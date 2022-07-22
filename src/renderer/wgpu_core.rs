use winit::window::Window;

pub struct WGPUCore {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
}

impl WGPUCore {
    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = Self::get_desktop_adapter(&instance, &surface);
        let (device, queue) = Self::get_desktop_device(&adapter);

        let surface_config = Self::get_desktop_surface_config(size, &surface, &adapter);
        surface.configure(&device, &surface_config);

        Self {
            size,
            instance,
            surface,
            device,
            queue,
            surface_config,
        }
    }

    fn get_desktop_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        }))
        .expect("Could not find any suitable adapter")
    }

    fn get_desktop_surface_config(
        size: winit::dpi::PhysicalSize<u32>,
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
    ) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(adapter)[0],
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Fifo,
        }
    }

    fn desktop_device_descriptor() -> wgpu::DeviceDescriptor<'static> {
        wgpu::DeviceDescriptor {
            label: Some("GB Desktop Device Descriptor"),
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        }
    }

    fn get_desktop_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        pollster::block_on(adapter.request_device(&Self::desktop_device_descriptor(), None))
            .expect("Could not find any suitable device")
    }
}

impl WGPUCore {
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}
