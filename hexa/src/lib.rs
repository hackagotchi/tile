pub use iced_wgpu;
pub use iced_winit;
pub use nalgebra as na;

pub use iced_wgpu::Renderer as IcedRenderer;
pub use iced_winit::Debug as IcedDebug;
use iced_wgpu::Primitive as GuiPrimitive;
use iced_winit::{winit, mouse, Size};
use winit::event::{WindowEvent, ModifiersState};

pub mod camera;
pub use camera::Camera;

pub struct Tile {
    pub position: nalgebra::Vector2<u32>,
    pub elevation: f32,
    pub butt_size: f32,
    pub hat: u32,
    pub butt: u32,
}

pub struct Sprite {
    pub image: u32,
    pub position: nalgebra::Vector2<f32>,
    pub scale: nalgebra::Vector2<f32>,
}

/// This trait specifies the methods that Scenes have access to.
pub trait Renderer {
    fn screen_size(&self) -> Size;
    fn set_tiles(&mut self, tiles: Vec<Vec<Tile>>);
    fn set_sprites(&mut self, sprites: Vec<Sprite>);
    fn set_camera(&mut self, camera: &Camera);
    fn iced_mut(&mut self) -> (&mut IcedRenderer, &mut IcedDebug);
}

pub trait Scene {
    fn event(
        &mut self,
        event: &WindowEvent,
        scale_factor: f64,
        modifiers: ModifiersState,
    );

    fn gui_primitive(&self) -> &(GuiPrimitive, mouse::Interaction);

    fn update(&mut self, renderer: &mut dyn Renderer);
}
