use iced_wgpu::Renderer;
use iced_winit::{slider, Align, Column, Command, Element, Program, Slider, Text};

#[derive(Debug, Clone)]
pub enum Message {
    ElevationChanged(f32),
    SizeChanged(u32),
    SeedChanged(u32),
    Retiled,
}

fn dirty_default() -> bool {
    true
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TilingControls {
    #[serde(skip, default = "dirty_default")]
    pub dirty: bool,
    #[serde(skip)]
    sliders: Sliders,
    pub data: Data,
}
impl Default for TilingControls {
    fn default() -> Self {
        Self {
            dirty: true,
            sliders: Default::default(),
            data: Default::default(),
        }
    }
}
impl std::ops::Deref for TilingControls {
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Default, Debug)]
struct Sliders {
    elevation: slider::State,
    size: slider::State,
    seed: slider::State,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Data {
    pub elevation: f32,
    pub size: u32,
    pub seed: u32,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            elevation: 0.1,
            size: 5,
            seed: 42,
        }
    }
}

impl Program for TilingControls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        use Message::*;

        match message {
            ElevationChanged(e) => {
                self.data.elevation = e;
                self.dirty = true;
            },
            SizeChanged(size) => {
                self.data.size = size;
                self.dirty = true;
            }
            SeedChanged(seed) => {
                self.data.seed = seed;
                self.dirty = true;
            }
            Retiled => {
                self.dirty = false;
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let Sliders { elevation, size, seed } = &mut self.sliders;
        let data = self.data.clone();

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
                "Elevation",
                Slider::new(elevation, 0.0..=3.0, data.elevation, move |elevation| {
                    Message::ElevationChanged(elevation)
                }),
            ))
            .push(labeled_slider(
                "Size",
                Slider::new(size, 1.0..=15.0, data.size as f32, move |size| {
                    Message::SizeChanged(size as u32)
                }),
            ))
            .push(labeled_slider(
                "Seed",
                Slider::new(seed, 1.0..=u32::MAX as f32, data.seed as f32, move |seed| {
                    Message::SeedChanged(seed as u32)
                }),
            ))
            .into()
    }
}
