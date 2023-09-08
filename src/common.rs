#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write, Cursor};
use glam::*;

pub fn read_float2<T>(file: &mut T) -> Vec2
    where T: Read {
    let x = file.read_f32::<LittleEndian>().unwrap();
    let y = file.read_f32::<LittleEndian>().unwrap();
    vec2(x, y)
}
pub fn write_float2<T>(file: &mut T, v: Vec2)
    where T: Write {
    file.write_f32::<LittleEndian>(v.x).unwrap();
    file.write_f32::<LittleEndian>(v.y).unwrap();
}

pub fn read_float3<T>(file: &mut T) -> Vec3
    where T: Read {
    let x = file.read_f32::<LittleEndian>().unwrap();
    let y = file.read_f32::<LittleEndian>().unwrap();
    let z = file.read_f32::<LittleEndian>().unwrap();
    vec3(x, y, z)
}
pub fn write_float3<T>(file: &mut T, v: Vec3)
    where T: Write {
    file.write_f32::<LittleEndian>(v.x).unwrap();
    file.write_f32::<LittleEndian>(v.y).unwrap();
    file.write_f32::<LittleEndian>(v.z).unwrap();
}

pub fn read_float4<T>(file: &mut T) -> Vec4
    where T: Read {
    let x = file.read_f32::<LittleEndian>().unwrap();
    let y = file.read_f32::<LittleEndian>().unwrap();
    let z = file.read_f32::<LittleEndian>().unwrap();
    let w = file.read_f32::<LittleEndian>().unwrap();
    vec4(x, y, z, w)
}

pub fn read_quat<T>(file: &mut T) -> Quat
    where T: Read {
    let x = file.read_f32::<LittleEndian>().unwrap();
    let y = file.read_f32::<LittleEndian>().unwrap();
    let z = file.read_f32::<LittleEndian>().unwrap();
    let w = file.read_f32::<LittleEndian>().unwrap();
    quat(x, y, z, w)
}
pub fn write_float4<T>(file: &mut T, v: Vec4)
    where T: Write {
    file.write_f32::<LittleEndian>(v.x).unwrap();
    file.write_f32::<LittleEndian>(v.y).unwrap();
    file.write_f32::<LittleEndian>(v.z).unwrap();
    file.write_f32::<LittleEndian>(v.w).unwrap();
}

pub fn write_quat<T>(file: &mut T, v: Quat)
    where T: Write {
    file.write_f32::<LittleEndian>(v.x).unwrap();
    file.write_f32::<LittleEndian>(v.y).unwrap();
    file.write_f32::<LittleEndian>(v.z).unwrap();
    file.write_f32::<LittleEndian>(v.w).unwrap();
}

pub fn write_int4<T>(file: &mut T, v: IVec4)
    where T: Write {
    file.write_i32::<LittleEndian>(v.x).unwrap();
    file.write_i32::<LittleEndian>(v.y).unwrap();
    file.write_i32::<LittleEndian>(v.z).unwrap();
    file.write_i32::<LittleEndian>(v.w).unwrap();
}

pub fn read_items<T, F>(file: &mut Cursor<Vec<u8>>, f: F) -> Vec<T>
    where F: Fn(&mut Cursor<Vec<u8>>) -> T {
    match file.read_u32::<LittleEndian>() {
        Ok(count) => {
            let mut items: Vec<T> = Vec::with_capacity(count as usize);
            for _ in 0..count {
                items.push(f(file));
            }
            items
        },
        _ => Vec::new()
    }
}

pub fn read_fix_items<T, F>(file: &mut Cursor<Vec<u8>>, count: usize, f: F) -> Vec<T>
    where F: Fn(&mut Cursor<Vec<u8>>) -> T {
    let mut items: Vec<T> = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(f(file));
    }
    items
}
pub fn write_items<W, T, F>(mut file: &mut W, content: &Vec<T>, f: F)
    where W: Write, F: Fn(&mut W, &T) -> () {
    let count = content.len();
    file.write_u32::<LittleEndian>(count as u32).unwrap();
    for i in 0..count {
        f(&mut file, &content[i]);
    }
}

pub fn benchmark<F, T>(func: F) -> T
    where F: FnOnce() -> T
{
    let s = std::time::Instant::now();
    let r = func();
    let d = s.elapsed();
    println!("{}ms", d.as_millis());
    r
}
