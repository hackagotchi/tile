use crate::render::Config;
use crate::render::{RenderingState, compile_shaders, texture};
use crate::camera::Camera;
use crate::tiling::Tile;

use iced_wgpu::wgpu;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    image: u16
}
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: (mem::size_of::<[f32; 3]>() + mem::size_of::<[f32; 2]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint,
                },
            ],
        }
    }
}

//main.rs
#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex { position: [ 0.000000, -0.000000,  0.0], tex_coords: [0.500000, 0.500000], image: 0 },
    Vertex { position: [ 0.866025, -0.500000,  0.0], tex_coords: [0.933012, 0.250000], image: 0 },
    Vertex { position: [ 0.000000, -1.000000,  0.0], tex_coords: [0.500000, 0.000000], image: 0 },
    Vertex { position: [-0.866025, -0.500000,  0.0], tex_coords: [0.066988, 0.250000], image: 0 },
    Vertex { position: [-0.866025,  0.500000,  0.0], tex_coords: [0.066988, 0.750000], image: 0 },
    Vertex { position: [-0.000000,  1.000000,  0.0], tex_coords: [0.500000, 1.000000], image: 0 },
    Vertex { position: [ 0.866025,  0.500000,  0.0], tex_coords: [0.933012, 0.750000], image: 0 },
    // bottom verts
    Vertex { position: [ 0.866025, -0.500000,  0.00], tex_coords: [0.000000, 0.000000], image: 1 },
    Vertex { position: [ 0.000000, -1.000000,  0.00], tex_coords: [1.000000, 0.000000], image: 1 },
    Vertex { position: [-0.866025, -0.500000,  0.00], tex_coords: [0.000000, 0.000000], image: 1 },
    Vertex { position: [-0.866025,  0.500000,  0.00], tex_coords: [1.000000, 0.000000], image: 1 },
    Vertex { position: [-0.000000,  1.000000,  0.00], tex_coords: [0.000000, 0.000000], image: 1 },
    Vertex { position: [ 0.866025,  0.500000,  0.00], tex_coords: [1.000000, 0.000000], image: 1 },
    Vertex { position: [ 0.866025, -0.500000, -1.00], tex_coords: [0.000000, 1.000000], image: 1 },
    Vertex { position: [ 0.000000, -1.000000, -1.00], tex_coords: [1.000000, 1.000000], image: 1 },
    Vertex { position: [-0.866025, -0.500000, -1.00], tex_coords: [0.000000, 1.000000], image: 1 },
    Vertex { position: [-0.866025,  0.500000, -1.00], tex_coords: [1.000000, 1.000000], image: 1 },
    Vertex { position: [-0.000000,  1.000000, -1.00], tex_coords: [0.000000, 1.000000], image: 1 },
    Vertex { position: [ 0.866025,  0.500000, -1.00], tex_coords: [1.000000, 1.000000], image: 1 },
];
const INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 1, // bottom verts
    7, 14, 8, 7, 13, 14, 8, 15, 9, 8, 14, 15, 9, 16, 10, 9, 15, 16, 10, 17, 11, 10, 16, 17, 11, 18,
    12, 11, 17, 18, 12, 13, 7, 12, 18, 13,
];

