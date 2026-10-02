use crate::context::GpuContext;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct DabUniforms {
    pub center: [i32; 2],
    pub radius: i32,
    pub _pad0: i32,
    pub size: [i32; 2],
    pub origin: [i32; 2],
    pub area: [i32; 2],
    pub inverse_radius: f32,
    pub hardness: f32,
    pub keep: f32,
    pub _pad1: f32,
    pub mov: [f32; 2],
}

pub struct WarpPipeline {
    pub shader: wgpu::ShaderModule,
    pub pick_up_pipeline: wgpu::ComputePipeline,
    pub smudge_pipeline: wgpu::ComputePipeline,
    pub push_pipeline: wgpu::ComputePipeline,
}

impl WarpPipeline {
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Warp Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/warp.wgsl").into()),
        });

        let create_pipeline = |name: &'static str| -> wgpu::ComputePipeline {
            gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: None, // Auto-layout from WGSL bindings
                module: &shader,
                entry_point: Some(name),
                compilation_options: Default::default(),
                cache: None,
            })
        };

        let pick_up_pipeline = create_pipeline("warp_pick_up");
        let smudge_pipeline = create_pipeline("warp_smudge");
        let push_pipeline = create_pipeline("warp_push");

        Self {
            shader,
            pick_up_pipeline,
            smudge_pipeline,
            push_pipeline,
        }
    }
}
