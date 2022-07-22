// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords.x = (model.position.x + 1.0) / 2.0;
    out.tex_coords.y = (-model.position.y + 1.0) / 2.0;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

let XBR_Y_WEIGHT: f32 = 48.0;
let XBR_EQ_THRESHOLD: f32 = 15.0;
let yuv: mat3x3<f32> = mat3x3<f32>(vec3<f32>(0.299, 0.587, 0.114), vec3<f32>(-0.169, -0.331, 0.499), vec3<f32>(0.499, -0.418, -0.0813));

fn RGBtoYUV(color: vec3<f32>) -> f32 {
    return dot(color, XBR_Y_WEIGHT*yuv[0]);
}

fn df(A: f32, B: f32) -> f32 {
    return abs(A-B);
}

fn eq(A: f32, B: f32) -> bool {
    return (df(A, B) < XBR_EQ_THRESHOLD);
}

fn weighted_distance(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32, g: f32, h: f32) -> f32 {
    return (df(a,b) + df(a,c) + df(d,e) + df(d,f) + 4.0*df(g,h));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var edr: bool;
    var px: bool;
    var interp_restriction_lv1: bool;
    var nc: bool;
    var fx: bool;

    var dim = vec2<f32>(textureDimensions(t_diffuse));
    var ps = vec2<f32>(1.0 / dim.x, 1.0 / dim.y);
    var dx = ps.x;
    var dy = ps.y;

    var t1: vec4<f32> = vec4<f32>(0.0, -dy, -dx, 0.0);

    var pos: vec2<f32> = fract(in.tex_coords * dim) - vec2(0.5, 0.5);
    var dir = sign(pos);

    var g1 = dir * t1.xy;
    var g2 = dir  * t1.zw;

    var B = textureSample(t_diffuse, s_diffuse, in.tex_coords + g1     ).xyz;
    var C = textureSample(t_diffuse, s_diffuse, in.tex_coords + g1 - g2).xyz;
    var D = textureSample(t_diffuse, s_diffuse, in.tex_coords      + g2).xyz;
    var E = textureSample(t_diffuse, s_diffuse, in.tex_coords          ).xyz;
    var F = textureSample(t_diffuse, s_diffuse, in.tex_coords      - g2).xyz;
    var G = textureSample(t_diffuse, s_diffuse, in.tex_coords - g1 + g2).xyz;
    var H = textureSample(t_diffuse, s_diffuse, in.tex_coords - g1     ).xyz;
    var I = textureSample(t_diffuse, s_diffuse, in.tex_coords - g1 - g2).xyz;

    var F4 = textureSample(t_diffuse, s_diffuse, in.tex_coords - 2.0 * g2).xyz;
    var I4 = textureSample(t_diffuse, s_diffuse, in.tex_coords - g1 - 2.0 * g2).xyz;
    var H5 = textureSample(t_diffuse, s_diffuse, in.tex_coords - 2.0 * g1).xyz;
    var I5 = textureSample(t_diffuse, s_diffuse, in.tex_coords - 2.0 * g1 - g2).xyz;

    var b = RGBtoYUV( B );
    var c = RGBtoYUV( C );
    var d = RGBtoYUV( D );
    var e = RGBtoYUV( E );
    var f = RGBtoYUV( F );
    var g = RGBtoYUV( G );
    var h = RGBtoYUV( H );
    var i = RGBtoYUV( I );

    var i4 = RGBtoYUV( I4 );
    var i5 = RGBtoYUV( I5 );
    var h5 = RGBtoYUV( H5 );
    var f4 = RGBtoYUV( F4 );

    var meow = (e != f);

    fx = ( dot(dir, pos) > 0.5 );
    interp_restriction_lv1 = ((e!=f) && (e!=h)  && ( !eq(f,b) && !eq(f,c) || !eq(h,d) && !eq(h,g) || eq(e,i) && (!eq(f,f4) && !eq(f,i4) || !eq(h,h5) && !eq(h,i5)) || eq(e,g) || eq(e,c)) );

    edr = (weighted_distance( e, c, g, i, h5, f4, h, f) < weighted_distance( h, d, i5, f, i4, b, e, i)) && interp_restriction_lv1;
    nc = (edr && fx);
    px = (df(e, f) <= df(e, h));

    // var colors: vec4<f32>;
    // colors = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // colors.y = colors.x;
    // colors.z = colors.x;

    var res: vec3<f32>;
    if (nc) {
        if (px) {
            res = F;
        } else {
            res = H;
        }
    } else {
        res = E;
    }

    res.y = res.x;
    res.z = res.x;

    return vec4(res, 1.0);
}