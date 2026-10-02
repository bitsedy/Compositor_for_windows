use std::f64::consts::PI;
use serde::{Deserialize, Serialize};
use compositor_pixel as pixel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterKind {
    #[serde(rename = "Gaussian Blur")]
    GaussianBlur,
    #[serde(rename = "Motion Blur")]
    MotionBlur,
    #[serde(rename = "Add Noise")]
    AddNoise,
    #[serde(rename = "Vignette")]
    Vignette,
    #[serde(rename = "Bloom / Glow")]
    BloomGlow,
    #[serde(rename = "Dither")]
    Dither,
    #[serde(rename = "Tonal Contrast")]
    TonalContrast,
    #[serde(rename = "Lens Correction")]
    LensCorrection,
    #[serde(rename = "Camera Raw Filter")]
    CameraRaw,
    #[serde(rename = "Remove Background")]
    RemoveBackground,
    #[serde(rename = "Content-Aware Fill")]
    ContentAwareFill,
    #[serde(rename = "Curves")]
    Curves,
    #[serde(rename = "Exposure")]
    Exposure,
    #[serde(rename = "Gradient Map")]
    GradientMap,
    #[serde(rename = "Grain")]
    Grain,
    #[serde(rename = "Black & White")]
    BlackWhite,
    #[serde(rename = "Color Balance")]
    ColorBalance,
    #[serde(rename = "Invert")]
    Invert,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurvesSettings {
    /// 4 channels: 0 = RGB composite, 1 = Red, 2 = Green, 3 = Blue
    pub channels: Vec<Vec<CurvePoint>>,
}

impl Default for CurvesSettings {
    fn default() -> Self {
        Self {
            channels: vec![
                vec![CurvePoint { x: 0.0, y: 0.0 }, CurvePoint { x: 255.0, y: 255.0 }],
                vec![CurvePoint { x: 0.0, y: 0.0 }, CurvePoint { x: 255.0, y: 255.0 }],
                vec![CurvePoint { x: 0.0, y: 0.0 }, CurvePoint { x: 255.0, y: 255.0 }],
                vec![CurvePoint { x: 0.0, y: 0.0 }, CurvePoint { x: 255.0, y: 255.0 }],
            ],
        }
    }
}

impl CurvesSettings {
    /// Evaluates Hermite cubic spline interpolation for a given channel (0..=3).
    pub fn value(&self, x: f64, channel: usize) -> f64 {
        if channel >= self.channels.len() {
            return x.clamp(0.0, 255.0);
        }
        let p = &self.channels[channel];
        if p.len() < 2 {
            return x.clamp(0.0, 255.0);
        }
        let i = p
            .iter()
            .rposition(|pt| pt.x <= x)
            .unwrap_or(0)
            .min(p.len() - 2);

        let d: Vec<f64> = p
            .windows(2)
            .map(|w| {
                let dx = w[1].x - w[0].x;
                if dx.abs() < 1e-6 {
                    0.0
                } else {
                    (w[1].y - w[0].y) / dx
                }
            })
            .collect();

        let slope = |j: usize| -> f64 {
            if j == 0 {
                return d[0];
            }
            if j == d.len() {
                return *d.last().unwrap_or(&0.0);
            }
            if d[j - 1] * d[j] <= 0.0 {
                return 0.0;
            }
            2.0 / (1.0 / d[j - 1] + 1.0 / d[j])
        };

        let h = p[i + 1].x - p[i].x;
        if h <= 1e-6 {
            return p[i].y.clamp(0.0, 255.0);
        }
        let t = ((x - p[i].x) / h).clamp(0.0, 1.0);
        let y = (2.0 * t * t * t - 3.0 * t * t + 1.0) * p[i].y
            + (t * t * t - 2.0 * t * t + t) * h * slope(i)
            + (-2.0 * t * t * t + 3.0 * t * t) * p[i + 1].y
            + (t * t * t - t * t) * h * slope(i + 1);
        y.clamp(0.0, 255.0)
    }

    /// Builds a 256 * 4 lookup table (R, G, B, A) compatible with levels_apply.
    pub fn build_table(&self) -> [f32; 256 * 4] {
        let mut tables = [0.0f32; 256 * 4];
        for ch in 1..=3 {
            for i in 0..256 {
                let v = self.value(self.value(i as f64, ch), 0) / 255.0;
                tables[(ch - 1) * 256 + i] = v.clamp(0.0, 1.0) as f32;
            }
        }
        // Alpha channel is identity
        for i in 0..256 {
            tables[3 * 256 + i] = (i as f32) / 255.0;
        }
        tables
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExposureSettings {
    pub exposure: f64, // -20.0..=20.0 stops
    pub offset: f64,   // -0.5..=0.5
    pub gamma: f64,    // 0.01..=9.99
}

impl Default for ExposureSettings {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            offset: 0.0,
            gamma: 1.0,
        }
    }
}

impl ExposureSettings {
    pub fn build_table(&self) -> [f32; 256 * 4] {
        let scale = 2.0f64.powf(self.exposure.clamp(-20.0, 20.0));
        let offset = self.offset.clamp(-0.5, 0.5);
        let gamma = self.gamma.clamp(0.01, 9.99);

        let mut channel_table = [0.0f32; 256];
        for i in 0..256 {
            let encoded = i as f64 / 255.0;
            let mut linear = if encoded <= 0.04045 {
                encoded / 12.92
            } else {
                ((encoded + 0.055) / 1.055).powf(2.4)
            };
            linear = (linear * scale + offset).max(0.0).powf(1.0 / gamma);
            let output = if linear <= 0.0031308 {
                linear * 12.92
            } else {
                1.055 * linear.powf(1.0 / 2.4) - 0.055
            };
            channel_table[i] = output.clamp(0.0, 1.0) as f32;
        }

        let mut tables = [0.0f32; 256 * 4];
        for ch in 0..3 {
            tables[ch * 256..(ch + 1) * 256].copy_from_slice(&channel_table);
        }
        for i in 0..256 {
            tables[3 * 256 + i] = (i as f32) / 255.0;
        }
        tables
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LevelRange {
    pub black: f64,        // 0..254
    pub gamma: f64,        // 0.1..9.99
    pub white: f64,        // (black+1)..255
    pub output_black: f64, // 0..255
    pub output_white: f64, // 0..255
}

impl Default for LevelRange {
    fn default() -> Self {
        Self {
            black: 0.0,
            gamma: 1.0,
            white: 255.0,
            output_black: 0.0,
            output_white: 255.0,
        }
    }
}

impl LevelRange {
    pub fn apply(&self, value: f64) -> f64 {
        let b = self.black.clamp(0.0, 254.0);
        let w = self.white.clamp(b + 1.0, 255.0);
        let g = self.gamma.clamp(0.1, 9.99);
        let ob = self.output_black.clamp(0.0, 255.0);
        let ow = self.output_white.clamp(0.0, 255.0);

        let input = ((value * 255.0 - b) / (w - b)).clamp(0.0, 1.0);
        (ob + input.powf(1.0 / g) * (ow - ob)) / 255.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelsSettings {
    /// 4 ranges: RGB (0), Red (1), Green (2), Blue (3)
    pub ranges: [LevelRange; 4],
}

impl Default for LevelsSettings {
    fn default() -> Self {
        Self {
            ranges: [LevelRange::default(); 4],
        }
    }
}

impl LevelsSettings {
    pub fn build_table(&self) -> [f32; 256 * 4] {
        let mut tables = [0.0f32; 256 * 4];
        for ch in 1..=3 {
            for i in 0..256 {
                let v = self.ranges[0].apply(self.ranges[ch].apply(i as f64 / 255.0));
                tables[(ch - 1) * 256 + i] = v.clamp(0.0, 1.0) as f32;
            }
        }
        for i in 0..256 {
            tables[3 * 256 + i] = (i as f32) / 255.0;
        }
        tables
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GradientMapSettings {
    pub shadow_color: [f32; 3],    // RGB in 0.0..=1.0
    pub highlight_color: [f32; 3], // RGB in 0.0..=1.0
    pub reversed: bool,
}

impl Default for GradientMapSettings {
    fn default() -> Self {
        Self {
            shadow_color: [0.0, 0.0, 0.0],
            highlight_color: [1.0, 1.0, 1.0],
            reversed: false,
        }
    }
}

impl GradientMapSettings {
    pub fn build_table(&self) -> [u8; 256 * 3] {
        let (dark, light) = if self.reversed {
            (self.highlight_color, self.shadow_color)
        } else {
            (self.shadow_color, self.highlight_color)
        };

        let mut table = [0u8; 256 * 3];
        for i in 0..256 {
            let t = i as f32 / 255.0;
            let r = ((dark[0] + (light[0] - dark[0]) * t) * 255.0).round().clamp(0.0, 255.0) as u8;
            let g = ((dark[1] + (light[1] - dark[1]) * t) * 255.0).round().clamp(0.0, 255.0) as u8;
            let b = ((dark[2] + (light[2] - dark[2]) * t) * 255.0).round().clamp(0.0, 255.0) as u8;
            table[i * 3] = r;
            table[i * 3 + 1] = g;
            table[i * 3 + 2] = b;
        }
        table
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BlackWhiteSettings {
    pub reds: f64,     // -200..300 (default 40)
    pub yellows: f64,  // -200..300 (default 60)
    pub greens: f64,   // -200..300 (default 40)
    pub cyans: f64,    // -200..300 (default 60)
    pub blues: f64,    // -200..300 (default 20)
    pub magentas: f64, // -200..300 (default 80)
    pub tint: bool,
    pub tint_hue: f64,        // 0..360
    pub tint_saturation: f64, // 0..100
}

impl Default for BlackWhiteSettings {
    fn default() -> Self {
        Self {
            reds: 40.0,
            yellows: 60.0,
            greens: 40.0,
            cyans: 60.0,
            blues: 20.0,
            magentas: 80.0,
            tint: false,
            tint_hue: 40.0,
            tint_saturation: 20.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorBalanceSettings {
    pub shadow_cyan_red: f32,       // -1.0..1.0
    pub shadow_magenta_green: f32,  // -1.0..1.0
    pub shadow_yellow_blue: f32,    // -1.0..1.0
    pub mid_cyan_red: f32,          // -1.0..1.0
    pub mid_magenta_green: f32,     // -1.0..1.0
    pub mid_yellow_blue: f32,       // -1.0..1.0
    pub highlight_cyan_red: f32,    // -1.0..1.0
    pub highlight_magenta_green: f32,// -1.0..1.0
    pub highlight_yellow_blue: f32, // -1.0..1.0
    pub preserve_luminosity: bool,
}

impl Default for ColorBalanceSettings {
    fn default() -> Self {
        Self {
            shadow_cyan_red: 0.0,
            shadow_magenta_green: 0.0,
            shadow_yellow_blue: 0.0,
            mid_cyan_red: 0.0,
            mid_magenta_green: 0.0,
            mid_yellow_blue: 0.0,
            highlight_cyan_red: 0.0,
            highlight_magenta_green: 0.0,
            highlight_yellow_blue: 0.0,
            preserve_luminosity: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GaussianBlurSettings {
    pub radius: f64, // 0.1..250.0 pixels
}

impl Default for GaussianBlurSettings {
    fn default() -> Self {
        Self { radius: 10.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MotionBlurSettings {
    pub angle: f64,    // -90.0..90.0 degrees
    pub distance: f64, // 1.0..2000.0 pixels
}

impl Default for MotionBlurSettings {
    fn default() -> Self {
        Self {
            angle: 0.0,
            distance: 10.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VignetteSettings {
    pub amount: f64,      // 0..100
    pub midpoint: f64,    // 0..100
    pub roundness: f64,   // -100..100
    pub feather: f64,     // 0..100
    pub highlights: f64,  // 0..100
    pub color: [f64; 3],  // RGB
    pub fills_clear: bool,
}

impl Default for VignetteSettings {
    fn default() -> Self {
        Self {
            amount: 35.0,
            midpoint: 50.0,
            roundness: 100.0,
            feather: 60.0,
            highlights: 25.0,
            color: [0.0, 0.0, 0.0],
            fills_clear: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BloomSettings {
    pub amount: f64, // 0..100
    pub radius: f64, // 1..150
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            amount: 40.0,
            radius: 24.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TonalContrastSettings {
    pub amount: f64,     // 0..100
    pub radius: f64,     // 1..100
    pub shadows: f64,    // -100..100
    pub midtones: f64,   // -100..100
    pub highlights: f64, // -100..100
}

impl Default for TonalContrastSettings {
    fn default() -> Self {
        Self {
            amount: 50.0,
            radius: 16.0,
            shadows: 40.0,
            midtones: 60.0,
            highlights: 30.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoiseSettings {
    pub amount: f32, // 0.1..400.0
    pub gaussian: bool,
    pub monochromatic: bool,
    pub seed: u32,
}

impl Default for NoiseSettings {
    fn default() -> Self {
        Self {
            amount: 10.0,
            gaussian: false,
            monochromatic: false,
            seed: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterParameters {
    Curves(CurvesSettings),
    Exposure(ExposureSettings),
    Levels(LevelsSettings),
    GradientMap(GradientMapSettings),
    BlackWhite(BlackWhiteSettings),
    ColorBalance(ColorBalanceSettings),
    GaussianBlur(GaussianBlurSettings),
    MotionBlur(MotionBlurSettings),
    Vignette(VignetteSettings),
    BloomGlow(BloomSettings),
    TonalContrast(TonalContrastSettings),
    Noise(NoiseSettings),
    LensCorrection { distortion: f64 },
    Invert,
}

/// Applies a 1D separable Gaussian blur pass horizontally or vertically to premultiplied RGBA.
pub fn gaussian_blur_rgba(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    radius: f64,
) {
    if radius < 0.1 || width == 0 || height == 0 {
        return;
    }
    let sigma = radius;
    let k_radius = (sigma * 3.0).ceil() as usize;
    let k_size = 2 * k_radius + 1;
    let mut kernel = vec![0.0f32; k_size];
    let two_sigma_sq = 2.0 * sigma * sigma;
    let mut sum = 0.0f32;
    for i in 0..k_size {
        let x = i as f64 - k_radius as f64;
        let w = (-(x * x) / two_sigma_sq).exp() as f32;
        kernel[i] = w;
        sum += w;
    }
    for w in &mut kernel {
        *w /= sum;
    }

    let mut temp = rgba.to_vec();

    // Horizontal pass: read temp -> write rgba
    for y in 0..height {
        let row_idx = y * stride;
        for x in 0..width {
            let mut acc = [0.0f32; 4];
            for k in 0..k_size {
                let tap_x = (x as isize + k as isize - k_radius as isize).clamp(0, width as isize - 1) as usize;
                let src_idx = row_idx + tap_x * 4;
                let kw = kernel[k];
                acc[0] += temp[src_idx] as f32 * kw;
                acc[1] += temp[src_idx + 1] as f32 * kw;
                acc[2] += temp[src_idx + 2] as f32 * kw;
                acc[3] += temp[src_idx + 3] as f32 * kw;
            }
            let dst_idx = row_idx + x * 4;
            rgba[dst_idx] = acc[0].round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 1] = acc[1].round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 2] = acc[2].round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 3] = acc[3].round().clamp(0.0, 255.0) as u8;
        }
    }

    temp.copy_from_slice(rgba);

    // Vertical pass: read temp -> write rgba
    for y in 0..height {
        let row_idx = y * stride;
        for x in 0..width {
            let mut acc = [0.0f32; 4];
            for k in 0..k_size {
                let tap_y = (y as isize + k as isize - k_radius as isize).clamp(0, height as isize - 1) as usize;
                let src_idx = tap_y * stride + x * 4;
                let kw = kernel[k];
                acc[0] += temp[src_idx] as f32 * kw;
                acc[1] += temp[src_idx + 1] as f32 * kw;
                acc[2] += temp[src_idx + 2] as f32 * kw;
                acc[3] += temp[src_idx + 3] as f32 * kw;
            }
            let dst_idx = row_idx + x * 4;
            rgba[dst_idx] = acc[0].round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 1] = acc[1].round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 2] = acc[2].round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 3] = acc[3].round().clamp(0.0, 255.0) as u8;
        }
    }
}

/// Applies directional motion blur matching Photoshop angle and streak length.
pub fn motion_blur_rgba(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    angle_degrees: f64,
    distance: f64,
) {
    if distance <= 1.0 || width == 0 || height == 0 {
        return;
    }
    let rad = angle_degrees * PI / 180.0;
    let dx = rad.cos();
    let dy = -rad.sin(); // screen coordinates y-down
    let steps = (distance.ceil() as usize).max(3);
    let half_dist = distance / 2.0;

    let temp = rgba.to_vec();
    for y in 0..height {
        let row_idx = y * stride;
        for x in 0..width {
            let mut acc = [0.0f32; 4];
            for s in 0..steps {
                let t = (s as f64 / (steps - 1) as f64) * distance - half_dist;
                let sample_x = (x as f64 + t * dx).round() as isize;
                let sample_y = (y as f64 + t * dy).round() as isize;
                let clamped_x = sample_x.clamp(0, width as isize - 1) as usize;
                let clamped_y = sample_y.clamp(0, height as isize - 1) as usize;
                let idx = clamped_y * stride + clamped_x * 4;
                acc[0] += temp[idx] as f32;
                acc[1] += temp[idx + 1] as f32;
                acc[2] += temp[idx + 2] as f32;
                acc[3] += temp[idx + 3] as f32;
            }
            let inv_steps = 1.0 / steps as f32;
            let dst_idx = row_idx + x * 4;
            rgba[dst_idx] = (acc[0] * inv_steps).round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 1] = (acc[1] * inv_steps).round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 2] = (acc[2] * inv_steps).round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 3] = (acc[3] * inv_steps).round().clamp(0.0, 255.0) as u8;
        }
    }
}

/// Inverts premultiplied RGBA pixels: c' = alpha - c, alpha is preserved.
pub fn invert_rgba(rgba: &mut [u8], count: usize) {
    for i in 0..count {
        let idx = i * 4;
        let a = rgba[idx + 3];
        rgba[idx] = a.saturating_sub(rgba[idx]);
        rgba[idx + 1] = a.saturating_sub(rgba[idx + 1]);
        rgba[idx + 2] = a.saturating_sub(rgba[idx + 2]);
    }
}

/// Applies bloom/glow: extracts bright areas, blurs, and screens/adds over base.
pub fn bloom_glow_rgba(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    amount: f64,
    radius: f64,
) {
    if amount <= 0.0 || radius <= 0.0 || width == 0 || height == 0 {
        return;
    }
    let mut highlight = rgba.to_vec();
    // Extract highlight luminance
    for y in 0..height {
        let row_idx = y * stride;
        for x in 0..width {
            let idx = row_idx + x * 4;
            let r = highlight[idx] as f32;
            let g = highlight[idx + 1] as f32;
            let b = highlight[idx + 2] as f32;
            let a = highlight[idx + 3] as f32;
            if a > 0.0 {
                let lum = (0.2126 * r + 0.7152 * g + 0.0722 * b) / a;
                let thresh = 0.5f32;
                let factor = ((lum - thresh) / (1.0 - thresh)).clamp(0.0, 1.0);
                highlight[idx] = (r * factor).round() as u8;
                highlight[idx + 1] = (g * factor).round() as u8;
                highlight[idx + 2] = (b * factor).round() as u8;
            } else {
                highlight[idx] = 0;
                highlight[idx + 1] = 0;
                highlight[idx + 2] = 0;
            }
        }
    }

    gaussian_blur_rgba(&mut highlight, width, height, stride, radius);

    let intensity = (amount / 50.0) as f32;
    for y in 0..height {
        let row_idx = y * stride;
        for x in 0..width {
            let idx = row_idx + x * 4;
            let r = rgba[idx] as f32 + highlight[idx] as f32 * intensity;
            let g = rgba[idx + 1] as f32 + highlight[idx + 1] as f32 * intensity;
            let b = rgba[idx + 2] as f32 + highlight[idx + 2] as f32 * intensity;
            let a = rgba[idx + 3] as f32;
            rgba[idx] = r.min(a).clamp(0.0, 255.0) as u8;
            rgba[idx + 1] = g.min(a).clamp(0.0, 255.0) as u8;
            rgba[idx + 2] = b.min(a).clamp(0.0, 255.0) as u8;
        }
    }
}

/// Applies any filter parameters to an RGBA buffer, blending through an optional selection mask.
pub fn apply_filter(
    rgba: &mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    params: &FilterParameters,
    selection_mask: Option<&[u8]>,
) {
    let original = if selection_mask.is_some() {
        Some(rgba.to_vec())
    } else {
        None
    };

    match params {
        FilterParameters::Curves(curves) => {
            let tables = curves.build_table();
            pixel::levels_apply(rgba, width * height, &tables);
        }
        FilterParameters::Exposure(exp) => {
            let tables = exp.build_table();
            pixel::levels_apply(rgba, width * height, &tables);
        }
        FilterParameters::Levels(levels) => {
            let tables = levels.build_table();
            pixel::levels_apply(rgba, width * height, &tables);
        }
        FilterParameters::GradientMap(gmap) => {
            let table = gmap.build_table();
            pixel::adjust_gradient_map(rgba, width, height, stride, &table);
        }
        FilterParameters::BlackWhite(bw) => {
            let weights = [
                (bw.reds / 100.0) as f32,
                (bw.yellows / 100.0) as f32,
                (bw.greens / 100.0) as f32,
                (bw.cyans / 100.0) as f32,
                (bw.blues / 100.0) as f32,
                (bw.magentas / 100.0) as f32,
            ];
            pixel::adjust_black_white(
                rgba,
                width,
                height,
                stride,
                &weights,
                bw.tint,
                bw.tint_hue,
                bw.tint_saturation / 100.0,
            );
        }
        FilterParameters::ColorBalance(cb) => {
            let shadows = [cb.shadow_cyan_red, cb.shadow_magenta_green, cb.shadow_yellow_blue];
            let midtones = [cb.mid_cyan_red, cb.mid_magenta_green, cb.mid_yellow_blue];
            let highlights = [cb.highlight_cyan_red, cb.highlight_magenta_green, cb.highlight_yellow_blue];
            pixel::adjust_color_balance(
                rgba,
                width,
                height,
                stride,
                &shadows,
                &midtones,
                &highlights,
                cb.preserve_luminosity,
            );
        }
        FilterParameters::GaussianBlur(blur) => {
            gaussian_blur_rgba(rgba, width, height, stride, blur.radius);
        }
        FilterParameters::MotionBlur(mb) => {
            motion_blur_rgba(rgba, width, height, stride, mb.angle, mb.distance);
        }
        FilterParameters::Vignette(v) => {
            pixel::adjust_colored_vignette(
                rgba,
                width,
                height,
                stride,
                0.0,
                0.0,
                width as f64,
                height as f64,
                v.fills_clear,
                v.amount,
                v.midpoint,
                v.roundness,
                v.feather,
                v.highlights,
                v.color[0],
                v.color[1],
                v.color[2],
            );
        }
        FilterParameters::BloomGlow(bloom) => {
            bloom_glow_rgba(rgba, width, height, stride, bloom.amount, bloom.radius);
        }
        FilterParameters::TonalContrast(tc) => {
            let mut blurred = rgba.to_vec();
            gaussian_blur_rgba(&mut blurred, width, height, stride, tc.radius);
            pixel::adjust_tonal_contrast(
                rgba,
                &blurred,
                width,
                height,
                stride,
                stride,
                tc.amount,
                tc.shadows,
                tc.midtones,
                tc.highlights,
            );
        }
        FilterParameters::Noise(noise) => {
            pixel::add_noise(
                rgba,
                width,
                height,
                stride,
                noise.amount,
                noise.gaussian,
                noise.monochromatic,
                noise.seed,
            );
        }
        FilterParameters::LensCorrection { distortion } => {
            let source = rgba.to_vec();
            pixel::lens_distort(
                &source,
                rgba,
                width,
                height,
                stride,
                distortion / 100.0 * 0.35,
            );
        }
        FilterParameters::Invert => {
            invert_rgba(rgba, width * height);
        }
    }

    // Blend through selection mask if provided
    if let (Some(orig), Some(mask)) = (original, selection_mask) {
        for y in 0..height {
            let row_idx = y * stride;
            let mask_row = y * width;
            for x in 0..width {
                let m = mask[mask_row + x] as f32 / 255.0;
                let idx = row_idx + x * 4;
                for c in 0..4 {
                    let base = orig[idx + c] as f32;
                    let filt = rgba[idx + c] as f32;
                    rgba[idx + c] = (base + (filt - base) * m).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FilterError {
    #[error("Invalid image dimensions: {0}x{1}")]
    InvalidDimensions(usize, usize),
    #[error("Resampling error: {0}")]
    ResizeError(String),
}

/// High-quality image resampling using fast_image_resize.
pub fn resample_rgba(
    src: &[u8],
    src_width: usize,
    src_height: usize,
    dst_width: usize,
    dst_height: usize,
    filter: fast_image_resize::FilterType,
) -> Result<Vec<u8>, FilterError> {
    use fast_image_resize::images::Image;
    use fast_image_resize::{PixelType, Resizer};

    if src_width == 0 || src_height == 0 {
        return Err(FilterError::InvalidDimensions(src_width, src_height));
    }
    if dst_width == 0 || dst_height == 0 {
        return Err(FilterError::InvalidDimensions(dst_width, dst_height));
    }

    let src_image = Image::from_vec_u8(src_width as u32, src_height as u32, src.to_vec(), PixelType::U8x4)
        .map_err(|e| FilterError::ResizeError(e.to_string()))?;
    let mut dst_image = Image::new(dst_width as u32, dst_height as u32, PixelType::U8x4);

    let mut resizer = Resizer::new();
    let resize_options = fast_image_resize::ResizeOptions::new().resize_alg(
        fast_image_resize::ResizeAlg::Interpolation(filter),
    );
    resizer
        .resize(&src_image, &mut dst_image, &resize_options)
        .map_err(|e| FilterError::ResizeError(e.to_string()))?;

    let mut dst_bytes = dst_image.into_vec();
    // Clamp premultiplied RGBA to avoid ringing exceeding alpha
    pixel::clamp_premultiplied(&mut dst_bytes, dst_width * dst_height);
    Ok(dst_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curves_evaluation() {
        let curves = CurvesSettings::default();
        // Default is diagonal line (0,0) to (255,255)
        assert!((curves.value(128.0, 0) - 128.0).abs() <= 1.0);
        let table = curves.build_table();
        assert!((table[128] - 128.0 / 255.0).abs() <= 0.02);
    }

    #[test]
    fn test_exposure_table() {
        let exp = ExposureSettings {
            exposure: 1.0, // +1 stop
            offset: 0.0,
            gamma: 1.0,
        };
        let table = exp.build_table();
        // Midtones should brighten
        assert!(table[128] > 128.0 / 255.0);
    }

    #[test]
    fn test_levels_table() {
        let levels = LevelsSettings::default();
        let table = levels.build_table();
        assert!((table[128] - 128.0 / 255.0).abs() <= 0.02);
    }

    #[test]
    fn test_gaussian_blur() {
        let w = 16;
        let h = 16;
        let stride = w * 4;
        let mut rgba = vec![0u8; w * h * 4];
        // Center pixel is white
        let center_idx = (8 * w + 8) * 4;
        rgba[center_idx] = 255;
        rgba[center_idx + 1] = 255;
        rgba[center_idx + 2] = 255;
        rgba[center_idx + 3] = 255;

        gaussian_blur_rgba(&mut rgba, w, h, stride, 2.0);
        // Center pixel spreads to neighbor (8, 9)
        let neighbor_idx = (8 * w + 9) * 4;
        assert!(rgba[neighbor_idx] > 0);
        assert!(rgba[center_idx] < 255);
    }

    #[test]
    fn test_motion_blur() {
        let w = 16;
        let h = 16;
        let stride = w * 4;
        let mut rgba = vec![0u8; w * h * 4];
        let center_idx = (8 * w + 8) * 4;
        rgba[center_idx] = 255;
        rgba[center_idx + 3] = 255;

        motion_blur_rgba(&mut rgba, w, h, stride, 0.0, 5.0);
        // Horizontal streak spreads to adjacent x
        let next_x_idx = (8 * w + 9) * 4;
        assert!(rgba[next_x_idx] > 0);
    }

    #[test]
    fn test_invert_rgba() {
        let mut rgba = vec![50u8, 100, 150, 200];
        invert_rgba(&mut rgba, 1);
        assert_eq!(rgba[0], 150); // 200 - 50
        assert_eq!(rgba[1], 100); // 200 - 100
        assert_eq!(rgba[2], 50);  // 200 - 150
        assert_eq!(rgba[3], 200); // alpha preserved
    }

    #[test]
    fn test_apply_filter_with_selection() {
        let w = 4;
        let h = 4;
        let stride = w * 4;
        let mut rgba = vec![100u8; w * h * 4];
        for i in 0..w * h {
            rgba[i * 4 + 3] = 255;
        }

        // Mask covers only pixel 0
        let mut mask = vec![0u8; w * h];
        mask[0] = 255;

        apply_filter(&mut rgba, w, h, stride, &FilterParameters::Invert, Some(&mask));
        // Pixel 0 should be inverted (255 - 100 = 155)
        assert_eq!(rgba[0], 155);
        // Pixel 1 should be untouched (100)
        assert_eq!(rgba[4], 100);
    }

    #[test]
    fn test_resample_rgba() {
        let src = vec![255u8; 8 * 8 * 4];
        let dst = resample_rgba(&src, 8, 8, 16, 16, fast_image_resize::FilterType::Bilinear).unwrap();
        assert_eq!(dst.len(), 16 * 16 * 4);
        assert_eq!(dst[0], 255);
    }
}
