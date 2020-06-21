use hexa::iced_wgpu::Renderer;
use hexa::iced_winit::{
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
    Camera(camera::Message),
    Tiling(tiling::Message),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Tab {
    Home,
    Camera,
    Tiling,
    Save,
}
impl Tab {
    const ALL: &'static [Tab] = &[Tab::Home, Tab::Camera, Tab::Tiling, Tab::Save];
}
impl Default for Tab {
    fn default() -> Self {
        Tab::Home
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
pub struct Controls {
    #[serde(skip)]
    pub tab: Tab,
    #[serde(skip)]
    pub tab_buttons: HashMap<Tab, button::State>,
    #[serde(skip)]
    pub home_button: button::State,
    pub camera_tab: CameraControls,
    pub tiling_tab: TilingControls,
}

impl Controls {
    pub fn new() -> Controls {
        dbg!(Self {
            tab: Tab::Home,
            tab_buttons: Tab::ALL
                .iter()
                .filter(|&&t| t != Tab::Home)
                .map(|t| (*t, Default::default()))
                .collect(),
            ..std::fs::read_to_string("save.json")
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| {
                    log::error!("no save found");
                    Default::default()
                })
        })
    }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        use Message::*;

        match message {
            SetTab(t) => self.tab = match t {
                Tab::Save => {
                    std::fs::write(
                        "save.json",
                        serde_json::to_string_pretty(self).expect("self not savaeable")
                    )
                    .expect("couldn't save");

                    Tab::Home
                },
                other => other,
            },
            Camera(msg) => { self.camera_tab.update(msg); },
            Tiling(msg) => { self.tiling_tab.update(msg); },
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
                Tab::Camera => camera_tab.view().map(|msg| Message::Camera(msg)),
                Tab::Tiling => tiling_tab.view().map(|msg| Message::Tiling(msg)),
                // defaults to showing the home tab
                _ => {
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
    use hexa::iced_wgpu::{button, container};
    use hexa::iced_winit::Color;

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
