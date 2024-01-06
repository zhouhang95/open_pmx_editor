struct VertexInput {
    @location(0) pos: vec3f,
    @location(1) nrm: vec3f,
    @location(2) uv: vec2f,
};

struct VertexOut {
    @location(0) nrm: vec3f,
    @location(1) opos: vec3f,
    @location(2) uv: vec2f,
    @builtin(position) pos: vec4f,
};

struct Uniforms {
    view_proj: mat4x4f,
    view: mat4x4f,
    proj: mat4x4f,
    cam_real_pos: vec4f,
    flag: vec4f,
};

struct MatUniforms {
    diffuse: vec4f,
    specular: vec4f,
    ambient: vec4f,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;
@group(1) @binding(2)
var<uniform> mat_uniforms: MatUniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOut {
    var out: VertexOut;

    out.pos = uniforms.view_proj * vec4f(model.pos, 1.0);
    out.opos = model.pos;
    out.nrm = model.nrm;
    out.uv = model.uv;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4f {
    if uniforms.flag.z > 0.0 {
        let view_dir = normalize(uniforms.cam_real_pos.xyz - in.opos);
        let ligth_dir = normalize(vec3f(1.0, 1.0, 1.0));
        let halfway = normalize(view_dir + ligth_dir);
        let nrm = normalize(in.nrm);
        let spec = pow(max(dot(nrm, halfway), 0.0), mat_uniforms.specular.w);
        let light = mat_uniforms.diffuse.xyz * 0.5 + mat_uniforms.ambient.xyz + mat_uniforms.specular.xyz * spec;
        return textureSample(t_diffuse, s_diffuse, in.uv) * vec4f(light, mat_uniforms.diffuse.w);
    }
    if uniforms.flag.y > 0.0 {
        return vec4f(0.7, 0.7, 0.7, 1.0);
    }
    var planer_nrm_os = -normalize(cross(dpdx(in.opos), dpdy(in.opos)));
    var nrm_os = mix(in.nrm, planer_nrm_os, uniforms.flag.x);
    var nrm_cs = uniforms.view * vec4f(nrm_os, 0.0);
    var ins = mix(0.4, 1.0, saturate(nrm_cs.z));
    return vec4f(ins, ins, ins, 1.0);
}

@fragment
fn wireframe_main(in: VertexOut) -> @location(0) vec4f {
    return vec4f(0.22, 0.22, 0.295, 1.0);
}
