use {
    crate::{
        aabb::AABB,
        camera::{Camera, CameraController, CameraUniform},
        chunk::{ChunkCoords, CHUNK_HEIGHT, CHUNK_WIDTH},
        texture::Texture,
        vertex::Vertex,
        world::{calculate_lod, World, RENDER_DISTANCE},
    },
    glam::Vec3,
    std::{collections::HashMap, f32::consts::SQRT_2, sync::Arc, time::Duration},
    wgpu::util::DeviceExt as _,
    wgpu_text::{
        glyph_brush::{ab_glyph::FontRef, HorizontalAlign, Layout, Section, Text, VerticalAlign},
        BrushBuilder, TextBrush,
    },
    winit::{dpi::PhysicalSize, event::*, window::Window},
};

struct ChunkRenderData {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    aabb: AABB,
    lod_step: usize,
}

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    chunk_render_data: HashMap<ChunkCoords, ChunkRenderData>,

    pub camera: Camera,
    pub camera_controller: CameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    depth_texture: Texture,
    diffuse_bind_group: wgpu::BindGroup,
    voxels_pipeline: wgpu::RenderPipeline,

    skybox_pipeline: wgpu::RenderPipeline,
    skybox_bind_group: wgpu::BindGroup,

    pub fps: f32,
    pub text_brush: TextBrush<FontRef<'a>>,
}

