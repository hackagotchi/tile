use iced_winit::winit;
use winit::{
    event::{Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod camera;
mod tiling;

mod render;
use render::Renderer;

mod scene;
use scene::Scene;

fn main() {
    pretty_env_logger::init();

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(1024.0, 1024.0))
        .build(&event_loop)
        .unwrap();

    let mut modifiers = ModifiersState::default();

    let (mut renderer, mut scene) = Renderer::new_with_scene(&window);

    event_loop.run(move |event, _, control_flow| {
        // You should change this if you want to render continuosly
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = *new_modifiers;
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        winit::event::KeyboardInput {
                            state: winit::event::ElementState::Pressed,
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                    &WindowEvent::Resized(new_size) => {
                        renderer.resize(new_size, &window);
                    }
                    _ => {}
                }

                scene.event(
                    event,
                    window.scale_factor(),
                    modifiers
                );
            }
            Event::MainEventsCleared => {
                scene.update(&mut renderer);

                // and request a redraw
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                renderer.render(&window, scene.gui_primitive())
            },
            _ => {}
        }
    });
}
