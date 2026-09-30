mod config;
mod eq;
mod midi;
mod params;
mod protocol;
mod state;
mod tui;

use std::{path::PathBuf, time::Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arg = std::env::args().nth(1);
    if arg.as_deref() == Some("--list-ports") {
        return midi::list_ports();
    }
    let cfg = config::Config::load(&PathBuf::from(arg.unwrap_or_else(|| "config.yaml".into())))?;
    let state = state::spawn();
    let midi = midi::start(&cfg, state.clone())?;
    let _ = midi.out.send(protocol::status_request(cfg.device.id));

    let term = ratatui::init();
    let res = tui::run(term, cfg.device.id, state, midi.out.clone(), Duration::from_secs(cfg.sync.interval.max(1)), midi.port_name.clone()).await;
    ratatui::restore();
    res
}
