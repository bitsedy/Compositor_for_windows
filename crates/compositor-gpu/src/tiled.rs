use crate::context::GpuContext;
use bytemuck::{Pod, Zeroable};
use compositor_core::LayerBlendMode;
use std::collections::HashSet;

pub const TILE_SIZE: u32 = 256;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct BlendUniforms {
    pub opacity: f32,
    pub blend_mode: u32,
    pub has_mask: u32,
    pub _pad: u32,
    pub transform: [f32; 12], // 3x3 matrix in 3 vec4s for GPU alignment
}

pub struct TiledLayerRenderer {
    pub composite_pipeline: wgpu::ComputePipeline,
    pub width: u32,
    pub height: u32,
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub dirty_tiles: HashSet<(u32, u32)>,
}

impl TiledLayerRenderer {
    pub fn new(gpu: &GpuContext, width: u32, height: u32) -> Self {
        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tiled Composite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/composite.wgsl").into()),
        });

        let composite_pipeline = gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("composite_tile"),
            layout: None,
            module: &shader,
            entry_point: Some("composite_tile"),
            compilation_options: Default::default(),
            cache: None,
        });

        let tiles_x = (width + TILE_SIZE - 1) / TILE_SIZE;
        let tiles_y = (height + TILE_SIZE - 1) / TILE_SIZE;

        let mut dirty = HashSet::new();
        for ty in 0..tiles_y {
            for tx in 0..tiles_x {
                dirty.insert((tx, ty));
            }
        }

        Self {
            composite_pipeline,
            width,
            height,
            tiles_x,
            tiles_y,
            dirty_tiles: dirty,
        }
    }

    /// Marks a bounding box region (e.g. from an edited layer or brush stroke) as dirty.
    pub fn mark_dirty_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        if w <= 0 || h <= 0 {
            return;
        }
        let min_tx = ((x.max(0)) as u32) / TILE_SIZE;
        let max_tx = (((x + w).max(0) as u32) / TILE_SIZE).min(self.tiles_x - 1);
        let min_ty = ((y.max(0)) as u32) / TILE_SIZE;
        let max_ty = (((y + h).max(0) as u32) / TILE_SIZE).min(self.tiles_y - 1);

        for ty in min_ty..=max_ty {
            for tx in min_tx..=max_tx {
                self.dirty_tiles.insert((tx, ty));
            }
        }
    }

    /// Clears the dirty set after compositing.
    pub fn clear_dirty(&mut self) {
        self.dirty_tiles.clear();
    }

    /// Helper to convert LayerBlendMode into integer index matching the WGSL switch.
    pub fn blend_mode_to_u32(mode: LayerBlendMode) -> u32 {
        match mode {
            LayerBlendMode::Normal => 0,
            LayerBlendMode::Multiply => 1,
            LayerBlendMode::Screen => 2,
            LayerBlendMode::Overlay => 3,
            LayerBlendMode::Darken => 4,
            LayerBlendMode::Lighten => 5,
            LayerBlendMode::ColorDodge => 6,
            LayerBlendMode::ColorBurn => 7,
            LayerBlendMode::HardLight => 8,
            LayerBlendMode::SoftLight => 9,
            LayerBlendMode::Difference => 10,
            LayerBlendMode::Exclusion => 11,
            LayerBlendMode::LinearDodge => 12,
            LayerBlendMode::LinearBurn => 13,
            _ => 0, // Fallback to normal
        }
    }
}
