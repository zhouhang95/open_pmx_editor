#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::io::{Cursor, Write};

use byteorder::{LE, WriteBytesExt};
use byteorder::{LittleEndian, ReadBytesExt};
use glam::*;

use super::common::*;
use super::pmx::*;


impl Pmx {
    fn write_string(file: &mut Cursor<Vec<u8>>, content: &str) {
        let mut string: Vec<u16> = content.encode_utf16().collect();        
        file.write_u32::<LE>((string.len() * 2 )as u32).unwrap();
        file.write(bytemuck::cast_slice_mut(&mut string)).unwrap();
    }

    fn get_int_size(s: usize) -> u8 {
        if s <= 127 {
            1
        } else if s <= 32767 {
            2
        } else {
            4
        }
    }

    fn get_uint_size(s: usize) -> u8 {
        if s <= 255 {
            1
        } else if s <= 65535 {
            2
        } else {
            4
        }
    }

    fn write_int(file: &mut Cursor<Vec<u8>>, v: i32, index_size: u8) {
        match index_size {
            1 => file.write_i8(v as _).unwrap(),
            2 => file.write_i16::<LE>(v as _).unwrap(),
            4 => file.write_i32::<LE>(v).unwrap(),
            _ => unreachable!(),
        }
    }

    fn write_uint(file: &mut Cursor<Vec<u8>>, v: i32, index_size: u8) {
        match index_size {
            1 => file.write_u8(v as _).unwrap(),
            2 => file.write_u16::<LE>(v as _).unwrap(),
            4 => file.write_i32::<LE>(v).unwrap(),
            _ => unreachable!(),
        }
    }

    pub fn write(&self) -> Vec<u8> {
        let content = Vec::new();
        let mut file = std::io::Cursor::new(content);
        file.write(b"PMX ").unwrap();
        file.write_f32::<LE>(2.0).unwrap(); // version
        file.write_u8(8).unwrap(); // unknown

        file.write_u8(0).unwrap(); // use uft-16
        file.write_u8(self.appendix_uvs.len() as _).unwrap(); // appendix_uv
        let vertex_index_size = Pmx::get_uint_size(self.verts.len());
        let texture_index_size = Pmx::get_int_size(self.texs.len());
        let material_index_size = Pmx::get_int_size(self.mats.len());
        let bone_index_size = Pmx::get_int_size(self.bones.len());
        let morph_index_size = Pmx::get_int_size(self.morphs.len());
        let rigidbody_index_size = Pmx::get_int_size(self.rigidbodys.len());
        file.write_u8(vertex_index_size).unwrap();
        file.write_u8(texture_index_size).unwrap();
        file.write_u8(material_index_size).unwrap();
        file.write_u8(bone_index_size).unwrap();
        file.write_u8(morph_index_size).unwrap();
        file.write_u8(rigidbody_index_size).unwrap();

        Self::write_string(&mut file, &self.name);
        Self::write_string(&mut file, &self.name_en);
        Self::write_string(&mut file, &self.comment);
        Self::write_string(&mut file, &self.comment_en);
        
        self.write_verts(&mut file, bone_index_size);
        self.write_faces(&mut file, vertex_index_size);
        self.write_texs(&mut file);
        self.write_mats(&mut file, texture_index_size);
        self.write_bones(&mut file, bone_index_size);
        self.write_morphs(
            &mut file,
            vertex_index_size,
            material_index_size,
            bone_index_size,
            morph_index_size,
            rigidbody_index_size
        );


        self.write_display_frames(&mut file, bone_index_size, morph_index_size);
        file.write_u32::<LE>(0).unwrap(); // rigidbodys
        file.write_u32::<LE>(0).unwrap(); // joints

        file.into_inner()
    }

    fn write_texs(&self, file: &mut Cursor<Vec<u8>>) {
        file.write_u32::<LE>(self.texs.len() as _).unwrap();
        for tex in &self.texs {
            Self::write_string(file, tex);
        }
    }

    fn write_verts(&self, file: &mut Cursor<Vec<u8>>, bone_index_size: u8) {
        file.write_u32::<LE>(self.verts.len() as _).unwrap();
        for (i, v) in self.verts.iter().enumerate() {
            write_float3(file, v.pos);
            write_float3(file, v.nrm);
            write_float2(file, v.uv);
            for uvs in &self.appendix_uvs {
                write_float4(file, uvs[i]);
            }
            match v.weight {
                VertexWeight::One(b0) => {
                    file.write_u8(0).unwrap();
                    Pmx::write_int(file, b0, bone_index_size);
                },
                VertexWeight::Two(b1, b2, w) => {
                    file.write_u8(1).unwrap();
                    Pmx::write_int(file, b1, bone_index_size);
                    Pmx::write_int(file, b2, bone_index_size);
                    file.write_f32::<LE>(w).unwrap();
                },
                VertexWeight::Four(bi, bw) => {
                    file.write_u8(2).unwrap();
                    Pmx::write_int(file, bi[0], bone_index_size);
                    Pmx::write_int(file, bi[1], bone_index_size);
                    Pmx::write_int(file, bi[2], bone_index_size);
                    Pmx::write_int(file, bi[3], bone_index_size);
                    write_float4(file, bw);
                },
                VertexWeight::Sphere(b1, b2, weight, c, r0, r1) => {
                    file.write_u8(3).unwrap();
                    Pmx::write_int(file, b1, bone_index_size);
                    Pmx::write_int(file, b2, bone_index_size);
                    file.write_f32::<LE>(weight).unwrap();
                    write_float3(file, c);
                    write_float3(file, r0);
                    write_float3(file, r1);
                },
                VertexWeight::Quat(bi, bw) => {
                    file.write_u8(4).unwrap();
                    Pmx::write_int(file, bi[0], bone_index_size);
                    Pmx::write_int(file, bi[1], bone_index_size);
                    Pmx::write_int(file, bi[2], bone_index_size);
                    Pmx::write_int(file, bi[3], bone_index_size);
                    write_float4(file, bw);
                },
            }
            file.write_f32::<LE>(v.edge_scale).unwrap();
        }
    }

