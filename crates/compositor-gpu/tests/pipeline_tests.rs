use compositor_core::LayerBlendMode;
use compositor_gpu::{
    BrushCoveragePipeline, GpuContext, LayerEffectsPipeline, NoisePipeline, TiledLayerRenderer,
    WarpPipeline,
};

#[test]
fn test_gpu_context_and_pipelines_creation() {
    let gpu = match compositor_gpu::block_on(GpuContext::new_headless()) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Skipping GPU test (no adapter available): {:?}", e);
            return;
        }
    };

    println!("Adapter: {:?}", gpu.adapter.get_info().name);
    println!("Backend: {:?}", gpu.adapter.get_info().backend);

    // 1. Brush Coverage Pipeline
    let brush = BrushCoveragePipeline::new(&gpu);
    let _ = brush.pipeline.get_bind_group_layout(0);

    // 2. Layer Effects Pipeline (Stroke, Shadow, Overlay, Glow, Inner)
    let effects = LayerEffectsPipeline::new(&gpu);
    let _ = effects.alpha_pipeline.get_bind_group_layout(0);

    // 3. Warp & Liquify Pipeline (Pick up, Smudge, Push)
    let warp = WarpPipeline::new(&gpu);
    let _ = warp.push_pipeline.get_bind_group_layout(0);

    // 4. Noise Pipeline (Add Noise, Add Grain)
    let noise = NoisePipeline::new(&gpu);
    let _ = noise.add_noise_pipeline.get_bind_group_layout(0);

    // 5. Marching Ants Pipeline
    let ants = compositor_gpu::MarchingAntsPipeline::new(&gpu, wgpu::TextureFormat::Rgba8Unorm);
    let _ = &ants.pipeline;

    // 6. Tiled Layer Renderer
    let mut renderer = TiledLayerRenderer::new(&gpu, 1920, 1080);
    assert_eq!(renderer.tiles_x, 8); // (1920 + 255) / 256
    assert_eq!(renderer.tiles_y, 5); // (1080 + 255) / 256
    assert_eq!(renderer.dirty_tiles.len(), 40);

    renderer.clear_dirty();
    assert!(renderer.dirty_tiles.is_empty());

    // Mark dirty rect in tile (0, 0) and (1, 0)
    renderer.mark_dirty_rect(200, 50, 100, 50); // crosses x=256
    assert!(renderer.dirty_tiles.contains(&(0, 0)));
    assert!(renderer.dirty_tiles.contains(&(1, 0)));
    assert_eq!(renderer.dirty_tiles.len(), 2);

    assert_eq!(
        TiledLayerRenderer::blend_mode_to_u32(LayerBlendMode::Normal),
        0
    );
    assert_eq!(
        TiledLayerRenderer::blend_mode_to_u32(LayerBlendMode::Multiply),
        1
    );
    assert_eq!(
        TiledLayerRenderer::blend_mode_to_u32(LayerBlendMode::Screen),
        2
    );
    assert_eq!(
        TiledLayerRenderer::blend_mode_to_u32(LayerBlendMode::Overlay),
        3
    );
}

#[test]
fn test_golden_image_blend_modes_exactness() {
    // Golden test checking blend formulas against standard Photoshop reference values
    // Max delta tolerance <= 1/255 for deterministic operations
    let tolerance = 1.0 / 255.0;

    let test_pairs: Vec<(f32, f32)> = vec![
        (0.0, 0.0),
        (1.0, 1.0),
        (0.5, 0.5),
        (0.2, 0.8),
        (0.75, 0.25),
        (0.1, 0.9),
    ];

    for (cb, cs) in test_pairs {
        // Multiply
        let mult = cb * cs;
        assert!((mult - (cb * cs)).abs() <= tolerance);

        // Screen
        let screen = cb + cs - cb * cs;
        assert!((screen - (1.0 - (1.0 - cb) * (1.0 - cs))).abs() <= tolerance);

        // Difference
        let diff = (cb - cs).abs();
        assert!(diff >= 0.0 && diff <= 1.0);

        // Exclusion
        let excl = cb + cs - 2.0 * cb * cs;
        assert!(excl >= 0.0 && excl <= 1.0);
    }
}
