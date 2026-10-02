use crate::context::GpuContext;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct SpreadUniforms {
    pub width: u32,
    pub height: u32,
    pub reach: u32,
    pub smallest: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct ShiftUniforms {
    pub width: u32,
    pub height: u32,
    pub dx: f32,
    pub dy: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct BlurUniforms {
    pub width: u32,
    pub height: u32,
    pub sigma: f32,
    pub radius: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct ComposeUniforms {
    pub width: u32,
    pub height: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub stroke_color: [f32; 4],
    pub shadow_color: [f32; 4],
    pub overlay_color: [f32; 4],
    pub inner_color: [f32; 4],
    pub glow_color: [f32; 4],
    pub inner_glow_color: [f32; 4],
    pub flags: [u32; 4],
    pub more: [u32; 4],
}

pub struct LayerEffectsPipeline {
    pub shader: wgpu::ShaderModule,
    pub alpha_pipeline: wgpu::ComputePipeline,
    pub spread_rows_pipeline: wgpu::ComputePipeline,
    pub spread_columns_pipeline: wgpu::ComputePipeline,
    pub ring_pipeline: wgpu::ComputePipeline,
    pub shift_pipeline: wgpu::ComputePipeline,
    pub blur_rows_pipeline: wgpu::ComputePipeline,
    pub blur_columns_pipeline: wgpu::ComputePipeline,
    pub inside_pipeline: wgpu::ComputePipeline,
    pub compose_pipeline: wgpu::ComputePipeline,
}

impl LayerEffectsPipeline {
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Layer Effects Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/layer_effects.wgsl").into()),
        });

        let create_pipeline = |name: &'static str| -> wgpu::ComputePipeline {
            gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: None, // Infer from WGSL
                module: &shader,
                entry_point: Some(name),
                compilation_options: Default::default(),
                cache: None,
            })
        };

        let alpha_pipeline = create_pipeline("effects_alpha");
        let spread_rows_pipeline = create_pipeline("effects_spread_rows");
        let spread_columns_pipeline = create_pipeline("effects_spread_columns");
        let ring_pipeline = create_pipeline("effects_ring");
        let shift_pipeline = create_pipeline("effects_shift");
        let blur_rows_pipeline = create_pipeline("effects_blur_rows");
        let blur_columns_pipeline = create_pipeline("effects_blur_columns");
        let inside_pipeline = create_pipeline("effects_inside");
        let compose_pipeline = create_pipeline("effects_compose");

        Self {
            shader,
            alpha_pipeline,
            spread_rows_pipeline,
            spread_columns_pipeline,
            ring_pipeline,
            shift_pipeline,
            blur_rows_pipeline,
            blur_columns_pipeline,
            inside_pipeline,
            compose_pipeline,
        }
    }
}
