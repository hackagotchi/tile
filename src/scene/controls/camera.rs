use iced_wgpu::Renderer;
use iced_winit::{slider, Align, Column, Command, Element, Program, Slider, Text};

#[derive(Debug, Clone)]
pub enum Message {
    FovChanged(f32),
    HeightChanged(f32),
    AngleChanged(f32),
    DistanceChanged(f32),
}

#[derive(Default, Debug)]
struct Sliders {
    fov: slider::State,
    height: slider::State,
    angle: slider::State,
    distance: slider::State,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CameraControls {
    pub fov: f32,
    pub height: f32,
    pub angle: f32,
    pub distance: f32,
    #[serde(skip)]
    sliders: Sliders,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            fov: std::f32::consts::PI / 2.0,
            height: 3.0,
            angle: std::f32::consts::PI / 2.0,
            distance: 6.0,
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
            FovChanged(fov) => self.fov = fov,
            HeightChanged(height) => self.height = height,
            AngleChanged(angle) => self.angle = angle,
            DistanceChanged(distance) => self.distance = distance,
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        use std::f32::consts::PI;
        const TAU: f32 = PI * 2.0;

        let Sliders {
            fov,
            distance,
            height,
            angle,
        } = &mut self.sliders;

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
                Slider::new(fov, 0.0..=PI, self.fov, move |f| {
                    Message::FovChanged(f)
                }),
            ))
            .push(labeled_slider(
                "Height",
                Slider::new(height, 0.0..=50.0, self.height, move |h| {
                    Message::HeightChanged(h)
                }),
            ))
            .push(labeled_slider(
                "Angle",
                Slider::new(angle, 0.0..=TAU, self.angle, move |a| {
                    Message::AngleChanged(a)
                }),
            ))
            .push(labeled_slider(
                "Distance",
                Slider::new(distance, 0.0..=50.0, self.distance, move |d| {
                    Message::DistanceChanged(d)
                }),
            ))
            .into()
    }
}
