pub mod c_bindings;

pub use c_bindings::*;

/// Calculates the bounding box of non-zero alpha in an RGBA buffer.
/// Returns Option<[usize; 4]>: `[left, top, right, bottom]`.
pub fn get_brush_alpha_bounds(
    bytes: &[u8],
    width: usize,
    height: usize,
    stride: usize,
) -> Option<[usize; 4]> {
    let mut bounds = [0usize; 4];
    unsafe {
        c_bindings::brush_alpha_bounds(
            bytes.as_ptr(),
            width,
            height,
            stride,
            bounds.as_mut_ptr(),
        );
    }
    if bounds[2] > bounds[0] && bounds[3] > bounds[1] {
        Some(bounds)
    } else {
        None
    }
}

/// Extracts alpha channel (4th byte of each RGBA pixel) into a grayscale buffer.
pub fn extract_alpha(
    rgba: &[u8],
    rgba_stride: usize,
    gray: &mut [u8],
    gray_stride: usize,
    width: usize,
    height: usize,
) {
    unsafe {
        c_bindings::layer_extract_alpha(
            rgba.as_ptr(),
            rgba_stride,
            gray.as_mut_ptr(),
            gray_stride,
            width,
            height,
        );
    }
}

/// Clamps premultiplied RGBA pixels to prevent color exceeding alpha.
pub fn clamp_premultiplied(rgba: &mut [u8], count: usize) {
    unsafe {
        c_bindings::rgba_clamp_premultiplied(rgba.as_mut_ptr(), count);
    }
}

/// Applies uniform or Gaussian noise to RGBA buffer.
pub fn add_noise(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    amount: f32,
    gaussian: bool,
    monochromatic: bool,
    seed: u32,
) {
    unsafe {
        c_bindings::noise_add(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            amount,
            if gaussian { 1 } else { 0 },
            if monochromatic { 1 } else { 0 },
            seed,
        );
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealMode {
    ContentAware = 0,
    CreateTexture = 1,
    ProximityMatch = 2,
}

pub fn spot_heal(
    rgba: &mut [u8],
    coverage: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    opacity: f32,
    mode: HealMode,
    seed: u32,
) -> Result<(), &'static str> {
    let res = unsafe {
        c_bindings::spot_heal(
            rgba.as_mut_ptr(),
            coverage.as_ptr(),
            width,
            height,
            stride,
            opacity,
            mode as i32,
            seed,
        )
    };
    if res == 0 {
        Ok(())
    } else {
        Err("Out of memory in spot heal")
    }
}

pub fn content_fill(
    rgba: &mut [u8],
    stride: usize,
    mask: &[u8],
    mask_stride: usize,
    width: i32,
    height: i32,
) -> Result<bool, &'static str> {
    let res = unsafe {
        c_bindings::content_fill(
            rgba.as_mut_ptr(),
            stride,
            mask.as_ptr(),
            mask_stride,
            width,
            height,
        )
    };
    match res {
        1 => Ok(true),
        0 => Ok(false),
        _ => Err("Allocation failure in content fill"),
    }
}

pub fn wand_mask(
    rgba: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    seed_x: usize,
    seed_y: usize,
    radius: usize,
    tolerance: i32,
    contiguous: bool,
    mask: &mut [u8],
) -> Result<usize, &'static str> {
    let count = unsafe {
        c_bindings::wand_mask(
            rgba.as_ptr(),
            width,
            height,
            stride,
            seed_x,
            seed_y,
            radius,
            tolerance,
            if contiguous { 1 } else { 0 },
            mask.as_mut_ptr(),
        )
    };
    if count >= 0 {
        Ok(count as usize)
    } else {
        Err("Out of memory in wand mask")
    }
}

pub fn color_range_mask(
    rgba: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    include: &[u8],
    exclude: &[u8],
    fuzziness: i32,
    invert: bool,
    mask: &mut [u8],
) -> usize {
    let inc_count = (include.len() / 3) as i32;
    let exc_count = (exclude.len() / 3) as i32;
    let count = unsafe {
        c_bindings::color_range_mask(
            rgba.as_ptr(),
            width,
            height,
            stride,
            include.as_ptr(),
            inc_count,
            exclude.as_ptr(),
            exc_count,
            fuzziness,
            if invert { 1 } else { 0 },
            mask.as_mut_ptr(),
        )
    };
    count.max(0) as usize
}

