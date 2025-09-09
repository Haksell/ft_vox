struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_proj_skybox_inverse: mat4x4<f32>,
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
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.atlas_offset = model.atlas_offset;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

const ATLAS_SHAPE: vec2<f32> = vec2(64.0, 32.0);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = (fract(in.tex_coords) + vec2<f32>(in.atlas_offset)) / ATLAS_SHAPE;
    return textureSample(t_diffuse, s_diffuse, uv);
}