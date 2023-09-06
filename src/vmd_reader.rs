#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, Cursor};
use std::path::Path;
use encoding::{Encoding, DecoderTrap};
use encoding::all::WINDOWS_31J;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::SeekFrom;

use crate::motion::*;
use crate::common::*;


const VERSION_1: &str = "Vocaloid Motion Data file";
pub const VERSION_2: &str = "Vocaloid Motion Data 0002";

pub fn read_string<T>(file: &mut T, len: usize) -> String
        where T: Read {
    let mut string_raw = vec![0u8; len];
    file.read(&mut string_raw).unwrap();
    read_string_raw(&string_raw)
}

fn read_string_raw(string_raw: &[u8]) -> String {
    WINDOWS_31J.decode(string_raw, DecoderTrap::Ignore).unwrap()
        .split('\0').next().unwrap()
        .to_string()
}

fn read_string_as_u128(file: &mut Cursor<Vec<u8>>) -> u128 {
    let mut name: [u128; 1] = [0];
    let name_ref: &mut [u8] = bytemuck::cast_slice_mut(&mut name);
    file.read(&mut name_ref[0..15]).unwrap();
    name[0]
}

fn case_string_from_u128(string_raw: u128) -> String {
    let k: &[u128; 1] = &[string_raw];
    let name: &[u8] = bytemuck::cast_slice(k);
    read_string_raw(&name[0..15])
}

pub fn read_bezier_control_point_pair4(file: &mut Cursor<Vec<u8>>) -> [f32; 4] {
    let x = (file.read_u32::<LittleEndian>().unwrap() & 0xFF) as f32 / 127f32;
    let y = (file.read_u32::<LittleEndian>().unwrap() & 0xFF) as f32 / 127f32;
    let z = (file.read_u32::<LittleEndian>().unwrap() & 0xFF) as f32 / 127f32;
    let w = (file.read_u32::<LittleEndian>().unwrap() & 0xFF) as f32 / 127f32;
    [x, y, z, w]
}

pub fn read_bezier_control_point_pair1<T>(file: &mut T) -> [f32; 4] 
    where T: Read {
    let x = file.read_u8().unwrap() as f32 / 127f32;
    let y = file.read_u8().unwrap() as f32 / 127f32;
    let z = file.read_u8().unwrap() as f32 / 127f32;
    let w = file.read_u8().unwrap() as f32 / 127f32;
    [x, y, z, w]
}

pub fn read_header(mut file: &mut Cursor<Vec<u8>>) -> String {
    let header_string = read_string(&mut file, 30);

    if header_string.starts_with(VERSION_1) {
        read_string(&mut file, 10)
    } else if header_string.starts_with(VERSION_2) {
        read_string(&mut file, 20)
    } else {
        unreachable!()
    }
}

pub fn read_bone_keyframe(mut file: &mut Cursor<Vec<u8>>) -> (u128, BoneKeyframe) {
    let name = read_string_as_u128(file);
    let keyframe = BoneKeyframe {
        frame: file.read_u32::<LittleEndian>().unwrap(),
        trans: read_float3(&mut file),
        rot: read_float4(&mut file),
        txc: read_bezier_control_point_pair4(&mut file),
        tyc: read_bezier_control_point_pair4(&mut file),
        tzc: read_bezier_control_point_pair4(&mut file),
        rc:  read_bezier_control_point_pair4(&mut file),
    };
    (name, keyframe)
}

pub fn read_camera_keyframe(mut file: &mut Cursor<Vec<u8>>) -> CameraKeyframe {
    CameraKeyframe {
        frame: file.read_u32::<LittleEndian>().unwrap(),
        dist: file.read_f32::<LittleEndian>().unwrap(),
        trans: read_float3(&mut file),
        rot: read_float3(&mut file),
        txc: read_bezier_control_point_pair1(&mut file),
        tyc: read_bezier_control_point_pair1(&mut file),
        tzc: read_bezier_control_point_pair1(&mut file),
        rc:  read_bezier_control_point_pair1(&mut file),
        dc:  read_bezier_control_point_pair1(&mut file),
        vc : read_bezier_control_point_pair1(&mut file),
        fov: file.read_u32::<LittleEndian>().unwrap(),
        perspective: file.read_u8().unwrap() == 0,
    }
}

pub fn read_morph_keyframe(mut file: &mut Cursor<Vec<u8>>) -> (String, MorphKeyframe) {
    let name = read_string(&mut file, 15);
    let keyframe =  MorphKeyframe {
        frame: file.read_u32::<LittleEndian>().unwrap(),
        weight: file.read_f32::<LittleEndian>().unwrap(),
    };
    (name, keyframe)
}

pub fn read_light_keyframe(file: &mut Cursor<Vec<u8>>) -> LightKeyframe {
    LightKeyframe {
        frame: file.read_u32::<LittleEndian>().unwrap(),
        color: read_float3(file),
        direction: read_float3(file),
    }
}

pub fn read_shadow_keyframe(file: &mut Cursor<Vec<u8>>) -> ShadowKeyframe {
    ShadowKeyframe {
        frame: file.read_u32::<LittleEndian>().unwrap(),
        mode:  file.read_u8().unwrap(),
        dist:  file.read_f32::<LittleEndian>().unwrap(),
    }
}

pub fn read_ik_keyframe(file: &mut Cursor<Vec<u8>>) -> IkKeyframe {
    let frame = file.read_u32::<LittleEndian>().unwrap();
    let show = file.read_u8().unwrap() == 0;
    let count = file.read_u32::<LittleEndian>().unwrap() as usize;
    let mut infos = Vec::with_capacity(count);
    for _ in 0..count {
        infos.push((
            read_string(file, 20),
            file.read_u8().unwrap() == 1
        ));
    }

    IkKeyframe {
        frame,
        show,
        infos,
    }
}

impl Motion {
    pub fn read_vmd(path: &Path) -> Motion {
        let content = std::fs::read(path).unwrap();
        let mut file = std::io::Cursor::new(content);

        let model_name = read_header(&mut file);
        let mut bone_keyframes: BTreeMap<String, Vec<BoneKeyframe>> = BTreeMap::new();
        {
            let bone_keyframe_list = read_items(&mut file, read_bone_keyframe);
            let mut bone_keyframes_inner: BTreeMap<u128, Vec<BoneKeyframe>> = BTreeMap::new();

            for (name, kf) in &bone_keyframe_list {
                bone_keyframes_inner.entry(name.clone()).or_insert(vec![]);
                bone_keyframes_inner.get_mut(name).unwrap().push(kf.clone());
            }
            for (string_raw, value) in bone_keyframes_inner {
                let name = case_string_from_u128(string_raw);
                bone_keyframes.insert(name, value);
            }
        }
        let mut morph_keyframes: BTreeMap<String, Vec<MorphKeyframe>> = BTreeMap::new();
        {
            let morph_keyframe_list = read_items(&mut file, read_morph_keyframe);
            for (name, kf) in &morph_keyframe_list {
                morph_keyframes.entry(name.clone()).or_insert(vec![]);
                morph_keyframes.get_mut(name).unwrap().push(kf.clone());
            }
        }
        Motion {
            model_name,
            bone_keyframes,
            morph_keyframes,
            camera_keyframes: read_items(&mut file, read_camera_keyframe),
            light_keyframes:  read_items(&mut file, read_light_keyframe),
            shadow_keyframes: read_items(&mut file, read_shadow_keyframe),
            ik_keyframes:     read_items(&mut file, read_ik_keyframe),
        }
    }
}