use std::ffi::OsStr;

use egui::{TextStyle, ScrollArea};

use crate::{motion::Motion, pmm::read_pmm};

pub struct TemplateApp {
    vmd_path: Option<std::path::PathBuf>,
    vmd_motion: Option<Motion>,

    bone_cur_value: usize,
    log_text: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            vmd_path: None,
            vmd_motion: None,
            bone_cur_value: 0,
            log_text: String::new(),
        }
    }
}
fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/SourceHanSans-Normal.otf"
        )),
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
        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.vmd_path = rfd::FileDialog::new().pick_file();
                        if let Some(p) = &self.vmd_path {
                            let ext = p.extension();
                            if ext == Some(OsStr::new("vmd")) || ext == Some(OsStr::new("VMD")) {
                                self.vmd_motion = Some(Motion::read_vmd(p));
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
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Log").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            let mut bone_names = Vec::new();
            let mut bone_keyframe_counts = Vec::new();
            if let Some(m) = &self.vmd_motion {
                ui.heading(&m.model_name);
                for (bn, kfs) in m.get_bone_keyframes() {
                    bone_names.push(bn);
                    bone_keyframe_counts.push(kfs.len());
                }
            }
            ui.horizontal(|ui| {
                let text = format!("Count: {}", bone_names.len());
                ui.heading(text);
            });
            ui.separator();
            let text_style = TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);
            let num_rows = bone_names.len();
            ScrollArea::vertical().auto_shrink([false; 2]).show_rows(
                ui,
                row_height,
                num_rows,
                |ui, row_range| {
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {                        
                        for row in row_range {
                            let text = format!("{:3}: {} ({})", row, bone_names[row], bone_keyframe_counts[row]);
                            ui.selectable_value(&mut self.bone_cur_value, row, text);
                        }
                    });
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            use egui_extras::{Column, TableBuilder};
            let mut bone_cur_keyframe = Vec::new();
            let mut bone_names = Vec::new();
            let mut bone_keyframe_counts = Vec::new();
            if let Some(m) = &self.vmd_motion {
                for (bn, kfs) in m.get_bone_keyframes() {
                    bone_names.push(bn);
                    bone_keyframe_counts.push(kfs.len());
                }
                if self.bone_cur_value < bone_names.len() {
                    let bone_name = &bone_names[self.bone_cur_value];
                    bone_cur_keyframe = m.get_bone_keyframes().get(bone_name).unwrap().clone();
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
                    body.rows(text_height, bone_cur_keyframe.len(), |row_index, mut row| {
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
                            ui.label(format!("{}, {}, {}, {}", rot[0], rot[1], rot[2], rot[3]));
                        });
                    });
                });
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}
