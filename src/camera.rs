use na::RealField;
use nalgebra as na;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    mouse::MouseState,
};

pub struct Camera {
    pub position: na::Vector3<f32>,
    pub screen_up: na::Unit<na::Vector3<f32>>,
    pub screen_right: na::Unit<na::Vector3<f32>>,
    pub view_matrix: na::Matrix4<f32>,
    pub rotation_matrix: na::Matrix3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub distance: f32,
    pub last_mouse_pos: Option<(i32, i32)>,
    pub sensitivity: f32,
}

impl Camera {
    fn build_view_projection_matrix(&mut self) -> na::Matrix4<f32> {
        let perspective = na::Perspective3::new(self.aspect, self.fovy, self.znear, self.zfar);
        let perspective_matrix = perspective.as_matrix();

        return perspective_matrix * self.view_matrix;
    }

    fn update_affine_matrix(&mut self) {
        let transform_matrix = na::Matrix4::from(na::Translation3::from(self.position));
        let rotation_matrix = self.rotation_matrix.to_homogeneous();

        let affine_matrix = rotation_matrix * transform_matrix;
        self.view_matrix = affine_matrix;
    }

    fn update_rotation_matrix(&mut self) {
        // This is supposed to be yaw, but this only works when the matrix models pitch
        let yaw_matrix = na::Matrix3::new(
            self.yaw.cos(),
            0.0,
            self.yaw.sin(),
            0.0,
            1.0,
            0.0,
            -self.yaw.sin(),
            0.0,
            self.yaw.cos(),
        );

        // This is supposed to be pitch, but this only works w hen the matrix models roll
        // I have no idea why this is the way it is, but it works so idk
        let pitch_matrix = na::Matrix3::new(
            1.0,
            0.0,
            0.0,
            0.0,
            self.pitch.cos(),
            -self.pitch.sin(),
            0.0,
            self.pitch.sin(),
            self.pitch.cos(),
        );

        self.rotation_matrix = yaw_matrix * pitch_matrix;
    }

    pub fn transform_camera(&mut self, scancodes: &Vec<Scancode>, mouse_state: &MouseState) {
        if scancodes.contains(&Scancode::W) {
            self.move_forward(self.distance);
        }
        if scancodes.contains(&Scancode::S) {
            self.move_backward(self.distance);
        }
        if scancodes.contains(&Scancode::D) {
            self.move_right(self.distance);
        }
        if scancodes.contains(&Scancode::A) {
            self.move_left(self.distance);
        }
        if scancodes.contains(&Scancode::Space) {
            self.move_up(self.distance);
        }
        if scancodes.contains(&Scancode::LCtrl) {
            self.move_down(self.distance);
        }

        let current_mouse_pos = (mouse_state.x(), mouse_state.y());

        if let Some(last_pos) = self.last_mouse_pos {
            let mouse_x_delta = current_mouse_pos.0 - last_pos.0;
            let mouse_y_delta = current_mouse_pos.1 - last_pos.1;
            self.rotate_yaw(mouse_x_delta as f32, self.sensitivity);
            self.rotate_pitch(mouse_y_delta as f32, self.sensitivity);
            self.update_rotation_matrix();
        }

        self.last_mouse_pos = Some(current_mouse_pos);

        self.update_affine_matrix();
    }

    fn move_forward(&mut self, distance: f32) {
        self.position += distance
            * self.rotation_matrix.try_inverse().unwrap()
            * na::Vector3::new(0.0, 0.0, 1.0);
    }

    fn move_backward(&mut self, distance: f32) {
        self.move_forward(-distance);
    }

    fn move_left(&mut self, distance: f32) {
        self.position += distance
            * self.rotation_matrix.try_inverse().unwrap()
            * na::Vector3::new(1.0, 0.0, 0.0);
    }

    fn move_right(&mut self, distance: f32) {
        self.move_left(-distance);
    }

    fn move_up(&mut self, distance: f32) {
        self.move_down(-distance);
    }

    fn move_down(&mut self, distance: f32) {
        self.position += distance
            * self.rotation_matrix.try_inverse().unwrap()
            * na::Vector3::new(0.0, 1.0, 0.0);
    }

    fn rotate_pitch(&mut self, rotation: f32, sensitivity: f32) {
        self.pitch += rotation * sensitivity;
    }

    fn rotate_yaw(&mut self, rotation: f32, sensitivity: f32) {
        self.yaw += rotation * sensitivity;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        CameraUniform {
            view_proj: na::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &mut Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
