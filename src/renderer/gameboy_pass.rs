use std::{num::NonZeroU32, sync::Arc};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::emulator;

use super::wgpu_core::WGPUCore;

const GAMEBOY_SCREEN: wgpu::Extent3d = wgpu::Extent3d {
    width: 160,
    height: 144,
    depth_or_array_layers: 1,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
    },
    Vertex {
        position: [3.0, -1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 3.0, 0.0],
    },
];

// const INDICES: &[u16] = &[0, 1, 2];
const INDICES: &[u16] = &[0, 1, 2];

/// This will eventually be chooseable through a menu
#[allow(dead_code)]
pub enum GameBoyPassPipelineChoice {
    Naive,
    Xbr,
}

pub struct GameBoyPass {
    buffer: Arc<emulator::DoubleBuffer>,
    texture: wgpu::Texture,
    texture_bind_group: wgpu::BindGroup,
    naive_pipeline: wgpu::RenderPipeline,
    xbr_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    pub pipeline_to_use: GameBoyPassPipelineChoice,
}

impl GameBoyPass {
    pub fn new(core: &WGPUCore, buffer: Arc<emulator::DoubleBuffer>) -> Self {
        let (texture, texture_bind_group_layout, texture_bind_group) =
            Self::create_framebuffer_texture(core);

        let naive_pipeline = Self::create_pipeline(
            core,
            "Naive",
            include_str!("gameboy_naive.wgsl"),
            &texture_bind_group_layout,
        );

        let xbr_pipeline = Self::create_pipeline(
            core,
            "XBR",
            include_str!("gameboy_xbr.wgsl"),
            &texture_bind_group_layout,
        );

        let vertex_buffer = core
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Gameboy Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = core
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Gameboy Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            buffer,
            texture,
            texture_bind_group,
            naive_pipeline,
            xbr_pipeline,
            vertex_buffer,
            index_buffer,
            pipeline_to_use: GameBoyPassPipelineChoice::Naive,
        }
    }

    fn create_pipeline(
        core: &WGPUCore,
        name: &str,
        shader_source: &str,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = core
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("{} Gameboy Shader", name)),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        let pipeline_layout = core
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Gameboy Pipeline Layout", name)),
                bind_group_layouts: &[texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = core
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{} Gameboy Pipeline", name)),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: core.surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        pipeline
    }

    fn create_framebuffer_texture(
        core: &WGPUCore,
    ) -> (wgpu::Texture, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let texture = core.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Gameboy Framebuffer Texture"),
            size: GAMEBOY_SCREEN,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = core.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Gameboy Framebuffer Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            core.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Gameboy Framebuffer Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let texture_bind_group = core.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gameboy Framebuffer Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        (texture, texture_bind_group_layout, texture_bind_group)
    }
}

impl GameBoyPass {
    pub fn render(&self, core: &WGPUCore, output: &wgpu::TextureView) {
        let data = self
            .buffer
            .get_current()
            .lock()
            .expect("Emulator thread poisoned");

        core.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &*data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(160),
                rows_per_image: NonZeroU32::new(144),
            },
            GAMEBOY_SCREEN,
        );

        drop(data);

        let mut encoder = core
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Gameboy Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Gameboy Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            let pipeline = match self.pipeline_to_use {
                GameBoyPassPipelineChoice::Naive => &self.naive_pipeline,
                GameBoyPassPipelineChoice::Xbr => &self.xbr_pipeline,
            };

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
        }

        core.queue.submit(std::iter::once(encoder.finish()));
    }
}
