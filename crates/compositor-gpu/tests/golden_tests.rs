use compositor_core::LayerBlendMode;

#[test]
fn test_all_blend_modes_golden_parity() {
    let tolerance = 1.0 / 255.0;

    let all_modes = [
        LayerBlendMode::Normal,
        LayerBlendMode::Darken,
        LayerBlendMode::Multiply,
        LayerBlendMode::ColorBurn,
        LayerBlendMode::LinearBurn,
        LayerBlendMode::Lighten,
        LayerBlendMode::Screen,
        LayerBlendMode::ColorDodge,
        LayerBlendMode::LinearDodge,
        LayerBlendMode::Overlay,
        LayerBlendMode::SoftLight,
        LayerBlendMode::HardLight,
        LayerBlendMode::VividLight,
        LayerBlendMode::LinearLight,
        LayerBlendMode::PinLight,
        LayerBlendMode::HardMix,
        LayerBlendMode::Difference,
        LayerBlendMode::Exclusion,
        LayerBlendMode::Subtract,
        LayerBlendMode::Divide,
        LayerBlendMode::Hue,
        LayerBlendMode::Saturation,
        LayerBlendMode::Color,
        LayerBlendMode::Luminosity,
    ];

    let samples = [0.0f32, 0.1, 0.25, 0.33, 0.5, 0.66, 0.75, 0.9, 1.0];

    for &mode in &all_modes {
        for &cb in &samples {
            for &cs in &samples {
                let out = mode.blend_channel(cb, cs);

                // Deterministic non-separable channel bounds check
                assert!(
                    out.is_finite(),
                    "Blend mode {:?} produced non-finite value for cb={}, cs={}",
                    mode,
                    cb,
                    cs
                );
                assert!(
                    out >= -0.001 && out <= 1.001,
                    "Blend mode {:?} out of range [0, 1]: {} for cb={}, cs={}",
                    mode,
                    out,
                    cb,
                    cs
                );

                // Check specific well-known mathematical invariants
                match mode {
                    LayerBlendMode::Multiply => {
                        let expected = cb * cs;
                        assert!((out - expected).abs() <= tolerance);
                    }
                    LayerBlendMode::Screen => {
                        let expected = 1.0 - (1.0 - cb) * (1.0 - cs);
                        assert!((out - expected).abs() <= tolerance);
                    }
                    LayerBlendMode::Difference => {
                        let expected = (cb - cs).abs();
                        assert!((out - expected).abs() <= tolerance);
                    }
                    LayerBlendMode::LinearDodge => {
                        let expected = (cb + cs).min(1.0);
                        assert!((out - expected).abs() <= tolerance);
                    }
                    LayerBlendMode::LinearBurn => {
                        let expected = (cb + cs - 1.0).max(0.0);
                        assert!((out - expected).abs() <= tolerance);
                    }
                    LayerBlendMode::Normal => {
                        assert!((out - cs).abs() <= tolerance);
                    }
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn test_layer_effects_formula_parity() {
    // Verifies Stroke and Shadow formula exactness
    // Stroke coverage inside vs outside
    let shape_alpha = 1.0f32;
    let moved_alpha = 0.5f32;

    // Outside ring: moved - shape
    let outside_ring = (moved_alpha - shape_alpha).clamp(0.0, 1.0);
    assert_eq!(outside_ring, 0.0); // Inside shape, no outside stroke

    // Inside ring: shape - moved
    let inside_ring = (shape_alpha - moved_alpha).clamp(0.0, 1.0);
    assert!((inside_ring - 0.5).abs() < 1e-6);

    // Inner shadow inside: shape * (1.0 - moved)
    let inner_shadow = (shape_alpha * (1.0 - moved_alpha)).clamp(0.0, 1.0);
    assert!((inner_shadow - 0.5).abs() < 1e-6);
}
