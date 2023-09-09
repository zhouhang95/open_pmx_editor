#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::io::{Cursor, Write};

use byteorder::{LE, WriteBytesExt};
use byteorder::{LittleEndian, ReadBytesExt};
use glam::*;

use crate::common::*;
use crate::pmx::*;

impl Pmx {
    fn write_string(file: &mut Cursor<Vec<u8>>, content: &str) {
        let mut string: Vec<u16> = content.encode_utf16().collect();        
        file.write_u32::<LE>((string.len() * 2 )as u32).unwrap();
        file.write(bytemuck::cast_slice_mut(&mut string)).unwrap();
    }

    pub fn write(&self) -> Vec<u8> {
        let content = Vec::new();
        let mut file = std::io::Cursor::new(content);
        file.write(b"PMX ").unwrap();
        file.write_f32::<LE>(2.0).unwrap(); // version
        file.write_u8(8).unwrap(); // unknown

        file.write_u8(0).unwrap(); // use uft-16
        file.write_u8(0).unwrap(); // appendix_uv
        file.write_u8(4).unwrap(); // vertex_index_size
        file.write_u8(4).unwrap(); // texture_index_size
        file.write_u8(4).unwrap(); // material_index_size
        file.write_u8(4).unwrap(); // bone_index_size
        file.write_u8(4).unwrap(); // morph_index_size
        file.write_u8(4).unwrap(); // rigidbody_index_size

        Self::write_string(&mut file, &self.name);
        Self::write_string(&mut file, &self.name_en);
        Self::write_string(&mut file, &self.comment);
        Self::write_string(&mut file, &self.comment_en);
        
        self.write_verts(&mut file);
        self.write_faces(&mut file);
        file.write_u32::<LE>(0).unwrap(); // tex
        self.write_mats(&mut file);
        self.write_bones(&mut file);
        self.write_morphs(&mut file);


        self.write_display_frames(&mut file);
        file.write_u32::<LE>(0).unwrap(); // rigidbodys
        file.write_u32::<LE>(0).unwrap(); // joints

        file.into_inner()
    }

    fn write_verts(&self, file: &mut Cursor<Vec<u8>>) {
        file.write_u32::<LE>(self.verts.len() as _).unwrap();
        for v in &self.verts {
            write_float3(file, v.pos);
            write_float3(file, v.nrm);
            write_float2(file, v.uv);
            match v.weight {
                VertexWeight::One(b0) => {
                    file.write_u8(0).unwrap();
                    file.write_i32::<LE>(b0).unwrap();
                },
                VertexWeight::Two(b1, b2, w) => {
                    file.write_u8(1).unwrap();
                    file.write_i32::<LE>(b1).unwrap();
                    file.write_i32::<LE>(b2).unwrap();
                    file.write_f32::<LE>(w).unwrap();
                },
                VertexWeight::Four(bi, bw) => {
                    file.write_u8(2).unwrap();
                    write_int4(file, bi);
                    write_float4(file, bw);
                },
                VertexWeight::Sphere(b1, b2, weight, c, r0, r1) => {
                    file.write_u8(3).unwrap();
                    file.write_i32::<LE>(b1).unwrap();
                    file.write_i32::<LE>(b2).unwrap();
                    file.write_f32::<LE>(weight).unwrap();
                    write_float3(file, c);
                    write_float3(file, r0);
                    write_float3(file, r1);
                },
                VertexWeight::Quat(bi, bw) => {
                    file.write_u8(4).unwrap();
                    write_int4(file, bi);
                    write_float4(file, bw);
                },
            }
            file.write_f32::<LE>(v.edge_scale).unwrap();
        }
    }

    fn write_faces(&self, file: &mut Cursor<Vec<u8>>) {
        file.write_u32::<LE>(3 * self.faces.len() as u32).unwrap();

        for f in &self.faces {
            file.write_u32::<LE>(f[0]).unwrap();
            file.write_u32::<LE>(f[1]).unwrap();
            file.write_u32::<LE>(f[2]).unwrap();
        }
    }

