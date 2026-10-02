use compositor_gpu::GpuContext;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    spikes::block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    env_logger::init();
    println!("=== SPIKE 1: wgpu 24 MP Tiled Pan/Zoom Performance ===");

    let gpu = GpuContext::new_headless().await?;
    println!("GPU Adapter: {:?}", gpu.adapter.get_info().name);
    println!("Backend: {:?}", gpu.adapter.get_info().backend);

    // 24 MP Document: 6000 x 4000
    let width = 6000;
    let height = 4000;
    println!("Allocating 24 MP image ({}x{}) textures...", width, height);

    // Create 24 MP texture in GPU memory (RGBA8)
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("24MP Layer Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    // Viewport render target: 4K display viewport (3840 x 2160)
    let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Viewport Target"),
        size: wgpu::Extent3d {
            width: 3840,
            height: 2160,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    // Simple blit / sampling shader in WGSL for pan/zoom
    let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("PanZoom Shader"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
            struct VertexOutput {
                @builtin(position) position: vec4<f32>,
                @location(0) uv: vec2<f32>,
            };

            struct Uniforms {
                offset: vec2<f32>,
                scale: vec2<f32>,
            };

            @group(0) @binding(0) var<uniform> u: Uniforms;
            @group(0) @binding(1) var t_diffuse: texture_2d<f32>;
            @group(0) @binding(2) var s_diffuse: sampler;

            @vertex
            fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
                var out: VertexOutput;
                var pos = array<vec2<f32>, 4>(
                    vec2<f32>(-1.0, -1.0),
                    vec2<f32>( 1.0, -1.0),
                    vec2<f32>(-1.0,  1.0),
                    vec2<f32>( 1.0,  1.0)
                );
                var uv = array<vec2<f32>, 4>(
                    vec2<f32>(0.0, 1.0),
                    vec2<f32>(1.0, 1.0),
                    vec2<f32>(0.0, 0.0),
                    vec2<f32>(1.0, 0.0)
                );
                out.position = vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
                out.uv = uv[in_vertex_index] * u.scale + u.offset;
                return out;
            }

            @fragment
            fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                return textureSample(t_diffuse, s_diffuse, in.uv);
            }
        "#
            .into(),
        ),
    });

    let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let uniform_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("PanZoom Uniforms"),
        size: 32, // 2 x vec2<f32> + padding
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout =
        gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PanZoom Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("PanZoom Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(
                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    let pipeline_layout =
        gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PanZoom Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

    let pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("PanZoom Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    // Run 120 simulated pan/zoom frames
    println!("Simulating 120 continuous pan/zoom frames...");
    let mut frame_durations = Vec::with_capacity(120);

    for i in 0..120 {
        let t = i as f32 * 0.05;
        let offset = [t.sin() * 0.1, t.cos() * 0.1];
        let scale = [1.0 + t.sin() * 0.2, 1.0 + t.sin() * 0.2];

        let frame_start = Instant::now();

        // Update uniforms
        let uniforms: [f32; 8] = [offset[0], offset[1], 0.0, 0.0, scale[0], scale[1], 0.0, 0.0];
        gpu.queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&uniforms));

        // Render pass
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("PanZoom Encoder"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("PanZoom Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..4, 0..1);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        gpu.device.poll(wgpu::Maintain::Wait);

        let duration = frame_start.elapsed();
        frame_durations.push(duration.as_secs_f64() * 1000.0);
    }

    frame_durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = frame_durations[frame_durations.len() / 2];
    let p95 = frame_durations[(frame_durations.len() as f64 * 0.95) as usize];
    let avg = frame_durations.iter().sum::<f64>() / frame_durations.len() as f64;
    let fps = 1000.0 / avg;

    println!("--------------------------------------------------");
    println!("SPIKE 1 RESULTS (24 MP Tiled Pan/Zoom on 4K Target):");
    println!("  Average Frame Time: {:.2} ms ({:.1} FPS)", avg, fps);
    println!("  Median Frame Time:  {:.2} ms", median);
    println!("  95th % Frame Time:  {:.2} ms", p95);
    println!("  Target Budget:      <= 16.67 ms (60 FPS)");
    if avg <= 16.67 {
        println!("  STATUS: PASSED (Budget Met)");
    } else {
        println!("  STATUS: FAILED (Budget Exceeded)");
    }
    println!("--------------------------------------------------");

    assert!(avg <= 16.67, "Spike 1 failed to meet 60 fps budget");
    Ok(())
}
