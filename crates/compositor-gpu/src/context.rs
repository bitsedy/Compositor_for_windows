use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use thiserror::Error;

struct NoopWake;
impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}

pub fn block_on<F: Future>(mut future: F) -> F::Output {
    let mut future = unsafe { std::pin::Pin::new_unchecked(&mut future) };
    let waker = Waker::from(Arc::new(NoopWake));
    let mut cx = Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

#[derive(Error, Debug)]
pub enum GpuError {
    #[error("Failed to find suitable GPU adapter")]
    NoAdapter,
    #[error("Failed to request GPU device: {0}")]
    DeviceRequest(#[from] wgpu::RequestDeviceError),
    #[error("Surface error: {0}")]
    Surface(#[from] wgpu::CreateSurfaceError),
}

pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuContext {
    /// Creates a headless GPU context using Direct3D 12 (primary) with Vulkan fallback.
    pub async fn new_headless() -> Result<Self, GpuError> {
        let (instance, adapter) = {
            let instance_dx12 = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::DX12,
                ..Default::default()
            });
            if let Some(adapter) = instance_dx12
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
            {
                (instance_dx12, adapter)
            } else {
                let instance_vk = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::VULKAN,
                    ..Default::default()
                });
                let adapter = instance_vk
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        compatible_surface: None,
                        force_fallback_adapter: false,
                    })
                    .await
                    .ok_or(GpuError::NoAdapter)?;
                (instance_vk, adapter)
            }
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Compositor GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }
}
