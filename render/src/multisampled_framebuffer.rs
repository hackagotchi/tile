use hexa::iced_wgpu::wgpu;

pub struct MultisampledFramebuffer {
    pub texture_view: wgpu::TextureView,
    pub no_srgb_texture_view: wgpu::TextureView,
}
impl MultisampledFramebuffer {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        sample_count: u32,
    ) -> Self {
        let multisampled_texture_extent = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };
        let multisampled_frame_descriptor = wgpu::TextureDescriptor {
            size: multisampled_texture_extent,
            array_layer_count: 1,
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: 1,
            sample_count,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: Some("multisampled framebuffer"),
        };

        Self {
            texture_view: device
                .create_texture(&multisampled_frame_descriptor)
                .create_default_view(),
            no_srgb_texture_view: device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("no_srgb_texture for dodging intel gpu bugs"),
                    usage: wgpu::TextureUsage::COPY_DST
                        | wgpu::TextureUsage::OUTPUT_ATTACHMENT
                        | wgpu::TextureUsage::SAMPLED,
                    sample_count: 1,
                    ..multisampled_frame_descriptor
                })
                .create_default_view(),
        }
    }
}
