#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::*;
use std::mem::transmute;
use std::path::Path;
use std::collections::*;

use byteorder::{LE, ReadBytesExt};
use glam::*;
use image::Rgba;
use image::RgbaImage;
use uuid::Uuid;
use image::ImageReader;
use image::ImageFormat;
use image::DynamicImage;

use super::motion::Motion;
use super::common::*;
use bitflags::bitflags;


#[derive(Clone)]
pub struct Pmx {
    pub name: String,
    pub name_en: String,
    pub comment: String,
    pub comment_en: String,
    pub verts: Vec<Vertex>,
    pub appendix_uvs: Vec<Vec<Vec4>>,
    pub faces: Vec<[u32; 3]>,
    pub texs: Vec<String>,
    pub mats: Vec<Mat>,
    pub bones: Vec<Bone>,
    pub iks: Vec<Ik>,
    pub morphs: Vec<MorphInfo>,
    pub rigidbodys: Vec<Rigidbody>,
    pub joints: Vec<Joint>,
    pub display_frames: Vec<DisplayFrame>,
    pub path: String,
    pub uuid: Uuid,
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: Vec3,
    pub nrm: Vec3,
    pub uv: Vec2,
    pub weight: VertexWeight,
    pub edge_scale: f32,
}

#[derive(Copy, Clone)]
pub enum VertexWeight {
    One(i32),
    Two(i32, i32, f32),
    Four(IVec4, Vec4),
    Sphere(i32, i32, f32, Vec3, Vec3, Vec3),
    Quat(IVec4, Vec4),
}

#[derive(Copy, Clone)]
pub enum Toon {
    Tex(i32),
    Inner(u8),
}

#[derive(Debug, Copy, Clone)]
pub enum BlendMode {
    Disable,
    Mul,
    Add,
    Other,
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct DrawFlags: u8 {
        const NO_CULL         = 0b00000001;
        const GROUND_SHADOW   = 0b00000010;
        const CAST_SHADOW     = 0b00000100;
        const RECEIVE_SHADOW  = 0b00001000;
        const HAS_EDGE        = 0b00010000;
        const VERTEX_COLOR    = 0b00100000;
        const FILL_MODE_POINT = 0b01000000;
        const FILL_MODE_EDGE  = 0b10000000;
    }
}

#[derive(Clone)]
pub struct Mat {
    pub name: String,
    pub name_en: String,
    pub diffuse: Vec4,
    pub specular: Vec4,
    pub ambient: Vec3,
    pub draw_flag: DrawFlags,
    pub edge_color: Vec4,
    pub edge_scale: f32,
    pub tex_index: i32,
    pub env_index: i32,
    pub env_blend_mode: BlendMode,
    pub toon: Toon,
    pub comment: String,
    pub associated_face_count: u32,
}
impl Default for Mat {
    fn default() -> Self {
        Self {
            name: "Mat".to_string(),
            name_en: "Mat".to_string(),
            diffuse: vec4(0.4, 0.4, 0.4, 1.0),
            specular: vec4(0.4, 0.4, 0.4, 5.0),
            ambient: Vec3::splat(0.2),
            draw_flag: DrawFlags::NO_CULL,
            edge_color: vec4(0.0, 0.0, 0.0, 1.0), 
            edge_scale: 1.0,
            tex_index: -1,
            env_index: -1,
            env_blend_mode: BlendMode::Disable,
            toon: Toon::Tex(-1),
            comment: Default::default(),
            associated_face_count: 0,
        }
    }
}


bitflags! {
    #[derive(Clone, Copy)]
    pub struct BoneFlags: u16 {
        const INDEXED_TAIL_BONE     = 0b0000000000000001;
        const ROTATABLE            = 0b0000000000000010;
        const TRANSLATABLE         = 0b0000000000000100;
        const VISIBLE              = 0b0000000000001000;
        const ENABLED              = 0b0000000000010000;
        const IK                   = 0b0000000000100000;
        const INHERIT_ROTATION     = 0b0000000100000000;
        const INHERIT_TRANSLATION  = 0b0000001000000000;
        const FIXED_AXIS           = 0b0000010000000000;
        const LOCAL_AXIS           = 0b0000100000000000;
        const PHYSICS_AFTER_DEFORM = 0b0001000000000000;
        const EXTERNAL_PARENT      = 0b0010000000000000;
    }
}

#[derive(Copy, Clone)]
pub enum BoneTailPos {
    Bone(i32),
    Pos(Vec3),
}

#[derive(Clone)]
pub struct Bone {
    pub name: String,
    pub name_en: String,
    pub pos: Vec3,
    pub parent_index: Option<usize>,
    pub layer: i32,
    pub bone_flags: BoneFlags,
    pub bone_tail_pos: BoneTailPos,
    pub inherit: Option<(i32, f32)>,
    pub fixed_axis: Option<Vec3>,
    pub local_axis: Option<(Vec3, Vec3)>,
    pub external_parent: Option<i32>,
}

