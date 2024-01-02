struct VertexInput {
    @location(0) pos: vec3f,
    @location(1) nrm: vec3f,
};

struct VertexOut {
    @location(0) nrm: vec3f,
    @builtin(position) pos: vec4f,
};

struct Uniforms {
    view_proj: mat4x4f,
    view: mat4x4f,
    proj: mat4x4f,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOut {
    var out: VertexOut;

    out.pos = uniforms.view_proj * vec4f(model.pos, 1.0);
    out.nrm = model.nrm;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    var nrm = uniforms.view * vec4f(in.nrm, 0.0);
    var ins = mix(0.4, 1.0, saturate(nrm.z));
    return vec4f(ins, ins, ins, 0.0);
}
