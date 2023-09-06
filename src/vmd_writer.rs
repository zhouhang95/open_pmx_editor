#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::motion::{Motion, ShadowKeyframe, LightKeyframe, CameraKeyframe, MorphKeyframe, BoneKeyframe};
use std::path::Path;
use crate::vmd_reader::VERSION_2;
use encoding::{Encoding, DecoderTrap, EncoderTrap};
use glam::Vec4;
use std::fs;
use std::io::Write;
use byteorder::{WriteBytesExt, LittleEndian};
use encoding::all::WINDOWS_31J;
use crate::common::{write_float3, write_float4, write_items, write_quat};
use std::cmp::max;
use std::collections::HashMap;

pub fn write_bezier_control_point_pair4<T>(file: &mut T, vec: Vec4)
    where T: Write {
    for v in &[vec.x, vec.y, vec.z, vec.w] {
        let v = max((v * 127f32) as i8, 0);
        file.write_i8(v).unwrap();
        file.write_i8(v).unwrap();
        file.write_i8(v).unwrap();
        file.write_i8(v).unwrap();
    }
}

pub fn write_bezier_control_point_pair1<T>(file: &mut T, v: Vec4)
    where T: Write {
    file.write_i8(max((v.x * 127f32) as i8, 0)).unwrap();
    file.write_i8(max((v.y * 127f32) as i8, 0)).unwrap();
    file.write_i8(max((v.z * 127f32) as i8, 0)).unwrap();
    file.write_i8(max((v.w * 127f32) as i8, 0)).unwrap();
}

pub fn write_bone_keyframe<T>(mut file: &mut T, name: &String, keyframe: &BoneKeyframe)
    where T: Write {
    write_string(&mut file, name, 15);
    file.write_u32::<LittleEndian>(keyframe.frame).unwrap();
    write_float3(&mut file, keyframe.trans);
    write_quat(&mut file, keyframe.rot);
    write_bezier_control_point_pair4(&mut file, keyframe.txc);
    write_bezier_control_point_pair4(&mut file, keyframe.tyc);
    write_bezier_control_point_pair4(&mut file, keyframe.tzc);
    write_bezier_control_point_pair4(&mut file, keyframe.rc);
}


pub fn write_string<T>(file: &mut T, content: &String, len: usize)
    where T: Write {
    let mut content_u8: Vec<u8> = Vec::new();
    for c in content.chars() {
        let mut char_u8 = WINDOWS_31J.encode(&c.to_string(), EncoderTrap::Ignore).unwrap();
        if content_u8.len() + char_u8.len() < len {
            content_u8.append(&mut char_u8);
        } else {
            break;
        }
    }

    file.write_all(&content_u8).unwrap();
    file.write_all(&vec![0u8; len - content_u8.len()]).unwrap();
}
pub fn write_item_string_cache<T>(file: &mut T, content: String, cache: &mut HashMap<String, Vec<u8>>)
    where T: Write {
    let mut content_u8: Vec<u8> = Vec::new();
    let len = 15;

    if cache.contains_key(&content) {
        content_u8 = cache[&content].clone();
    } else {
        for c in content.chars() {
            let mut char_u8 = WINDOWS_31J.encode(&c.to_string(), EncoderTrap::Ignore).unwrap();
            if content_u8.len() + char_u8.len() < len {
                content_u8.append(&mut char_u8);
            } else {
                break;
            }
        }
    }

    file.write_all(&content_u8).unwrap();
    file.write_all(&vec![0u8; len - content_u8.len()]).unwrap();
}

pub fn write_camera_keyframe<T>(mut file: &mut T, keyframe: &CameraKeyframe)
    where T: Write {
    file.write_u32::<LittleEndian>(keyframe.frame).unwrap();
    file.write_f32::<LittleEndian>(keyframe.dist).unwrap();

    write_float3(&mut file, keyframe.trans);
    write_float3(&mut file, keyframe.rot);

    write_bezier_control_point_pair1(&mut file, keyframe.txc);
    write_bezier_control_point_pair1(&mut file, keyframe.tyc);
    write_bezier_control_point_pair1(&mut file, keyframe.tzc);
    write_bezier_control_point_pair1(&mut file, keyframe.rc);
    write_bezier_control_point_pair1(&mut file, keyframe.dc);
    write_bezier_control_point_pair1(&mut file, keyframe.vc);

    file.write_u32::<LittleEndian>(keyframe.fov).unwrap();
    file.write_u8(if keyframe.perspective {0} else {1}).unwrap();
}

pub fn write_morph_keyframe<T>(mut file: &mut T, name: &String, keyframe: &MorphKeyframe)
    where T: Write {
    write_string(&mut file, name, 15);
    file.write_u32::<LittleEndian>(keyframe.frame).unwrap();
    file.write_f32::<LittleEndian>(keyframe.weight).unwrap();
}

pub fn write_light_keyframe<T>(mut file: &mut T, keyframe: &LightKeyframe)
    where T: Write {
    file.write_u32::<LittleEndian>(keyframe.frame).unwrap();
    write_float3(&mut file, keyframe.color);
    write_float3(&mut file, keyframe.direction);
}

pub fn write_shadow_keyframe<T>(file: &mut T, keyframe: &ShadowKeyframe)
    where T: Write {
    file.write_u32::<LittleEndian>(keyframe.frame).unwrap();
    file.write_u8(keyframe.mode).unwrap();
    file.write_f32::<LittleEndian>(keyframe.dist).unwrap();
}

impl Motion {
    pub fn write_vmd(&self, path: &String) {
        let mut file = vec![];
        write_string(&mut file, &VERSION_2.to_string(), 30);
        write_string(&mut file, &self.model_name, 20);
        {
            let mut bone_kf_count = 0;
            for (_, list) in &self.bone_keyframes {
                bone_kf_count += list.len();
            }
            file.write_u32::<LittleEndian>(bone_kf_count as u32).unwrap();
            for (name, list) in &self.bone_keyframes {
                for keyframe in list {
                    write_bone_keyframe(&mut file, name, keyframe)
                }
            }
        }
        {
            let mut morph_kf_count = 0;
            for (_, list) in &self.morph_keyframes {
                morph_kf_count += list.len();
            }
            file.write_u32::<LittleEndian>(morph_kf_count as u32).unwrap();
            for (name, list) in &self.morph_keyframes {
                for keyframe in list {
                    write_morph_keyframe(&mut file, name, keyframe)
                }
            }
        }
        write_items(&mut file, &self.camera_keyframes, write_camera_keyframe);
        write_items(&mut file, &self.light_keyframes, write_light_keyframe);
        write_items(&mut file, &self.shadow_keyframes, write_shadow_keyframe);
        fs::write(path, file).unwrap();
    }
}