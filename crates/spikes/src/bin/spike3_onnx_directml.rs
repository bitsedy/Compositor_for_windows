use compositor_ai::ForegroundSegmenter;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    println!("=== SPIKE 3: ONNX Runtime & Foreground Mask Inference ===");

    let segmenter = ForegroundSegmenter::new()?;
    println!("Execution Provider: {}", segmenter.execution_provider);

    // Test image: 1024 x 1024
    let width = 1024;
    let height = 1024;
    let mut rgba = vec![0u8; width * height * 4];

    // Create synthetic test image with central subject
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let dx = (x as f32 - width as f32 / 2.0) / (width as f32 / 2.0);
            let dy = (y as f32 - height as f32 / 2.0) / (height as f32 / 2.0);
            let r = (dx * dx + dy * dy).sqrt();

            if r < 0.4 {
                // Central subject
                rgba[idx] = 220;
                rgba[idx + 1] = 80;
                rgba[idx + 2] = 60;
                rgba[idx + 3] = 255;
            } else {
                // Background
                rgba[idx] = 30;
                rgba[idx + 1] = 40;
                rgba[idx + 2] = 50;
                rgba[idx + 3] = 255;
            }
        }
    }

    println!("Running foreground mask generation on {}x{} image...", width, height);
    let start = Instant::now();
    let mask = segmenter.generate_mask(&rgba, width, height)?;
    let elapsed = start.elapsed();

    let center_val = mask[(height / 2) * width + (width / 2)];
    let corner_val = mask[0];

    println!("--------------------------------------------------");
    println!("SPIKE 3 RESULTS (Foreground Mask Generation):");
    println!("  Inference Latency:   {:.2} ms", elapsed.as_secs_f64() * 1000.0);
    println!("  Center Mask Value:   {} (expected > 200)", center_val);
    println!("  Corner Mask Value:   {} (expected < 50)", corner_val);
    println!("  Target Budget:       < 500 ms");
    if elapsed.as_millis() < 500 {
        println!("  STATUS: PASSED (Budget Met)");
    } else {
        println!("  STATUS: FAILED (Budget Exceeded)");
    }
    println!("--------------------------------------------------");

    assert!(elapsed.as_millis() < 500, "Spike 3 failed latency budget");
    assert!(center_val > 200, "Center subject not detected");
    assert!(corner_val < 50, "Background not suppressed");

    Ok(())
}
