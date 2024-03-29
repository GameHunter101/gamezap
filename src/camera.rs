use nalgebra as na;
use sdl2::{keyboard::Scancode, mouse::RelativeMouseState};

pub struct Camera {
    pub position: na::Vector3<f32>,
    pub screen_right: na::Unit<na::Vector3<f32>>,
    pub view_matrix: na::Matrix4<f32>,
    pub rotation_matrix: na::Matrix4<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub distance: f32,
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

        let affine_matrix = self.rotation_matrix * transform_matrix;
        self.view_matrix = affine_matrix;
    }

    fn update_rotation_matrix(&mut self) {
        let yaw_rotation_axis = na::Vector3::new(0.0, 1.0, 0.0);
        let yaw_matrix = na::Matrix4::new_rotation((yaw_rotation_axis * self.yaw).xyz());

        let pitch_rotation_axis = na::Vector3::new(1.0, 0.0, 0.0);

        let pitch_matrix = na::Matrix4::new_rotation((pitch_rotation_axis * self.pitch).xyz());

        self.rotation_matrix = pitch_matrix * yaw_matrix;
    }

    pub fn transform_camera(
        &mut self,
        scancodes: &Vec<Scancode>,
        mouse_state: &RelativeMouseState,
        relative_mouse: bool,
    ) {
        if scancodes.contains(&Scancode::W) {
            self.move_forward(self.distance);
        }
        if scancodes.contains(&Scancode::S) {
            self.move_backward(self.distance);
        }
        if scancodes.contains(&Scancode::D) {
            self.move_left(self.distance);
        }
        if scancodes.contains(&Scancode::A) {
            self.move_right(self.distance);
        }
        if scancodes.contains(&Scancode::Space) {
            self.move_up(self.distance);
        }
        if scancodes.contains(&Scancode::LCtrl) {
            self.move_down(self.distance);
        }

        if relative_mouse {
            self.rotate_yaw(mouse_state.x() as f32, self.sensitivity);
            self.rotate_pitch(mouse_state.y() as f32, self.sensitivity);
            self.update_rotation_matrix();
        }

        self.update_affine_matrix();
    }

    fn move_forward(&mut self, distance: f32) {
        self.position += (distance
            * self.rotation_matrix.try_inverse().unwrap()
            * na::Vector3::new(0.0, 0.0, 1.0).to_homogeneous())
        .xyz();
    }

    fn move_backward(&mut self, distance: f32) {
        self.move_forward(-distance);
    }

    fn move_right(&mut self, distance: f32) {
        self.position += (distance
            * self.rotation_matrix.try_inverse().unwrap()
            * na::Vector3::new(1.0, 0.0, 0.0).to_homogeneous())
        .xyz();
    }

    fn move_left(&mut self, distance: f32) {
        self.move_right(-distance);
    }

    fn move_up(&mut self, distance: f32) {
        self.move_down(-distance);
    }

    fn move_down(&mut self, distance: f32) {
        self.position += (distance
            * self.rotation_matrix.try_inverse().unwrap()
            * na::Vector3::new(0.0, 1.0, 0.0).to_homogeneous())
        .xyz();
    }

    fn rotate_pitch(&mut self, rotation: f32, sensitivity: f32) {
        self.pitch += rotation * sensitivity;

        // self.screen_down = na::Unit::new_normalize(
        //     na::UnitQuaternion::from_axis_angle(&self.screen_right, self.pitch)
        //         .transform_vector(&self.screen_down),
        // );
    }

    fn rotate_yaw(&mut self, rotation: f32, sensitivity: f32) {
        self.yaw += rotation * sensitivity;

        // self.screen_right = na::Unit::new_normalize(
        //     na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), self.yaw)
        //         .transform_vector(&self.screen_right),
        // );
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_pos: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        CameraUniform {
            view_pos: [0.0; 4],
            view_proj: na::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &mut Camera) {
        self.view_pos = camera.position.to_homogeneous().into();
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
