use crate::context::GpuContext;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct NoiseParameters {
    pub amount: f32,
    pub gaussian: u32,
    pub monochromatic: u32,
    pub seed: u32,
    pub corner: [i32; 2],
    pub _pad: [i32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct GrainParameters {
    pub strength: f32,
    pub roughness: f32,
    pub size: f32,
    pub detail_size: f32,
    pub origin: [f32; 2],
    pub units_per_pixel: f32,
    pub seed: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct PlaceUniforms {
    pub place: [i32; 4],
}

pub struct NoisePipeline {
    pub shader: wgpu::ShaderModule,
    pub add_noise_pipeline: wgpu::ComputePipeline,
    pub add_grain_pipeline: wgpu::ComputePipeline,
}

impl NoisePipeline {
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Noise Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/noise.wgsl").into()),
        });

        let add_noise_pipeline = gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("add_noise"),
            layout: None,
            module: &shader,
            entry_point: Some("add_noise"),
            compilation_options: Default::default(),
            cache: None,
        });

        let add_grain_pipeline = gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("add_grain"),
            layout: None,
            module: &shader,
            entry_point: Some("add_grain"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            shader,
            add_noise_pipeline,
            add_grain_pipeline,
        }
    }
}
