use std::collections::HashMap;

use once_cell::sync::Lazy;

static MORPH_JP_TO_EN: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("まばたき", "blink"),
        ("笑い", "happy_eyes"),
        ("ウィンク", "happy_eye_l"),
        ("ウィンク右", "happy_eye_r"),
        ("ウィンク２", "blink_l"),
        ("ウィンク２右", "blink_r"),
        ("ｳｨﾝｸ２右", "blink_r"),
        ("なごみ", "blink_2"),
        ("はぅ", ">_<"),
        ("＞＜", ">_<"),
        ("びっくり", "surprised"),
        ("じと目", "-_-"),
        ("ジト目", "-_-"),
        ("ｷﾘｯ", "powerful_eyes"),
        ("はちゅ目", "white_eyes"),
        ("はちゅ目縦潰れ", "white_eyes_v"),
        ("はちゅ目横潰れ", "white_eyes_h"),
        ("あ", "a"),
        ("い", "i"),
        ("う", "u"),
        ("え", "e"),
        ("お", "o"),
        ("あ２", "aa"),
        ("ん", "n"),
        ("▲", "A"),
        ("∧", "^"),
        ("□", "[]"),
        ("ワ", "wa"),
        ("ω", "w"),
        ("ω□", "W"),
        ("にやり", "smile"),
        ("にやり２", "smile/"),
        ("にっこり", "\\smile/"),
        ("ぺろっ", "tongue_out"),
        ("舌", "tongue"),
        ("てへぺろ", "tongue_out_1"),
        ("てへぺろ２", "tongue_out_2"),
        ("口角上げ", "\\mouth/"),
        ("口角下げ", "/mouth_down\\"),
        ("口横広げ", "<mouth>"),
        ("口角広げ", "<mouth>"),
        ("口角狭め", ">mouth<"),
        ("歯無し上", "no_up_tooth"),
        ("歯無し下", "no_down_tooth"),
        ("涙", "tear"),
        ("輪郭", "contour"),
        ("星目", "star_eyes"),
        ("しいたけ", "star_eyes"),
        ("はぁと", "heart"),
        ("ハート", "heart"),
        ("瞳小", "small_pupil"),
        ("瞳縦潰れ", "pupil_v"),
        ("光下", "highlight_down"),
        ("恐ろしい子！", "horrible"),
        ("照れ", "embarrassed"),
        ("赤面", "embarrassed"),
        ("がーん", "bann"),
        ("真面目", "serious"),
        ("困る", "in_trouble"),
        ("にこり", "happy_eyebow"),
        ("寄せ", "happy_eyebow"),
        ("怒り", "angry"),
        ("上", "up"),
        ("下", "down"),
        ("前", "front"),
        ("眉頭左", "eyebow_l"),
        ("眉頭右", "eyebow_r"),
        ("メガネ", "glass"),
        ("ハイライト消", "no_highlight"),
        ("映り込み消", "no_eye_reflect"),
        ("青ざめる", "pale"),
        ("髪影消", "no_hair_shadow"),
        ("照れ消", "no_embarrassed"),
        ("AL未使用", "disable_al"),
        ("ハンサム", "\\jaw/"),
        ("ヤンデレ", "yandere"),
    ])
});

