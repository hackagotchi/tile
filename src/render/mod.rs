use crate::{controls::{self, Controls}, camera::Camera};
use iced_wgpu::{wgpu, Renderer};
use iced_winit::{program, Debug};

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
    pub controls: program::State<Controls>,
}
impl Scene {
    pub fn new(rs: &RenderingState, renderer: &mut Renderer, debug: &mut Debug) -> Self {
        let camera = Camera::new(
            rs.swap_chain_descriptor.width as f32,
            rs.swap_chain_descriptor.height as f32,
        );
        let mut controls = program::State::new(
            Controls::new(camera),
            rs.viewport.logical_size(),
            renderer,
            debug,
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
            rs,
            controls.program(),
            multisampled_framebuffer.texture_view
        );
        let no_srgb_pipeline = NoSrgbPipeline::new(
            rs,
            multisampled_framebuffer.no_srgb_texture_view
        );

        Self {
            no_srgb_pipeline,
            main_pipeline,
            controls
        }
    }

    pub fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) {
        self.controls.queue_message(
            controls::Message::Resize(sc_desc.width, sc_desc.height)
        );

        self.main_pipeline.resize(sc_desc, device);

        let multisampled_framebuffer = MultisampledFramebuffer::new(
            device,
            sc_desc,
            pipeline::world::SAMPLES
        );
        self.no_srgb_pipeline.no_srgb_framebuffer = multisampled_framebuffer.no_srgb_texture_view;
        self.main_pipeline.framebuffer = multisampled_framebuffer.texture_view;
    }

    pub fn update(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        rs: &RenderingState,
    ) {
        self.main_pipeline.update(encoder, rs, &mut self.controls);
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        rs: &RenderingState
    ) {
        self.main_pipeline.render(encoder, &self.no_srgb_pipeline.no_srgb_framebuffer);
        self.no_srgb_pipeline.render(encoder, frame_view);
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
