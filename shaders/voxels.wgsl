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

const ATLAS_SHAPE: vec2<f32> = vec2(64.0, 64.0);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = (fract(in.tex_coords) + vec2<f32>(in.atlas_offset)) / ATLAS_SHAPE;
    let t16 = textureSample(t_diffuse, s_diffuse, vec2(uv.x, uv.y));
    let t8 = textureSample(t_diffuse, s_diffuse, vec2(uv.x / 2.0, uv.y / 2.0 + 0.5));
    let t4 = textureSample(t_diffuse, s_diffuse, vec2(uv.x / 4.0, uv.y / 4.0 + 0.75));
    let t2 = textureSample(t_diffuse, s_diffuse, vec2(uv.x / 8.0, uv.y / 8.0 + 0.875));
    let t1 = textureSample(t_diffuse, s_diffuse, vec2(uv.x / 16.0, uv.y / 16.0 + 0.9375));
    let d = in.dist;
    if d < 32.0 {
        return t16;
    } else if d < 64.0 {
        return mix(t16, t8, (d - 32.0) / 32.0);
    } else if d < 128.0 {
        return mix(t8, t4, (d - 64.0) / 64.0);
    } else if d < 256.0 {
        return mix(t4, t2, (d - 128.0) / 128.0);
    } else if d < 512.0 {
        return mix(t2, t1, (d - 256.0) / 256.0);
    } else {
        return t1;
    }
}