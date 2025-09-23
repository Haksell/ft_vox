struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_proj_skybox_inverse: mat4x4<f32>,
    pos: vec3<f32>,
    _pad0: f32,
    near: f32,
    far: f32,
    _pad1: vec2<f32>,
};

@group(0) @binding(0) var sceneTexture : texture_2d<f32>;
@group(0) @binding(1) var sceneSampler : sampler;
@group(0) @binding(2) var depthTexture : texture_depth_2d;
@group(0) @binding(3) var depthSampler : sampler;
@group(0) @binding(4) var<uniform> cam : CameraUniform;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Fullscreen triangle
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );
    var uv = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 2.0),
        vec2<f32>(2.0, 0.0),
        vec2<f32>(0.0, 0.0),
    );

    var out: VertexOutput;
    out.pos = vec4<f32>(pos[vi], 0.0, 1.0);
    out.uv = uv[vi];
    return out;
}

fn linearize_depth(d: f32, near: f32, far: f32) -> f32 {
    // d is depth buffer value in [0,1] (post-projection z)
    // assumes standard perspective projection mapping
    return (2.0 * near) / (far + near - d * (far - near));
}

fn luma(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texSize = vec2<f32>(textureDimensions(sceneTexture, 0));
    let uv = in.uv;

    // Gather depth (linearized) at center + 4-neighborhood
    let duv = 1.0 / texSize;
    let dC = linearize_depth(textureSample(depthTexture, depthSampler, uv), cam.near, cam.far);
    let dL = linearize_depth(textureSample(depthTexture, depthSampler, uv + vec2<f32>(-duv.x, 0.0)), cam.near, cam.far);
    let dR = linearize_depth(textureSample(depthTexture, depthSampler, uv + vec2<f32>(duv.x, 0.0)), cam.near, cam.far);
    let dT = linearize_depth(textureSample(depthTexture, depthSampler, uv + vec2<f32>(0.0, -duv.y)), cam.near, cam.far);
    let dB = linearize_depth(textureSample(depthTexture, depthSampler, uv + vec2<f32>(0.0, duv.y)), cam.near, cam.far);

    // Depth edge magnitude (Sobel-ish)
    let dx = (dR - dL);
    let dy = (dB - dT);
    let depthEdge = sqrt(dx * dx + dy * dy);

    // Optional: color contrast to catch coplanar albedo edges
    let cC = textureSample(sceneTexture, sceneSampler, uv).rgb;
    let cL = textureSample(sceneTexture, sceneSampler, uv + vec2<f32>(-duv.x, 0.0)).rgb;
    let cR = textureSample(sceneTexture, sceneSampler, uv + vec2<f32>(duv.x, 0.0)).rgb;
    let cT = textureSample(sceneTexture, sceneSampler, uv + vec2<f32>(0.0, -duv.y)).rgb;
    let cB = textureSample(sceneTexture, sceneSampler, uv + vec2<f32>(0.0, duv.y)).rgb;

    let lumC = luma(cC);
    let colorEdge = abs(luma(cR) - luma(cL)) + abs(luma(cB) - luma(cT));

    // Tunables
    let depthThreshold: f32 = 0.002;  // increase for fewer outlines
    let colorThreshold: f32 = 0.20;   // increase to rely less on color edges
    let outlineStrength: f32 = 1.0;    // 0..1 mix of black

    let isDepthEdge = depthEdge > depthThreshold;
    let isColorEdge = colorEdge > colorThreshold;

    let edge = select(0.0, 1.0, isDepthEdge || isColorEdge);

    let base = cC;
    let outlined = mix(base, vec3<f32>(0.0), edge * outlineStrength);
    return vec4<f32>(outlined, 1.0);
}
