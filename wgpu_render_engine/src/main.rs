use winit::{
    event::{Event, WindowEvent, DeviceEvent, ElementState, MouseButton},
    event_loop::EventLoop,
    window::{WindowBuilder, CursorGrabMode},
};
mod renderer;
mod vertex;
mod camera;

use renderer::Renderer;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WGPU Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::new(&window).await;
    let mut mouse_pressed = false;

    event_loop.run(move |event, _, control_flow| {
    match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !renderer.input(event) {
                match event {
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size);
                    }
                    _ => {}
                }
            }
        }
        Event::DeviceEvent { event, .. } => {
            match event {
                DeviceEvent::MouseMotion { delta } => {
                    renderer.process_mouse_movement(
                        delta.0 as f32, 
                        delta.1 as f32
                    );
                }
                _ => {}
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            renderer.update();
            match renderer.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit(),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    }
});
}
