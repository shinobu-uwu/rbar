use anyhow::Result;
use smithay_client_toolkit::{
    compositor::CompositorState, output::OutputState, registry::RegistryState, seat::SeatState,
    shell::wlr_layer::LayerShell,
};
use wayland_client::{Connection, QueueHandle, globals::registry_queue_init};

use crate::app::App;

mod app;
mod bar;

#[tokio::main]
async fn main() -> Result<()> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh: QueueHandle<App> = event_queue.handle();

    let compositor_state = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let output_state = OutputState::new(&globals, &qh);
    let seat_state = SeatState::new(&globals, &qh);
    let registry_state = RegistryState::new(&globals);

    let mut app = App::new(
        output_state,
        seat_state,
        layer_shell,
        compositor_state,
        registry_state,
    )
    .await?;

    loop {
        event_queue.blocking_dispatch(&mut app)?;
    }
}
