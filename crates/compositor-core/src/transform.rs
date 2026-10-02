use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerSampling {
    #[serde(rename = "Nearest")]
    Nearest,
    #[serde(rename = "Smooth")]
    Smooth,
    #[serde(rename = "High quality")]
    High,
}

impl Default for LayerSampling {
    fn default() -> Self {
        Self::High
    }
}

/// Unrotated bounds in document pixels; rotation is clockwise around their center.
/// Ported 1:1 from `LayerTransform.swift`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayerTransform {
    pub origin: [f64; 2],
    pub size: [f64; 2],
    #[serde(default)]
    pub rotation: f64,
    #[serde(default, rename = "flipX")]
    pub flip_x: bool,
    #[serde(default, rename = "flipY")]
    pub flip_y: bool,
    #[serde(default)]
    pub sampling: LayerSampling,
}

impl Default for LayerTransform {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0],
            size: [1.0, 1.0],
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            sampling: LayerSampling::High,
        }
    }
}

impl LayerTransform {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: [x, y],
            size: [width, height],
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            sampling: LayerSampling::High,
        }
    }

    pub fn center(&self) -> [f64; 2] {
        [
            self.origin[0] + self.size[0] / 2.0,
            self.origin[1] + self.size[1] / 2.0,
        ]
    }

    pub fn radians(&self) -> f64 {
        (self.rotation % 360.0) * PI / 180.0
    }

    pub fn is_valid(&self) -> bool {
        self.origin[0].is_finite()
            && self.origin[1].is_finite()
            && self.size[0].is_finite()
            && self.size[1].is_finite()
            && self.rotation.is_finite()
            && (1.0..=300_000.0).contains(&self.size[0])
            && (1.0..=300_000.0).contains(&self.size[1])
            && self.origin[0].abs() <= 1_000_000.0
            && self.origin[1].abs() <= 1_000_000.0
    }

    pub fn point(&self, unit: [f64; 2]) -> [f64; 2] {
        let x = (unit[0] - 0.5) * self.size[0];
        let y = (unit[1] - 0.5) * self.size[1];
        let rad = self.radians();
        let c = rad.cos();
        let s = rad.sin();
        let center = self.center();
        [
            center[0] + x * c - y * s,
            center[1] + x * s + y * c,
        ]
    }

    pub fn contains(&self, p: [f64; 2]) -> bool {
        let center = self.center();
        let x = p[0] - center[0];
        let y = p[1] - center[1];
        let rad = self.radians();
        let c = rad.cos();
        let s = rad.sin();
        (x * c + y * s).abs() <= self.size[0] / 2.0
            && (-x * s + y * c).abs() <= self.size[1] / 2.0
    }

    pub fn scale_percent(&self, pixel_size: [f64; 2]) -> f64 {
        self.size[0] / pixel_size[0].max(1.0) * 100.0
    }

    pub fn scaled(&self, percent: f64, pixel_size: [f64; 2]) -> Self {
        let center = self.center();
        let new_w = pixel_size[0] * percent / 100.0;
        let new_h = pixel_size[1] * percent / 100.0;
        Self {
            origin: [center[0] - new_w / 2.0, center[1] - new_h / 2.0],
            size: [new_w, new_h],
            rotation: self.rotation,
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            sampling: self.sampling,
        }
    }

    pub fn rounded(&self) -> Self {
        Self {
            origin: [self.origin[0].round(), self.origin[1].round()],
            size: [self.size[0].round().max(1.0), self.size[1].round().max(1.0)],
            rotation: self.rotation.round(),
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            sampling: self.sampling,
        }
    }

    pub const HANDLES: [[f64; 2]; 8] = [
        [0.0, 0.0],
        [0.5, 0.0],
        [1.0, 0.0],
        [1.0, 0.5],
        [1.0, 1.0],
        [0.5, 1.0],
        [0.0, 1.0],
        [0.0, 0.5],
    ];
}

/// Moving a layer snaps its edges and center to the canvas and to the other layers.
pub struct TransformSnap;

impl TransformSnap {
    pub const DISTANCE: f64 = 10.0;

    pub fn offset(
        box_rect: [f64; 4], // [min_x, min_y, width, height]
        xs: &[f64],
        ys: &[f64],
        tolerance: f64,
    ) -> ([f64; 2], Option<f64>, Option<f64>) {
        let min_x = box_rect[0];
        let mid_x = box_rect[0] + box_rect[2] / 2.0;
        let max_x = box_rect[0] + box_rect[2];

        let min_y = box_rect[1];
        let mid_y = box_rect[1] + box_rect[3] / 2.0;
        let max_y = box_rect[1] + box_rect[3];

        let (h_move, h_target) = Self::shift(&[min_x, mid_x, max_x], xs, tolerance);
        let (v_move, v_target) = Self::shift(&[min_y, mid_y, max_y], ys, tolerance);

        ([h_move, v_move], h_target, v_target)
    }

    fn shift(guides: &[f64], targets: &[f64], tolerance: f64) -> (f64, Option<f64>) {
        let mut best: Option<(f64, f64)> = None;
        for &guide in guides {
            for &target in targets {
                let m = target - guide;
                if m.abs() > tolerance {
                    continue;
                }
                if let Some((best_move, _)) = best {
                    if best_move.abs() <= m.abs() {
                        continue;
                    }
                }
                best = Some((m, target));
            }
        }
        match best {
            Some((m, target)) => (m, Some(target)),
            None => (0.0, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_validity() {
        let mut t = LayerTransform::new(0.0, 0.0, 100.0, 200.0);
        assert!(t.is_valid());
        assert_eq!(t.center(), [50.0, 100.0]);
        assert!(t.contains([50.0, 100.0]));
        assert!(!t.contains([200.0, 200.0]));

        t.origin[0] = 2_000_000.0;
        assert!(!t.is_valid());
    }

    #[test]
    fn test_transform_handles_and_points() {
        let t = LayerTransform::new(0.0, 0.0, 100.0, 100.0);
        assert_eq!(t.point([0.0, 0.0]), [0.0, 0.0]);
        assert_eq!(t.point([1.0, 1.0]), [100.0, 100.0]);
        assert_eq!(t.point([0.5, 0.5]), [50.0, 50.0]);
    }
}