impl Default for Bone {
    fn default() -> Self {
        Self {
            name: "センター".to_string(),
            name_en: "center".to_string(),
            pos: Default::default(),
            parent_index: Default::default(),
            layer: Default::default(),
            bone_flags: BoneFlags::empty(),
            bone_tail_pos: BoneTailPos::Pos(Vec3::ZERO),
            inherit: Default::default(),
            fixed_axis: Default::default(),
            local_axis: Default::default(),
            external_parent: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct Ik {
    pub bone: i32,
    pub effector: i32,
    pub loop_count: i32,
    pub limit_angle: f32,
    pub ik_joints: Vec<IkJoint>,
}

#[derive(Copy, Clone)]
pub struct IkJoint {
    pub bone: i32,
    pub limit: Option<(Vec3, Vec3)>,
}

#[derive(Clone)]
pub struct Joint {
    pub name: String,
    pub name_en: String,
    pub category: u8,
    pub rigidbody_a: i32,
    pub rigidbody_b: i32,
    pub pos: Vec3,
    pub rot: Vec3,
    pub pos_min: Vec3,
    pub pos_max: Vec3,
    pub rot_min: Vec3,
    pub rot_max: Vec3,
    pub pos_spring: Vec3,
    pub rot_spring: Vec3,
    pub uuid: Uuid,
}

#[derive(Copy, Clone)]
pub enum RigidbodyShape {
    Shpere,
    Box,
    Capsule,
}

#[derive(Copy, Clone)]
pub enum RigidbodyMode {
    Kinematics,
    Dynamics,
    DynamicsPassRotation,
}

#[derive(Clone)]
pub struct Rigidbody {
    pub name: String,
    pub name_en: String,
    pub bone: i32,
    pub group: u8,
    pub collision_group: u16,
    pub shape: RigidbodyShape,
    pub size: Vec3,
    pub pos: Vec3,
    pub rot: Vec3,
    pub mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub restitution: f32,
    pub friction: f32,
    pub mode: RigidbodyMode,
    pub uuid: Uuid,
}

#[derive(Copy, Clone)]
pub enum DisplayFrameIndex {
    Bone(u32),
    Morph(u32),
}

#[derive(Clone)]
pub struct DisplayFrame {
    pub name: String,
    pub name_en: String,
    pub deletable: bool,
    pub morph_items: Vec<DisplayFrameIndex>,
}

#[derive(Clone)]
pub struct MorphInfo {
    pub name: String,
    pub name_en: String,
    pub panel: i8,
    pub category: i8,
    pub data: Morph,
}

#[derive(Clone)]
pub enum Morph {
    MorphGroup(Vec<MorphGroupItem>),
    MorphFlip(Vec<MorphFlipItem>),
    MorphVertex(Vec<MorphVertexItem>),
    MorphBone(Vec<MorphBoneItem>),
    MorphUv(Vec<MorphUvItem>),
    MorphRigidbody(Vec<MorphRigidbodyItem>),
    MorphMat(Vec<MorphMatItem>),
}

#[derive(Copy, Clone)]
pub struct MorphGroupItem {
    pub index: u32,
    pub affect: f32,
}

#[derive(Copy, Clone)]
pub struct MorphFlipItem {
    pub index: u32,
    pub affect: f32,
}

#[derive(Copy, Clone)]
pub struct MorphVertexItem {
    pub index: u32,
    pub trans: Vec3,
}

#[derive(Copy, Clone)]
pub struct MorphBoneItem {
    pub index: u32,
    pub trans: Vec3,
    pub rot: Quat,
}

#[derive(Copy, Clone)]
pub struct MorphUvItem {
    pub index: u32,
    pub trans: Vec4,
}

#[derive(Copy, Clone)]
pub struct MorphRigidbodyItem {
    pub index: u32,
    pub local: bool,
    pub trans_speed: Vec3,
    pub rot_torque: Vec3,
}

#[derive(Copy, Clone)]
pub struct MorphMatItem {
    pub index: u32,
    pub blend_mode: BlendMode,
    pub diffuse: Vec4,
    pub specular: Vec3,
    pub specularity: f32,
    pub ambient: Vec3,
    pub edge_color: Vec4,
    pub edge_size: f32,
    pub texture_tint: Vec4,
    pub environment_tint: Vec4,
    pub toon_tint: Vec4,
}

impl Pmx {
    pub fn delete_mats(&mut self, mats: &Vec<String>) {
        let mut new_mats = Vec::new();
        let mut new_faces: Vec<[u32; 3]> = Vec::new();
        let mut start: usize = 0;
        for mat in &self.mats {
            if mats.contains(&mat.name) == false {
                new_mats.push(mat.clone());
                new_faces.extend_from_slice(&self.faces[start..(start + mat.associated_face_count as usize)]);
            }
            start = mat.associated_face_count as _;
        }
        self.mats = new_mats;
        self.faces = new_faces;
        self.delete_unref_point();
    }
    pub fn delete_unref_point(&mut self) {
        let mut ref_set = BTreeSet::new();
        for [x, y, z] in &self.faces {
            ref_set.insert(x);
            ref_set.insert(y);
            ref_set.insert(z);
        }
        let mut new_verts = Vec::new();
        let mut mapping: BTreeMap<u32, u32> = BTreeMap::new();
        for (new_index, old_index) in ref_set.iter().enumerate() {
            new_verts.push(self.verts[**old_index as usize]);
            mapping.insert(**old_index, new_index as _);
        }
        self.verts = new_verts;
        let mut new_faces: Vec<[u32; 3]> = Vec::new();
        for [x, y, z] in &self.faces {
            new_faces.push([
                mapping[x],
                mapping[y],
                mapping[z],
            ]);
        }
        self.faces = new_faces;
    }
    pub fn calc_connected_nrms_to_uv1(&mut self) {
        let mut mapping = Vec::new();
        let mut cache: BTreeMap<u128, usize> = BTreeMap::new();
        for v in &self.verts {
            unsafe {
                let mut tmp = 0u128;
                std::ptr::copy(&v.pos, &mut tmp as *mut u128 as *mut Vec3, 1);
                let n = if let Some(n) = cache.get(&tmp) {
                    *n
                } else {
                    let n = cache.len();
                    cache.insert(tmp, n);
                    n
                };
                mapping.push(n);
            }
        }
        let mut face_nrms = Vec::new();
        let mut vert_associate_face: Vec<HashSet<usize>> = vec![HashSet::new(); cache.len()];
        for (i, [v0, v1, v2]) in self.faces.iter().enumerate() {
            vert_associate_face[mapping[*v0 as usize]].insert(i);
            vert_associate_face[mapping[*v1 as usize]].insert(i);
            vert_associate_face[mapping[*v2 as usize]].insert(i);
            let a = self.verts[*v0 as usize].pos;
            let b = self.verts[*v1 as usize].pos;
            let c = self.verts[*v2 as usize].pos;
            let ab = (b - a).normalize();
            let ac = (c - a).normalize();
            let face_nrm = ab.cross(ac);
            face_nrms.push(face_nrm);
        }
        let mut vert_nrms = Vec::new();
        for s in vert_associate_face {
            let mut nrm = glam::Vec3::ZERO;
            for i in s {
                nrm += face_nrms[i];
            }
            vert_nrms.push(nrm.normalize());
        }
        let mut con_nrms = Vec::with_capacity(self.verts.len());
        for (i, v) in self.verts.iter_mut().enumerate() {
            let nrm = vert_nrms[mapping[i]];
            con_nrms.push(vec4(nrm.x, nrm.y, -nrm.z, 0.0));
        }
        if self.appendix_uvs.len() == 0 {
            self.appendix_uvs.push(Vec::new());
        }
        self.appendix_uvs[0] = con_nrms;
    }
    pub fn load_tex(&self) -> HashMap<i32, RgbaImage> {
        let mut res: HashMap<i32, RgbaImage> = HashMap::new();
        for (i, tex) in self.texs.iter().enumerate() {
            let tex_path = Path::new(&self.path).parent().unwrap().join(tex);
            if let Ok(mut reader) = ImageReader::open(tex_path) {
                let tl = tex.to_lowercase();
                if tl.ends_with("sph") || tl.ends_with("spa") {
                    reader.set_format(ImageFormat::Bmp);
                }
                let img = reader.decode().unwrap().into_rgba8();
                res.insert(i as _, img);
            } else {
                eprintln!("tex {}: {} miss", i, tex);
            }
        }
        let default_image = RgbaImage::from_pixel(64, 64, Rgba::<u8>([255; 4]));
        res.insert(-1, default_image);
        res
    }
    fn read_string(file: &mut Cursor<Vec<u8>>, utf8: bool) -> String {
        let len = file.read_i32::<LE>().unwrap() as usize;
        if len == 0 {
            return String::new();
        };
        let mut content = vec![0u8; len];
        file.read(&mut content).unwrap();
        if utf8 {
            String::from_utf8(content).unwrap()
        } else {
            String::from_utf16(bytemuck::cast_slice_mut(&mut content)).unwrap()
        }
    }
    pub fn read_with_preset(content: Vec<u8>, path: &str) -> Self {
        let mut pmx = Self::read(content, path);
        pmx.reverse_ik_joints();
        pmx.linear_four_weight();
        pmx.scale(0.08);
        pmx.right_hand();
        pmx
    }

    pub fn read(content: Vec<u8>, path: &str) -> Self {
        let file = &mut std::io::Cursor::new(content);
        let mut magic = vec![0u8; 3];
        file.read(&mut magic).unwrap();
        file.read_u8().unwrap();
        assert_eq!(String::from_utf8(magic), Ok("PMX".to_string()));
        let version = file.read_f32::<LE>().unwrap();
        file.read_u8().unwrap();
        let utf8 = file.read_u8().unwrap() == 1;
        let appendix_uv = file.read_u8().unwrap();
        let vertex_index_size = file.read_u8().unwrap();
        let texture_index_size = file.read_u8().unwrap();
        let material_index_size = file.read_u8().unwrap();
        let bone_index_size = file.read_u8().unwrap();
        let morph_index_size = file.read_u8().unwrap();
        let rigidbody_index_size = file.read_u8().unwrap();
        let name = Pmx::read_string(file, utf8);
        let name_en = Pmx::read_string(file, utf8);
        let comment = Pmx::read_string(file, utf8);
        let comment_en = Pmx::read_string(file, utf8);
        let (verts, appendix_uvs) = Pmx::read_verts(file, bone_index_size, appendix_uv);
        let faces = Pmx::read_faces(file, vertex_index_size);
        let texs = Pmx::read_texs(file, utf8);
        let mats = Pmx::read_mats(file, utf8, texture_index_size);
        let (bones, iks) = Pmx::read_bones(file, utf8, bone_index_size);
        let morphs = Pmx::read_morphs(
            file,
            utf8,
            vertex_index_size,
            material_index_size,
            bone_index_size,
            morph_index_size,
            rigidbody_index_size
        );
        let display_frames = Pmx::read_display_frames(file, utf8, bone_index_size, morph_index_size);
        let rigidbodys = Pmx::read_rigidbodys(file, utf8, bone_index_size);
        let joints = Pmx::read_joints(file, utf8, rigidbody_index_size);

        
        Self {
            name,
            name_en,
            comment,
            comment_en,
            verts,
            appendix_uvs,
            faces,
            texs,
            mats,
            bones,
            iks,
            morphs,
            rigidbodys,
            joints,
            path: path.to_string(),
            uuid: Uuid::new_v4(),
            display_frames,
        }

    }

    fn read_mats(file: &mut Cursor<Vec<u8>>, utf8: bool, texture_index_size: u8) -> Vec<Mat> {
        let len = file.read_u32::<LE>().unwrap();
        let mut vct = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let name = Pmx::read_string(file, utf8);
            let name_en = Pmx::read_string(file, utf8);
            let diffuse = read_float4(file);
            let specular = read_float4(file);
            let ambient = read_float3(file);
            let draw_flag = DrawFlags::from_bits(file.read_u8().unwrap()).unwrap();
            let edge_color = read_float4(file);
            let edge_scale = file.read_f32::<LE>().unwrap();
            let tex_index = Pmx::read_int(file, texture_index_size);
            let env_index = Pmx::read_int(file, texture_index_size);
            let env_blend_mode = match file.read_u8().unwrap() {
                0 => BlendMode::Disable,
                1 => BlendMode::Mul,
                2 => BlendMode::Add,
                3 => BlendMode::Other,
                _ => unreachable!(),
            };
            let toon_ref = file.read_u8().unwrap();
            let toon = if toon_ref == 0 {
                Toon::Tex(Pmx::read_int(file, texture_index_size))
            } else {
                Toon::Inner(file.read_u8().unwrap())
            };
            let comment = Pmx::read_string(file, utf8);
            let associated_face_count = Pmx::read_int(file, 4) as u32 / 3;
            vct.push(Mat {
                name,
                name_en,
                diffuse,
                specular,
                ambient,
                draw_flag,
                edge_color,
                edge_scale,
                tex_index,
                env_index,
                env_blend_mode,
                toon,
                comment,
                associated_face_count,
            })
        }
        vct
    }

    fn read_bones(file: &mut Cursor<Vec<u8>>, utf8: bool, bone_index_size: u8) -> (Vec<Bone>, Vec<Ik>) {
        let len = file.read_u32::<LE>().unwrap();
        let mut vct = Vec::with_capacity(len as usize);
        let mut iks = Vec::with_capacity(len as usize);
        for i in 0..len {
            let name = Pmx::read_string(file, utf8);
            let name_en = Pmx::read_string(file, utf8);
            let pos = read_float3(file);
            let parent_index = Pmx::read_int(file, bone_index_size);
            let parent_index = if parent_index >= 0 {
                Some(parent_index as usize)
            } else {
                None
            };
            let layer = file.read_i32::<LE>().unwrap();
            let bone_flags = BoneFlags::from_bits(file.read_u16::<LE>().unwrap()).unwrap();
            let bone_tail_pos = if bone_flags.contains(BoneFlags::INDEXED_TAIL_BONE) {
                BoneTailPos::Bone(Pmx::read_int(file, bone_index_size))
            } else {
                BoneTailPos::Pos(read_float3(file))
            };
            let inherit = if bone_flags.contains(BoneFlags::INHERIT_ROTATION) || bone_flags.contains(BoneFlags::INHERIT_TRANSLATION) {
                let parent_index = Pmx::read_int(file, bone_index_size);
                let affect = file.read_f32::<LE>().unwrap();
                Some((parent_index, affect))
            } else {
                None
            };
            let fixed_axis = if bone_flags.contains(BoneFlags::FIXED_AXIS) {
                Some(read_float3(file))
            } else {
                None
            };
            let local_axis = if bone_flags.contains(BoneFlags::LOCAL_AXIS) {
                Some((read_float3(file), read_float3(file)))
            } else {
                None
            };
            let external_parent = if bone_flags.contains(BoneFlags::EXTERNAL_PARENT) {
                Some(Pmx::read_int(file, bone_index_size))
            } else {
                None
            };
            if bone_flags.contains(BoneFlags::IK) {
                let effector = Pmx::read_int(file, bone_index_size);
                let loop_count = file.read_i32::<LE>().unwrap();
                let limit_angle = file.read_f32::<LE>().unwrap();
                let link_count = file.read_i32::<LE>().unwrap();
                let mut ik_joints = Vec::new();
                for i in 0..link_count {
                    let bone = Pmx::read_int(file, bone_index_size);
                    let limit = if file.read_u8().unwrap() == 1 {
                        let limit_min = read_float3(file);
                        let limit_max = read_float3(file);
                        Some((limit_min, limit_max))
                    } else {
                        None
                    };
                    ik_joints.push(IkJoint {
                        bone,
                        limit,
                    });
                }
                iks.push(Ik {
                    bone: i as i32,
                    effector,
                    loop_count,
                    limit_angle,
                    ik_joints,
                });
            }
            vct.push(Bone {
                name,
                name_en,
                pos,
                parent_index,
                layer,
                bone_flags,
                bone_tail_pos,
                inherit,
                fixed_axis,
                local_axis,
                external_parent,
            })
        }
        (vct, iks)
    }

    fn read_texs(file: &mut Cursor<Vec<u8>>, utf8: bool) -> Vec<String> {
        let len = file.read_u32::<LE>().unwrap();
        let mut vct = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let tex = Pmx::read_string(file, utf8);
            vct.push(tex)
        }
        vct
    }
    fn read_joints(file: &mut Cursor<Vec<u8>>, utf8: bool, rigidbody_index_size: u8) -> Vec<Joint> {
        let len = file.read_u32::<LE>().unwrap();
        let mut vct = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let name = Pmx::read_string(file, utf8);
            let name_en = Pmx::read_string(file, utf8);
            let category = file.read_u8().unwrap();
            assert_eq!(category, 0);
            let rigidbody_a = Pmx::read_int(file, rigidbody_index_size);
            let rigidbody_b = Pmx::read_int(file, rigidbody_index_size);
            let pos = read_float3(file);
            let rot = read_float3(file);
            let pos_min = read_float3(file);
            let pos_max = read_float3(file);
            let rot_min = read_float3(file);
            let rot_max = read_float3(file);
            let pos_spring = read_float3(file);
            let rot_spring = read_float3(file);
            vct.push(Joint {
                name,
                name_en,
                category,
                rigidbody_a,
                rigidbody_b,
                pos,
                rot,
                pos_min,
                pos_max,
                rot_min,
                rot_max,
                pos_spring,
                rot_spring,
                uuid: Uuid::new_v4(),
            });
        }
        vct
    }

