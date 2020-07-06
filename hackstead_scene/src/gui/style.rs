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