static BONE_JP_TO_EN: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("操作中心", "op_center"),
        ("全ての親", "root"),
        ("エッジ倍率", "edge_factor"),
        ("センター", "center"),
        ("センター2", "center_2"),
        ("グルーブ", "groove"),
        ("グルーブ2", "groove_2"),
        ("腰", "waist"),
        ("上半身", "upper_half"),
        ("上半身1", "upper_half_1"),
        ("上半身2", "upper_half_2"),
        ("上半身3", "upper_half_3"),
        ("下半身", "pelvis"),
        ("首", "neck"),
        ("頭", "head"),

        ("スカート親", "skirt_root"),
        ("メガネ", "glass"),
        ("ﾈｸﾀｲ１", "necktie_1"),
        ("ﾈｸﾀｲ２", "necktie_2"),
        ("ﾈｸﾀｲ３", "necktie_3"),
        ("ﾈｸﾀｲＩＫ", "ik_necktie"),

        ("両目", "eyes"),
        ("左目", "eye_l"),
        ("右目", "eye_r"),
        
        ("左胸１", "brest_1_l"),
        ("左胸２", "brest_2_l"),

        ("左肩", "clavicle_l"),
        ("左腕", "upperarm_l"),
        ("左腕捩", "upperarm_T_l"),
        ("左腕捩1", "upperarm_T_1_l"),
        ("左腕捩2", "upperarm_T_2_l"),
        ("左腕捩3", "upperarm_T_3_l"),
        ("左ひじ", "lowerarm_l"),
        ("左ひじ補助", "lowerarm_H_l"),
        ("左手捩", "lowerarm_T_l"),
        ("左手捩1", "lowerarm_T_1_l"),
        ("左手捩2", "lowerarm_T_2_l"),
        ("左手捩3", "lowerarm_T_3_l"),
        ("左手首", "hand_l"),
        ("左手先", "hand_tip_l"),
        ("左親指０", "thumb_1_L"),
        ("左親指１", "thumb_2_L"),
        ("左親指２", "thumb_3_L"),
        ("左親指先", "thumb_tip_L"),
        ("左人指０", "index_0_l"),
        ("左人指１", "index_1_l"),
        ("左人指２", "index_2_l"),
        ("左人指３", "index_3_l"),
        ("左人指先", "index_tip_l"),
        ("左中指０", "middle_0_l"),
        ("左中指１", "middle_1_l"),
        ("左中指２", "middle_2_l"),
        ("左中指３", "middle_3_l"),
        ("左中指先", "middle_tip_l"),
        ("左薬指０", "ring_0_l"),
        ("左薬指１", "ring_1_l"),
        ("左薬指２", "ring_2_l"),
        ("左薬指３", "ring_3_l"),
        ("左薬指先", "ring_tip_l"),
        ("左小指０", "pingk_0_l"),
        ("左小指１", "pinky_1_l"),
        ("左小指２", "pinky_2_l"),
        ("左小指３", "pinky_3_l"),
        ("左小指先", "pinky_tip_l"),

        ("腰キャンセル左", "waist_C_l"),

        ("左足", "thigh_l"),
        ("左足D", "thigh_deform_l"),
        ("左ひざ", "calf_l"),
        ("左ひざD", "calf_deform_l"),
        ("左足首", "foot_l"),
        ("左足首D", "foot_deform_l"),
        ("左足ＩＫ", "ik_foot_l"),
        ("左足先EX", "ball_l"),
        ("左つま先ＩＫ", "ik_toe_l"),
        ("左つま先", "toe_l"),

        ("左肩C", "clavicle_C_l"),
        ("左肩P", "clavicle_P_l"),
        ("左ダミー", "dummy_l"),
        ("左足IK親", "ik_foot_root_l"),

        ("右胸１", "brest_1_r"),
        ("右胸２", "brest_2_r"),

        ("右肩", "clavicle_r"),
        ("右腕", "upperarm_r"),
        ("右腕捩", "upperarm_T_r"),
        ("右腕捩1", "upperarm_T_1_r"),
        ("右腕捩2", "upperarm_T_2_r"),
        ("右腕捩3", "upperarm_T_3_r"),
        ("右ひじ", "lowerarm_r"),
        ("右ひじ補助", "lowerarm_H_r"),
        ("右手捩", "lowerarm_T_r"),
        ("右手捩1", "lowerarm_T_1_r"),
        ("右手捩2", "lowerarm_T_2_r"),
        ("右手捩3", "lowerarm_T_3_r"),
        ("右手首", "hand_r"),
        ("右手先", "hand_tip_r"),
        ("右親指０", "thumb_1_r"),
        ("右親指１", "thumb_2_r"),
        ("右親指２", "thumb_3_r"),
        ("右親指先", "thumb_tip_r"),
        ("右人指０", "index_0_r"),
        ("右人指１", "index_1_r"),
        ("右人指２", "index_2_r"),
        ("右人指３", "index_3_r"),
        ("右人指先", "index_tip_r"),
        ("右中指０", "middle_0_r"),
        ("右中指１", "middle_1_r"),
        ("右中指２", "middle_2_r"),
        ("右中指３", "middle_3_r"),
        ("右中指先", "middle_tip_r"),
        ("右薬指０", "ring_0_r"),
        ("右薬指１", "ring_1_r"),
        ("右薬指２", "ring_2_r"),
        ("右薬指３", "ring_3_r"),
        ("右薬指先", "ring_tip_r"),
        ("右小指０", "pingk_0_r"),
        ("右小指１", "pinky_1_r"),
        ("右小指２", "pinky_2_r"),
        ("右小指３", "pinky_3_r"),
        ("右小指先", "pinky_tip_r"),

        
        ("腰キャンセル右", "waist_C_r"),
        ("右足", "thigh_r"),
        ("右足D", "thigh_deform_r"),
        ("右ひざ", "calf_r"),
        ("右ひざD", "calf_deform_r"),
        ("右足首", "foot_r"),
        ("右足首D", "foot_deform_r"),
        ("右足ＩＫ", "ik_foot_r"),
        ("右足先EX", "ball_r"),
        ("右つま先ＩＫ", "ik_toe_r"),
        ("右つま先", "toe_r"),

        ("右肩C", "clavicle_C_r"),
        ("右肩P", "clavicle_P_r"),
        ("右ダミー", "dummy_r"),
        ("右足IK親", "ik_foot_root_r"),
    ])
});
static UTILS_JP_TO_EN: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("０", "_0"),
        ("１", "_1"),
        ("２", "_2"),
        ("３", "_3"),
        ("左", "_l_"),
        ("右", "_r_"),

        ("スカート", "skirt"),
    ])
});

