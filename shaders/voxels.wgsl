struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_proj_skybox_inverse: mat4x4<f32>,
    pos: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) atlas_offset: vec2<u32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) atlas_offset: vec2<u32>,
    @location(2) dist: f32,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.atlas_offset = model.atlas_offset;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.dist = distance(model.position.xyz, camera.pos);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

const ATLAS_SHAPE: vec2<f32> = vec2(128.0, 64.0);

fn t16(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, vec2(uv.x, uv.y));
}

fn t8(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, vec2(uv.x / 2.0 + 0.5, uv.y / 2.0 + 0.5));
}

fn t4(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, vec2(uv.x / 4.0 + 0.75, uv.y / 4.0 + 0.75));
}

fn t2(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, vec2(uv.x / 8.0 + 0.875, uv.y / 8.0 + 0.875));
}

fn t1(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, vec2(uv.x / 16.0 + 0.9375, uv.y / 16.0 + 0.9375));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = (fract(in.tex_coords) + vec2<f32>(in.atlas_offset)) / ATLAS_SHAPE;

    let d = max(in.dist * 0.5, 1e-5);
    var lod = clamp(log2(d / 32.0), 0.0, 4.0);
    let level_lo = u32(clamp(floor(lod), 0.0, 3.0));
    let frac_lod = fract(lod);

    var a: vec4<f32>;
    var b: vec4<f32>;

    switch level_lo {
        case 0u: { a = t16(uv); b = t8(uv); }
        case 1u: { a = t8(uv);  b = t4(uv); }
        case 2u: { a = t4(uv);  b = t2(uv); }
        case 3u: { a = t2(uv);  b = t1(uv); }
        default: { a = t1(uv);  b = t1(uv); }
    }

    return mix(a, b, frac_lod);
}