    fn write_faces(&self, file: &mut Cursor<Vec<u8>>, vertex_index_size: u8) {
        file.write_u32::<LE>(3 * self.faces.len() as u32).unwrap();

        for f in &self.faces {
            Pmx::write_uint(file, f[0] as _, vertex_index_size);
            Pmx::write_uint(file, f[1] as _, vertex_index_size);
            Pmx::write_uint(file, f[2] as _, vertex_index_size);
        }
    }

    fn write_morphs(
        &self,
        file: &mut Cursor<Vec<u8>>,
        vertex_index_size: u8,
        material_index_size: u8,
        bone_index_size: u8,
        morph_index_size: u8,
        rigidbody_index_size: u8
    )  {
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
                        Pmx::write_int(file, v.index as _, morph_index_size);
                        file.write_f32::<LE>(v.affect).unwrap();
                    }
                },
                Morph::MorphFlip(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        Pmx::write_int(file, v.index as _, morph_index_size);
                        file.write_f32::<LE>(v.affect).unwrap();
                    }
                },
                Morph::MorphVertex(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        Pmx::write_int(file, v.index as _, vertex_index_size);
                        write_float3(file, v.trans);
                    }
                },
                Morph::MorphBone(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        Pmx::write_int(file, v.index as _, bone_index_size);
                        write_float3(file, v.trans);
                        write_quat(file, v.rot);
                    }
                },
                Morph::MorphUv(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        Pmx::write_int(file, v.index as _, vertex_index_size);
                        write_float4(file, v.trans);
                    }
                },
                Morph::MorphRigidbody(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        Pmx::write_int(file, v.index as _, rigidbody_index_size);
                        file.write_i8(if v.local { 1 } else {0}).unwrap();
                        write_float3(file, v.trans_speed);
                        write_float3(file, v.rot_torque);
                    }
                },
                Morph::MorphMat(vs) => {
                    file.write_u32::<LE>(vs.len() as _).unwrap();
                    for v in vs {
                        Pmx::write_int(file, v.index as _, morph_index_size);
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

    fn write_mats(&self, file: &mut Cursor<Vec<u8>>, texture_index_size: u8) {
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
            write_float4(file, m.specular);
            write_float3(file, m.ambient);
            file.write_u8(m.draw_flag.bits()).unwrap();
            write_float4(file, m.edge_color);
            file.write_f32::<LE>(m.edge_scale).unwrap();
            Pmx::write_int(file, m.tex_index, texture_index_size);
            Pmx::write_int(file, m.env_index, texture_index_size);
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
                    Pmx::write_int(file, i, texture_index_size);
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
    fn write_bones(&self, file: &mut Cursor<Vec<u8>>, bone_index_size: u8) {
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
                Pmx::write_int(file, p as _, bone_index_size);
            } else {
                Pmx::write_int(file, -1, bone_index_size);
            };
            file.write_i32::<LE>(b.layer).unwrap();

            file.write_u16::<LE>(b.bone_flags.bits()).unwrap();
            match b.bone_tail_pos {
                BoneTailPos::Bone(bi) => {
                    Pmx::write_int(file, bi, bone_index_size);
                },
                BoneTailPos::Pos(pos) => {
                    write_float3(file, pos);
                },
            }
            if let Some((parent_index, affect)) = b.inherit {
                Pmx::write_int(file, parent_index, bone_index_size);
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
                Pmx::write_int(file, external_parent, bone_index_size);
            }
            for ik in &self.iks {
                if ik.bone == i as _ {
                    Pmx::write_int(file, ik.effector, bone_index_size);
                    file.write_i32::<LE>(ik.loop_count).unwrap();
                    file.write_f32::<LE>(ik.limit_angle).unwrap();
                    file.write_i32::<LE>(ik.ik_joints.len() as _).unwrap();
                    for j in &ik.ik_joints {
                        Pmx::write_int(file, j.bone, bone_index_size);
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
    fn write_display_frames(&self, file: &mut Cursor<Vec<u8>>, bone_index_size: u8, morph_index_size: u8) {
        let display_frames: Vec<DisplayFrame> = if self.display_frames.len() < 2 {
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
                        Pmx::write_int(file, *bi as _, bone_index_size);
                    },
                    DisplayFrameIndex::Morph(mi) => {
                        file.write_u8(1).unwrap();
                        Pmx::write_int(file, *mi as _, morph_index_size);
                    },
                }
            }
        }
    }


}

