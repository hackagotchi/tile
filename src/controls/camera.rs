use crate::camera::Camera;
use iced_wgpu::Renderer;
use iced_winit::{slider, Align, Column, Command, Element, Program, Slider, Text};

#[derive(Debug, Clone)]
pub enum Message {
    FovChanged(f32),
    HeightChanged(f32),
    AngleChanged(f32),
    DistanceChanged(f32),
    Resize(u32, u32),
}

#[derive(Default)]
struct Sliders {
    fov: slider::State,
    height: slider::State,
    angle: slider::State,
    distance: slider::State,
}

pub struct CameraControls {
    pub camera: Camera,
    angle: f32,
    distance: f32,
    sliders: Sliders,
}

impl CameraControls {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            angle: Camera::DEFAULT_ANGLE,
            distance: Camera::DEFAULT_DISTANCE,
            sliders: Default::default(),
        }
    }
}

impl Program for CameraControls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        use Message::*;

        match message {
            FovChanged(fov) => self.camera.fovy = fov,
            HeightChanged(height) => self.camera.eye.z = height,
            AngleChanged(angle) => {
                self.angle = angle;
                self.camera.set_angle(angle, self.distance);
            }
            DistanceChanged(distance) => {
                self.distance = distance;
                self.camera.set_angle(self.angle, distance);
            }
            Resize(w, h) => self.camera.resize(w as f32, h as f32),
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        use std::f32::consts::PI;

        let Self {
            camera,
            distance,
            angle,
            sliders,
        } = self;
        const TAU: f32 = PI * 2.0;

        let labeled_slider = |label, slider| {
            Column::new()
                .spacing(2)
                .align_items(Align::Center)
                .push(Text::new(label).size(20))
                .push(slider)
        };

        Column::new()
            .spacing(25)
            .padding(10)
            .push(labeled_slider(
                "FOV",
                Slider::new(&mut sliders.fov, 0.0..=TAU, camera.fovy, move |f| {
                    Message::FovChanged(f)
                }),
            ))
            .push(labeled_slider(
                "Height",
                Slider::new(&mut sliders.height, 0.0..=20.0, camera.eye.z, move |h| {
                    Message::HeightChanged(h)
                }),
            ))
            .push(labeled_slider(
                "Angle",
                Slider::new(&mut sliders.angle, 0.0..=TAU, *angle, move |a| {
                    Message::AngleChanged(a)
                }),
            ))
            .push(labeled_slider(
                "Distance",
                Slider::new(&mut sliders.distance, 0.0..=50.0, *distance, move |d| {
                    Message::DistanceChanged(d)
                }),
            ))
            .into()
    }
}
