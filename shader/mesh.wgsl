struct VertexInput {
    @location(0) pos: vec3f,
    @location(1) nrm: vec3f,
};

struct VertexOut {
    @location(0) nrm: vec3f,
    @location(1) opos: vec3f,
    @builtin(position) pos: vec4f,
};

struct Uniforms {
    view_proj: mat4x4f,
    view: mat4x4f,
    proj: mat4x4f,
    flag: vec4f,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOut {
    var out: VertexOut;

    out.pos = uniforms.view_proj * vec4f(model.pos, 1.0);
    out.opos = model.pos;
    out.nrm = model.nrm;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    if uniforms.flag.y > 0.0 {
        return vec4f(0.7, 0.7, 0.7, 0.0);
    }
    var planer_nrm_os = -normalize(cross(dpdx(in.opos), dpdy(in.opos)));
    var nrm_os = mix(in.nrm, planer_nrm_os, uniforms.flag.x);
    var nrm_cs = uniforms.view * vec4f(nrm_os, 0.0);
    var ins = mix(0.4, 1.0, saturate(nrm_cs.z));
    return vec4f(ins, ins, ins, 0.0);
}

@fragment
fn wireframe_main(in: VertexOut) -> @location(0) vec4f {
    return vec4f(0.22, 0.22, 0.295, 0.0);
}
