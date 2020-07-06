use hexa::iced_wgpu::Renderer;
use hexa::iced_winit::{Align, Command, Container, Element, Length, Program};

pub mod dev_controls;
use dev_controls::Controls;

mod style;
pub use style::DarkIce;

#[derive(Debug)]
pub enum Message {
    DevControls(dev_controls::Message),
}

#[derive(Default, Debug)]
pub struct Gui {
    pub dev_controls: Controls
}
impl Gui {
    pub fn new() -> Self {
        Self {
            dev_controls: Controls::new(),
        }
    }
}
impl Program for Gui {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        use Message::*;

        match message {
            DevControls(msg) => {
                self.dev_controls.update(msg);
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        Container::new(
            Container::new(
                    self
                        .dev_controls
                        .view()
                        .map(|msg| Message::DevControls(msg))
                )
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
