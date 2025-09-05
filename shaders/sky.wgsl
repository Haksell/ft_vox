// shaders/sky.wgsl

struct Camera {
    // Must match your CameraUniform layout in Rust/WGSL used by shader.wgsl.
    // Existing code already provides a view-projection matrix to the vertex stage.
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
};

@group(1) @binding(0) var<uniform> camera : Camera;

@group(0) @binding(0) var skyTex : texture_2d<f32>;
@group(0) @binding(1) var skySampler : sampler;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,  // clip-space xy in [-1, 1]
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vi: u32) -> VSOut {
    // Fullscreen triangle positions in clip space.
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );

    var out: VSOut;
    out.pos = vec4<f32>(pos[vi], 0.0, 1.0);
    out.ndc = pos[vi];
    return out;
}

fn world_dir_from_ndc(ndc: vec2<f32>) -> vec3<f32> {
    // Reconstruct two world points along the view ray using the inverse ViewProjection.
    let inv_vp = camera.inverse_view_proj;

    // Note: In wgpu, NDC z is in [0, 1]. Sample two depths (near-ish and far) to get a ray direction.
    let p0 = inv_vp * vec4<f32>(ndc, 0.0, 1.0);
    let p1 = inv_vp * vec4<f32>(ndc, 1.0, 1.0);

    let w0 = p0.xyz / p0.w;
    let w1 = p1.xyz / p1.w;

    return normalize(w1 - w0);
}

fn pano_uv(dir: vec3<f32>) -> vec2<f32> {
    // Convert direction to equirectangular UVs.
    let phi = atan2(dir.z, dir.x);                 // [-pi, pi]
    let theta = acos(clamp(dir.y, -1.0, 1.0));       // [0, pi]
    let u = phi / (2.0 * 3.141592653589793) + 0.5;   // [0, 1]
    let v = 1.0 - (theta / 3.141592653589793);       // [0, 1], flip so v=0 is top
    return vec2<f32>(u, v);
}

@fragment
fn fs_sky(in: VSOut) -> @location(0) vec4<f32> {
    let dir = world_dir_from_ndc(in.ndc);
    let uv = pano_uv(dir);
    let c = textureSample(skyTex, skySampler, uv);
    return vec4<f32>(c.rgb, 1.0);
}
