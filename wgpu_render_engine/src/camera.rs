use cgmath::{perspective, Matrix4, Point3, Rad, Vector3, InnerSpace};
use winit::event::*;

#[derive(Debug)]
pub struct Camera {
    position: Point3<f32>,
    direction: Vector3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
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
        }
    }

    pub fn build_view_projection_matrix(&self) -> CameraUniform {
        let view = Matrix4::look_to_rh(self.position, self.direction, self.up);
        let proj = perspective(Rad(self.fovy.to_radians()), self.aspect, self.znear, self.zfar);
        CameraUniform {
            view_proj: (proj * view).into(),
        }
    }

    pub fn update(&mut self, controller: &CameraController) {
        // Update position based on movement
        let forward = self.direction.normalize();
        let right = forward.cross(self.up).normalize();
        
        self.position += forward * (controller.amount_forward - controller.amount_backward) * controller.speed;
        self.position += right * (controller.amount_right - controller.amount_left) * controller.speed;
        self.position.y += (controller.amount_up - controller.amount_down) * controller.speed;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}

pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
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
}
