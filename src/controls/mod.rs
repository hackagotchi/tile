use crate::camera::Camera;
use iced_wgpu::Renderer;
use iced_winit::{
    button, Align, Button, Column, Command, Container, Element, HorizontalAlignment, Length,
    Program, Row, Text,
};
use std::collections::HashMap;

mod camera;
use camera::CameraControls;

pub mod tiling;
pub use style::DarkIce;

use tiling::TilingControls;

#[derive(Debug, Clone)]
pub enum Message {
    SetTab(Tab),
    Resize(u32, u32),
    Camera(camera::Message),
    Tiling(tiling::Message),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Tab {
    Home,
    Camera,
    Tiling,
}
impl Tab {
    const ALL: &'static [Tab] = &[Tab::Home, Tab::Camera, Tab::Tiling];
}

pub struct Controls {
    pub tab: Tab,
    pub tab_buttons: HashMap<Tab, button::State>,
    pub home_button: button::State,
    pub camera_tab: CameraControls,
    pub tiling_tab: TilingControls,
}

impl Controls {
    pub fn new(camera: Camera) -> Controls {
        Self {
            tab: Tab::Home,
            camera_tab: CameraControls::new(camera),
            tiling_tab: Default::default(),
            home_button: Default::default(),
            tab_buttons: Tab::ALL
                .iter()
                .filter(|&&t| t != Tab::Home)
                .map(|t| (*t, Default::default()))
                .collect(),
        }
    }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        use Message::*;

        match message {
            SetTab(t) => self.tab = t,
            Resize(w, h) => {
                self.camera_tab.update(camera::Message::Resize(w, h));
            }
            Camera(msg) => {
                self.camera_tab.update(msg);
            }
            Tiling(msg) => {
                match msg {
                    tiling::Message::SizeChanged(size) => {
                        self.camera_tab.camera.target = nalgebra::Point3::new(
                            size as f32 / 2.0 + 1.0,
                            size as f32 / 2.0 + 1.0,
                            0.0,
                        );
                    }
                    _ => {},
                };
                self.tiling_tab.update(msg);
            }
        };

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let Self {
            camera_tab,
            tiling_tab,
            home_button,
            tab_buttons,
            tab,
        } = self;

        let content = Column::new()
            .spacing(35)
            .push({
                let mut r =
                    Row::new().push(Text::new(format!("{:?}", tab).to_uppercase()).size(30));

                if *tab != Tab::Home {
                    let go_home = Button::new(home_button, Text::new("Home").size(18))
                        .padding(5)
                        .on_press(Message::SetTab(Tab::Home))
                        .style(DarkIce);

                    r = r.push(
                        Container::new(go_home)
                            .width(Length::Fill)
                            .align_x(Align::End),
                    );
                }

                r
            })
            .push(match tab {
                Tab::Home => {
                    let mut c = Column::new().spacing(20);
                    for (tab, button) in tab_buttons.iter_mut() {
                        c = c.push(
                            Button::new(
                                button,
                                Text::new(format!("{:?}", tab).to_uppercase())
                                    .width(Length::Fill)
                                    .horizontal_alignment(HorizontalAlignment::Center)
                                    .size(25),
                            )
                            .width(Length::Fill)
                            .on_press(Message::SetTab(*tab))
                            .style(DarkIce),
                        );
                    }

                    c.into()
                }
                Tab::Camera => camera_tab.view().map(|msg| Message::Camera(msg)),
                Tab::Tiling => tiling_tab.view().map(|msg| Message::Tiling(msg)),
            });

        Container::new(
            Container::new(content)
                .padding(35)
                .width(Length::Units(320))
                .height(Length::Shrink)
                .style(DarkIce),
        )
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Align::End)
        .align_y(Align::End)
        .into()
    }
}

mod style {
    use iced_wgpu::{button, container};
    use iced_winit::Color;

    pub struct DarkIce;

    impl container::StyleSheet for DarkIce {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(
                    Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 0.75,
                    }
                    .into(),
                ),
                border_width: 5,
                border_radius: 15,
                border_color: Color {
                    r: 0.05,
                    g: 0.1,
                    b: 0.15,
                    a: 0.5,
                },
                text_color: Some(Color {
                    r: 0.94,
                    g: 0.94,
                    b: 1.00,
                    a: 1.00,
                }),
            }
        }
    }
    impl button::StyleSheet for DarkIce {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(
                    Color {
                        r: 0.203,
                        g: 0.745,
                        b: 0.356,
                        a: 0.40,
                    }
                    .into(),
                ),
                text_color: Color {
                    r: 0.94,
                    g: 0.94,
                    b: 1.00,
                    a: 1.00,
                },
                border_radius: 3,
                ..Default::default()
            }
        }
    }
}
