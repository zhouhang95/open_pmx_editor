#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
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
    WINDOWS_31J.decode(&string_raw, DecoderTrap::Ignore).unwrap()
}

pub fn read_bezier_control_point_pair4(file: &mut File) -> [f32; 4] {
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

pub fn read_header(mut file: &mut File) -> String {
    let header_string = read_string(&mut file, 30);

    if header_string.starts_with(VERSION_1) {
        read_string(&mut file, 10)
    } else if header_string.starts_with(VERSION_2) {
        read_string(&mut file, 20)
    } else {
        unreachable!()
    }
}

pub fn read_bone_keyframe(mut file: &mut File) -> BoneKeyframe {
    BoneKeyframe {
        name: read_string(&mut file, 15),
        frame: file.read_u32::<LittleEndian>().unwrap(),
        trans: read_float3(&mut file),
        rot: read_float4(&mut file),
        txc: read_bezier_control_point_pair4(&mut file),
        tyc: read_bezier_control_point_pair4(&mut file),
        tzc: read_bezier_control_point_pair4(&mut file),
        rc:  read_bezier_control_point_pair4(&mut file),
    }
}

pub fn read_camera_keyframe(mut file: &mut File) -> CameraKeyframe {
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

pub fn read_morph_keyframe(mut file: &mut File) -> MorphKeyframe {
    MorphKeyframe {
        name: read_string(&mut file, 15),
        frame: file.read_u32::<LittleEndian>().unwrap(),
        weight: file.read_f32::<LittleEndian>().unwrap(),
    }
}

pub fn read_light_keyframe(file: &mut File) -> LightKeyframe {
    LightKeyframe {
        frame: file.read_u32::<LittleEndian>().unwrap(),
        color: read_float3(file),
        direction: read_float3(file),
    }
}

pub fn read_shadow_keyframe(file: &mut File) -> ShadowKeyframe {
    ShadowKeyframe {
        frame: file.read_u32::<LittleEndian>().unwrap(),
        mode:  file.read_u8().unwrap(),
        dist:  file.read_f32::<LittleEndian>().unwrap(),
    }
}

pub fn read_ik_keyframe(file: &mut File) -> IkKeyframe {
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
        let mut file = File::open(path).unwrap();
        Motion {
            model_name:       read_header(&mut file),
            bone_keyframes:   read_items(&mut file, read_bone_keyframe),
            morph_keyframes:  read_items(&mut file, read_morph_keyframe),
            camera_keyframes: read_items(&mut file, read_camera_keyframe),
            light_keyframes:  read_items(&mut file, read_light_keyframe),
            shadow_keyframes: read_items(&mut file, read_shadow_keyframe),
            ik_keyframes:     read_items(&mut file, read_ik_keyframe),
        }
    }
}