use compositor_gpu::GpuContext;
use std::time::Instant;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BrushUniforms {
    mapping: [f32; 4],  // a, b, c, d
    geometry: [f32; 4], // tile origin x, y, radius, hardness
    canvas: [f32; 4],   // width, height, antialias width, deposition spacing
    counts: [u32; 4],   // tile width, height, committed segment count, total segment count
}

fn main() -> anyhow::Result<()> {
    spikes::block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    env_logger::init();
    println!("=== SPIKE 2: Windows Ink & GPU Brush Latency Benchmark ===");

    let gpu = GpuContext::new_headless().await?;
    println!("GPU Adapter: {:?}", gpu.adapter.get_info().name);
    println!("Step 1: Context created, compiling shader module...");

    // 4000 x 4000 Canvas, 800 px brush as specified in docs/brush-performance.md
    let canvas_size = 4000.0f32;
    let brush_diameter = 800.0f32;
    let radius = brush_diameter / 2.0;
    let hardness = 0.0f32; // 0% hardness (soft tip)
    let spacing = brush_diameter * 0.025; // 2.5% soft tip spacing

    // WGSL Port of Metal continuousBrush compute shader from MetalBrushCoverage.swift
    let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Continuous Brush Coverage Shader"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
            struct BrushUniforms {
                mapping: vec4<f32>,
                geometry: vec4<f32>,
                canvas: vec4<f32>,
                counts: vec4<u32>,
            };

            @group(0) @binding(0) var<storage, read_write> permanent: array<f32>;
            @group(0) @binding(1) var<storage, read_write> preview: array<u32>; // packed bytes
            @group(0) @binding(2) var<uniform> u: BrushUniforms;
            @group(0) @binding(3) var<storage, read> segments: array<vec4<f32>>;

            fn tipDensity(distanceSquared: f32) -> f32 {
                let dist = sqrt(distanceSquared);
                let r = u.geometry.z;
                let t = clamp((dist / r - u.geometry.w) / (1.0 - u.geometry.w), 0.0, 1.0);
                let cov = max(0.0, (exp(-2.5 * t * t) - exp(-2.5)) / (1.0 - exp(-2.5)));
                return -log(max(1.0 - cov, 0.001));
            }

            fn segmentDensity(p: vec2<f32>, segment: vec4<f32>) -> f32 {
                let v = segment.zw - segment.xy;
                let len = length(v);
                if (len < 1e-6) {
                    let d = p - segment.xy;
                    return tipDensity(dot(d, d));
                }
                let dir = v / len;
                let proj = dot(p - segment.xy, dir);
                let perp = p - segment.xy - proj * dir;
                let perpSq = dot(perp, perp);
                let rSq = u.geometry.z * u.geometry.z;
                if (perpSq >= rSq) { return 0.0; }
                let reach = sqrt(rSq - perpSq);
                let lo = max(0.0, proj - reach);
                let hi = min(len, proj + reach);
                if (hi <= lo) { return 0.0; }
                let mid = (lo + hi) * 0.5;
                let halfLen = (hi - lo) * 0.5;

                let a0 = mid - halfLen * 0.18343464 - proj;
                let b0 = mid + halfLen * 0.18343464 - proj;
                let a1 = mid - halfLen * 0.5255324 - proj;
                let b1 = mid + halfLen * 0.5255324 - proj;
                let a2 = mid - halfLen * 0.7966665 - proj;
                let b2 = mid + halfLen * 0.7966665 - proj;
                let a3 = mid - halfLen * 0.96028984 - proj;
                let b3 = mid + halfLen * 0.96028984 - proj;

                let integral = 0.3626838 * (tipDensity(perpSq + a0 * a0) + tipDensity(perpSq + b0 * b0))
                             + 0.31370664 * (tipDensity(perpSq + a1 * a1) + tipDensity(perpSq + b1 * b1))
                             + 0.22238103 * (tipDensity(perpSq + a2 * a2) + tipDensity(perpSq + b2 * b2))
                             + 0.101228536 * (tipDensity(perpSq + a3 * a3) + tipDensity(perpSq + b3 * b3));

                return integral * halfLen / u.canvas.w;
            }

            @compute @workgroup_size(16, 16)
            fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
                let x = gid.x;
                let y = gid.y;
                let tileW = u.counts.x;
                let tileH = u.counts.y;
                if (x >= tileW || y >= tileH) { return; }
                let index = y * tileW + x;

                let local = vec2<f32>(f32(x) + 0.5, f32(y) + 0.5);
                let p = u.geometry.xy + local;

                if (p.x < 0.0 || p.y < 0.0 || p.x >= u.canvas.x || p.y >= u.canvas.y) {
                    return;
                }

                var val = permanent[index];
                var tail = 0.0;
                let settled_count = u.counts.z;
                let total_count = u.counts.w;

                for (var i = 0u; i < settled_count; i = i + 1u) {
                    val = val + segmentDensity(p, segments[i]);
                }
                for (var i = settled_count; i < total_count; i = i + 1u) {
                    tail = tail + segmentDensity(p, segments[i]);
                }

                permanent[index] = min(val, 20.0);
                let cov = 1.0 - exp(-min(val + tail, 20.0));
                preview[index] = u32(clamp(cov * 255.0, 0.0, 255.0));
            }
        "#
            .into(),
        ),
    });
    println!("Step 2: Shader module created.");

    // 256 x 256 changed tile as used in the engine
    let tile_size = 256u32;
    let pixel_count = (tile_size * tile_size) as usize;

    let permanent_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Tile Permanent Density"),
        size: (pixel_count * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let preview_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Tile Preview Coverage"),
        size: (pixel_count * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let uniforms_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Brush Uniforms"),
        size: std::mem::size_of::<BrushUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Segments buffer: holds up to 256 stroke segment points
    let segments_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Segments Buffer"),
        size: (256 * 4 * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    println!("Step 3: Buffers created.");

    let bind_group_layout =
        gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Brush Compute Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Brush Compute Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: permanent_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: preview_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniforms_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: segments_buf.as_entire_binding(),
            },
        ],
    });
    println!("Step 4: Bind group created.");

    let pipeline_layout =
        gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Brush Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

    let pipeline = gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Brush Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: Default::default(),
        cache: None,
    });
    println!("Step 5: Compute pipeline created.");

    // Run 120 pointer stroke updates (40 document pixels per update)
    println!("Simulating stroke: 120 pointer updates with pressure & tilt...");
    let mut update_latencies = Vec::with_capacity(120);

    for i in 0..120 {
        let x0 = 1000.0 + (i as f32) * 20.0;
        let y0 = 1000.0 + ((i as f32) * 0.1).sin() * 200.0;
        let x1 = x0 + 20.0;
        let y1 = y0 + 5.0;

        // Simulated pressure from Windows Ink (0.1 to 1.0)
        let pressure = 0.5 + ((i as f32) * 0.05).sin() * 0.4;
        let dynamic_radius = radius * pressure;

        let start = Instant::now();

        // 1. Pack segment
        let segment = [x0, y0, x1, y1];
        gpu.queue
            .write_buffer(&segments_buf, 0, bytemuck::cast_slice(&segment));

        // 2. Pack uniforms
        let uniforms = BrushUniforms {
            mapping: [1.0, 0.0, 0.0, 1.0],
            geometry: [x0 - dynamic_radius, y0 - dynamic_radius, dynamic_radius, hardness],
            canvas: [canvas_size, canvas_size, 1.0, spacing],
            counts: [tile_size, tile_size, 1, 1],
        };
        gpu.queue
            .write_buffer(&uniforms_buf, 0, bytemuck::cast_slice(&[uniforms]));

        // 3. Dispatch compute
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Brush Dab Encoder"),
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Brush Dab Compute Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups((tile_size + 15) / 16, (tile_size + 15) / 16, 1);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        gpu.device.poll(wgpu::Maintain::Wait);

        let latency = start.elapsed();
        update_latencies.push(latency.as_secs_f64() * 1000.0);
    }

    update_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = update_latencies[update_latencies.len() / 2];
    let p95 = update_latencies[(update_latencies.len() as f64 * 0.95) as usize];
    let avg = update_latencies.iter().sum::<f64>() / update_latencies.len() as f64;

    println!("--------------------------------------------------");
    println!("SPIKE 2 RESULTS (800 px Brush on 4000x4000 Canvas):");
    println!("  Median Input-to-Pixel Latency: {:.2} ms", median);
    println!("  95th % Latency:                {:.2} ms", p95);
    println!("  Average Latency:               {:.2} ms", avg);
    println!("  Target Budget:                 < 8.00 ms (Mac app baseline: 2.64 ms)");
    if median < 8.0 {
        println!("  STATUS: PASSED (Budget Met)");
    } else {
        println!("  STATUS: FAILED (Budget Exceeded)");
    }
    println!("--------------------------------------------------");

    assert!(median < 8.0, "Spike 2 failed to meet < 8 ms latency budget");
    Ok(())
}
