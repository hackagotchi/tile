use crate::Config;
use crate::{compile_shaders, texture, RenderingState, Sprite};
use hexa::{iced_wgpu::wgpu, Camera};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
}
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[wgpu::VertexAttributeDescriptor {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float3,
            }],
        }
    }
}

#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5, 0.0] },
    Vertex { position: [ 0.5, -0.5, 0.0] },
    Vertex { position: [ 0.5,  0.5, 0.0] },
    Vertex { position: [-0.5,  0.5, 0.0] },
];
const INDICES: &[u16] = &[2, 1, 0, 0, 3, 2];

#[repr(C)]
#[derive(Copy, Clone)]
struct Uniforms {
    camera_up: nalgebra::Vector4<f32>,
    camera_right: nalgebra::Vector4<f32>,
    view_proj: nalgebra::Matrix4<f32>,
}
unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}
impl Uniforms {
    fn new() -> Self {
        Self {
            camera_right: nalgebra::Vector4::identity(),
            camera_up: nalgebra::Vector4::identity(),
            view_proj: nalgebra::Matrix4::identity(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        let view = camera.view();
        let proj = camera.projection();

        self.camera_right =
            nalgebra::Vector4::new(view.column(0)[0], view.column(1)[0], view.column(2)[0], 1.0);
        self.camera_up =
            nalgebra::Vector4::new(view.column(0)[1], view.column(1)[1], view.column(2)[1], 1.0);
        self.view_proj = proj * view;
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InstanceRaw {
    position: nalgebra::Vector3<f32>,
    scale: nalgebra::Vector3<f32>,
    texture_indexes: nalgebra::Vector2<u32>,
}
unsafe impl bytemuck::Pod for InstanceRaw {}
unsafe impl bytemuck::Zeroable for InstanceRaw {}

pub struct Quad {
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

impl Quad {
    pub fn new(rs: &RenderingState, camera: &Camera, config: &Config) -> Self {
        // UNIFORMS
        let instance_buffer_size =
            (std::mem::size_of::<InstanceRaw>() * 250) as wgpu::BufferAddress;
        let instance_buffer = rs.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad instance buffer"),
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
                    label: Some("quad_uniform_bind_group_layout"),
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
            label: Some("quad_uniform_bind_group"),
        });

        // IMAGE
        let (diffuse_texture, cmd_buffer) = texture::Texture::from_bytes(
            &rs.device,
            vec![(include_bytes!("../../../img/sprite/stump.png"), "stump.png")],
            "quad textures",
        )
        .unwrap();
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
                    label: Some("quad_texture_bind_group_layout"),
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
            label: Some("quad_diffuse_bind_group"),
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
            (
                include_str!("../../../shader/quad/shader.vert"),
                "quad/shader.vert",
            ),
            (
                include_str!("../../../shader/quad/shader.frag"),
                "quad/shader.frag",
            ),
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
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Max,
                    },
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

    pub fn set_sprites(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        rs: &RenderingState,
        sprites: Vec<Sprite>,
    ) {
        let instance_data = sprites
            .into_iter()
            .map(|t| {
                use nalgebra::Vector3 as Vec3;
                let Sprite {
                    image,
                    position: pos,
                    scale,
                } = t;

                InstanceRaw {
                    position: Vec3::new(pos.x, pos.y, 0.275),
                    scale: Vec3::new(scale.x, scale.y, 1.0),
                    texture_indexes: nalgebra::Vector2::x() * image,
                }
            })
            .collect::<Vec<_>>();
        self.instances_count = instance_data.len();

        let staging_buffer_size = instance_data.len() * std::mem::size_of::<InstanceRaw>();
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

    pub fn render<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); // NEW!
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        render_pass.set_index_buffer(&self.index_buffer, 0, 0);
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..self.instances_count as u32);
    }
}