impl<'a> State<'a> {
    pub async fn new(window: Arc<Window>) -> State<'a> {
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

        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            wgpu::PresentMode::Mailbox
        } else if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Immediate)
        {
            wgpu::PresentMode::Immediate
        } else {
            wgpu::PresentMode::Fifo
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let diffuse_texture = Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("../assets/atlas.png"),
            "../assets/atlas.png",
        )
        .unwrap();
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

        let camera_distance_xy = (RENDER_DISTANCE + 1) as f32 * SQRT_2 * CHUNK_WIDTH as f32;
        let camera_distance = (camera_distance_xy.powi(2) + (CHUNK_HEIGHT as f32).powi(2)).sqrt();
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, CHUNK_HEIGHT as f32),
            Vec3::new(0.0, 0.0, 1.0),
            config.width as f32 / config.height as f32,
            (80.0_f32).to_radians(),
            0.1,
            camera_distance,
        );

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::new(&camera)]),
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

        let camera_controller = CameraController::new();

        // === VOXELS ===
        let voxels_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/voxels.wgsl"));
        let voxels_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("voxels_pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("voxels_pipeline_layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &voxels_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &voxels_shader,
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
                cull_mode: Some(wgpu::Face::Back),
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

        // === SKYBOX ===
        let skybox_texture = Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("../assets/space.tif"),
            "../assets/space.tif",
        )
        .expect("failed to load skybox");

        let skybox_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&skybox_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&skybox_texture.sampler),
                },
            ],
            label: Some("skybox_bind_group"),
        });

        let skybox_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/skybox.wgsl"));
        let skybox_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("skybox_pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("skybox_pipeline_layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &skybox_shader,
                entry_point: Some("vs_main"),
                buffers: &[], // fullscreen triangle: no vertex buffers needed.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &skybox_shader,
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
                ..Default::default()
            },
            depth_stencil: None, // infinite depth
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let text_brush =
            BrushBuilder::using_font_bytes(include_bytes!("../assets/EP-Boxi-Bold.otf"))
                .unwrap()
                .with_depth_stencil(None)
                .build(&device, config.width, config.height, config.format);

        let fps = 60.0; // dummy value before first calculation

        Self {
            surface,
            device,
            queue,
            config,
            size,
            voxels_pipeline,
            chunk_render_data,
            diffuse_bind_group,
            depth_texture,
            camera,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            skybox_pipeline,
            skybox_bind_group,
            fps,
            text_brush,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;

        self.surface.configure(&self.device, &self.config);

        self.camera.resize(new_size.width, new_size.height);
        self.text_brush
            .resize_view(new_size.width as f32, new_size.height as f32, &self.queue);

        self.depth_texture = Texture::create_depth_texture(&self.device, &self.config);
    }

    pub fn update_chunks(&mut self, world: &mut World) {
        let camera_pos = self.camera.position();
        let render_distance = RENDER_DISTANCE as i32;
        let camera_chunk = world.get_chunk_index_from_position(camera_pos.x, camera_pos.y);

        let mut chunks_in_range = HashMap::new();

        for dx in -render_distance..=render_distance {
            for dy in -render_distance..=render_distance {
                let chunk_coords = (camera_chunk.0 + dx, camera_chunk.1 + dy);
                let lod_step = calculate_lod(camera_chunk, chunk_coords);
                chunks_in_range.insert(chunk_coords, lod_step);
                world.get_chunk(chunk_coords);
            }
        }

        self.chunk_render_data
            .retain(|&coords, _| chunks_in_range.contains_key(&coords));

        for (&chunk_coords, &lod_step) in &chunks_in_range {
            if self
                .chunk_render_data
                .get(&chunk_coords)
                .is_none_or(|crd| crd.lod_step != lod_step)
            {
                self.generate_chunk_mesh(world, camera_chunk, lod_step, chunk_coords);
            }
        }
    }

    fn generate_chunk_mesh(
        &mut self,
        world: &mut World,
        camera_chunk: ChunkCoords,
        lod_step: usize,
        chunk_coords: ChunkCoords,
    ) {
        let (chunk_x, chunk_y) = chunk_coords;
        let (mut vertices, indices) =
            world.generate_chunk_mesh(camera_chunk, lod_step, chunk_coords);
        if vertices.is_empty() || indices.is_empty() {
            return;
        }

        let world_offset_x = chunk_x as f32 * CHUNK_WIDTH as f32;
        let world_offset_y = chunk_y as f32 * CHUNK_WIDTH as f32;

        for vertex in &mut vertices {
            vertex.position[0] += world_offset_x;
            vertex.position[1] += world_offset_y;
        }

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

        let chunk = world.get_chunk_if_loaded(chunk_coords).unwrap();
        let aabb = chunk.bounding_box();

        let render_data = ChunkRenderData {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            aabb,
            lod_step,
        };

        self.chunk_render_data
            .insert((chunk_x, chunk_y), render_data);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_keyboard(event)
    }

    pub fn update(&mut self, dt: Duration) {
        self.camera_controller
            .update(&mut self.camera, dt.as_secs_f32());
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[CameraUniform::new(&self.camera)]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        fn render_skybox(
            state: &State,
            encoder: &mut wgpu::CommandEncoder,
            texture_view: &wgpu::TextureView,
        ) {
            let mut skybox_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("skybox_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            skybox_pass.set_pipeline(&state.skybox_pipeline);
            skybox_pass.set_bind_group(0, &state.skybox_bind_group, &[]);
            skybox_pass.set_bind_group(1, &state.camera_bind_group, &[]);
            skybox_pass.draw(0..3, 0..1); // fullscreen triangle: 3 vertices, 1 instance.
        }

        fn render_voxels(
            state: &State,
            encoder: &mut wgpu::CommandEncoder,
            texture_view: &wgpu::TextureView,
        ) {
            let mut voxels_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("voxels_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // load previous color (the skybox)
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &state.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            voxels_pass.set_pipeline(&state.voxels_pipeline);
            voxels_pass.set_bind_group(0, &state.diffuse_bind_group, &[]);
            voxels_pass.set_bind_group(1, &state.camera_bind_group, &[]);

            let frustum = state.camera.get_frustum();
            for render_data in state.chunk_render_data.values() {
                if frustum.intersects_aabb(&render_data.aabb) {
                    voxels_pass.set_vertex_buffer(0, render_data.vertex_buffer.slice(..));
                    voxels_pass.set_index_buffer(
                        render_data.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    voxels_pass.draw_indexed(0..render_data.num_indices, 0, 0..1);
                }
            }
        }

        fn render_text(
            state: &mut State,
            encoder: &mut wgpu::CommandEncoder,
            texture_view: &wgpu::TextureView,
        ) {
            let fps_text = format!("FPS:{:.0}", state.fps);

            let section = Section::default()
                .with_layout(
                    Layout::default()
                        .h_align(HorizontalAlign::Left)
                        .v_align(VerticalAlign::Top),
                )
                .with_screen_position((12.0, 12.0))
                .add_text(
                    Text::new(&fps_text)
                        .with_scale(24.0)
                        .with_color([1.0, 0.15, 0.15, 1.0]),
                );

            if let Err(brush_error) = state
                .text_brush
                .queue(&state.device, &state.queue, [section])
            {
                log::warn!("Brush error: {:?}", brush_error);
            }

            let mut text_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("text_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            state.text_brush.draw(&mut text_pass);
        }

        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                self.surface.get_current_texture()?
            }
            Err(e) => return Err(e),
        };

        let texture_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        render_skybox(self, &mut encoder, &texture_view);
        render_voxels(self, &mut encoder, &texture_view);
        render_text(self, &mut encoder, &texture_view);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