pub fn levels_apply(pixels: &mut [u8], count: usize, tables: &[f32; 256 * 4]) {
    unsafe {
        c_bindings::levels_apply(pixels.as_mut_ptr(), count, tables.as_ptr());
    }
}

pub fn lens_distort(
    source: &[u8],
    destination: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    k: f64,
) {
    unsafe {
        c_bindings::lens_distort(
            source.as_ptr(),
            destination.as_mut_ptr(),
            width,
            height,
            stride,
            k,
        );
    }
}

pub fn adjust_gradient_map(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    table: &[u8; 256 * 3],
) {
    unsafe {
        c_bindings::adjust_gradient_map(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            table.as_ptr(),
        );
    }
}

pub fn adjust_color_balance(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    shadows: &[f32; 3],
    midtones: &[f32; 3],
    highlights: &[f32; 3],
    preserve_luminosity: bool,
) {
    unsafe {
        c_bindings::adjust_color_balance(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            shadows.as_ptr(),
            midtones.as_ptr(),
            highlights.as_ptr(),
            if preserve_luminosity { 1 } else { 0 },
        );
    }
}

pub fn adjust_black_white(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    weights: &[f32; 6],
    tint: bool,
    tint_hue: f64,
    tint_saturation: f64,
) {
    unsafe {
        c_bindings::adjust_black_white(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            weights.as_ptr(),
            if tint { 1 } else { 0 },
            tint_hue,
            tint_saturation,
        );
    }
}

pub fn adjust_grain(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    amount: f64,
    size: f64,
    roughness: f64,
    seed: u32,
    origin_x: f64,
    origin_y: f64,
    units_per_pixel: f64,
) {
    unsafe {
        c_bindings::adjust_grain(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            amount,
            size,
            roughness,
            seed,
            origin_x,
            origin_y,
            units_per_pixel,
        );
    }
}

pub fn adjust_tonal_contrast(
    rgba: &mut [u8],
    blurred: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    blurred_stride: usize,
    amount: f64,
    shadows: f64,
    midtones: f64,
    highlights: f64,
) {
    unsafe {
        c_bindings::adjust_tonal_contrast(
            rgba.as_mut_ptr(),
            blurred.as_ptr(),
            width,
            height,
            stride,
            blurred_stride,
            amount,
            shadows,
            midtones,
            highlights,
        );
    }
}

pub fn adjust_colored_vignette(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    frame_x: f64,
    frame_y: f64,
    frame_width: f64,
    frame_height: f64,
    fills_clear: bool,
    amount: f64,
    midpoint: f64,
    roundness: f64,
    feather: f64,
    highlights: f64,
    red: f64,
    green: f64,
    blue: f64,
) {
    unsafe {
        c_bindings::adjust_colored_vignette(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            frame_x,
            frame_y,
            frame_width,
            frame_height,
            if fills_clear { 1 } else { 0 },
            amount,
            midpoint,
            roundness,
            feather,
            highlights,
            red,
            green,
            blue,
        );
    }
}

pub fn cube_apply(pixels: &mut [u8], count: usize, cube: &[f32], dimension: i32) {
    unsafe {
        c_bindings::cube_apply(pixels.as_mut_ptr(), count, cube.as_ptr(), dimension);
    }
}

pub fn levels_histogram(
    pixels: &[u8],
    coverage: Option<&[u8]>,
    count: usize,
    bins: &mut [f64; 1024],
) {
    unsafe {
        c_bindings::levels_histogram(
            pixels.as_ptr(),
            coverage.map_or(std::ptr::null(), |c| c.as_ptr()),
            count,
            bins.as_mut_ptr(),
        );
    }
}

