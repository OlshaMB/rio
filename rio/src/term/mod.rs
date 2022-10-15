use crate::bar::{self, BarBrush};
use crate::style;
use crate::text::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text};
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Term {
    device: wgpu::Device,
    surface: wgpu::Surface,
    queue: wgpu::Queue,
    render_format: wgpu::TextureFormat,
    staging_belt: wgpu::util::StagingBelt,
    text_brush: GlyphBrush<()>,
    size: winit::dpi::PhysicalSize<u32>,
    bar: BarBrush,
}

impl Term {
    pub async fn new(
        winit_window: &winit::window::Window,
    ) -> Result<Term, Box<dyn Error>> {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&winit_window) };

        let (device, queue) = (async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .expect("Request adapter");

            adapter
                .request_device(&wgpu::DeviceDescriptor::default(), None)
                .await
                .expect("Request device")
        })
        .await;

        let staging_belt = wgpu::util::StagingBelt::new(64);
        let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let size = winit_window.inner_size();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../bar/bar.wgsl").into()),
        });

        let bar: BarBrush = BarBrush::new(&device, shader);

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: render_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
            },
        );

        let font = ab_glyph::FontArc::try_from_slice(style::FONT_FIRA_MONO)?;
        let text_brush =
            GlyphBrushBuilder::using_font(font).build(&device, render_format);

        Ok(Term {
            device,
            surface,
            staging_belt,
            text_brush,
            size,
            render_format,
            bar,
            queue,
        })
    }

    pub fn set_size(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        self.configure_surface();
    }

    fn configure_surface(&mut self) {
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.render_format,
                width: self.size.width,
                height: self.size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
            },
        );
    }

    pub fn draw(&mut self, output: &Arc<Mutex<String>>) {
        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Redraw"),
                });

        let frame = self.surface.get_current_texture().expect("Get next frame");
        let view = &frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            self.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &self.bar.shader,
                        entry_point: "vs_main",
                        buffers: &[bar::Vertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &self.bar.shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: self.render_format,
                            blend: crate::style::gpu::BLEND,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        ..Default::default()
                    },
                    depth_stencil: None, // 1.
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear frame"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(style::DEFAULT_COLOR_BACKGROUND),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_vertex_buffer(0, self.bar.buffers.0.slice(..));
            render_pass.set_index_buffer(
                self.bar.buffers.1.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw(0..self.bar.num_indices, 0..1);
        }

        {
            self.text_brush.queue(Section {
                screen_position: (24.0, 120.0),
                bounds: ((self.size.width - 40) as f32, self.size.height as f32),
                text: vec![Text::new(&output.lock().unwrap())
                    .with_color([1.0, 1.0, 1.0, 1.0])
                    .with_scale(36.0)],
                ..Section::default()
            });

            self.text_brush
                .draw_queued(
                    &self.device,
                    &mut self.staging_belt,
                    &mut encoder,
                    view,
                    (self.size.width, self.size.height),
                )
                .expect("Draw queued");
        }

        self.staging_belt.finish();
        self.queue.submit(Some(encoder.finish()));
        frame.present();

        // Recall unused staging buffers
        self.staging_belt.recall();
    }
}
