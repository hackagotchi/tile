use iced_wgpu::{wgpu, Backend, Renderer, Settings};
use iced_winit::{futures, winit, Debug};
use winit::{
    event::{Event, ModifiersState, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod camera;
mod controls;
mod rendering_state;
mod scene;
mod texture;
use rendering_state::RenderingState;
use scene::Scene;

fn main() {
    use futures::executor::block_on;

    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(1024.0, 1024.0))
        .build(&event_loop)
        .unwrap();

    let mut rs = RenderingState::new(&window);

    // Initialize iced
    let mut modifiers = ModifiersState::default();
    let mut debug = Debug::new();
    let mut renderer = Renderer::new(Backend::new(&mut rs.device, Settings::default()));

    // Since main can't be async, we're going to need to block
    let mut scene = block_on(Scene::new(&rs, &mut renderer, &mut debug));

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
                        rs.resize((new_size.width, new_size.height), &window);
                        scene.resize(&rs.swap_chain_descriptor, &rs.device);
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
                // We update iced
                let _ = scene.controls.update(
                    None,
                    rs.viewport.logical_size(),
                    &mut renderer,
                    &mut debug,
                );

                // and request a redraw
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let frame = rs
                    .swap_chain
                    .get_next_texture()
                    .expect("Timeout getting texture");

                let mut encoder = rs
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                // We draw the scene first
                // let program = scene.controls.program();
                scene.update(&mut encoder, &rs);
                scene.render(&mut encoder, &frame.view);

                // then iced on top
                let mouse_interaction = renderer.backend_mut().draw(
                    &mut rs.device,
                    &mut encoder,
                    &frame.view,
                    &rs.viewport,
                    scene.controls.primitive(),
                    &debug.overlay(),
                );

                rs.queue.submit(&[encoder.finish()]);

                // And update the mouse cursor
                window
                    .set_cursor_icon(iced_winit::conversion::mouse_interaction(mouse_interaction));
            }
            _ => {}
        }
    });
}