#[repr(C)]
#[derive(Copy, Clone)]
struct Uniforms {
    view_proj: nalgebra::Matrix4<f32>,
}
unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}
impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: nalgebra::Matrix4::identity(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix();
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InstanceRaw {
    model: nalgebra::Matrix4<f32>,
    texture_indexes: nalgebra::Vector4<u32>,
}
unsafe impl bytemuck::Pod for InstanceRaw {}
unsafe impl bytemuck::Zeroable for InstanceRaw {}

pub struct Hex {
    /// Stores vertex data
    vertex_buffer: wgpu::Buffer,
    /// Stores index data
    index_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    instances_count: usize,
}

impl Hex {
    pub fn new(
        rs: &RenderingState,
        camera: &Camera,
        config: &Config,
    ) -> Self {
        // UNIFORMS
        let instance_buffer_size =
            (std::mem::size_of::<InstanceRaw>() * 250) as wgpu::BufferAddress;
        let instance_buffer = rs.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tile instance buffer"),
            size: instance_buffer_size,
            usage: wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST,
        });

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = rs.device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let uniform_bind_group_layout =
            rs.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    bindings: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStage::VERTEX,
                            ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStage::VERTEX,
                            ty: wgpu::BindingType::StorageBuffer {
                                dynamic: false,
                                readonly: true,
                            },
                        },
                    ],
                    label: Some("uniform_bind_group_layout"),
                });
        let uniform_bind_group = rs.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        // FYI: you can share a single buffer between bindings.
                        range: 0..std::mem::size_of_val(&uniforms) as wgpu::BufferAddress,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &instance_buffer,
                        range: 0..instance_buffer_size,
                    },
                },
            ],
            label: Some("uniform_bind_group"),
        });

        // IMAGE
        let (diffuse_texture, cmd_buffer) = texture::Texture::from_bytes(
            &rs.device,
            vec![
                (include_bytes!("../../../img/hex/ice_hat.png"), "ice_hat.png"),
                (include_bytes!("../../../img/hex/ice_butt.png"), "ice_butt.png"),
                (include_bytes!("../../../img/hex/snow_hat.png"), "snow_hat.png"),
                (include_bytes!("../../../img/hex/snow_butt.png"), "snow_butt.png"),
            ],
            "tile textures"
        ).unwrap();
        rs.queue.submit(&[cmd_buffer]);

        let texture_bind_group_layout =
            rs.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    bindings: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::SampledTexture {
                                multisampled: false,
                                dimension: wgpu::TextureViewDimension::D2Array,
                                component_type: wgpu::TextureComponentType::Uint,
                            },
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::Sampler { comparison: false },
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let diffuse_bind_group = rs.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        // VERTEXES (and by extension, indexes)
        let vertex_buffer = rs
            .device
            .create_buffer_with_data(bytemuck::cast_slice(VERTICES), wgpu::BufferUsage::VERTEX);
        let index_buffer = rs
            .device
            .create_buffer_with_data(bytemuck::cast_slice(INDICES), wgpu::BufferUsage::INDEX);

        // SHADERS
        let (vs_module, fs_module) = compile_shaders(
            (include_str!("../../../shader/hex/shader.vert"), "hex/shader.vert"),
            (include_str!("../../../shader/hex/shader.frag"), "hex/shader.frag"),
            rs,
        );

        let render_pipeline_layout =
            rs.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                });
        let render_pipeline = rs
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: &render_pipeline_layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vs_module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fs_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: wgpu::CullMode::Back,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                }),
                color_states: &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                    format: texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    stencil_read_mask: 0,
                    stencil_write_mask: 0,
                }),
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[Vertex::desc()],
                },
                sample_count: config.msaa,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        Self {
            instances_count: 0,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            diffuse_texture,
            diffuse_bind_group,
            instance_buffer,
        }
    }

    pub fn set_camera(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        rs: &RenderingState,
        camera: &Camera,
    ) {
        self.uniforms.update_view_proj(&camera);
        let staging_buffer = rs.device.create_buffer_with_data(
            bytemuck::cast_slice(&[self.uniforms]),
            wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
        );
    }

    pub fn set_tiles(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        rs: &RenderingState,
        tiles: Vec<Tile>
    ) {
        let instance_data = tiles
            .into_iter()
            .map(|t| {
                use nalgebra::Vector3 as Vec3;
                let Tile { elevation, position: p, hat, butt, butt_size } = t;
                let w: f32 = 3.0_f32.sqrt();
                let h: f32 = 2.0;

                let position = Vec3::new(
                    (p.x * 2 + (p.y & 1)) as f32 / 2.0 * w,
                    ((3.0 / 4.0) * p.y as f32) * h,
                    elevation
                );
                let model = nalgebra::Matrix4::new_nonuniform_scaling(&Vec3::new(1.0, 1.0, butt_size))
                    * nalgebra::Matrix4::new_translation(&position);
                    //* nalgebra::Matrix4::new_rotation(nalgebra::Vector3::y() * rotation);

                InstanceRaw {
                    model,
                    texture_indexes: nalgebra::Vector4::new(hat, butt, 0, 0),
                }
            })
            .collect::<Vec<_>>();
        self.instances_count = instance_data.len();

        let staging_buffer_size =
            instance_data.len() * std::mem::size_of::<InstanceRaw>();
        let staging_buffer = rs.device.create_buffer_with_data(
            bytemuck::cast_slice(&instance_data),
            wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.instance_buffer,
            0,
            staging_buffer_size as u64,
        );
    }

    pub fn render<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); // NEW!
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        render_pass.set_index_buffer(&self.index_buffer, 0, 0);
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..self.instances_count as u32);
    }
}
