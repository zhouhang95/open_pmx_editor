#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::collections::BTreeMap;

pub struct Motion {
    pub model_name:       String,
    pub bone_keyframes:   BTreeMap<String, Vec<BoneKeyframe>>,
    pub morph_keyframes:  BTreeMap<String, Vec<MorphKeyframe>>,
    pub camera_keyframes: Vec<CameraKeyframe>,
    pub light_keyframes:  Vec<LightKeyframe>,
    pub shadow_keyframes: Vec<ShadowKeyframe>,
    pub ik_keyframes:     Vec<IkKeyframe>,
}

#[derive(Clone, Copy)]
pub struct BoneKeyframe {
    pub frame: u32,
    pub trans: [f32; 3],
    pub rot:   [f32; 4],
    pub txc:   [f32; 4],
    pub tyc:   [f32; 4],
    pub tzc:   [f32; 4],
    pub rc:    [f32; 4],
}

impl BoneKeyframe {
    fn empty(&self) -> bool {
        if self.trans != [0.0; 3] {
            return false;
        }
        if self.rot != [0.0, 0.0, 0.0, 1.0] {
            return false;
        }
        return true;
    }
}

#[derive(Copy, Clone)]
pub struct MorphKeyframe {
    pub frame:  u32,
    pub weight: f32,
}

#[derive(Copy, Clone)]
pub struct CameraKeyframe {
    pub frame: u32,
    pub dist:  f32,
    pub trans: [f32; 3],
    pub rot:   [f32; 3],
    pub txc:   [f32; 4],
    pub tyc:   [f32; 4],
    pub tzc:   [f32; 4],
    pub rc :   [f32; 4],
    pub dc :   [f32; 4],
    pub vc :   [f32; 4],
    pub fov:   u32,
    pub perspective: bool,
}


#[derive(Copy, Clone)]
pub struct LightKeyframe {
    pub frame:     u32,
    pub color:     [f32; 3],
    pub direction: [f32; 3],
}

#[derive(Copy, Clone)]
pub struct ShadowKeyframe {
    pub frame: u32,
    pub mode:  u8,
    pub dist:  f32,
}

#[derive(Clone)]
pub struct IkKeyframe {
    pub frame: u32,
    pub show:  bool,
    pub infos:  Vec<(String, bool)>,
}

impl Motion {
    pub fn new() -> Motion {
        Motion {
            model_name:       String::new(),
            bone_keyframes:   BTreeMap::new(),
            morph_keyframes:  BTreeMap::new(),
            camera_keyframes: Vec::new(),
            light_keyframes:  Vec::new(),
            shadow_keyframes: Vec::new(),
            ik_keyframes:     Vec::new(),
        }
    }

    pub fn clear_empty_morph(&self) -> BTreeMap<String, Vec<MorphKeyframe>> {
        let mut keyframes: BTreeMap<String, Vec<MorphKeyframe>> = BTreeMap::new();
        for (k, v) in &self.morph_keyframes {
            let mut not_zero = false;
            for kf in v {
                if kf.weight != 0.0 {
                    not_zero = true;
                    break;
                }
            }
            if not_zero {
                if v.len() >= 3 {
                    let mut need_remove = Vec::new();
                    for i in 1..(v.len()-1) {
                        if v[i-1].weight == v[i].weight && v[i].weight == v[i+1].weight {
                            need_remove.push(i);
                        }
                    }
                    let mut nv = Vec::new();
                    for i in 0..v.len() {
                        if !need_remove.contains(&i) {
                            nv.push(v[i])
                        }
                    }
                    keyframes.insert(k.clone(), nv);
                } else {
                    keyframes.insert(k.clone(), v.clone());
                }
            }
        }
        return keyframes;
    }

    pub fn clear_empty_bone(&self) -> BTreeMap<String, Vec<BoneKeyframe>> {
        let mut keyframes: BTreeMap<String, Vec<BoneKeyframe>> = BTreeMap::new();
        for (k, v) in &self.bone_keyframes {
            let mut not_zero = false;
            for kf in v {
                if !kf.empty() {
                    not_zero = true;
                    break;
                }
            }
            if not_zero {
                keyframes.insert(k.clone(), v.clone());
            }
        }
        return keyframes;
    }
    
    pub fn clear_empty_keyframe(&self) -> Motion {
        let bone_keyframes = self.clear_empty_bone();
        let morph_keyframes = self.clear_empty_morph();
        
        Motion {
            model_name: self.model_name.clone(),
            bone_keyframes,
            morph_keyframes,
            camera_keyframes: self.camera_keyframes.clone(),
            light_keyframes: self.light_keyframes.clone(),
            shadow_keyframes: self.shadow_keyframes.clone(),
            ik_keyframes: self.ik_keyframes.clone(),
        }
    }

    pub fn summary(&self) -> String {
        let mut buf = String::new();
        buf += &format!("Model Name: {}\n", self.model_name);

        if self.bone_keyframes.len() > 0 {
            buf += &format!("Bone Frames: {}\n", self.bone_keyframes.len());
            buf += &format!("Bone Count: {}\n", self.bone_keyframes.len());
            for (k, v) in &self.bone_keyframes {
                buf += &format!("\t{}: {}\n", k, v.len());
            }
        }

        if self.morph_keyframes.len() > 0 {
            buf += &format!("Morph Frames: {}\n", self.morph_keyframes.len());
            buf += &format!("Morph Count: {}\n", self.morph_keyframes.len());
            for (k, v) in &self.morph_keyframes {
                buf += &format!("\t{}: {}\n", k, v.len());
            }
        }

        if self.camera_keyframes.len() >0 {
            buf += &format!("Camera Frames: {}\n", self.camera_keyframes.len());
        }

        buf += "\n";
        buf
    }
}