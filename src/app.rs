#![allow(dead_code, unused_imports, unused_variables)]
use std::{collections::{BTreeMap, BTreeSet, HashSet}, ffi::OsStr, fmt::format, path::PathBuf, str::FromStr, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use egui::{TextStyle, ScrollArea, mutex::Mutex, viewport, ViewportId};
use egui_extras::{Column, TableBuilder};

use crate::{format::{motion::{BoneKeyframe, MorphKeyframe, Motion}, pmm::read_pmm, pmx::Pmx}, misc::add_sphere};
use crate::dict::{bone_jap_to_eng, morph_jap_to_eng};
use crate::custom3d::{Custom3d, self};

#[derive(PartialEq)]
enum Page {
    Info,
    Material,
    Bone,
    Morph,
    Frame,
    RigidBody,
    Joint,
    VmdBone,
    VmdMorph,
    VmdCamera,
}

pub struct TemplateApp {
    vmd_motion: Option<Motion>,
    pmx_data: Option<Arc<Mutex<Pmx>>>,
    pmx_mat_cur_value: BTreeSet<usize>,
    pmx_bone_cur_value: usize,
    pmx_morph_cur_value: usize,
    page: Page,

    bone_cur_value: usize,
    morph_cur_value: usize,
    log_text: String,
    info_text: String,
    info_window_open: bool,
    show_model_view: Arc<Mutex<bool>>,
    custom3d: Arc<Mutex<Custom3d>>,
    model_viewport_id: ViewportId,
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/SourceHanSans-Normal.otf"
        ))),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        let mut s = Self {
            vmd_motion: None,
            bone_cur_value: 0,
            morph_cur_value: 0,
            log_text: String::new(),
            info_text: String::new(),
            info_window_open: false,
            page: Page::VmdBone,
            pmx_data: None,
            pmx_bone_cur_value: 0,
            pmx_mat_cur_value: BTreeSet::new(),
            pmx_morph_cur_value: 0,
            show_model_view: Arc::new(Mutex::new(true)),
            custom3d: Arc::new(Mutex::new(Custom3d::new(cc))),
            model_viewport_id: egui::ViewportId::from_hash_of("model_viewport"),
        };
        s.load_file(&PathBuf::from_str("./assets/ImagineGirls_Iris_v102_mmd/Iris_mmd/Iris.pmx").unwrap());
        s
    }
    fn load_file(&mut self, p: &PathBuf) {
        let ext = p.extension().unwrap_or_default().to_ascii_lowercase();
        if ext == OsStr::new("vmd") {
            let content = std::fs::read(p).unwrap();
            self.vmd_motion = Some(Motion::read(content, p.to_str().unwrap()));
            self.page = Page::VmdBone;
        } else if ext == OsStr::new("pmx") {
            let content = std::fs::read(p).unwrap();
            let pmx_data = Arc::new(Mutex::new(Pmx::read(content, p.to_str().unwrap())));
            pmx_data.lock().right_hand();
            self.pmx_data = Some(pmx_data.clone());
            self.page = Page::Material;
            self.custom3d.lock().load_mesh(pmx_data);
        }
    }
}