    fn write_morphs(&self, file: &mut Cursor<Vec<u8>>)  {
        file.write_u32::<LE>(self.morphs.len() as _).unwrap();
        for morph in &self.morphs {
            Self::write_string(file, &morph.name);
            Self::write_string(file, &morph.name_en);
            file.write_i8(morph.panel).unwrap();
            file.write_i8(morph.category).unwrap();
            match &morph.data {
                Morph::MorphGroup(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        file.write_u32::<LE>(v.index).unwrap();
                        file.write_f32::<LE>(v.affect).unwrap();
                    }
                },
                Morph::MorphFlip(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        file.write_u32::<LE>(v.index).unwrap();
                        file.write_f32::<LE>(v.affect).unwrap();
                    }
                },
                Morph::MorphVertex(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        file.write_u32::<LE>(v.index).unwrap();
                        write_float3(file, v.trans);
                    }
                },
                Morph::MorphBone(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        file.write_u32::<LE>(v.index).unwrap();
                        write_float3(file, v.trans);
                        write_quat(file, v.rot);
                    }
                },
                Morph::MorphUv(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        file.write_u32::<LE>(v.index).unwrap();
                        write_float4(file, v.trans);
                    }
                },
                Morph::MorphRigidbody(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        file.write_u32::<LE>(v.index).unwrap();
                        file.write_i8(if v.local { 1 } else {0}).unwrap();
                        write_float3(file, v.trans_speed);
                        write_float3(file, v.rot_torque);
                    }
                },
                Morph::MorphMat(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        file.write_u32::<LE>(v.index).unwrap();
                        match v.blend_mode {
                            BlendMode::Mul => {
                                file.write_u8(0).unwrap();
                            },
                            BlendMode::Add => {
                                file.write_u8(1).unwrap();
                            },
                            BlendMode::Disable => todo!(),
                            BlendMode::Other => todo!(),
                        }
                        write_float4(file, v.diffuse);
                        write_float3(file, v.specular);
                        file.write_f32::<LE>(v.specularity).unwrap();
                        write_float3(file, v.ambient);
                        write_float4(file, v.edge_color);
                        file.write_f32::<LE>(v.edge_size).unwrap();
                        write_float4(file, v.texture_tint);
                        write_float4(file, v.environment_tint);
                        write_float4(file, v.toon_tint);
                    }
                },
            }
        }
    }

    fn write_mats(&self, file: &mut Cursor<Vec<u8>>) {
        if self.faces.is_empty() {
            file.write_u32::<LE>(0).unwrap();
            return;
        }
        let default_mats = vec![
            Mat {
                associated_face_count: self.faces.len() as u32,
                ..Default::default()
            }
        ];
        let mats = if self.mats.is_empty() {
            &default_mats
        } else {
            &self.mats
        };
        file.write_u32::<LE>(mats.len() as _).unwrap();
        for m in mats {
            Self::write_string(file, &m.name);
            Self::write_string(file, &m.name_en);
            write_float4(file, m.diffuse);
            write_float3(file, m.specular);
            file.write_f32::<LE>(m.specular_strength).unwrap();
            write_float3(file, m.ambient);
            file.write_u8(m.draw_flag.bits()).unwrap();
            write_float4(file, m.edge_color);
            file.write_f32::<LE>(m.edge_scale).unwrap();
            file.write_i32::<LE>(m.tex_index).unwrap();
            file.write_i32::<LE>(m.env_index).unwrap();
            let env_blend_mode = match m.env_blend_mode {
                BlendMode::Disable => 0,
                BlendMode::Mul => 1,
                BlendMode::Add => 2,
                BlendMode::Other => 3,
            };
            file.write_u8(env_blend_mode).unwrap();
            match m.toon {
                Toon::Tex(i) => {
                    file.write_u8(0).unwrap();
                    file.write_i32::<LE>(i).unwrap();
                },
                Toon::Inner(i) => {
                    file.write_u8(1).unwrap();
                    file.write_u8(i).unwrap();
                },
            }

            Self::write_string(file, &m.comment);
            file.write_u32::<LE>(m.associated_face_count * 3).unwrap();
        }
    }
    fn write_bones(&self, file: &mut Cursor<Vec<u8>>) {
        let default = vec![ Bone::default() ];

        let bones = if self.bones.is_empty() {
            &default
        } else {
            &self.bones
        };
        file.write_u32::<LE>(bones.len() as _).unwrap();
        for (i, b) in bones.iter().enumerate() {
            Self::write_string(file, &b.name);
            Self::write_string(file, &b.name_en);
            write_float3(file, b.pos);
            if let Some(p) = b.parent_index {
                file.write_i32::<LE>(p as _).unwrap();
            } else {
                file.write_i32::<LE>(-1).unwrap();
            };
            file.write_i32::<LE>(b.layer).unwrap();

            file.write_u16::<LE>(b.bone_flags.bits()).unwrap();
            match b.bone_tail_pos {
                BoneTailPos::Bone(bi) => {
                    file.write_i32::<LE>(bi).unwrap();
                },
                BoneTailPos::Pos(pos) => {
                    write_float3(file, pos);
                },
            }
            if let Some((parent_index, affect)) = b.inherit {
                file.write_i32::<LE>(parent_index).unwrap();
                file.write_f32::<LE>(affect).unwrap();
            }
            if let Some(v) = b.fixed_axis {
                write_float3(file, v);
            }
            if let Some((x, y)) = b.local_axis {
                write_float3(file, x);
                write_float3(file, y);
            }
            if let Some(external_parent) = b.external_parent {
                file.write_i32::<LE>(external_parent).unwrap();
            }
            for ik in &self.iks {
                if ik.bone == i as _ {
                    file.write_i32::<LE>(ik.effector).unwrap();
                    file.write_i32::<LE>(ik.loop_count).unwrap();
                    file.write_f32::<LE>(ik.limit_angle).unwrap();
                    file.write_i32::<LE>(ik.ik_joints.len() as _).unwrap();
                    for j in &ik.ik_joints {
                        file.write_i32::<LE>(j.bone).unwrap();
                        if let Some((limit_min, limit_max)) = j.limit {
                            file.write_i8(1).unwrap();
                            write_float3(file, limit_min);
                            write_float3(file, limit_max);
                            
                        } else {
                            file.write_i8(0).unwrap();
                        }
                    }
                }
            }
        }
    }
    fn write_display_frames(&self, file: &mut Cursor<Vec<u8>>) {
        let display_frames: Vec<DisplayFrame> = if self.display_frames.is_empty() {
            vec![
                DisplayFrame { name: "Root".to_string(), name_en: "Root".to_string(), deletable: true, morph_items: vec![DisplayFrameIndex::Bone(0)] },
                DisplayFrame { name: "表情".to_string(), name_en: "Exp".to_string(), deletable: true, morph_items: vec![] },
            ]
        } else {
            self.display_frames.clone()
        };

        file.write_u32::<LE>(display_frames.len() as _).unwrap();
        for df in &display_frames {
            Self::write_string(file, &df.name);
            Self::write_string(file, &df.name_en);
            file.write_u8(if df.deletable { 1 } else { 0 }).unwrap();
            file.write_i32::<LE>(df.morph_items.len() as _).unwrap();
            for index in &df.morph_items {
                match index {
                    DisplayFrameIndex::Bone(bi) => {
                        file.write_u8(0).unwrap();
                        file.write_u32::<LE>(*bi).unwrap();
                        
                    },
                    DisplayFrameIndex::Morph(mi) => {
                        file.write_u8(1).unwrap();
                        file.write_u32::<LE>(*mi).unwrap();
                    },
                }
            }
        }
    }


}

