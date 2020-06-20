use iced_wgpu::{Renderer as IcedRenderer, Primitive as GuiPrimitive};
use iced_winit::{program, winit, Debug as IcedDebug, mouse, Size};
use winit::event::{WindowEvent, ModifiersState};
use crate::{camera, sprite, tiling, render};
use crate::Renderer;
use camera::Camera;
use tiling::Tile;
use sprite::Sprite;

mod controls;
use controls::Controls;

pub struct Scene {
    gui: program::State<Controls>,
    camera: Camera,
    first_frame: bool,
}
impl Scene {
    pub fn new(
        screen: Size,
        iced_renderer: &mut IcedRenderer,
        iced_debug: &mut IcedDebug,
    ) -> (Self, render::Config) {
        let camera = Camera::new(screen.width as f32, screen.height as f32);
        let gui = program::State::new(
            Controls::new(),
            screen,
            iced_renderer,
            iced_debug,
        );

        (
            Self {
                gui,
                camera: camera.clone(),
                first_frame: true,
            },
            render::Config {
                camera,
                msaa: 16,
            }
        )
    }

    pub fn event(
        &mut self,
        event: &WindowEvent,
        scale_factor: f64,
        modifiers: ModifiersState,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                self.camera.resize(
                    new_size.width as f32,
                    new_size.height as f32
                );
            }
            _ => {}
        }

        // Map window event to iced event
        if let Some(event) =
            iced_winit::conversion::window_event(&event, scale_factor, modifiers)
        {
            self.gui.queue_event(event);
        }
    }

    pub fn gui_primitive(&self) -> &(GuiPrimitive, mouse::Interaction) {
        self.gui.primitive()
    }

    pub fn update(&mut self, renderer: &mut Renderer) {
        let _ = self.gui.update(
            None,
            renderer.screen_size(),
            &mut renderer.iced_renderer,
            &mut renderer.iced_debug,
        );

        if self.first_frame {

            renderer.set_sprites(vec![Sprite {
                image: 0,
                position: nalgebra::Vector2::new(8.0, 7.5),
                scale: nalgebra::Vector2::repeat(1.0),
            }]);

            self.first_frame = false;
        }

        let Controls { tiling_tab, camera_tab, .. } = self.gui.program();

        renderer.set_camera({
            self.camera.eye.z = camera_tab.height;
            self.camera.set_angle(camera_tab.angle, camera_tab.distance);
            self.camera.target = nalgebra::Point3::new(1.0, 1.0, 0.0)
                * (tiling_tab.size as f32 / 2.0 + 1.0);
            self.camera.fovy = camera_tab.fov;
            &self.camera
        });

        if tiling_tab.dirty {
            use noise::{Seedable, NoiseFn};

            let perlin = noise::Perlin::new().set_seed(tiling_tab.seed);
            let g = tiling_tab.size as f64;
            let e = tiling_tab.elevation as f32;

            renderer.set_tiles(
                (0..tiling_tab.size)
                    .flat_map(|x| (0..tiling_tab.size).filter_map(move |y| {
                        use nalgebra::Vector2 as Vec2;
                        let position = Vec2::new(x, y);
                        let elevation = perlin.get([x as f64 / g, y as f64 / g]) as f32 * e;
                        let distance = (Vec2::new(x as f64, y as f64) - Vec2::repeat(g/2.0)).magnitude();

                        if distance < g / 4.0 {
                            Some(Tile {
                                position,
                                elevation: elevation + 0.9,
                                hat: 2,
                                butt: 3,
                                butt_size: 1.7,
                            })
                        } else if distance < g / 2.0 {
                            Some(Tile {
                                position,
                                elevation,
                                hat: 0,
                                butt: 1,
                                butt_size: 1.0,
                            })
                        } else {
                            None
                        }
                    }))
                    .collect()
            );

            self.gui.queue_message(
                controls::Message::Tiling(controls::tiling::Message::Retiled)
            );
        }
    }
}