pub fn adjust_camera_raw(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    red_gain: f64,
    green_gain: f64,
    blue_gain: f64,
    exposure: f64,
    contrast: f64,
    highlights: f64,
    shadows: f64,
    whites: f64,
    blacks: f64,
    vibrance: f64,
    saturation: f64,
    clipping: i32,
) {
    unsafe {
        c_bindings::adjust_camera_raw(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            red_gain,
            green_gain,
            blue_gain,
            exposure,
            contrast,
            highlights,
            shadows,
            whites,
            blacks,
            vibrance,
            saturation,
            clipping,
        );
    }
}

pub fn adjust_camera_raw_clip_overlay(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    shadows: bool,
    highlights: bool,
) {
    unsafe {
        c_bindings::adjust_camera_raw_clip_overlay(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            if shadows { 1 } else { 0 },
            if highlights { 1 } else { 0 },
        );
    }
}

pub fn adjust_camera_raw_curve_color(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    tone_lut: &[f32; 256],
    red_lut: &[f32; 256],
    green_lut: &[f32; 256],
    blue_lut: &[f32; 256],
    refine_saturation: f64,
    mixer: &[f32; 24],
    points: &[f32],
    point_count: usize,
    grade: &[f32; 12],
    blending: f64,
    balance: f64,
    visualize: i32,
) {
    unsafe {
        c_bindings::adjust_camera_raw_curve_color(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            tone_lut.as_ptr(),
            red_lut.as_ptr(),
            green_lut.as_ptr(),
            blue_lut.as_ptr(),
            refine_saturation,
            mixer.as_ptr(),
            point_count as i32,
            points.as_ptr(),
            grade.as_ptr(),
            blending,
            balance,
            visualize,
        );
    }
}

pub fn adjust_camera_raw_effects(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    texture: f64,
    clarity: f64,
    dehaze: f64,
    glow: f64,
    glow_style: i32,
    glow_range: f64,
    glow_spread: f64,
    glow_warmth: f64,
    vignette_amount: f64,
    vignette_midpoint: f64,
    vignette_roundness: f64,
    vignette_feather: f64,
    vignette_highlights: f64,
    vignette_style: i32,
    scale: f64,
) {
    unsafe {
        c_bindings::adjust_camera_raw_effects(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            texture,
            clarity,
            dehaze,
            glow,
            glow_style,
            glow_range,
            glow_spread,
            glow_warmth,
            vignette_amount,
            vignette_midpoint,
            vignette_roundness,
            vignette_feather,
            vignette_highlights,
            vignette_style,
            scale,
        );
    }
}

pub fn adjust_camera_raw_detail(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    sharpen_amount: f64,
    sharpen_radius: f64,
    sharpen_detail: f64,
    sharpen_masking: f64,
    noise_luminance: f64,
    noise_luminance_detail: f64,
    noise_luminance_contrast: f64,
    noise_color: f64,
    noise_color_detail: f64,
    noise_color_smoothness: f64,
    scale: f64,
) {
    unsafe {
        c_bindings::adjust_camera_raw_detail(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            sharpen_amount,
            sharpen_radius,
            sharpen_detail,
            sharpen_masking,
            noise_luminance,
            noise_luminance_detail,
            noise_luminance_contrast,
            noise_color,
            noise_color_detail,
            noise_color_smoothness,
            scale,
        );
    }
}

pub fn adjust_camera_raw_sharpen_mask_overlay(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    sharpen_radius: f64,
    sharpen_detail: f64,
    sharpen_masking: f64,
    scale: f64,
) {
    unsafe {
        c_bindings::adjust_camera_raw_sharpen_mask_overlay(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            sharpen_radius,
            sharpen_detail,
            sharpen_masking,
            scale,
        );
    }
}

pub fn adjust_camera_raw_optics(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    remove_chromatic: bool,
    lens_profile: bool,
    profile_distortion: f64,
    profile_vignetting: f64,
    distortion_k: f64,
    purple_amount: f64,
    purple_hue_low: f64,
    purple_hue_high: f64,
    green_amount: f64,
    green_hue_low: f64,
    green_hue_high: f64,
    vignette_amount: f64,
    vignette_midpoint: f64,
    scale: f64,
) {
    unsafe {
        c_bindings::adjust_camera_raw_optics(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            if remove_chromatic { 1 } else { 0 },
            if lens_profile { 1 } else { 0 },
            profile_distortion,
            profile_vignetting,
            distortion_k,
            purple_amount,
            purple_hue_low,
            purple_hue_high,
            green_amount,
            green_hue_low,
            green_hue_high,
            vignette_amount,
            vignette_midpoint,
            scale,
        );
    }
}