pub fn bone_jap_to_eng(name: &str) -> String {
    BONE_JP_TO_EN.iter().for_each(|(_, v) | {
        if v.len() >= 15 {
            println!("B: {} {}", v, v.len());
        }
    });
    MORPH_JP_TO_EN.iter().for_each(|(_, v) | {
        if v.len() >= 15 {
            println!("M: {} {}", v, v.len());
        }
    });
    if name.is_ascii() {
        return name.to_string();
    }
    if let Some(n) = BONE_JP_TO_EN.get(name) {
        return n.to_string();
    }
    let mut n = name.to_string();
    UTILS_JP_TO_EN.iter().for_each(|(k, v)| {
        n = n.replace(k, v);
    });
    eprintln!("Undefined bone rename: {} -> {}", name, n);
    n
}

pub fn morph_jap_to_eng(name: &str) -> String {
    if name.is_ascii() {
        return name.to_string();
    }
    if let Some(n) = MORPH_JP_TO_EN.get(name) {
        return n.to_string();
    }
    let mut n = name.to_string();
    UTILS_JP_TO_EN.iter().for_each(|(k, v)| {
        n = n.replace(k, v);
    });
    if !n.is_ascii() {
        let mut m = String::new();
        for ch in n.chars() {
            if ch.is_ascii() {
                m.push(ch);
            } else {
                let mut b = [0; 2];
                ch.encode_utf16(&mut b);
                m.push_str(&format!("{:04X}", b[0]));
            }
        }
        n = String::new();
        for (i, ch) in m.chars().enumerate() {
            if i < 14 {
                n.push(ch);
            }
        }
    }
    eprintln!("Undefined morph rename: {} -> {}", name, n);
    n
}