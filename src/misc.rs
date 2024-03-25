use std::f32::consts::PI;

use glam::*;

use crate::format::pmx::*;


pub fn add_sphere(m: &mut Pmx, segments: u32, rings: u32, radius: f32) {
    let sphere_name = format!("Sphere_{}", m.bones.len());

    {
        for i in 0..=rings {
            let v = i as f32 / rings as f32;
            let theta = v * PI;
            let y = theta.cos();
            let r = theta.sin();
            for j in 0..=segments {
                let u = j as f32 / segments as f32;
                let phi = u * PI * 2.0;
                let z = phi.cos() * r;
                let x = phi.sin() * r;

                let vertex = Vertex {
                    pos: vec3(x, y, z) * radius,
                    nrm: vec3(x, y, z),
                    uv: vec2(u, v),
                    weight: VertexWeight::One(m.bones.len() as _),
                    edge_scale: 0.0,
                };
                m.verts.push(vertex);
            }
        }
    }
    {
        for i in 0..rings {
            for j in 0..segments {
                let face_0 = (segments + 1) * i + j + 0;
                let face_1 = (segments + 1) * i + j + segments + 1;
                let face_2 = (segments + 1) * i + j + segments + 1 + 1;
                let face_3 = (segments + 1) * i + j + 1;
                m.faces.push([face_0, face_1, face_2]);
                m.faces.push([face_0, face_2, face_3]);
            }
        }
    }
    {
        let mut mat = Mat::default();
        mat.name = sphere_name.clone();
        mat.name_en = sphere_name.clone();
        mat.associated_face_count = segments * rings * 2;
        m.mats.push(mat);
    }
    {
        let mut bone_flags = BoneFlags::empty();
        bone_flags.insert(BoneFlags::TRANSLATABLE);
        bone_flags.insert(BoneFlags::ROTATABLE);
        bone_flags.insert(BoneFlags::VISIBLE);
        bone_flags.insert(BoneFlags::ENABLED);
        let bone = Bone {
            name: sphere_name.clone(),
            name_en: sphere_name.clone(),
            pos: Vec3::ZERO,
            parent_index: None,
            layer: 0,
            bone_flags: bone_flags,
            bone_tail_pos: BoneTailPos::Pos(Vec3::ZERO),
            inherit: None,
            fixed_axis: None,
            local_axis: None,
            external_parent: None,
        };
        m.bones.push(bone);
    }
}