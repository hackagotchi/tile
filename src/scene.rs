use super::{camera::Camera, controls, rendering_state::RenderingState, texture};
use controls::Controls;
use iced_wgpu::{wgpu, Renderer};
use iced_winit::{program, Debug};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
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
            ],
        }
    }
}

//main.rs
#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex { position: [ 0.000000, -0.000000,  0.0], tex_coords: [ 0.25, 0.25 ] },
    Vertex { position: [ 0.866025, -0.500000,  0.0], tex_coords: [0.466506, 0.125000] },
    Vertex { position: [ 0.000000, -1.000000,  0.0], tex_coords: [0.250000, 0.000000] },
    Vertex { position: [-0.866025, -0.500000,  0.0], tex_coords: [0.033494, 0.125000] },
    Vertex { position: [-0.866025,  0.500000,  0.0], tex_coords: [0.033494, 0.375000] },
    Vertex { position: [-0.000000,  1.000000,  0.0], tex_coords: [0.250000, 0.500000] },
    Vertex { position: [ 0.866025,  0.500000,  0.0], tex_coords: [0.466506, 0.375000] },
    // bottom verts
    Vertex { position: [ 0.866025, -0.500000,  0.00], tex_coords: [0.500000, 0.000000] },
    Vertex { position: [ 0.000000, -1.000000,  0.00], tex_coords: [1.000000, 0.000000] },
    Vertex { position: [-0.866025, -0.500000,  0.00], tex_coords: [0.500000, 0.000000] },
    Vertex { position: [-0.866025,  0.500000,  0.00], tex_coords: [1.000000, 0.000000] },
    Vertex { position: [-0.000000,  1.000000,  0.00], tex_coords: [0.500000, 0.000000] },
    Vertex { position: [ 0.866025,  0.500000,  0.00], tex_coords: [1.000000, 0.000000] },
    Vertex { position: [ 0.866025, -0.500000, -1.00], tex_coords: [0.500000, 0.500000] },
    Vertex { position: [ 0.000000, -1.000000, -1.00], tex_coords: [1.000000, 0.500000] },
    Vertex { position: [-0.866025, -0.500000, -1.00], tex_coords: [0.500000, 0.500000] },
    Vertex { position: [-0.866025,  0.500000, -1.00], tex_coords: [1.000000, 0.500000] },
    Vertex { position: [-0.000000,  1.000000, -1.00], tex_coords: [0.500000, 0.500000] },
    Vertex { position: [ 0.866025,  0.500000, -1.00], tex_coords: [1.000000, 0.500000] },
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

struct Instance {
    position: nalgebra::Vector3<f32>,
    rotation: f32,
}
impl Default for Instance {
    fn default() -> Self {
        Self {
            position: nalgebra::zero(),
            rotation: Default::default(),
        }
    }
}
impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        let model = nalgebra::Matrix4::new_translation(&self.position)
            * nalgebra::Matrix4::new_rotation(nalgebra::Vector3::y() * self.rotation);

        InstanceRaw { model }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InstanceRaw {
    model: nalgebra::Matrix4<f32>,
}
unsafe impl bytemuck::Pod for InstanceRaw {}
unsafe impl bytemuck::Zeroable for InstanceRaw {}

pub struct Scene {
    /// Stores vertex data
    vertex_buffer: wgpu::Buffer,
    /// Stores index data
    index_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    instances_count: usize,
    pub controls: program::State<Controls>,
}

impl Scene {
    pub async fn new(rs: &RenderingState, renderer: &mut Renderer, debug: &mut Debug) -> Self {
        // UNIFORMS
        let instance_buffer_size =
            (std::mem::size_of::<InstanceRaw>() * 250) as wgpu::BufferAddress;
        let instance_buffer = rs.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tile instance buffer"),
            size: instance_buffer_size,
            usage: wgpu::BufferUsage::STORAGE_READ | wgpu::BufferUsage::COPY_DST,
        });

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

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&controls.program().camera_tab.camera);

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
        let depth_texture = texture::Texture::create_depth_texture(
            &rs.device,
            &rs.swap_chain_descriptor,
            "depth_texture",
        );

        let diffuse_bytes = include_bytes!("dirt.png");
        let (diffuse_texture, cmd_buffer) =
            texture::Texture::from_bytes(&rs.device, diffuse_bytes, "hat.png").unwrap();
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
                                dimension: wgpu::TextureViewDimension::D2,
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
        let vs_src = include_str!("shader.vert");
        let fs_src = include_str!("shader.frag");

        let mut compiler = shaderc::Compiler::new().unwrap();
        let vs_spirv = compiler
            .compile_into_spirv(
                vs_src,
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap();
        let fs_spirv = compiler
            .compile_into_spirv(
                fs_src,
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None,
            )
            .unwrap();

        let vs_data = wgpu::read_spirv(std::io::Cursor::new(vs_spirv.as_binary_u8())).unwrap();
        let fs_data = wgpu::read_spirv(std::io::Cursor::new(fs_spirv.as_binary_u8())).unwrap();

        let vs_module = rs.device.create_shader_module(&vs_data);
        let fs_module = rs.device.create_shader_module(&fs_data);

        // PIPELINE & SHADERS
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
                    format: rs.swap_chain_descriptor.format,
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
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        Self {
            controls,
            instances_count: 0,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            depth_texture,
            diffuse_texture,
            diffuse_bind_group,
            instance_buffer,
        }
    }

    pub fn resize(&mut self, (w, h): (u32, u32), rs: &RenderingState) {
        self.controls.queue_message(controls::Message::Resize(w, h));

        self.depth_texture = texture::Texture::create_depth_texture(
            &rs.device,
            &rs.swap_chain_descriptor,
            "depth_texture",
        );
    }

    pub fn update(&mut self, encoder: &mut wgpu::CommandEncoder, rs: &RenderingState) {
        let controls = self.controls.program();

        self.uniforms
            .update_view_proj(&controls.camera_tab.camera);
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

        if controls.tiling_tab.dirty {
            use noise::{Seedable, NoiseFn};

            let perlin = noise::Perlin::new().set_seed(controls.tiling_tab.seed);
            let g = controls.tiling_tab.size as f64;
            let e = controls.tiling_tab.elevation as f32;

            let instances: Vec<Instance> = (0..controls.tiling_tab.size)
                .flat_map(|x| {
                    (0..controls.tiling_tab.size).map(move |y| {
                        let w: f32 = 3.0_f32.sqrt();
                        let h: f32 = 2.0;

                        Instance {
                            position: nalgebra::Vector3::new(
                                (x * 2 + (y & 1)) as f32 / 2.0 * w,
                                ((3.0 / 4.0) * y as f32) * h,
                                dbg!(perlin.get([x as f64 / g, y as f64 / g]) as f32 * e),
                            ),
                            ..Default::default()
                        }
                    })
                })
                .collect();
            self.instances_count = instances.len();

            let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<InstanceRaw>>();
            let staging_buffer_size =
                instance_data.len() * std::mem::size_of::<nalgebra::Matrix4<f32>>();
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

            self.controls.queue_message(
                controls::Message::Tiling(controls::tiling::Message::Retiled)
            );
        }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, frame_view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: frame_view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                stencil_load_op: wgpu::LoadOp::Clear,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); // NEW!
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        render_pass.set_index_buffer(&self.index_buffer, 0, 0);
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..self.instances_count as u32);
    }
}
