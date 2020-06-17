use crate::{controls::{self, Controls}, camera::Camera};
use iced_wgpu::{wgpu, Renderer};
use iced_winit::{winit, program, Debug};
use winit::window::Window;

mod pipeline;
mod texture;
mod rendering_state;
mod multisampled_framebuffer;

pub use rendering_state::RenderingState;
use multisampled_framebuffer::MultisampledFramebuffer;
use pipeline::{WorldPipeline, NoSrgbPipeline};

pub struct Scene {
    main_pipeline: WorldPipeline,
    no_srgb_pipeline: NoSrgbPipeline,
    rs: RenderingState,
    renderer: Renderer,
    debug: Debug,
    pub controls: program::State<Controls>,
}
impl Scene {
    pub fn new(window: &Window) -> Self {
        use iced_wgpu::{Backend, Settings};

        let mut rs = RenderingState::new(&window);

        // Initialize iced
        let mut debug = Debug::new();
        let mut renderer = Renderer::new(Backend::new(&mut rs.device, Settings::default()));

        let camera = Camera::new(
            rs.swap_chain_descriptor.width as f32,
            rs.swap_chain_descriptor.height as f32,
        );
        let mut controls = program::State::new(
            Controls::new(camera),
            rs.viewport.logical_size(),
            &mut renderer,
            &mut debug,
        );
        controls.queue_message(
            controls::Message::Tiling(
                controls::tiling::Message::SizeChanged(
                    controls.program().tiling_tab.size
                )
            )
        );

        let multisampled_framebuffer = MultisampledFramebuffer::new(
            &rs.device,
            &rs.swap_chain_descriptor,
            pipeline::world::SAMPLES
        );

        let main_pipeline = WorldPipeline::new(
            &rs,
            controls.program(),
            multisampled_framebuffer.texture_view
        );
        let no_srgb_pipeline = NoSrgbPipeline::new(
            &rs,
            multisampled_framebuffer.no_srgb_texture_view
        );

        Self {
            no_srgb_pipeline,
            main_pipeline,
            controls,
            rs,
            renderer,
            debug
        }
    }

    pub fn resize(
        &mut self,
        (w, h): (u32, u32),
        window: &Window
    ) {
        self.rs.resize((w, h), &window);

        self.controls.queue_message(
            controls::Message::Resize(w, h)
        );

        self.main_pipeline.resize(&self.rs.swap_chain_descriptor, &self.rs.device);

        let multisampled_framebuffer = MultisampledFramebuffer::new(
            &self.rs.device,
            &self.rs.swap_chain_descriptor,
            pipeline::world::SAMPLES
        );
        self.no_srgb_pipeline.resize(
            multisampled_framebuffer.no_srgb_texture_view,
            &self.rs
        );
        self.main_pipeline.framebuffer = multisampled_framebuffer.texture_view;
    }

    pub fn update(&mut self) {
        let mut encoder = self
            .rs
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let _ = self.controls.update(
            None,
            self.rs.viewport.logical_size(),
            &mut self.renderer,
            &mut self.debug,
        );

        self.main_pipeline.update(&mut encoder, &self.rs, &mut self.controls);

        self.rs.queue.submit(&[encoder.finish()]);
    }

    pub fn render(&mut self, window: &Window) {
        let frame = self
            .rs
            .swap_chain
            .get_next_texture()
            .expect("Timeout getting texture");

        let mut encoder = self
            .rs
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.main_pipeline.render(&mut encoder, &self.no_srgb_pipeline.no_srgb_framebuffer);
        self.no_srgb_pipeline.render(&mut encoder, &frame.view);

        // And update the mouse cursor
        window.set_cursor_icon(iced_winit::conversion::mouse_interaction(
            self
                .renderer
                .backend_mut()
                .draw(
                    &mut self.rs.device,
                    &mut encoder,
                    &frame.view,
                    &self.rs.viewport,
                    self.controls.primitive(),
                    &self.debug.overlay(),
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
