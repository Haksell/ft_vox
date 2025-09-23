struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_proj_skybox_inverse: mat4x4<f32>,
    pos: vec3<f32>,
    _pad0: f32,
    near: f32,
    far: f32,
    _pad1: vec2<f32>,
};

@group(0) @binding(0) var sceneTexture  : texture_2d<f32>;
@group(0) @binding(1) var sceneSampler : sampler;
@group(0) @binding(2) var depthTexture  : texture_depth_2d;
@group(0) @binding(3) var depthSampler : sampler;
@group(0) @binding(4) var<uniform> cam : CameraUniform;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    var uv = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(2.0, 0.0)
    );

    var out: VSOut;
    out.pos = vec4<f32>(pos[vi], 0.0, 1.0);
    out.uv = uv[vi];
    return out;
}

// Linearize D3D/WebGPU depth in [0,1] to a positive view-space Z (>0 forward)
fn linearize_depth(d: f32, n: f32, f: f32) -> f32 {
    // Derived from perspective projection with [0,1] depth range.
    // Returns view-space distance (meters) along +Z (magnitude only).
    return (n * f) / (f - d * (f - n));
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let texSize = vec2<f32>(textureDimensions(sceneTexture, 0));
    let duv = 1.0 / texSize;

    let baseColor = textureSample(sceneTexture, sceneSampler, uv).rgb;

    // Center depth; if it's background, pass through (prevents skybox outlines)
    let dC_ndc = textureSample(depthTexture, depthSampler, uv);
    if dC_ndc >= 0.9995 {
        return vec4<f32>(baseColor, 1.0);
    }

    // Neighbor depths
    let dL_ndc = textureSample(depthTexture, depthSampler, uv + vec2<f32>(-duv.x, 0.0));
    let dR_ndc = textureSample(depthTexture, depthSampler, uv + vec2<f32>(duv.x, 0.0));
    let dT_ndc = textureSample(depthTexture, depthSampler, uv + vec2<f32>(0.0, -duv.y));
    let dB_ndc = textureSample(depthTexture, depthSampler, uv + vec2<f32>(0.0, duv.y));

    // Treat background neighbors as "same depth" as center so edges don't appear next to sky.
    let dL_lin = linearize_depth(select(dC_ndc, dL_ndc, dL_ndc < 0.999999), cam.near, cam.far);
    let dR_lin = linearize_depth(select(dC_ndc, dR_ndc, dR_ndc < 0.999999), cam.near, cam.far);
    let dT_lin = linearize_depth(select(dC_ndc, dT_ndc, dT_ndc < 0.999999), cam.near, cam.far);
    let dB_lin = linearize_depth(select(dC_ndc, dB_ndc, dB_ndc < 0.999999), cam.near, cam.far);
    let dC_lin = linearize_depth(dC_ndc, cam.near, cam.far);

    // Simple gradient magnitude on linear depth
    let dx = dR_lin - dL_lin;
    let dy = dB_lin - dT_lin;
    let edgeMag = sqrt(dx * dx + dy * dy);

    if edgeMag >= 1.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else {
        return vec4<f32>(baseColor, 1.0);
    }

    // Depth-aware thresholding:
    // normalize center depth to [0,1] across clip range to relax threshold with distance
    let dC_norm = clamp((dC_lin - cam.near) / (cam.far - cam.near), 0.0, 1.0);
    let baseThreshold = 0.002;      // near-field sensitivity
    let farScale = 6.0;        // how much easier to pass threshold up close than far away
    let threshold = baseThreshold * mix(1.0, farScale, dC_norm);

    // Outline only on foreground pixels (center valid)
    let edge = select(0.0, 1.0, edgeMag > threshold);

    let outlineStrength = 1.0;
    let outlined = mix(baseColor, vec3<f32>(0.0), edge * outlineStrength);
    return vec4<f32>(outlined, 1.0);
}
