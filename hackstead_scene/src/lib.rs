use hexa::{camera::Camera, Renderer, Scene, Sprite, Tile};
use hexa::{iced_wgpu, iced_winit};
use iced_wgpu::Primitive as GuiPrimitive;
use iced_winit::{mouse, program, winit};
use winit::event::{ModifiersState, WindowEvent};

mod controls;
use controls::Controls;

#[no_mangle]
pub fn _scene_init(r: &mut dyn Renderer) -> *mut dyn Scene {
    println!("initializing scene");
    Box::into_raw(Box::new(HacksteadScene::new(r)))
}

pub struct HacksteadScene {
    gui: program::State<Controls>,
    camera: Camera,
    first_frame: bool,
}
impl HacksteadScene {
    pub fn new(r: &mut dyn Renderer) -> Self {
        let screen = r.screen_size();
        let camera = Camera::new(screen.width as f32, screen.height as f32);
        let (render, debug) = r.iced_mut();
        let gui = program::State::new(Controls::new(), screen, render, debug);

        Self {
            gui,
            camera: camera.clone(),
            first_frame: true,
        }
    }
}
impl Scene for HacksteadScene {
    fn event(&mut self, event: &WindowEvent, scale_factor: f64, modifiers: ModifiersState) {
        match event {
            WindowEvent::Resized(new_size) => {
                self.camera
                    .resize(new_size.width as f32, new_size.height as f32);
            }
            _ => {}
        }

        // Map window event to iced event
        if let Some(event) = iced_winit::conversion::window_event(&event, scale_factor, modifiers) {
            self.gui.queue_event(event);
        }
    }

    fn gui_primitive(&self) -> &(GuiPrimitive, mouse::Interaction) {
        self.gui.primitive()
    }

    fn update(&mut self, renderer: &mut dyn Renderer) {
        let screen = renderer.screen_size();
        let (render, debug) = renderer.iced_mut();
        let _ = self.gui.update(None, screen, render, debug);

        if self.first_frame {
            renderer.set_sprites(vec![Sprite {
                image: 0,
                position: hexa::na::Vector2::new(8.0, 7.5),
                scale: hexa::na::Vector2::repeat(1.0),
            }]);

            self.first_frame = false;
        }

        let Controls {
            tiling_tab,
            camera_tab,
            ..
        } = self.gui.program();

        renderer.set_camera({
            self.camera.eye.z = camera_tab.height;
            self.camera.set_angle(camera_tab.angle, camera_tab.distance);
            self.camera.target =
                hexa::na::Point3::new(1.0, 1.0, 0.0) * (tiling_tab.size as f32 / 2.0 + 1.0);
            self.camera.fovy = camera_tab.fov;
            &self.camera
        });

        if tiling_tab.dirty {
            use noise::{NoiseFn, Seedable};

            let perlin = noise::Perlin::new().set_seed(tiling_tab.seed);
            let g = tiling_tab.size as f64;
            let e = tiling_tab.elevation as f32;

            renderer.set_tiles(
                (0..tiling_tab.size)
                    .flat_map(|x| (0..tiling_tab.size).filter_map(move |y| {
                        use hexa::na::Vector2 as Vec2;
                        let position = Vec2::new(x, y);
                        let noise = perlin.get([x as f64 / g, y as f64 / g]) as f32 * e;

                        let mut tiles = vec![Tile {
                            position,
                            elevation: 0.0,
                            hat: 2,
                            butt: 3,
                            butt_size: noise + 0.3,
                        }];

                        if noise > e / 2.0 {
                            tiles.push(Tile {
                                position,
                                elevation: 0.0,
                                hat: 0,
                                butt: 1,
                                butt_size: noise * (noise / 1.5) * 0.4,
                            });
                        }

                        if noise > 0.0 {
                            Some(tiles)
                        } else {
                            None
                        }
                    }))
                    .collect()
            );

            self.gui.queue_message(controls::Message::Tiling(
                controls::tiling::Message::Retiled,
            ));
        }
    }
}
