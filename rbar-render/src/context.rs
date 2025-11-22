use wgpu::{Adapter, Device, Queue};

#[derive(Debug, Clone)]
pub struct WgpuContext {
    pub device: Device,
    pub queue: Queue,
    pub adapter: Adapter,
}

impl WgpuContext {
    pub fn new(device: Device, queue: Queue, adapter: Adapter) -> Self {
        Self {
            device,
            queue,
            adapter,
        }
    }
}
