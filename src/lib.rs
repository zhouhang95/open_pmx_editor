#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod common;
mod pmm;
mod motion;
mod vmd_reader;
mod vmd_writer;
mod pmx;
pub use app::TemplateApp;