pub fn adjust_camera_raw_calibration(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    shadow_tint: f64,
    red_hue: f64,
    red_saturation: f64,
    green_hue: f64,
    green_saturation: f64,
    blue_hue: f64,
    blue_saturation: f64,
    process_version: i32,
) {
    unsafe {
        c_bindings::adjust_camera_raw_calibration(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            shadow_tint,
            red_hue,
            red_saturation,
            green_hue,
            green_saturation,
            blue_hue,
            blue_saturation,
            process_version,
        );
    }
}

pub fn dither_apply(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    params: &c_bindings::CDitherParams,
) -> Result<(), &'static str> {
    let res = unsafe {
        c_bindings::dither_apply(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            params as *const _,
        )
    };
    if res != 0 {
        Ok(())
    } else {
        Err("Dither apply failed")
    }
}

pub fn dither_dots(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    block: i32,
    gap: &[u8; 3],
) {
    unsafe {
        c_bindings::dither_dots(
            rgba.as_mut_ptr(),
            width,
            height,
            stride,
            block,
            gap.as_ptr(),
        );
    }
}

pub fn dither_glow(
    rgba: &mut [u8],
    glow: &[u8],
    width: usize,
    height: usize,
    stride: usize,
    amount: f32,
) {
    unsafe {
        c_bindings::dither_glow(
            rgba.as_mut_ptr(),
            glow.as_ptr(),
            width,
            height,
            stride,
            amount,
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpha_bounds_empty() {
        let buffer = vec![0u8; 16 * 16 * 4];
        let bounds = get_brush_alpha_bounds(&buffer, 16, 16, 16 * 4);
        assert_eq!(bounds, None);
    }

    #[test]
    fn test_alpha_bounds_with_pixel() {
        let mut buffer = vec![0u8; 16 * 16 * 4];
        // Set a pixel at (4, 5) with non-zero alpha
        let idx = (5 * 16 + 4) * 4 + 3;
        buffer[idx] = 255;
        let bounds = get_brush_alpha_bounds(&buffer, 16, 16, 16 * 4).unwrap();
        assert_eq!(bounds[0], 4);
        assert_eq!(bounds[1], 5);
        assert_eq!(bounds[2], 5);
        assert_eq!(bounds[3], 6);
    }

    #[test]
    fn test_extract_alpha() {
        let rgba = vec![10, 20, 30, 200, 40, 50, 60, 150];
        let mut gray = vec![0u8; 2];
        extract_alpha(&rgba, 8, &mut gray, 2, 2, 1);
        assert_eq!(gray, vec![200, 150]);
    }

    #[test]
    fn test_wand_mask() {
        let w = 16;
        let h = 16;
        let stride = w * 4;
        let mut rgba = vec![0u8; w * h * 4];
        // Fill with blue [0, 0, 255, 255]
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) * 4;
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 255;
                rgba[idx + 3] = 255;
            }
        }
        // Center 4x4 (from 6..10) is red [255, 0, 0, 255]
        for y in 6..10 {
            for x in 6..10 {
                let idx = (y * w + x) * 4;
                rgba[idx] = 255;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 255;
            }
        }

        let mut mask = vec![0u8; w * h];
        let count = wand_mask(&rgba, w, h, stride, 8, 8, 0, 10, true, &mut mask).unwrap();
        assert_eq!(count, 16);
        for y in 6..10 {
            for x in 6..10 {
                assert_eq!(mask[y * w + x], 255);
            }
        }
        assert_eq!(mask[0], 0);
    }

    #[test]
    fn test_color_range_mask() {
        let w = 8;
        let h = 8;
        let stride = w * 4;
        let mut rgba = vec![0u8; w * h * 4];
        // Half red, half green
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) * 4;
                if x < 4 {
                    rgba[idx] = 255; // Red
                } else {
                    rgba[idx + 1] = 255; // Green
                }
                rgba[idx + 3] = 255;
            }
        }

        let include = [255u8, 0, 0];
        let exclude = [];
        let mut mask = vec![0u8; w * h];
        let count = color_range_mask(&rgba, w, h, stride, &include, &exclude, 20, false, &mut mask);
        assert!(count > 0);
        // Red side should be selected (255), green side should be 0
        assert_eq!(mask[0], 255);
        assert_eq!(mask[7], 0);
    }

    #[test]
    fn test_spot_heal() {
        let w = 16;
        let h = 16;
        let stride = w * 4;
        let mut rgba = vec![128u8; w * h * 4];
        let mut coverage = vec![0u8; w * h];
        // Defect at (8, 8)
        coverage[8 * w + 8] = 255;
        let res = spot_heal(&mut rgba, &coverage, w, h, stride, 1.0, HealMode::ProximityMatch, 42);
        assert!(res.is_ok());
    }

    #[test]
    fn test_content_fill() {
        let w = 16;
        let h = 16;
        let stride = w * 4;
        let mut rgba = vec![100u8; w * h * 4];
        for i in 0..w * h {
            rgba[i * 4 + 3] = 255;
        }
        let mut mask = vec![0u8; w * h];
        // Mask a 2x2 hole in the center
        mask[7 * w + 7] = 255;
        mask[7 * w + 8] = 255;
        mask[8 * w + 7] = 255;
        mask[8 * w + 8] = 255;

        let res = content_fill(&mut rgba, stride, &mask, w, w as i32, h as i32);
        assert!(res.is_ok());
    }

    #[test]
    fn test_levels_apply() {
        let mut pixels = vec![100u8, 150, 200, 255];
        let mut tables = [0.0f32; 256 * 4];
        // Invert lookup table for all channels
        for ch in 0..4 {
            for i in 0..256 {
                tables[ch * 256 + i] = (255 - i) as f32 / 255.0;
            }
        }
        levels_apply(&mut pixels, 1, &tables);
        assert_eq!(pixels[0], 155); // 255 - 100
        assert_eq!(pixels[1], 105); // 255 - 150
        assert_eq!(pixels[2], 55);  // 255 - 200
        assert_eq!(pixels[3], 255); // alpha preserved
    }

    #[test]
    fn test_cube_apply() {
        // Identity 2x2x2 cube
        let mut cube = [0.0f32; 2 * 2 * 2 * 4];
        for b in 0..2 {
            for g in 0..2 {
                for r in 0..2 {
                    let idx = ((b * 2 + g) * 2 + r) * 4;
                    cube[idx] = r as f32;
                    cube[idx + 1] = g as f32;
                    cube[idx + 2] = b as f32;
                    cube[idx + 3] = 1.0;
                }
            }
        }

        let mut pixels = vec![64u8, 128, 192, 255];
        cube_apply(&mut pixels, 1, &cube, 2);
        // Should remain approximately the same within rounding
        assert!((pixels[0] as i32 - 64).abs() <= 2);
        assert!((pixels[1] as i32 - 128).abs() <= 2);
        assert!((pixels[2] as i32 - 192).abs() <= 2);
        assert_eq!(pixels[3], 255);
    }

    #[test]
    fn test_lens_distort() {
        let w = 16;
        let h = 16;
        let stride = w * 4;
        let mut src = vec![0u8; w * h * 4];
        let mut dst = vec![0u8; w * h * 4];
        for i in 0..w * h {
            src[i * 4] = 200;
            src[i * 4 + 3] = 255;
        }
        lens_distort(&src, &mut dst, w, h, stride, 0.05);
        // Center pixel should be sampled and non-zero
        let center_idx = (8 * w + 8) * 4;
        assert_eq!(dst[center_idx + 3], 255);
    }

    #[test]
    fn test_adjust_gradient_map() {
        let w = 4;
        let h = 4;
        let stride = w * 4;
        let mut rgba = vec![128u8; w * h * 4];
        // 256 * 3 table mapping luminance to pure red
        let mut table = [0u8; 256 * 3];
        for i in 0..256 {
            table[i * 3] = 255;     // R
            table[i * 3 + 1] = 0;   // G
            table[i * 3 + 2] = 0;   // B
        }
        adjust_gradient_map(&mut rgba, w, h, stride, &table);
        assert_eq!(rgba[0], 128); // premultiplied against alpha=128 gives 255 * (128/255) = 128
        assert_eq!(rgba[1], 0);
        assert_eq!(rgba[2], 0);
        assert_eq!(rgba[3], 128);
    }

    #[test]
    fn test_adjust_color_balance() {
        let w = 4;
        let h = 4;
        let stride = w * 4;
        let mut rgba = vec![128u8; w * h * 4];
        for i in 0..w * h {
            rgba[i * 4 + 3] = 255;
        }
        let shadows = [0.2f32, 0.0, -0.2];
        let midtones = [0.1f32, 0.0, -0.1];
        let highlights = [0.0f32, 0.0, 0.0];
        adjust_color_balance(&mut rgba, w, h, stride, &shadows, &midtones, &highlights, true);
        // Red should increase, blue should decrease
        assert!(rgba[0] >= 128);
        assert!(rgba[2] <= 128);
    }

    #[test]
    fn test_adjust_black_white() {
        let w = 4;
        let h = 4;
        let stride = w * 4;
        let mut rgba = vec![0u8; w * h * 4];
        // Red pixel
        rgba[0] = 255;
        rgba[3] = 255;
        let weights = [0.4f32, 0.6, 0.4, 0.6, 0.2, 0.8];
        adjust_black_white(&mut rgba, w, h, stride, &weights, false, 0.0, 0.0);
        // Pure red at 40% weight becomes ~40% gray
        assert_eq!(rgba[0], rgba[1]);
        assert_eq!(rgba[1], rgba[2]);
        assert!((rgba[0] as i32 - 102).abs() <= 5);
    }

    #[test]
    fn test_adjust_grain() {
        let w = 8;
        let h = 8;
        let stride = w * 4;
        let mut rgba = vec![128u8; w * h * 4];
        for i in 0..w * h {
            rgba[i * 4 + 3] = 255;
        }
        adjust_grain(&mut rgba, w, h, stride, 50.0, 1.5, 50.0, 12345, 0.0, 0.0, 1.0);
        // Some pixels should have changed from 128
        let changed = rgba.chunks(4).any(|p| p[0] != 128);
        assert!(changed);
    }

    #[test]
    fn test_adjust_colored_vignette() {
        let w = 16;
        let h = 16;
        let stride = w * 4;
        let mut rgba = vec![255u8; w * h * 4];
        adjust_colored_vignette(
            &mut rgba,
            w,
            h,
            stride,
            0.0,
            0.0,
            w as f64,
            h as f64,
            false,
            50.0,
            50.0,
            100.0,
            50.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        // Corners should be darker than center
        let corner = rgba[0];
        let center = rgba[(8 * w + 8) * 4];
        assert!(corner < center);
    }

    #[test]
    fn test_add_noise() {
        let w = 8;
        let h = 8;
        let stride = w * 4;
        let mut rgba = vec![128u8; w * h * 4];
        for i in 0..w * h {
            rgba[i * 4 + 3] = 255;
        }
        add_noise(&mut rgba, w, h, stride, 20.0, false, false, 999);
        let changed = rgba.chunks(4).any(|p| p[0] != 128);
        assert!(changed);
    }

    #[test]
    fn test_adjust_camera_raw() {
        let w = 4;
        let h = 4;
        let stride = w * 4;
        let mut rgba = vec![100u8; w * h * 4];
        for i in 0..w * h {
            rgba[i * 4 + 3] = 255;
        }
        // Exposure +1 stop
        adjust_camera_raw(
            &mut rgba,
            w,
            h,
            stride,
            1.0, 1.0, 1.0, // gains
            1.0, // exposure +1
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0,
        );
        // Pixel should be brighter than 100
        assert!(rgba[0] > 100);
    }
}
