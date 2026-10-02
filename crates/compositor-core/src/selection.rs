use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Mode for combining new selections with existing ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionMode {
    #[serde(rename = "New")]
    Replace,
    #[serde(rename = "Add")]
    Add,
    #[serde(rename = "Subtract")]
    Subtract,
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::Replace
    }
}

/// A document-space selection outline clipped to canvas.
/// An empty path or empty points slice is an explicit empty selection,
/// which later edits treat as "touch nothing", never as "touch everything".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSelection {
    /// Ordered points forming closed polygon loop(s).
    pub points: Vec<Vec2>,
    pub antialiased: bool,
    /// Edge fade in document pixels (0.0 is hard edge).
    pub feather: f32,
}

impl DocumentSelection {
    pub fn new(points: Vec<Vec2>, antialiased: bool, feather: f32) -> Self {
        Self {
            points,
            antialiased,
            feather: feather.max(0.0),
        }
    }

    pub fn from_rect(min: Vec2, max: Vec2, antialiased: bool, feather: f32) -> Self {
        let x0 = min.x.min(max.x);
        let x1 = min.x.max(max.x);
        let y0 = min.y.min(max.y);
        let y1 = min.y.max(max.y);
        let points = vec![
            Vec2::new(x0, y0),
            Vec2::new(x1, y0),
            Vec2::new(x1, y1),
            Vec2::new(x0, y1),
        ];
        Self::new(points, antialiased, feather)
    }

    pub fn from_ellipse(center: Vec2, radius: Vec2, segments: usize, antialiased: bool, feather: f32) -> Self {
        let n = segments.max(16);
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            let theta = 2.0 * std::f32::consts::PI * (i as f32) / (n as f32);
            points.push(Vec2::new(
                center.x + radius.x * theta.cos(),
                center.y + radius.y * theta.sin(),
            ));
        }
        Self::new(points, antialiased, feather)
    }

    pub fn is_empty(&self) -> bool {
        if self.points.len() < 3 {
            return true;
        }
        let b = self.bounding_box();
        b.2 <= b.0 || b.3 <= b.1
    }

    /// (min_x, min_y, max_x, max_y)
    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        if self.points.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let mut min_x = self.points[0].x;
        let mut max_x = self.points[0].x;
        let mut min_y = self.points[0].y;
        let mut max_y = self.points[0].y;
        for p in &self.points[1..] {
            if p.x < min_x { min_x = p.x; }
            if p.x > max_x { max_x = p.x; }
            if p.y < min_y { min_y = p.y; }
            if p.y > max_y { max_y = p.y; }
        }
        (min_x, min_y, max_x, max_y)
    }

    /// Bounding box expanded by feather radius (two standard deviations).
    pub fn coverage_bounds(&self) -> (f32, f32, f32, f32) {
        let (min_x, min_y, max_x, max_y) = self.bounding_box();
        let pad = (self.feather * 2.0).ceil();
        (min_x - pad, min_y - pad, max_x + pad, max_y + pad)
    }

    /// Point-in-polygon test using ray casting.
    pub fn contains_point(&self, p: Vec2) -> bool {
        if self.is_empty() {
            return false;
        }
        let (min_x, min_y, max_x, max_y) = self.bounding_box();
        if p.x < min_x || p.x > max_x || p.y < min_y || p.y > max_y {
            return false;
        }
        let mut inside = false;
        let n = self.points.len();
        let mut j = n - 1;
        for i in 0..n {
            let pi = self.points[i];
            let pj = self.points[j];
            if ((pi.y > p.y) != (pj.y > p.y))
                && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x)
            {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    /// Offsets the selection outline by (dx, dy).
    pub fn offset(&mut self, delta: Vec2) {
        for p in &mut self.points {
            *p += delta;
        }
    }

    /// Rasterizes the polygon to an 8-bit grayscale mask at specified dimensions.
    /// Pixels inside receive 255; feathered edge smoothly falls off.
    pub fn rasterize_mask(&self, width: u32, height: u32) -> Vec<u8> {
        let mut mask = vec![0u8; (width * height) as usize];
        if self.is_empty() || width == 0 || height == 0 {
            return mask;
        }

        let (min_x, min_y, max_x, max_y) = self.bounding_box();
        let x_start = (min_x.floor() as i32).max(0) as u32;
        let x_end = (max_x.ceil() as u32 + 1).min(width);
        let y_start = (min_y.floor() as i32).max(0) as u32;
        let y_end = (max_y.ceil() as u32 + 1).min(height);

        for y in y_start..y_end {
            let py = y as f32 + 0.5;
            for x in x_start..x_end {
                let px = x as f32 + 0.5;
                if self.contains_point(Vec2::new(px, py)) {
                    mask[(y * width + x) as usize] = 255;
                }
            }
        }

        if self.feather > 0.0 {
            gaussian_blur_mask(&mut mask, width, height, self.feather);
        }

        mask
    }
}

/// Applies a separable 1D Gaussian blur on a 1-channel mask.
fn gaussian_blur_mask(mask: &mut [u8], width: u32, height: u32, sigma: f32) {
    if sigma <= 0.0 || width == 0 || height == 0 {
        return;
    }
    let radius = (sigma * 3.0).ceil() as usize;
    if radius == 0 {
        return;
    }

    // Compute 1D Gaussian kernel
    let mut kernel = Vec::with_capacity(2 * radius + 1);
    let mut sum = 0.0f32;
    let two_sigma_sq = 2.0 * sigma * sigma;
    for i in -(radius as i32)..=(radius as i32) {
        let val = (-(i * i) as f32 / two_sigma_sq).exp();
        kernel.push(val);
        sum += val;
    }
    for k in &mut kernel {
        *k /= sum;
    }

    let w = width as usize;
    let h = height as usize;
    let mut temp = vec![0.0f32; w * h];

    // Horizontal pass
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0f32;
            for (ki, &weight) in kernel.iter().enumerate() {
                let sample_x = (x as i32 + ki as i32 - radius as i32).clamp(0, w as i32 - 1) as usize;
                acc += mask[y * w + sample_x] as f32 * weight;
            }
            temp[y * w + x] = acc;
        }
    }

    // Vertical pass
    for x in 0..w {
        for y in 0..h {
            let mut acc = 0.0f32;
            for (ki, &weight) in kernel.iter().enumerate() {
                let sample_y = (y as i32 + ki as i32 - radius as i32).clamp(0, h as i32 - 1) as usize;
                acc += temp[sample_y * w + x] * weight;
            }
            mask[y * w + x] = acc.round().clamp(0.0, 255.0) as u8;
        }
    }
}
