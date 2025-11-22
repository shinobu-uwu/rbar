pub mod module;
pub mod node;
pub mod style;

use anyhow::Result;
use rbar_render::SurfaceRenderer;
use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use wayland_client::protocol::wl_output::WlOutput;

use crate::bar::module::Module;

pub struct Bar {
    pub layer_surface: LayerSurface,
    pub output: WlOutput,
    pub width: u32,
    pub height: u32,
    pub modules: Vec<Module>,
    surface_renderer: SurfaceRenderer,
}

impl Bar {
    pub fn new(
        layer_surface: LayerSurface,
        output: WlOutput,
        surface_renderer: SurfaceRenderer,
    ) -> Result<Self> {
        Ok(Self {
            layer_surface,
            output,
            width: 0,
            height: 0,
            modules: vec![],
            surface_renderer,
        })
    }

    pub fn configure(&mut self, width: u32, height: u32) -> Result<()> {
        self.width = width;
        self.height = height;
        self.surface_renderer.render()?;
        self.surface_renderer.set_size(self.width, self.height);

        Ok(())
    }
}