impl eframe::App for TemplateApp {

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let need_repaint_model_viewport = ctx.input(|i| {
            if i.raw.dropped_files.len() == 1 {
                let f = i.raw.dropped_files[0].clone();
                if let Some(p) = &f.path {
                    self.load_file(p);
                }
            }
            i.raw.dropped_files.len() == 1
        });
        if need_repaint_model_viewport {
            ctx.request_repaint_of(self.model_viewport_id);
        }

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        let path = rfd::FileDialog::new().pick_file();
                        if let Some(p) = &path {
                            self.load_file(p);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save PMX As ...").clicked() {
                        if let Some(m) = &self.pmx_data {
                            let path = rfd::FileDialog::new()
                                .add_filter("Poygon Mesh data eXtension", &["pmx"])
                                .save_file();
                            if let Some(p) = &path {
                                let m = m.lock();
                                let mut nm = m.clone();
                                nm.right_hand();
                                let contents = nm.write();
                                std::fs::write(p, contents).unwrap();
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save VMD As ...").clicked() {
                        if let Some(m) = &self.vmd_motion {
                            let path = rfd::FileDialog::new()
                                .add_filter("Vocaloid Motion Data", &["vmd"])
                                .save_file();
                            if let Some(p) = &path {
                                m.write_vmd(p.to_str().unwrap());
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Calc Connected Normal to UV1").clicked() {
                        if let Some(m) = &mut self.pmx_data {
                            let mut m = m.lock();
                            m.calc_connected_nrms_to_uv1();
                        }
                        ui.close_menu();
                    }
                    if ui.button("Japanese to Engligh").clicked() {
                        if let Some(m) = &mut self.pmx_data {
                            let mut m = m.lock();
                            for b in &mut m.bones {
                                b.name = bone_jap_to_eng(&b.name);
                            }
                            for morph in &mut m.morphs {
                                morph.name = morph_jap_to_eng(&morph.name);
                            }
                        }
                        if let Some(m) = &mut self.vmd_motion {
                            {
                                let mut new_bone_keyframes: BTreeMap<String, Vec<BoneKeyframe>> = BTreeMap::new();
                                for (k, v) in &m.bone_keyframes {
                                    let name = bone_jap_to_eng(&k);
                                    new_bone_keyframes.insert(name, v.clone());
                                }
                                m.bone_keyframes = new_bone_keyframes;
                            }
                            {
                                let mut new_morph_keyframes: BTreeMap<String, Vec<MorphKeyframe>> = BTreeMap::new();
                                for (k, v) in &m.morph_keyframes {
                                    let name = morph_jap_to_eng(&k);
                                    new_morph_keyframes.insert(name, v.clone());
                                }
                                m.morph_keyframes = new_morph_keyframes;
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Clean Empty Keyframes").clicked() {
                        if let Some(m) = &mut self.vmd_motion {
                            *m = m.clear_empty_keyframe();
                        }
                        ui.close_menu();
                    }
                    if ui.button("Extract PMM into VMDs").clicked() {
                        if let Some(p) = rfd::FileDialog::new().pick_file() {
                            let pmm_path = p.display().to_string();
                            let motions = read_pmm(&p);
                            let mut buf = String::new();
                            for (i, m) in motions.iter().enumerate() {
                                let new_m = m.clear_empty_keyframe();
                                new_m.write_vmd(&format!("{}.{:0>2}.vmd", pmm_path, i));
                                buf += &format!("Index: {}\n", i);
                                buf += &new_m.summary();
                            }
                            self.log_text += &buf;
                        }
                        ui.close_menu();
                    }
                    if ui.button("Check missing bones and morphs").clicked() {
                        if let Some(pd) = &self.pmx_data {
                            let pd = pd.lock();
                            if let Some(vm) = &self.vmd_motion {
                                let missing_bones = pd.check_missing_bones(&vm.get_useful_bone_names());
                                let missing_morphs = pd.check_missing_morphs(&vm.get_useful_morph_names());

                                let mut info = String::new();
                                info += "Missing Bones:\n";
                                for n in &missing_bones {
                                    info += &format!("{}\n", n);
                                }
                                info += "\nMissing Morphs:\n";
                                for n in &missing_morphs {
                                    info += &format!("{}\n", n);
                                }
                                self.info_text = info;
                                self.info_window_open = true;
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Add UV Sphere").clicked() {
                        if let Some(m) = &mut self.pmx_data {
                            let mut m = m.lock();
                            add_sphere(&mut m, 32, 16, 1.0);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.menu_button("Material", |ui| {
                        if ui.button("Merge").clicked() {
                            if let Some(m) = &mut self.pmx_data {
                                {
                                    let mut m = m.lock();
                                    m.mat_merge(&self.pmx_mat_cur_value);
                                }
                                self.pmx_mat_cur_value.clear();
                                self.custom3d.lock().load_mesh(m.clone());
                            }
                            ui.close_menu();
                        }
                    });
                });
                ui.menu_button("View", |ui| {
                    if ui.checkbox(&mut self.show_model_view.lock(), "Show Model View").clicked() {
                        ui.close_menu();
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("Log").clicked() {
                        ui.close_menu();
                    }
                });
            });
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.page, Page::Info, "Info");
                ui.selectable_value(&mut self.page, Page::Material, "Material");
                ui.selectable_value(&mut self.page, Page::Bone, "Bone");
                ui.selectable_value(&mut self.page, Page::Morph, "Morph");
                // ui.selectable_value(&mut self.page, Page::Frame, "Frame");
                // ui.selectable_value(&mut self.page, Page::RigidBody, "RigidBody");
                // ui.selectable_value(&mut self.page, Page::Joint, "Joint");
                ui.selectable_value(&mut self.page, Page::VmdBone, "VmdBone");
                ui.selectable_value(&mut self.page, Page::VmdMorph, "VmdMorph");
                ui.selectable_value(&mut self.page, Page::VmdCamera, "VmdCamera");
            });
        });

        if self.page != Page::Info {
            egui::SidePanel::left("side_panel").show(ctx, |ui| match self.page {
                Page::VmdBone => {
                    let mut bone_names = Vec::new();
                    let mut bone_keyframe_counts = Vec::new();
                    if let Some(m) = &self.vmd_motion {
                        ui.heading(&m.model_name);
                        for (bn, kfs) in &m.bone_keyframes {
                            bone_names.push(bn);
                            bone_keyframe_counts.push(kfs.len());
                        }
                    }
                    ui.horizontal(|ui| {
                        let text = format!("Count: {}", bone_names.len());
                        ui.heading(text);
                    });
                    ui.separator();
                    ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
                        ui,
                        ui.text_style_height(&TextStyle::Body),
                        bone_names.len(),
                        |ui, row_range| {
                            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                                for row in row_range {
                                    let text = format!("{:3}: {} ({})", row, bone_names[row], bone_keyframe_counts[row]);
                                    ui.selectable_value(&mut self.bone_cur_value, row, text);
                                }
                            });
                        },
                    );
                },
                Page::VmdMorph => {
                    let mut morph_names = Vec::new();
                    let mut morph_keyframe_counts = Vec::new();
                    if let Some(m) = &self.vmd_motion {
                        ui.heading(&m.model_name);
                        for (bn, kfs) in &m.morph_keyframes {
                            morph_names.push(bn);
                            morph_keyframe_counts.push(kfs.len());
                        }
                    }
                    ui.horizontal(|ui| {
                        let text = format!("Count: {}", morph_names.len());
                        ui.heading(text);
                    });
                    ui.separator();
                    ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
                        ui,
                        ui.text_style_height(&TextStyle::Body),
                        morph_names.len(),
                        |ui, row_range| {
                            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                                for row in row_range {
                                    let text = format!("{:3}: {} ({})", row, morph_names[row], morph_keyframe_counts[row]);
                                    ui.selectable_value(&mut self.morph_cur_value, row, text);
                                }
                            });
                        },
                    );
                },
                Page::VmdCamera => {
                    let mut count = 0;
                    if let Some(m) = &self.vmd_motion {
                        ui.heading(&m.model_name);
                        count = m.camera_keyframes.len();
                    }
                    ui.horizontal(|ui| {
                        let text = format!("Count: {}", count);
                        ui.heading(text);
                    });
                    ui.separator();
                },
                Page::Bone => {
                    let mut names = Vec::new();
                    if let Some(m) = &self.pmx_data {
                        let m = m.lock();
                        for b in &m.bones {
                            names.push(b.name.clone());
                        }
                    }
                    ui.horizontal(|ui| {
                        let text = format!("Count: {}", names.len());
                        ui.heading(text);
                    });
                    ui.separator();
                    ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
                        ui,
                        ui.text_style_height(&TextStyle::Body),
                        names.len(),
                        |ui, row_range| {
                            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                                for row in row_range {
                                    let text = format!("{:3}: {}", row, names[row]);
                                    ui.selectable_value(&mut self.pmx_bone_cur_value, row, text);
                                }
                            });
                        },
                    );
                },
                Page::Material => {
                    let mut names = Vec::new();
                    let mut alphas = Vec::new();
                    if let Some(m) = &self.pmx_data {
                        let m = m.lock();
                        for mat in &m.mats {
                            names.push(mat.name.clone());
                            alphas.push(if mat.diffuse.w == 0.0 { "◻" } else { "◼" });
                        }
                    }
                    ui.horizontal(|ui| {
                        let text = format!("Count: {}", names.len());
                        ui.heading(text);
                    });
                    ui.separator();
                    ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
                        ui,
                        ui.text_style_height(&TextStyle::Body),
                        names.len(),
                        |ui, row_range| {
                            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                                for row in row_range {
                                    let text = format!("{}{:3}: {}", alphas[row], row, names[row]);
                                    let selected = self.pmx_mat_cur_value.contains(&row);
                                    if ui.selectable_label(selected, text).clicked() {
                                        if ui.input(|i| i.modifiers.ctrl) {
                                            if selected {
                                                self.pmx_mat_cur_value.remove(&row);
                                            } else {
                                                self.pmx_mat_cur_value.insert(row);
                                            }
                                        } else {
                                            self.pmx_mat_cur_value.clear();
                                            self.pmx_mat_cur_value.insert(row);
                                        }
                                    }
                                }
                            });
                        },
                    );
                },
                Page::Morph => {
                    let mut names = Vec::new();
                    if let Some(m) = &self.pmx_data {
                        let m = m.lock();
                        for morph in &m.morphs {
                            names.push(morph.name.clone());
                        }
                    }
                    ui.horizontal(|ui| {
                        let text = format!("Count: {}", names.len());
                        ui.heading(text);
                    });
                    ui.separator();
                    ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
                        ui,
                        ui.text_style_height(&TextStyle::Body),
                        names.len(),
                        |ui, row_range| {
                            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                                for row in row_range {
                                    let text = format!("{:3}: {}", row, names[row]);
                                    ui.selectable_value(&mut self.pmx_morph_cur_value, row, text);
                                }
                            });
                        },
                    );
                },
                _ => {},
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.page {
            Page::Info => {
                if let Some(m) = &self.pmx_data {
                    let m = m.lock();
                    ui.heading(&m.name);
                    ui.label(&m.comment);
                }
            },
            Page::Material => {
                if let Some(m) = &self.pmx_data {
                    let m = m.lock();
                    if m.mats.len() > 0 && self.pmx_mat_cur_value.len() > 0 {
                        let mat = &m.mats[*self.pmx_mat_cur_value.first().unwrap()];
                        ui.heading(format!("Name: {}", mat.name));
                        ui.label(format!("Associated Face Count: {}", mat.associated_face_count));
                        ui.label(format!("NameEn: {}", mat.name_en));
                        ui.label(format!("Comment: {}", mat.comment));
                        ui.label(format!("Diffuse: {}", mat.diffuse));
                        ui.label(format!("Specular: {}", mat.specular));
                        ui.label(format!("Ambient: {}", mat.ambient));
                        ui.label(format!("Edge Color: {}", mat.edge_color));
                        ui.label(format!("Edge Scale: {}", mat.edge_scale));
                        if mat.tex_index >= 0 {
                            ui.label(format!("Tex: {}", m.texs[mat.tex_index as usize]));
                        } else {
                            ui.label("Tex: None");
                        }
                        if mat.env_index >= 0 {
                            ui.label(format!("MatCap Tex: {}", m.texs[mat.env_index as usize]));
                        } else {
                            ui.label("MatCap Tex: None");
                        }
                        ui.label(format!("Matcap Blend Mode: {:?}", mat.env_blend_mode));
                        match mat.toon {
                            crate::format::pmx::Toon::Tex(i) => {
                                if i >= 0 {
                                    ui.label(format!("Toon Tex: {}", m.texs[i as usize]));
                                } else {
                                    ui.label("Toon Tex: None");
                                }
                            },
                            crate::format::pmx::Toon::Inner(i) => {
                                ui.label(format!("Toon Tex: toon{:02}.bmp", i));
                            },
                        }
                    }
                }
            },
            Page::VmdBone => {
                let mut bone_cur_keyframe = Vec::new();
                let mut bone_names = Vec::new();
                let mut bone_keyframe_counts = Vec::new();
                if let Some(m) = &self.vmd_motion {
                    for (bn, kfs) in &m.bone_keyframes {
                        bone_names.push(bn);
                        bone_keyframe_counts.push(kfs.len());
                    }
                    if self.bone_cur_value < bone_names.len() {
                        let bone_name = bone_names[self.bone_cur_value];
                        bone_cur_keyframe = m.bone_keyframes.get(bone_name).unwrap().clone();
                    }
                }
    
                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
    
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::initial(100.0))
                    .column(Column::remainder())
                    .min_scrolled_height(0.0);
    
                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Index");
                        });
                        header.col(|ui| {
                            ui.strong("Frame");
                        });
                        header.col(|ui| {
                            ui.strong("Translate");
                        });
                        header.col(|ui| {
                            ui.strong("Rotation");
                        });
                    })
                    .body(|body|  {
                        body.rows(text_height, bone_cur_keyframe.len(), |mut row| {
                            let row_index = row.index();
                            row.col(|ui| {
                                ui.label(row_index.to_string());
                            });
                            row.col(|ui| {
                                let frame = bone_cur_keyframe[row_index].frame;
                                ui.label(frame.to_string());
                            });
                            row.col(|ui| {
                                let trans = bone_cur_keyframe[row_index].trans;
                                ui.label(format!("{:.2}, {:.2}, {:.2}", trans[0], trans[1], trans[2]));
                            });
                            row.col(|ui| {
                                let rot = bone_cur_keyframe[row_index].rot;
                                ui.label(format!("{}, {}, {}, {}", rot.x, rot.y, rot.z, rot.w));
                            });
                        });
                    });
            },
            Page::VmdCamera => {
                let mut cur_keyframe = Vec::new();
                if let Some(m) = &self.vmd_motion {
                    cur_keyframe = m.camera_keyframes.clone();
                }

                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::initial(100.0))
                    .column(Column::remainder())
                    .min_scrolled_height(0.0);

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Index");
                        });
                        header.col(|ui| {
                            ui.strong("Frame");
                        });
                        header.col(|ui| {
                            ui.strong("FOV");
                        });
                        header.col(|ui| {
                            ui.strong("Dist");
                        });
                        header.col(|ui| {
                            ui.strong("Translate");
                        });
                        header.col(|ui| {
                            ui.strong("Rotation");
                        });
                    })
                    .body(|body|  {
                        body.rows(text_height, cur_keyframe.len(), |mut row| {
                            let row_index = row.index();
                            row.col(|ui| {
                                ui.label(row_index.to_string());
                            });
                            row.col(|ui| {
                                let frame = cur_keyframe[row_index].frame;
                                ui.label(frame.to_string());
                            });
                            row.col(|ui| {
                                let fov = cur_keyframe[row_index].fov;
                                ui.label(fov.to_string());
                            });
                            row.col(|ui| {
                                let dist = cur_keyframe[row_index].dist;
                                ui.label(dist.to_string());
                            });
                            row.col(|ui| {
                                let trans = cur_keyframe[row_index].trans;
                                ui.label(format!("{:.2}, {:.2}, {:.2}", trans[0], trans[1], trans[2]));
                            });
                            row.col(|ui| {
                                let rot = cur_keyframe[row_index].rot;
                                ui.label(format!("{}, {}, {}", rot.x, rot.y, rot.z));
                            });
                        });
                    });
            },
            Page::VmdMorph => {
                let mut morph_cur_keyframe = Vec::new();
                let mut morph_names = Vec::new();
                let mut morph_keyframe_counts = Vec::new();
                if let Some(m) = &self.vmd_motion {
                    for (bn, kfs) in &m.morph_keyframes {
                        morph_names.push(bn);
                        morph_keyframe_counts.push(kfs.len());
                    }
                    if self.morph_cur_value < morph_names.len() {
                        let morph_name = morph_names[self.morph_cur_value];
                        morph_cur_keyframe = m.morph_keyframes.get(morph_name).unwrap().clone();
                    }
                }
    
                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
    
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::remainder())
                    .min_scrolled_height(0.0);
    
                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Index");
                        });
                        header.col(|ui| {
                            ui.strong("Frame");
                        });
                        header.col(|ui| {
                            ui.strong("Weight");
                        });
                    })
                    .body(|body|  {
                        body.rows(text_height, morph_cur_keyframe.len(), |mut row| {
                            let row_index = row.index();
                            row.col(|ui| {
                                ui.label(row_index.to_string());
                            });
                            row.col(|ui| {
                                let frame = morph_cur_keyframe[row_index].frame;
                                ui.label(frame.to_string());
                            });
                            row.col(|ui| {
                                let weight = morph_cur_keyframe[row_index].weight;
                                ui.label(weight.to_string());
                            });
                        });
                    });
            },
            _ => {},
        });

        {
            let window = egui::Window::new("Info")
                                            .scroll([false, true])
                                            .collapsible(false)
                                            .open(&mut self.info_window_open)
                                            .show(ctx, |ui| {
                ui.text_edit_multiline(&mut self.info_text);
            });
        }
        {
            let show_model_view = self.show_model_view.clone();
            if *show_model_view.lock() {
                let custom3d = self.custom3d.clone();
                let pmx_data = self.pmx_data.clone();
                let model_viewport_id = self.model_viewport_id;
                ctx.show_viewport_deferred(
                    model_viewport_id,
                    egui::ViewportBuilder::default()
                        .with_title("Model View")
                        .with_inner_size([500.0, 500.0]),
                    move |ctx, class| {
                        if custom3d.lock().show_material_filter {
                            let custom3d = custom3d.clone();
                            let pmx_data = pmx_data.clone();
                            ctx.show_viewport_deferred(
                                egui::ViewportId::from_hash_of("deferred_viewport"),
                                egui::ViewportBuilder::default()
                                    .with_title("Material Filter")
                                    .with_inner_size([200.0, 400.0]),
                                move |ctx, class| {
                                    let mut custom3d = custom3d.lock();
                                    egui::CentralPanel::default().show(ctx, |ui| {
                                        ui.horizontal(|ui| {
                                            if ui.button("All").clicked() {
                                                for (_, checked) in custom3d.filters.iter_mut() {
                                                    *checked = true;
                                                    ctx.request_repaint_of(model_viewport_id);
                                                }
                                            }
                                            if ui.button("None").clicked() {
                                                for (_, checked) in custom3d.filters.iter_mut() {
                                                    *checked = false;
                                                    ctx.request_repaint_of(model_viewport_id);
                                                }
                                            }
                                            if ui.button("Invert").clicked() {
                                                for (_, checked) in custom3d.filters.iter_mut() {
                                                    *checked = !*checked;
                                                    ctx.request_repaint_of(model_viewport_id);
                                                }
                                            }
                                            if ui.button("Delete").clicked() {
                                                let mut mats_need_delete = Vec::new();
                                                for (mat, checked) in custom3d.filters.iter_mut() {
                                                    if *checked {
                                                        mats_need_delete.push(mat.clone());
                                                    }
                                                }
                                                if let Some(m) = &pmx_data {
                                                    {
                                                        let mut m = m.lock();
                                                        m.delete_mats(&mats_need_delete);
                                                    }
                                                    custom3d.load_mesh(m.clone());
                                                }
                                                ctx.request_repaint_of(model_viewport_id);
                                            }
                                        });
                                        let text_style = TextStyle::Body;
                                        let row_height = ui.text_style_height(&text_style);
                                        let num_rows = custom3d.filters.len();
                                        ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
                                            ui,
                                            row_height,
                                            num_rows,
                                            |ui, row_range| {
                                                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                                                    for i in row_range {
                                                        let (name, check) = &mut custom3d.filters[i];
                                                        if ui.checkbox(check, format!("{}: {}", i, name)).clicked() {
                                                            ctx.request_repaint_of(model_viewport_id);
                                                        }
                                                    }
                                                });
                                            },
                                        );
                                    });
                                    if ctx.input(|i| i.viewport().close_requested()) {
                                        custom3d.show_material_filter = false;
                                    }
                                },
                            );
                        }

                        egui::CentralPanel::default().show(ctx, |ui| {
                            let mut custom3d = custom3d.lock();
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut custom3d.draw_flag.planer, "planer");
                                ui.checkbox(&mut custom3d.draw_flag.wireframe, "wireframe");
                                ui.checkbox(&mut custom3d.draw_flag.gray, "gray");
                                ui.checkbox(&mut custom3d.draw_flag.use_texture, "texture");
                                ui.checkbox(&mut custom3d.show_material_filter, "filter");
                            });
                            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                                custom3d.custom_painting(ui);
                            });
                        });
                        if ctx.input(|i| i.viewport().close_requested()) {
                            // Tell parent to close us.
                            *show_model_view.lock() = false;
                        }
                    },
                );
            }
        }
    }
}
