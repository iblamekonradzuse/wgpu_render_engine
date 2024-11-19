use cgmath::{perspective, Matrix4, Point3, Rad, Vector3, InnerSpace, Euler, Deg};
use winit::event::*;

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    pub yaw: f32,
    pub pitch: f32,
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_position: [f32; 4], // Change to [f32; 4] to ensure 16-byte alignment
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Point3::new(0.0, 1.0, 2.0),
            direction: Vector3::new(0.0, 0.0, -1.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            aspect: width as f32 / height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            yaw: -90.0, // Start facing negative Z
            pitch: 0.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> CameraUniform {
    let view = Matrix4::look_to_rh(self.position, self.direction, self.up);
    let proj = perspective(Rad(self.fovy.to_radians()), self.aspect, self.znear, self.zfar);
    CameraUniform {
        view_proj: (proj * view).into(),
        view_position: [self.position.x, self.position.y, self.position.z, 0.0], // Add 0.0 as the fourth component
    }
}

    pub fn update(&mut self, controller: &CameraController) {
        // Update direction based on mouse movement
        self.yaw += controller.rotate_horizontal;
        self.pitch += controller.rotate_vertical;

        // Clamp pitch to prevent camera flipping
        self.pitch = self.pitch.clamp(-89.0, 89.0);

        // Compute new direction vector
        let direction = Vector3::new(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos()
        ).normalize();
        self.direction = direction;

        // Compute camera right vector
        let right = direction.cross(self.up).normalize();
        
        // Update position based on movement
        self.position += self.direction * (controller.amount_forward - controller.amount_backward) * controller.speed;
        self.position += right * (controller.amount_right - controller.amount_left) * controller.speed;
        self.position.y += (controller.amount_up - controller.amount_down) * controller.speed;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}

pub struct CameraController {
    pub amount_left: f32,
    pub amount_right: f32,
    pub amount_forward: f32,
    pub amount_backward: f32,
    pub amount_up: f32,
    pub amount_down: f32,
    pub rotate_horizontal: f32,
    pub rotate_vertical: f32,
    pub scroll: f32,
    pub speed: f32,
    pub sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse_movement(&mut self, delta_x: f32, delta_y: f32) {
        self.rotate_horizontal = -delta_x * self.sensitivity;
        self.rotate_vertical = -delta_y * self.sensitivity;
    }

    pub fn reset_mouse_movement(&mut self) {
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }
}
