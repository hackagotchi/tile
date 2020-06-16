use iced_winit::winit;
use winit::{
    event::{Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod camera;
mod controls;
mod render;
use render::Scene;

fn main() {
    pretty_env_logger::init();

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(1024.0, 1024.0))
        .build(&event_loop)
        .unwrap();

    let mut modifiers = ModifiersState::default();

    // Since main can't be async, we're going to need to block
    let mut scene = Scene::new(&window);

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
                    WindowEvent::Resized(new_size) => {
                        scene.resize((new_size.width, new_size.height), &window);
                    }
                    _ => {}
                }

                // Map window event to iced event
                if let Some(event) =
                    iced_winit::conversion::window_event(&event, window.scale_factor(), modifiers)
                {
                    scene.controls.queue_event(event);
                }
            }
            Event::MainEventsCleared => {
                // We update the scene
                scene.update();

                // and request a redraw
                window.request_redraw();
            }
            Event::RedrawRequested(_) => scene.render(&window),
            _ => {}
        }
    });
}
