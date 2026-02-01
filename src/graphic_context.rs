use std::sync::Arc;
use winit::window::Window;
use wgpu::util::DeviceExt;
use glyphon::{Attrs, Buffer, Cache, Color as TextColor, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextRenderer, Viewport};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

pub struct TextEntry {
    pub text: String,
    pub x: f32, // Logical X
    pub y: f32, // Logical Y
    pub color: [f32; 4],
    pub scale: f32,
}

pub struct TextSystem {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub viewport: Viewport,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                }
            ]
        }
    }
}

pub struct GraphicContext {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub text_system: TextSystem,
}

impl GraphicContext {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        
        // The surface is the part of the window that we draw to
        // Using Arc<Window> allows the surface into be 'static
        let surface = instance.create_surface(window.clone()).unwrap();

        // The adapter is a handle to our actual graphics card.
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                ..Default::default()
            },
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Load shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[Vertex::desc()], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, 
            multisample: wgpu::MultisampleState {
                count: 1, 
                mask: !0, 
                alpha_to_coverage_enabled: false, 
            },
            cache: None,
            multiview_mask: None,
        });

        // Initialize with a dummy triangle so we don't crash before first update
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, 
            }
        );

        // --- Text System Init ---
        let mut font_system = FontSystem::new();
        // Load embedded font
        let font_data = include_bytes!("../assets/font.ttf").to_vec();
        font_system.db_mut().load_font_data(font_data);

        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, config.format);
        let text_renderer = TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);
        let viewport = Viewport::new(&device, &cache);

        let text_system = TextSystem {
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices: 0,
            text_system,
        }
    }


    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update_buffers(&mut self, vertices: &[Vertex]) {
        self.num_vertices = vertices.len() as u32;
        
        // Recreate buffer if it's too small or just create new one every time (simple but inefficient)
        // For Tetris, vertex count is low, so recreating is fine or writing to existing if mapped.
        // COPY_DST allows write_buffer.
        
        self.vertex_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
    }

    pub fn render(&mut self, text_entries: &[TextEntry]) -> Result<(), wgpu::SurfaceError> {
        // --- 1. Prepare Text ---
        // We use a buffer to hold the text area
        let _buffer = Buffer::new(&mut self.text_system.font_system, Metrics::new(30.0, 42.0));
        let width = self.size.width as f32;
        let height = self.size.height as f32;
        
        self.text_system.viewport.update(&self.queue, Resolution { width: self.size.width, height: self.size.height });

        // We will create a TextArea for each entry (inefficient but simple)
        // Actually processing all text this way every frame is okay for this complexity.
        // glyphon works best with 'Buffer's.
        
        // For multiple independent text blocks, we create a generic text processor or just loop.
        // glyphon Prepare step takes a slice of TextAreas.
        
        let mut text_areas = Vec::new();
        // Since Buffer cannot be easily moved into the Vec because of lifetimes/borrowing within the loop,
        // we might need to recreate them or have a different strategy. 
        // glyphon::TextArea stores reference to buffer.
        // So we need a Vec<Buffer> that lives long enough.
        
        let mut buffers = Vec::new();

        for entry in text_entries {
             let physical_font_size = entry.scale * 30.0; // Base size multiplier
             let mut buff = Buffer::new(&mut self.text_system.font_system, Metrics::new(physical_font_size, physical_font_size * 1.2));
             
             // Convert Logical X/Y to Physical
             // VertexData Logical mapping:
             // WIDTH=800, HEIGHT=800.
             // logical_width = 10 + 16 = 26.
             // logical_height = 29.
             // unit_size_y = 1.9 / 29.0
             // unit_size_x = unit_size_y / aspect.
             
             // We need to inverse the projection in VertexData to get Pixel Coords,
             // OR simplest way: Just pass Pixel Coords from VertexData?
             // VertexData doesn't know Screen Size properly without querying.
             // Let's rely on stored conversion in VertexData or main.
             // Just assuming entry.x/y are PRE-CALCULATED PIXEL COORDINATES?
             // No, "Logical X" in VertexData is like "Grid Cell 5".
             // We need to convert Grid Cell 5 -> Pixel.
             
             // Let's do the conversion here.
             let logical_w = crate::game::WIDTH as f32 + 16.0;
             let logical_h = 29.0;
             
             let aspect = width / height;
             let unit_scale_y = 1.9 / logical_h;
             let unit_scale_x = unit_scale_y / aspect;
             
             let total_ndc_w = unit_scale_x * logical_w; 
             let start_ndc_x = -total_ndc_w / 2.0;

             let total_ndc_h = unit_scale_y * logical_h; 
             let start_ndc_y = total_ndc_h / 2.0;
             
             // NDC X = start_ndc_x + (entry.x * unit_scale_x)
             // NDC Y = start_ndc_y - (entry.y * unit_scale_y)
             
             let ndc_x = start_ndc_x + (entry.x * unit_scale_x);
             let ndc_y = start_ndc_y - (entry.y * unit_scale_y);
             
             // Pixel X = (ndc_x + 1.0) * 0.5 * width
             // Pixel Y = (1.0 - ndc_y) * 0.5 * height
             
             let screen_x = (ndc_x + 1.0) * 0.5 * width;
             let screen_y = (1.0 - ndc_y) * 0.5 * height;

             buff.set_size(&mut self.text_system.font_system, Some(width), Some(height));
             buff.set_text(&mut self.text_system.font_system, &entry.text, &Attrs::new().family(Family::Name("Press Start 2P")), Shaping::Advanced, None);
             buffers.push((buff, screen_x, screen_y, entry.color));
        }

        for (buff, x, y, color) in buffers.iter() {
             let color = TextColor::rgba((color[0] * 255.0) as u8, (color[1] * 255.0) as u8, (color[2] * 255.0) as u8, (color[3] * 255.0) as u8);
             text_areas.push(TextArea {
                 buffer: buff,
                 left: *x,
                 top: *y,
                 scale: 1.0,
                 bounds: glyphon::TextBounds {
                     left: 0,
                     top: 0,
                     right: width as i32,
                     bottom: height as i32,
                 },
                 default_color: color,
                 custom_glyphs: &[],
             });
        }

        self.text_system.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.text_system.font_system,
            &mut self.text_system.atlas,
            &self.text_system.viewport,
            text_areas,
            &mut self.text_system.swash_cache,
        ).unwrap();


        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
            
            self.text_system.text_renderer.render(&self.text_system.atlas, &self.text_system.viewport, &mut render_pass).unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        // Cleanup atlas to prevent infinite growth
        self.text_system.atlas.trim();

        Ok(())
    }
}
