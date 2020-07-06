use hexa::iced_wgpu::Renderer;
use hexa::iced_winit::{
    button, Align, Button, Column, Command, Container, Element, HorizontalAlignment, Length,
    Program, Row, Text,
};
use std::collections::HashMap;

mod camera;
use camera::CameraControls;

pub mod tiling;
use tiling::TilingControls;

pub use super::style::DarkIce;

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
        Self {
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
        }
    }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        use Message::*;

        match message {
            SetTab(t) => {
                self.tab = match t {
                    Tab::Save => {
                        std::fs::write(
                            "save.json",
                            serde_json::to_string_pretty(self).expect("self not savaeable"),
                        )
                        .expect("couldn't save");

                        Tab::Home
                    }
                    other => other,
                }
            }
            Camera(msg) => {
                self.camera_tab.update(msg);
            }
            Tiling(msg) => {
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

        Column::new()
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
            })
            .into()
    }
}

