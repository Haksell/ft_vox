mod aabb;
mod app;
mod block;
mod camera;
mod chunk;
mod face;
mod frustum;
mod noise;
mod texture;
mod vertex;
mod world;

use {
    crate::{
        aabb::AABB,
        app::Application,
        camera::{Camera, CameraController, CameraUniform},
        chunk::CHUNK_WIDTH,
        texture::Texture,
        vertex::Vertex,
        world::World,
    },
    std::{collections::HashMap, sync::Arc, time::Duration},
    wgpu::util::DeviceExt as _,
    winit::{event::*, event_loop::EventLoop, window::Window},
};

struct ChunkRenderData {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    aabb: AABB,
}

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    chunk_render_data: HashMap<(i32, i32), ChunkRenderData>,

    diffuse_texture: Texture,
    depth_texture: Texture,
    diffuse_bind_group: wgpu::BindGroup,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,

    sky_pipeline: wgpu::RenderPipeline,
    sky_texture: Texture,
    sky_bind_group: wgpu::BindGroup,
}

impl<'a> State<'a> {
    async fn new(window: Arc<Window>) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: true,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("../assets/atlas.png");
        let diffuse_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "../assets/atlas.png").unwrap();
        let depth_texture = Texture::create_depth_texture(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera = Camera::new(
            glam::Vec3::new(0.0, 256.0, 0.0),
            glam::Vec3::new(0.0, 1.0, 0.0),
            config.width as f32 / config.height as f32,
            80.0,
            0.1,
            500.0,
        );

        let camera_uniform = CameraUniform::new(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let camera_controller = CameraController::new(20.0, 0.2);

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Front),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let chunk_render_data = HashMap::new();

        // --- SKY TEXTURE ---
        let sky_bytes = include_bytes!("../assets/space.tif");
        let sky_texture = Texture::from_bytes(&device, &queue, sky_bytes, "../assets/space.tif")
            .expect("failed to load sky panorama");

        let sky_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout, // same layout: texture @binding(0), sampler @binding(1)
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sky_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sky_texture.sampler),
                },
            ],
            label: Some("sky_bind_group"),
        });

        // --- SKY PIPELINE ---
        let sky_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/sky.wgsl"));

        let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sky Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sky Pipeline"),
            layout: Some(&sky_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sky_shader,
                entry_point: Some("vs_fullscreen"),
                // Fullscreen triangle: no vertex buffers needed.
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sky_shader,
                entry_point: Some("fs_sky"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            // Sky doesn't use depth.
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            chunk_render_data,
            diffuse_bind_group,
            diffuse_texture,
            depth_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            sky_pipeline,
            sky_texture,
            sky_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.camera
                .set_aspect_ratio(new_size.width as f32 / new_size.height as f32);

            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config);
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update_chunks(&mut self, world: &mut World) {
        let camera_pos = self.camera.position();
        let render_distance = world.get_render_distance() as i32;
        let current_chunk = world.get_chunk_index_from_position(camera_pos.x, camera_pos.z);

        let mut chunks_in_range = std::collections::HashSet::new();

        for dx in -render_distance..=render_distance {
            for dy in -render_distance..=render_distance {
                let chunk_coords = (current_chunk.0 + dx, current_chunk.1 + dy);
                chunks_in_range.insert(chunk_coords);

                world.get_chunk(chunk_coords.0, chunk_coords.1);
            }
        }

        self.chunk_render_data
            .retain(|&coords, _| chunks_in_range.contains(&coords));

        for chunk_coords in &chunks_in_range {
            if !self.chunk_render_data.contains_key(chunk_coords) {
                let (chunk_x, chunk_y) = *chunk_coords;
                self.generate_chunk_mesh(world, chunk_x, chunk_y);
            }
        }
    }

    fn generate_chunk_mesh(&mut self, world: &mut World, chunk_x: i32, chunk_y: i32) {
        let (mut vertices, indices) = world.generate_chunk_mesh(chunk_x, chunk_y);

        let world_offset_x = chunk_x as f32 * CHUNK_WIDTH as f32;
        let world_offset_y = chunk_y as f32 * CHUNK_WIDTH as f32;

        for vertex in &mut vertices {
            vertex.position[0] += world_offset_x;
            vertex.position[1] += world_offset_y;
        }

        if !vertices.is_empty() && !indices.is_empty() {
            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Chunk ({}, {}) Vertex Buffer", chunk_x, chunk_y)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

            let index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Chunk ({}, {}) Index Buffer", chunk_x, chunk_y)),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

            let chunk = world.get_chunk_if_loaded(chunk_x, chunk_y).unwrap();
            let aabb = chunk.bounding_box();

            let render_data = ChunkRenderData {
                vertex_buffer,
                index_buffer,
                num_indices: indices.len() as u32,
                aabb,
            };

            self.chunk_render_data
                .insert((chunk_x, chunk_y), render_data);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_keyboard(event)
    }

    fn update(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32();

        self.camera_controller.update(&mut self.camera, dt);

        self.camera_uniform.update(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let frustum = self.camera.get_frustum();

        // --- PASS 1: Sky ---
        {
            let mut sky_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sky Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Clear once at the beginning; the fullscreen triangle will cover it anyway.
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            sky_pass.set_pipeline(&self.sky_pipeline);
            sky_pass.set_bind_group(0, &self.sky_bind_group, &[]);
            sky_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            // Fullscreen triangle: 3 vertices, 1 instance.
            sky_pass.draw(0..3, 0..1);
        }

        // --- PASS 2: World ---
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("World Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    // IMPORTANT: load previous color (the sky) so chunks draw over it.
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            for render_data in self.chunk_render_data.values() {
                if frustum.intersects_aabb(&render_data.aabb) {
                    render_pass.set_vertex_buffer(0, render_data.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        render_data.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    render_pass.draw_indexed(0..render_data.num_indices, 0, 0..1);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = Application::new();
    event_loop.run_app(&mut app).expect("Failed to run app");
}

fn main() {
    let _ = pollster::block_on(run());
}
