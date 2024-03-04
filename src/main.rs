#![warn(clippy::all, rust_2018_idioms)]
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::Arc;

use eframe::{egui_wgpu::{WgpuConfiguration, SurfaceErrorAction}, wgpu};

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let wgpu_options = WgpuConfiguration {
        supported_backends: wgpu::util::backend_bits_from_env()
            .unwrap_or(wgpu::Backends::PRIMARY),

        device_descriptor: Arc::new(|_adapter| {
            wgpu::DeviceDescriptor {
                label: Some("egui wgpu device"),
                required_features: wgpu::Features::POLYGON_MODE_LINE,
                required_limits: wgpu::Limits {
                    max_texture_dimension_2d: 8192,
                    ..Default::default()
                },
            }
        }),

        present_mode: wgpu::PresentMode::AutoVsync,

        power_preference: wgpu::util::power_preference_from_env()
            .unwrap_or(wgpu::PowerPreference::LowPower),

        on_surface_error: Arc::new(|err| {
            if err == wgpu::SurfaceError::Outdated {
            } else {
                log::warn!("Dropped frame with error: {err}");
            }
            SurfaceErrorAction::SkipFrame
        }),
        desired_maximum_frame_latency: None,
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
        Box::new(|cc| Box::new(open_pmx_editor::TemplateApp::new(cc))),
    )
}

