use iced_wgpu::wgpu;
use image::{ImageError, GenericImageView, DynamicImage};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

    pub fn from_bytes(
        device: &wgpu::Device,
        image_bytes: Vec<(&[u8], &str)>,
        main_label: &str,
    ) -> Result<(Self, wgpu::CommandBuffer), failure::Error> {
        Self::from_image(
            device,
            image_bytes
                .into_iter()
                .map(|(b, l)| Ok((image::load_from_memory(b)?, l)))
                .collect::<Result<Vec<(DynamicImage, &str)>, ImageError>>()?,
            main_label,
        )
    }

    pub fn from_image(
        device: &wgpu::Device,
        imgs: Vec<(DynamicImage, &str)>,
        main_label: &str,
    ) -> Result<(Self, wgpu::CommandBuffer), failure::Error> {
        let dimensions = imgs.first().expect("no images").0.dimensions();

        for (img, label) in &imgs {
            assert_eq!(
                img.dimensions(),
                dimensions,
                "image labeled {} has dimensions that don't match!",
                label
            )
        }

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(main_label),
            size,
            array_layer_count: 2,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("texture_buffer_copy_encoder"),
        });

        for (i, (img, label)) in imgs.into_iter().enumerate() {
            let rgba = img
                .as_rgba8()
                .unwrap_or_else(|| panic!("image with {} label invalid", label));
            let buffer = device.create_buffer_with_data(&rgba, wgpu::BufferUsage::COPY_SRC);
            encoder.copy_buffer_to_texture(
                wgpu::BufferCopyView {
                    buffer: &buffer,
                    offset: 0,
                    bytes_per_row: 4 * dimensions.0,
                    rows_per_image: dimensions.1,
                },
                wgpu::TextureCopyView {
                    texture: &texture,
                    mip_level: 0,
                    array_layer: i as u32,
                    origin: wgpu::Origin3d::ZERO,
                },
                size,
            );
        }

        let cmd_buffer = encoder.finish();

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            //label: Some(main_label),
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            dimension: wgpu::TextureViewDimension::D2Array,
            aspect: Default::default(),
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            array_layer_count: 2,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        Ok((
            Self {
                texture,
                view,
                sampler,
            },
            cmd_buffer,
        ))
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        sample_count: u32,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            // 2.
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::SAMPLED
                | wgpu::TextureUsage::COPY_SRC,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_default_view();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // 4.
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::LessEqual, // 5.
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}