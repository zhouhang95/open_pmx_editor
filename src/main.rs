#![warn(clippy::all, rust_2018_idioms)]
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::Arc;

use eframe::{egui_wgpu::{SurfaceErrorAction, WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew}, wgpu};

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let wgpu_options = WgpuConfiguration {
        present_mode: wgpu::PresentMode::AutoVsync,
        
        on_surface_error: Arc::new(|err| {
            if err == wgpu::SurfaceError::Outdated {
            } else {
                log::warn!("Dropped frame with error: {err}");
            }
            SurfaceErrorAction::SkipFrame
        }),
        desired_maximum_frame_latency: None,
        wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
            device_descriptor: Arc::new(|_adapter| {
                wgpu::DeviceDescriptor {
                    label: Some("egui wgpu device"),
                    required_features: wgpu::Features::POLYGON_MODE_LINE,
                    required_limits: wgpu::Limits {
                        max_texture_dimension_2d: 8192,
                        ..Default::default()
                    },
                    memory_hints: wgpu::MemoryHints::default(),
                }
            }),
            ..Default::default()
        }),
    };

    let native_options = eframe::NativeOptions {
        wgpu_options,
        depth_buffer: 32,
        viewport: egui::ViewportBuilder::default()
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Open MMD Editor",
        native_options,
        Box::new(|cc| Ok(Box::new(open_pmx_editor::TemplateApp::new(cc)))),
    )
}