    fn read_rigidbodys(file: &mut Cursor<Vec<u8>>, utf8: bool, bone_index_size: u8) -> Vec<Rigidbody> {
        let len = file.read_u32::<LE>().unwrap();
        let mut vct = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let name = Pmx::read_string(file, utf8);
            let name_en = Pmx::read_string(file, utf8);
            let bone = Pmx::read_int(file, bone_index_size);
            let group = file.read_u8().unwrap();
            let collision_group = file.read_u16::<LE>().unwrap();
            let shape = match file.read_u8().unwrap() {
                0 => RigidbodyShape::Shpere,
                1 => RigidbodyShape::Box,
                2 => RigidbodyShape::Capsule,
                _ => unreachable!(),
            };
            let size = read_float3(file);
            let pos = read_float3(file);
            let rot = read_float3(file);
            let mass = file.read_f32::<LE>().unwrap();
            let linear_damping = file.read_f32::<LE>().unwrap();
            let angular_damping = file.read_f32::<LE>().unwrap();
            let restitution = file.read_f32::<LE>().unwrap();
            let friction = file.read_f32::<LE>().unwrap();
            let mode = match file.read_u8().unwrap() {
                0 => RigidbodyMode::Kinematics,
                1 => RigidbodyMode::Dynamics,
                2 => RigidbodyMode::DynamicsPassRotation,
                _ => unreachable!(),
            };
            vct.push(Rigidbody {
                name,
                name_en,
                bone,
                group,
                collision_group,
                shape,
                size,
                pos,
                rot,
                mass,
                linear_damping,
                angular_damping,
                restitution,
                friction,
                mode,
                uuid: Uuid::new_v4(),
            });
        }
        vct
    }

    fn read_display_frames(file: &mut Cursor<Vec<u8>>, utf8: bool, bone_index_size: u8, morph_index_size: u8) -> Vec<DisplayFrame> {
        let len = file.read_u32::<LE>().unwrap();
        let mut vct = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let name = Pmx::read_string(file, utf8);
            let name_en = Pmx::read_string(file, utf8);
            let deletable = file.read_i8().unwrap() == 1;
            let frame_count = file.read_i32::<LE>().unwrap();
            let mut morph_items = Vec::new();
            for __ in 0..frame_count {
                let is_morph_frame = file.read_u8().unwrap() == 1;
                morph_items.push(if is_morph_frame {
                    DisplayFrameIndex::Morph(Pmx::read_int(file, morph_index_size) as u32)
                } else {
                    DisplayFrameIndex::Bone(Pmx::read_int(file, bone_index_size) as u32)
                });
            }
            vct.push(DisplayFrame {
                name,
                name_en,
                deletable,
                morph_items,
            });
        }
        vct
    }

    fn read_morphs(
        file: &mut Cursor<Vec<u8>>, 
        utf8: bool,
        vertex_index_size: u8,
        material_index_size: u8,
        bone_index_size: u8,
        morph_index_size: u8,
        rigidbody_index_size: u8
    ) -> Vec<MorphInfo> {
        let len = file.read_u32::<LE>().unwrap();
        let mut vct = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let name = Pmx::read_string(file, utf8);
            let name_en = Pmx::read_string(file, utf8);
            let panel = file.read_i8().unwrap();
            let category = file.read_i8().unwrap();
            let count = file.read_i32::<LE>().unwrap();
            if category == 0 {
                let mut v = Vec::new();
                for __ in 0..count {
                    let index = Pmx::read_int(file, morph_index_size) as u32;
                    let affect = file.read_f32::<LE>().unwrap();
                    v.push(MorphGroupItem {
                        index,
                        affect,
                    });
                }
                vct.push(MorphInfo {
                    name,
                    name_en,
                    panel,
                    category,
                    data: Morph::MorphGroup(v),
                });
            } else if category == 1 {
                let mut v = Vec::new();
                for __ in 0..count {
                    let index = Pmx::read_uint(file, vertex_index_size) as u32;
                    let trans = read_float3(file);
                    v.push(MorphVertexItem {
                        index,
                        trans,
                    });
                }
                vct.push(MorphInfo {
                    name,
                    name_en,
                    panel,
                    category,
                    data: Morph::MorphVertex(v),
                });
            } else if category == 2 {
                let mut v = Vec::new();
                for __ in 0..count {
                    let index = Pmx::read_int(file, bone_index_size) as u32;
                    let trans = read_float3(file);
                    let rot = read_quat(file);
                    v.push(MorphBoneItem {
                        index,
                        trans,
                        rot,
                    })
                }
                vct.push(MorphInfo {
                    name,
                    name_en,
                    panel,
                    category,
                    data: Morph::MorphBone(v),
                });
            } else if category == 3 {
                let mut v = Vec::new();
                for __ in 0..count {
                    let index = Pmx::read_uint(file, vertex_index_size) as u32;
                    let trans = read_float4(file);
                    v.push(MorphUvItem {
                        index,
                        trans,
                    })
                }
                vct.push(MorphInfo {
                    name,
                    name_en,
                    panel,
                    category,
                    data: Morph::MorphUv(v),
                });
            } else if category == 4 || category == 5 || category == 6 || category == 7 {
                unreachable!()
            } else if category == 8 {
                let mut v = Vec::new();
                for __ in 0..count {
                    let index = Pmx::read_int(file, material_index_size) as u32;
                    let blend_mode = match file.read_u8().unwrap() {
                        0 => BlendMode::Mul, 
                        1 => BlendMode::Add, 
                        _ => unreachable!(),
                    };
                    let diffuse = read_float4(file);
                    let specular = read_float3(file);
                    let specularity = file.read_f32::<LE>().unwrap();
                    let ambient = read_float3(file);
                    let edge_color = read_float4(file);
                    let edge_size = file.read_f32::<LE>().unwrap();
                    let texture_tint = read_float4(file);
                    let environment_tint = read_float4(file);
                    let toon_tint = read_float4(file);
                    v.push(MorphMatItem {
                        index,
                        blend_mode,
                        diffuse,
                        specular,
                        specularity,
                        ambient,
                        edge_color,
                        edge_size,
                        texture_tint,
                        environment_tint,
                        toon_tint,
                    });
                }
                vct.push(MorphInfo {
                    name,
                    name_en,
                    panel,
                    category,
                    data: Morph::MorphMat(v),
                });
            } else if category == 9 {
                let mut v = Vec::new();
                for __ in 0..count {
                    let index = Pmx::read_int(file, morph_index_size) as u32;
                    let affect = file.read_f32::<LE>().unwrap();
                    v.push(MorphFlipItem {
                        index,
                        affect,
                    })
                }
                vct.push(MorphInfo {
                    name,
                    name_en,
                    panel,
                    category,
                    data: Morph::MorphFlip(v),
                });
            } else if category == 10 {
                let mut v = Vec::new();
                for __ in 0..count {
                    let index = Pmx::read_int(file, rigidbody_index_size) as u32;
                    let local = file.read_u8().unwrap() == 1;
                    let trans_speed = read_float3(file);
                    let rot_torque = read_float3(file);
                    v.push(MorphRigidbodyItem {
                        index,
                        local,
                        trans_speed,
                        rot_torque,
                    });
                }
                vct.push(MorphInfo {
                    name,
                    name_en,
                    panel,
                    category,
                    data: Morph::MorphRigidbody(v),
                });
            }
        }
        vct
    }

    fn read_faces(file: &mut Cursor<Vec<u8>>, vertex_index_size: u8) -> Vec<[u32; 3]> {
        let len = file.read_u32::<LE>().unwrap() / 3;
        let mut vct = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let a = Pmx::read_uint(file, vertex_index_size) as u32;
            let b = Pmx::read_uint(file, vertex_index_size) as u32;
            let c = Pmx::read_uint(file, vertex_index_size) as u32;
            vct.push([a, b, c])
        }
        vct
    }
    fn read_verts(file: &mut Cursor<Vec<u8>>, bone_index_size: u8, appendix_uv: u8) -> (Vec<Vertex>, Vec<Vec<Vec4>>) {
        let len = file.read_u32::<LE>().unwrap();
        let mut appendix_uvs: Vec<Vec<Vec4>> = vec![Vec::with_capacity(len as usize); appendix_uv as usize];
        let mut vct = Vec::with_capacity(len as usize);
        for i in 0..len {
            let pos = read_float3(file);
            let nrm = read_float3(file);
            let uv = read_float2(file);
            for j in 0..appendix_uv {
                appendix_uvs[j as usize].push(read_float4(file));
            }

            let weight_type = file.read_u8().unwrap();
            let weight = if weight_type == 0 {
                let a = Pmx::read_int(file, bone_index_size);
                VertexWeight::One(a)
            } else if weight_type == 1 {
                let a = Pmx::read_int(file, bone_index_size);
                let b = Pmx::read_int(file, bone_index_size);
                let weight = file.read_f32::<LE>().unwrap();
                VertexWeight::Two(a, b, weight)
            } else if weight_type == 2 {
                let a = Pmx::read_int(file, bone_index_size);
                let b = Pmx::read_int(file, bone_index_size);
                let c = Pmx::read_int(file, bone_index_size);
                let d = Pmx::read_int(file, bone_index_size);
                let index = ivec4(a, b, c, d);
                let weight = read_float4(file);
                VertexWeight::Four(index, weight)
            } else if weight_type == 3 {
                let a = Pmx::read_int(file, bone_index_size);
                let b = Pmx::read_int(file, bone_index_size);
                let weight = file.read_f32::<LE>().unwrap();
                let c = read_float3(file);
                let r0 = read_float3(file);
                let r1 = read_float3(file);
                VertexWeight::Sphere(a, b, weight, c, r0, r1)
            } else if weight_type == 4 {
                let a = Pmx::read_int(file, bone_index_size);
                let b = Pmx::read_int(file, bone_index_size);
                let c = Pmx::read_int(file, bone_index_size);
                let d = Pmx::read_int(file, bone_index_size);
                let index = ivec4(a, b, c, d);
                let weight = read_float4(file);
                VertexWeight::Quat(index, weight)
            } else {
                unreachable!()
            };
            let edge_scale = file.read_f32::<LE>().unwrap();
            vct.push(Vertex {
                pos,
                nrm,
                uv,
                weight,
                edge_scale,
            })
        }
        (vct, appendix_uvs)
    }
    

    fn read_int(file: &mut Cursor<Vec<u8>>, index_size: u8) -> i32 {
        match index_size {
            1 => file.read_i8().unwrap() as i32,
            2 => file.read_i16::<LE>().unwrap() as i32,
            4 => file.read_i32::<LE>().unwrap(),
            _ => unreachable!(),
        }
    }

    fn read_uint(file: &mut Cursor<Vec<u8>>, index_size: u8) -> i32 {
        match index_size {
            1 => file.read_u8().unwrap() as i32,
            2 => file.read_u16::<LE>().unwrap() as i32,
            4 => file.read_i32::<LE>().unwrap(),
            _ => unreachable!(),
        }
    }

    pub fn scale(&mut self, scale: f32) {
        for v in &mut self.verts {
            v.pos *= scale;
        }
        for b in &mut self.bones {
            b.pos *= scale;
            if let BoneTailPos::Pos(pos) = b.bone_tail_pos {
                b.bone_tail_pos = BoneTailPos::Pos(pos * scale);
            }
            if let Some(axis) = b.fixed_axis {
                b.fixed_axis = Some(axis * scale);
            }
            if let Some((x, z)) = b.local_axis {
                b.local_axis = Some((x * scale, z * scale));
            }
        }
        for r in &mut self.rigidbodys {
            r.size *= scale;
            r.pos *= scale;
        }
        for j in &mut self.joints {
            j.pos *= scale;
            j.pos_min *= scale;
            j.pos_max *= scale;
        }
    }

    pub fn linear_four_weight(&mut self) {
        for v in &mut self.verts {
            v.weight = match v.weight {
                VertexWeight::One(i) => {
                    VertexWeight::Four(
                        ivec4(i, -1, -1, -1),
                        vec4(1.0, 0.0, 0.0, 0.0)
                    )
                },
                VertexWeight::Two(i0, i1, w) => {
                    if w == 0.0 {
                        VertexWeight::Four(
                            ivec4(i1, -1, -1, -1),
                            vec4(1.0, 0.0, 0.0, 0.0)
                        )
                    } else if w == 1.0 {
                        VertexWeight::Four(
                            ivec4(i0, -1, -1, -1),
                            vec4(1.0, 0.0, 0.0, 0.0)
                        )
                    } else {
                        VertexWeight::Four(
                            ivec4(i0, i1, -1, -1),
                            vec4(w, 1.0 - w, 0.0, 0.0)
                        )
                    }
                },
                VertexWeight::Sphere(i0, i1, w, _, _, _) => {
                    if w == 0.0 {
                        VertexWeight::Four(
                            ivec4(i1, -1, -1, -1),
                            vec4(1.0, 0.0, 0.0, 0.0)
                        )
                    } else if w == 1.0 {
                        VertexWeight::Four(
                            ivec4(i0, -1, -1, -1),
                            vec4(1.0, 0.0, 0.0, 0.0)
                        )
                    } else {
                        VertexWeight::Four(
                            ivec4(i0, i1, -1, -1),
                            vec4(w, 1.0 - w, 0.0, 0.0)
                        )
                    }
                },
                VertexWeight::Quat(i, w) => {
                    VertexWeight::Four(i, w)
                },
                VertexWeight::Four(i, w) => {
                    let mut i = i;
                    for j in 0..4 {
                        if w[j] == 0.0 {
                            i[j] = -1;
                        }
                    }
                    VertexWeight::Four(i, w)
                },
            }
        }
    }
    pub fn right_hand(&mut self) {
        for v in &mut self.verts {
            v.pos.z *= -1.0;
            v.nrm.z *= -1.0;
        }
        for f in &mut self.faces {
            f.swap(1, 2);
        }
        for b in &mut self.bones {
            b.pos.z *= -1.0;
            if let BoneTailPos::Pos(ref mut pos) = b.bone_tail_pos {
                pos.z *= -1.0;
            }
            if let Some(ref mut axis) = b.fixed_axis {
                axis.z *= -1.0;
            }
            if let Some((ref mut x, ref mut z)) = b.local_axis {
                x.z *= -1.0;
                z.z *= -1.0;
            }
        }
        for r in &mut self.rigidbodys {
            r.pos.z *= -1.0;
            r.rot.x *= -1.0;
            r.rot.y *= -1.0;
        }
        for j in &mut self.joints {
            j.pos.z *= -1.0;
            j.rot.x *= -1.0;
            j.rot.y *= -1.0;
        }
    }
    pub fn flip_uv(&mut self) {
        for v in &mut self.verts {
            v.uv.y = 1.0 - v.uv.y;
        }
    }
    pub fn reverse_ik_joints(&mut self) {
        for ik in &mut self.iks {
            ik.ik_joints.reverse();
        }
    }
    pub fn check_missing_bones(&self, vmd_bone_names: &BTreeSet<String>) -> BTreeSet<String> {
        let mut bone_names: BTreeSet<String> = BTreeSet::new();
        for b in &self.bones {
            bone_names.insert(b.name.clone());
        }

        let mut res = BTreeSet::new();
        for n in vmd_bone_names {
            if !bone_names.contains(n) {
                res.insert(n.clone());
            }
        }
        res
    }
    pub fn check_missing_morphs(&self, vmd_morph_names: &BTreeSet<String>) -> BTreeSet<String> {
        let mut morph_names: BTreeSet<String> = BTreeSet::new();
        for m in &self.morphs {
            morph_names.insert(m.name.clone());
        }

        let mut res = BTreeSet::new();
        for n in vmd_morph_names {
            if !morph_names.contains(n) {
                res.insert(n.clone());
            }
        }
        res
    }
}
