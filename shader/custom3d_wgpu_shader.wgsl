struct VertexInput {
    @location(0) pos: vec3f,
    @location(1) color: vec3f,
};

struct VertexOut {
    @location(0) color: vec4f,
    @builtin(position) pos: vec4f,
};

struct Uniforms {
    @size(16) angle: f32, // pad to 16 bytes
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOut {
    var out: VertexOut;

    out.pos = vec4f(model.pos, 1.0);
    out.pos.x = out.pos.x * cos(uniforms.angle);
    out.color = vec4f(model.color, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    return in.color;
}
