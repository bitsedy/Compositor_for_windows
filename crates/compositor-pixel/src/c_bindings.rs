use std::os::raw::{c_double, c_float, c_int, c_long};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CDitherParams {
    pub style: c_int,
    pub levels: c_int,
    pub diffusion: c_float,
    pub density: c_float,
    pub contrast: c_float,
    pub cell: c_int,
    pub angle: c_float,
    pub light_on_dark: c_int,
    pub original_colors: c_int,
    pub dark: [u8; 3],
    pub light: [u8; 3],
    pub glyph_width: c_int,
    pub glyph_height: c_int,
    pub glyphs: *const u8,
    pub glyph_coverage: *const c_float,
    pub glyph_count: c_int,
    pub dots: c_float,
    pub wobble: c_float,
}

extern "C" {
    // BrushPixels.h
    pub fn brush_alpha_bounds(
        bytes: *const u8,
        width: usize,
        height: usize,
        stride: usize,
        bounds: *mut usize,
    );
    pub fn layer_extract_alpha(
        rgba: *const u8,
        rgba_stride: usize,
        gray: *mut u8,
        gray_stride: usize,
        width: usize,
        height: usize,
    );
    pub fn layer_unpremultiply_opaque(
        rgba: *mut u8,
        stride: usize,
        width: usize,
        height: usize,
    );
    pub fn layer_restore_alpha(
        rgba: *mut u8,
        stride: usize,
        alpha: *const u8,
        alpha_stride: usize,
        width: usize,
        height: usize,
    );

    // HealPixels.h
    pub fn heal_coverage_bounds(
        gray: *const u8,
        width: usize,
        height: usize,
        stride: usize,
        bounds: *mut c_long,
    );
    pub fn spot_heal(
        rgba: *mut u8,
        coverage: *const u8,
        width: usize,
        height: usize,
        stride: usize,
        opacity: c_float,
        mode: c_int,
        seed: u32,
    ) -> c_int;

    // LevelsPixels.h
    pub fn levels_apply(pixels: *mut u8, count: usize, tables: *const c_float);
    pub fn levels_histogram(
        pixels: *const u8,
        coverage: *const u8,
        count: usize,
        bins: *mut c_double,
    );
    pub fn cube_apply(
        pixels: *mut u8,
        count: usize,
        cube: *const c_float,
        dimension: c_int,
    );

    // AdjustPixels.h
    pub fn adjust_gradient_map(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        table: *const u8,
    );
    pub fn adjust_grain(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        amount: c_double,
        size: c_double,
        roughness: c_double,
        seed: u32,
        origin_x: c_double,
        origin_y: c_double,
        units_per_pixel: c_double,
    );
    pub fn adjust_black_white(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        weights: *const c_float,
        tint: c_int,
        tint_hue: c_double,
        tint_saturation: c_double,
    );
    pub fn adjust_color_balance(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        shadows: *const c_float,
        midtones: *const c_float,
        highlights: *const c_float,
        preserve_luminosity: c_int,
    );
    pub fn adjust_camera_raw(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        red_gain: c_double,
        green_gain: c_double,
        blue_gain: c_double,
        exposure: c_double,
        contrast: c_double,
        highlights: c_double,
        shadows: c_double,
        whites: c_double,
        blacks: c_double,
        vibrance: c_double,
        saturation: c_double,
        clipping: c_int,
    );
    pub fn adjust_camera_raw_clip_overlay(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        shadows: c_int,
        highlights: c_int,
    );
    pub fn adjust_camera_raw_curve_color(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        tone_lut: *const c_float,
        red_lut: *const c_float,
        green_lut: *const c_float,
        blue_lut: *const c_float,
        refine_saturation: c_double,
        mixer: *const c_float,
        point_count: c_int,
        points: *const c_float,
        grade: *const c_float,
        blending: c_double,
        balance: c_double,
        visualize: c_int,
    );
    pub fn adjust_camera_raw_effects(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        texture: c_double,
        clarity: c_double,
        dehaze: c_double,
        glow: c_double,
        glow_style: c_int,
        glow_range: c_double,
        glow_spread: c_double,
        glow_warmth: c_double,
        vignette_amount: c_double,
        vignette_midpoint: c_double,
        vignette_roundness: c_double,
        vignette_feather: c_double,
        vignette_highlights: c_double,
        vignette_style: c_int,
        scale: c_double,
    );
    pub fn adjust_colored_vignette(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        frame_x: c_double,
        frame_y: c_double,
        frame_width: c_double,
        frame_height: c_double,
        fills_clear: c_int,
        amount: c_double,
        midpoint: c_double,
        roundness: c_double,
        feather: c_double,
        highlights: c_double,
        red: c_double,
        green: c_double,
        blue: c_double,
    );
    pub fn adjust_tonal_contrast(
        rgba: *mut u8,
        blurred: *const u8,
        width: usize,
        height: usize,
        stride: usize,
        blurred_stride: usize,
        amount: c_double,
        shadows: c_double,
        midtones: c_double,
        highlights: c_double,
    );
    pub fn adjust_camera_raw_detail(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        sharpen_amount: c_double,
        sharpen_radius: c_double,
        sharpen_detail: c_double,
        sharpen_masking: c_double,
        noise_luminance: c_double,
        noise_luminance_detail: c_double,
        noise_luminance_contrast: c_double,
        noise_color: c_double,
        noise_color_detail: c_double,
        noise_color_smoothness: c_double,
        scale: c_double,
    );
    pub fn adjust_camera_raw_sharpen_mask_overlay(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        sharpen_radius: c_double,
        sharpen_detail: c_double,
        sharpen_masking: c_double,
        scale: c_double,
    );
    pub fn adjust_camera_raw_optics(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        remove_chromatic: c_int,
        lens_profile: c_int,
        profile_distortion: c_double,
        profile_vignetting: c_double,
        distortion_k: c_double,
        purple_amount: c_double,
        purple_hue_low: c_double,
        purple_hue_high: c_double,
        green_amount: c_double,
        green_hue_low: c_double,
        green_hue_high: c_double,
        vignette_amount: c_double,
        vignette_midpoint: c_double,
        scale: c_double,
    );
    pub fn adjust_camera_raw_calibration(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        shadow_tint: c_double,
        red_hue: c_double,
        red_saturation: c_double,
        green_hue: c_double,
        green_saturation: c_double,
        blue_hue: c_double,
        blue_saturation: c_double,
        process_version: c_int,
    );
    pub fn rgba_clamp_premultiplied(rgba: *mut u8, count: usize);

    // LensPixels.h
    pub fn lens_distort(
        source: *const u8,
        destination: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        k: c_double,
    );

    // NoisePixels.h
    pub fn noise_add(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        amount: c_float,
        gaussian: c_int,
        monochromatic: c_int,
        seed: u32,
    );
    pub fn noise_add_at(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        amount: c_float,
        gaussian: c_int,
        monochromatic: c_int,
        seed: u32,
        origin_x: i64,
        origin_y: i64,
    );

    // WandPixels.h
    pub fn wand_mask(
        rgba: *const u8,
        width: usize,
        height: usize,
        stride: usize,
        seed_x: usize,
        seed_y: usize,
        radius: usize,
        tolerance: c_int,
        contiguous: c_int,
        mask: *mut u8,
    ) -> c_long;
    pub fn color_range_mask(
        rgba: *const u8,
        width: usize,
        height: usize,
        stride: usize,
        include: *const u8,
        include_count: c_int,
        exclude: *const u8,
        exclude_count: c_int,
        fuzziness: c_int,
        invert: c_int,
        mask: *mut u8,
    ) -> c_long;

    // ContentFill.h
    pub fn content_fill(
        rgba: *mut u8,
        stride: usize,
        mask: *const u8,
        mask_stride: usize,
        width: c_int,
        height: c_int,
    ) -> c_int;

    // DitherPixels.h
    pub fn dither_apply(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        params: *const CDitherParams,
    ) -> c_int;
    pub fn dither_dots(
        rgba: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        block: c_int,
        gap: *const u8,
    );
    pub fn dither_glow(
        rgba: *mut u8,
        glow: *const u8,
        width: usize,
        height: usize,
        stride: usize,
        amount: c_float,
    );
}
