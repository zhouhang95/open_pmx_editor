#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::collections::HashMap;

pub struct Motion {
    pub model_name:       String,
    pub bone_keyframes:   Vec<BoneKeyframe>,
    pub morph_keyframes:  Vec<MorphKeyframe>,
    pub camera_keyframes: Vec<CameraKeyframe>,
    pub light_keyframes:  Vec<LightKeyframe>,
    pub shadow_keyframes: Vec<ShadowKeyframe>,
    pub ik_keyframes:     Vec<IkKeyframe>,
}

pub struct BoneKeyframe {
    pub name:  String,
    pub frame: u32,
    pub trans: [f32; 3],
    pub rot:   [f32; 4],
    pub txc:   [f32; 4],
    pub tyc:   [f32; 4],
    pub tzc:   [f32; 4],
    pub rc:    [f32; 4],
}

impl Clone for BoneKeyframe {
    fn clone(&self) -> Self {
        BoneKeyframe {
            name: self.name.clone(),
            frame: self.frame,
            trans: self.trans,
            rot:   self.rot,
            txc:   self.txc,
            tyc:   self.tyc,
            tzc:   self.tzc,
            rc:    self.rc,
        }

    }
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

pub struct MorphKeyframe {
    pub name:   String,
    pub frame:  u32,
    pub weight: f32,
}

impl Clone for MorphKeyframe {
    fn clone(&self) -> Self {
        MorphKeyframe {
            name: self.name.clone(),
            frame: self.frame,
            weight: self.weight,
        }

    }
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
            bone_keyframes:   Vec::new(),
            morph_keyframes:  Vec::new(),
            camera_keyframes: Vec::new(),
            light_keyframes:  Vec::new(),
            shadow_keyframes: Vec::new(),
            ik_keyframes:     Vec::new(),
        }
    }

    pub fn get_bone_keyframes(&self) -> HashMap<String, Vec<BoneKeyframe>> {
        let mut keyframes_map: HashMap<String, Vec<BoneKeyframe>> = HashMap::new();
        for kf in &self.bone_keyframes {
            keyframes_map.entry(kf.name.clone()).or_insert(vec![]);
            keyframes_map.get_mut(&kf.name).unwrap().push(kf.clone());
        }
        keyframes_map
    }

    pub fn get_morph_keyframes(&self) -> HashMap<String, Vec<MorphKeyframe>> {
        let mut keyframes_map: HashMap<String, Vec<MorphKeyframe>> = HashMap::new();
        for kf in &self.morph_keyframes {
            keyframes_map.entry(kf.name.clone()).or_insert(vec![]);
            keyframes_map.get_mut(&kf.name).unwrap().push(kf.clone());
        }
        keyframes_map
    }

    pub fn clear_empty_morph(&self) -> Vec<MorphKeyframe> {
        let mut keyframes: Vec<MorphKeyframe> = vec![];
        for (k, v) in &self.get_morph_keyframes() {
            let mut not_zero = false;
            for kf in v {
                if kf.weight != 0.0 {
                    not_zero = true;
                    break;
                }
            }
            if not_zero {
                keyframes.extend_from_slice(v);
            }
        }
        return keyframes;
    }

    pub fn clear_empty_bone(&self) -> Vec<BoneKeyframe> {
        let mut keyframes: Vec<BoneKeyframe> = vec![];
        for (k, v) in &self.get_bone_keyframes() {
            let mut not_zero = false;
            for kf in v {
                if !kf.empty() {
                    not_zero = true;
                    break;
                }
            }
            if not_zero {
                keyframes.extend_from_slice(v);
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
            let bone_keyframes_map = self.get_bone_keyframes();
            buf += &format!("Bone Count: {}\n", bone_keyframes_map.len());
            for (k, v) in &bone_keyframes_map {
                buf += &format!("\t{}: {}\n", k, v.len());
            }
        }

        if self.morph_keyframes.len() > 0 {
            buf += &format!("Morph Frames: {}\n", self.morph_keyframes.len());
            let morph_keyframes_map = self.get_morph_keyframes();
            buf += &format!("Morph Count: {}\n", morph_keyframes_map.len());
            for (k, v) in &morph_keyframes_map {
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