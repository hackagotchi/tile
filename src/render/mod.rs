use crate::camera::Camera;
use crate::tiling::Tile;
use iced_wgpu::{wgpu, Renderer as IcedRenderer, Primitive as GuiPrimitive};
use iced_winit::{winit, Debug as IcedDebug, mouse, Size};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::Scene;

mod pipeline;
mod texture;
mod rendering_state;
mod multisampled_framebuffer;

pub use rendering_state::RenderingState;
use multisampled_framebuffer::MultisampledFramebuffer;
use pipeline::{WorldPipeline, NoSrgbPipeline};

pub struct Config {
    pub msaa: u32,
    pub camera: Camera,
}

pub struct Renderer {
    config: Config,
    main_pipeline: WorldPipeline,
    no_srgb_pipeline: NoSrgbPipeline,
    rs: RenderingState,
    pub iced_renderer: IcedRenderer,
    pub iced_debug: IcedDebug,
}
impl Renderer {
    pub fn new_with_scene(window: &Window) -> (Self, Scene) {
        use iced_wgpu::{Backend, Settings};

        let mut rs = RenderingState::new(&window);
        let screen = rs.viewport.logical_size();

        let mut iced_debug = IcedDebug::new();
        let mut iced_renderer = IcedRenderer::new(Backend::new(&mut rs.device, Settings::default()));

        let (scene, config) = Scene::new(
            screen,
            &mut iced_renderer,
            &mut iced_debug,
        );

        let multisampled_framebuffer = MultisampledFramebuffer::new(
            &rs.device,
            &rs.swap_chain_descriptor,
            config.msaa,
        );

        let main_pipeline = WorldPipeline::new(
            &rs,
            multisampled_framebuffer.texture_view,
            &config.camera,
            &config
        );
        let no_srgb_pipeline = NoSrgbPipeline::new(
            &rs,
            multisampled_framebuffer.no_srgb_texture_view
        );

        (
            Self {
                no_srgb_pipeline,
                main_pipeline,
                rs,
                config,
                iced_renderer,
                iced_debug
            },
            scene
        )
    }

    pub fn screen_size(&self) -> Size {
        self.rs.viewport.logical_size()
    }

    pub fn resize(
        &mut self,
        screen: PhysicalSize<u32>,
        window: &Window
    ) {
        self.rs.resize(screen, &window);

        self.main_pipeline.resize(
            &self.rs.swap_chain_descriptor,
            &self.rs.device,
            &self.config
        );

        let multisampled_framebuffer = MultisampledFramebuffer::new(
            &self.rs.device,
            &self.rs.swap_chain_descriptor,
            self.config.msaa
        );
        self.no_srgb_pipeline.resize(
            multisampled_framebuffer.no_srgb_texture_view,
            &self.rs
        );
        self.main_pipeline.framebuffer = multisampled_framebuffer.texture_view;
    }

    pub fn set_tiles(&mut self, tiles: Vec<Tile>) {
        let mut encoder = self
            .rs
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.main_pipeline.set_tiles(&mut encoder, &self.rs, tiles);

        self.rs.queue.submit(&[encoder.finish()]);
    }

    pub fn set_camera(&mut self, camera: &Camera) {
        let mut encoder = self
            .rs
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.main_pipeline.set_camera(&mut encoder, &self.rs, camera);

        self.rs.queue.submit(&[encoder.finish()]);
    }

    pub fn render(&mut self, window: &Window, gui: &(GuiPrimitive, mouse::Interaction)) {
        let frame = self
            .rs
            .swap_chain
            .get_next_texture()
            .expect("Timeout getting texture");

        let mut encoder = self
            .rs
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.main_pipeline.render(
            &mut encoder,
            &self.no_srgb_pipeline.no_srgb_framebuffer,
            &self.config
        );
        self.no_srgb_pipeline.render(&mut encoder, &frame.view);

        // And update the mouse cursor
        window.set_cursor_icon(iced_winit::conversion::mouse_interaction(
            self
                .iced_renderer
                .backend_mut()
                .draw(
                    &mut self.rs.device,
                    &mut encoder,
                    &frame.view,
                    &self.rs.viewport,
                    gui,
                    &self.iced_debug.overlay(),
                )
        ));

        self.rs.queue.submit(&[encoder.finish()]);
    }
}

fn compile_shaders(
    (vs_src, vs_lbl): (&str, &str),
    (fs_src, fs_lbl): (&str, &str),
    rs: &RenderingState,
) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    let mut compiler = shaderc::Compiler::new().unwrap();
    let vs_spirv = compiler
        .compile_into_spirv(
            vs_src,
            shaderc::ShaderKind::Vertex,
            vs_lbl,
            "main",
            None,
        )
        .unwrap_or_else(|e| panic!("couldn't compile vertex shader: {}", e));
    let fs_spirv = compiler
        .compile_into_spirv(
            fs_src,
            shaderc::ShaderKind::Fragment,
            fs_lbl,
            "main",
            None,
        )
        .unwrap_or_else(|e| panic!("couldn't compile fragment shader: {}", e));

    let vs_data = wgpu::read_spirv(std::io::Cursor::new(vs_spirv.as_binary_u8())).unwrap();
    let fs_data = wgpu::read_spirv(std::io::Cursor::new(fs_spirv.as_binary_u8())).unwrap();

    (
        rs.device.create_shader_module(&vs_data),
        rs.device.create_shader_module(&fs_data)
    )
}
