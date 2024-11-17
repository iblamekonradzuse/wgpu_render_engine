use wgpu::util::DeviceExt;
use winit::window::Window;
use winit::event::*;
use cgmath::{Matrix4, Deg, SquareMatrix};

use crate::camera::{Camera, CameraController};
use crate::vertex::Vertex;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TransformUniform {
    model: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    position: [f32; 3],
    _padding1: u32,
    color: [f32; 3],
    ambient: f32,
    diffuse: f32,
    specular: f32,
    _padding2: [u32; 2],
}

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    camera: Camera,
    camera_controller: CameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    rotation: f32,
    transform_buffer: wgpu::Buffer,
    transform_bind_group: wgpu::BindGroup,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let camera = Camera::new(config.width, config.height);
        let camera_controller = CameraController::new(0.2, 0.4);
        let camera_uniform = camera.build_view_projection_matrix();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let transform_uniform = TransformUniform {
            model: Matrix4::identity().into(),
        };

        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[transform_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let transform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Transform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Transform Bind Group"),
            layout: &transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        });

        // Create light uniform and buffer
        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding1: 0,
            color: [1.0, 1.0, 1.0],
            ambient: 0.1,
            diffuse: 0.5,
            specular: 0.5,
            _padding2: [0; 2],
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &transform_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let vertices = [
    // Front face
    Vertex {
        position: [0.0, 0.866, 0.0],    // Top vertex
        color: [1.0, 0.0, 0.0],         // Red
        normal: [0.0, 0.5, 1.0],
    },
    Vertex {
        position: [-0.5, -0.288, 0.5],  // Bottom left
        color: [0.0, 1.0, 0.0],         // Green
        normal: [0.0, 0.5, 1.0],
    },
    Vertex {
        position: [0.5, -0.288, 0.5],   // Bottom right
        color: [0.0, 0.0, 1.0],         // Blue
        normal: [0.0, 0.5, 1.0],
    },
    
    // Left face
    Vertex {
        position: [0.0, 0.866, 0.0],    // Top vertex
        color: [1.0, 0.0, 0.0],         // Red
        normal: [-1.0, 0.5, 0.0],
    },
    Vertex {
        position: [-0.5, -0.288, -0.5], // Back left
        color: [0.0, 1.0, 0.0],         // Green
        normal: [-1.0, 0.5, 0.0],
    },
    Vertex {
        position: [-0.5, -0.288, 0.5],  // Front left
        color: [0.0, 0.0, 1.0],         // Blue
        normal: [-1.0, 0.5, 0.0],
    },

    // Right face
    Vertex {
        position: [0.0, 0.866, 0.0],    // Top vertex
        color: [1.0, 0.0, 0.0],         // Red
        normal: [1.0, 0.5, 0.0],
    },
    Vertex {
        position: [0.5, -0.288, 0.5],   // Front right
        color: [0.0, 1.0, 0.0],         // Green
        normal: [1.0, 0.5, 0.0],
    },
    Vertex {
        position: [0.5, -0.288, -0.5],  // Back right
        color: [0.0, 0.0, 1.0],         // Blue
        normal: [1.0, 0.5, 0.0],
    },

    // Back face
    Vertex {
        position: [0.0, 0.866, 0.0],    // Top vertex
        color: [1.0, 0.0, 0.0],         // Red
        normal: [0.0, 0.5, -1.0],
    },
    Vertex {
        position: [0.5, -0.288, -0.5],  // Back right
        color: [0.0, 1.0, 0.0],         // Green
        normal: [0.0, 0.5, -1.0],
    },
    Vertex {
        position: [-0.5, -0.288, -0.5], // Back left
        color: [0.0, 0.0, 1.0],         // Blue
        normal: [0.0, 0.5, -1.0],
    },
];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            camera,
            camera_controller,
            camera_buffer,
            camera_bind_group,
            rotation: 0.0,
            transform_buffer,
            transform_bind_group,
            light_buffer,
            light_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.resize(new_size.width, new_size.height);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::KeyboardInput {
            input: KeyboardInput {
                state,
                virtual_keycode: Some(key),
                ..
            },
            ..
        } = event {
            return self.camera_controller.process_keyboard(*key, *state);
        }
        false
    }

    pub fn update(&mut self) {
    self.camera.update(&self.camera_controller);
    let camera_uniform = self.camera.build_view_projection_matrix();
    self.queue.write_buffer(
        &self.camera_buffer,
        0,
        bytemuck::cast_slice(&[camera_uniform]),
    );

      self.rotation += 0.02;
    let model = Matrix4::from_angle_x(Deg(self.rotation * 0.7)) * 
                Matrix4::from_angle_y(Deg(self.rotation)) *
                Matrix4::from_angle_z(Deg(self.rotation * 0.3));
    
    let transform_uniform = TransformUniform {
        model: model.into(),
    };
    self.queue.write_buffer(
        &self.transform_buffer,
        0,
        bytemuck::cast_slice(&[transform_uniform]),
    );
}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.transform_bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..12, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
