#![allow(dead_code, unused_imports, unused_variables)]
use glam::*;

pub struct Camera {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub dist: f32,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub perspective: bool,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            pos: Vec3::new(0.0, 1.0, 0.0),
            yaw: 10.0,
            pitch: 0.0,
            roll: 0.0,
            dist: 4.0,
            fov: 45.0,
            aspect_ratio: 16.0 / 9.0,
            perspective: true,
        }
    }

    pub fn rot(&self) -> Mat4 {
        let yaw = Mat4::from_rotation_y(self.yaw.to_radians());
        let pitch = Mat4::from_rotation_x(self.pitch.to_radians());
        let roll = Mat4::from_rotation_z(self.roll.to_radians());
        roll * pitch * yaw
    }

    pub fn rot3(&self) -> Mat3 {
        let yaw = Mat3::from_rotation_y(self.yaw.to_radians());
        let pitch = Mat3::from_rotation_x(self.pitch.to_radians());
        let roll = Mat3::from_rotation_z(self.roll.to_radians());
        roll * pitch * yaw
    }

    pub fn view(&self) -> Mat4 {
        let v = Mat4::from_translation(-self.pos);
        let v = self.rot() * v;
        let v = Mat4::from_translation(Vec3::new(0.0, 0.0, -self.dist)) * v;
        v
    }

    pub fn proj(&self) -> Mat4 {
        if self.perspective {
            Mat4::perspective_rh((self.fov).to_radians(), self.aspect_ratio, 0.01, 1000.0)
        } else {
            let height = (self.fov / 2.0).to_radians().tan() * self.dist.abs();
            let width = self.aspect_ratio * height;
            Mat4::orthographic_rh(-width, width, -height, height, -1000.0, 1000.0)
        }
    }

    pub fn real_pos(&self) -> Vec3 {
        self.pos - self.direction() * self.dist
    }

    pub fn direction(&self) -> Vec3 {
        self.rot3().transpose() * Vec3::new(0.0, 0.0, -1.0)
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.perspective = true;
        self.yaw += dx * 0.5;
        self.pitch += dy * 0.5;
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        let speed = 0.01;
        let dx = - dx * speed;
        let dy = dy * speed;
        let dv = Vec3::new(dx, dy, 0.0);
        let rot = self.rot3().transpose();
        let dv = rot * dv;
        self.pos += dv;
    }

    pub fn dolly(&mut self, d: f32) {
        self.dist -= d * 0.2;
    }
    pub fn set_not_perspective(&mut self) {
        self.perspective = false;
        self.yaw = 0.0;
        self.pitch = 0.0;
        self.roll = 0.0;
    }
    pub fn front(&mut self) {
        self.set_not_perspective();
    }

    pub fn back(&mut self) {
        self.set_not_perspective();
        self.yaw = 180.0;
    }

    pub fn left(&mut self) {
        self.set_not_perspective();
        self.yaw = 90.0;
    }

    pub fn right(&mut self) {
        self.set_not_perspective();
        self.yaw = -90.0;
    }
    pub fn up(&mut self) {
        self.set_not_perspective();
        self.pitch = 90.0;
    }
    pub fn down(&mut self) {
        self.set_not_perspective();
        self.pitch = -90.0;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform {
    view_proj: Mat4,
    view: Mat4,
    proj: Mat4,
    planer: Vec4,
}

unsafe impl bytemuck::Pod for CameraUniform {}
unsafe impl bytemuck::Zeroable for CameraUniform {}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
            proj: Mat4::IDENTITY,
            planer: Vec4::ZERO,
        }
    }

    pub fn from_camera(camera: &Camera, planer: bool) -> Self {
        let proj = camera.proj();
        let view = camera.view();
        let view_proj = camera.proj() * camera.view();
        Self {
            view_proj,
            view,
            proj,
            planer: vec4(if planer {1.0} else {0.0}, 0.0, 0.0, 0.0),
        }
    }
}
