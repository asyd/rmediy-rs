//! Pont MIDI : entrée -> acteur d'état, file de sortie -> port MIDI.

use crate::config::Config;
use crate::protocol::{self, Device};
use crate::state::{Cmd, StateHandle};
use anyhow::anyhow;
use midir::{MidiInput, MidiOutput};
use tokio::sync::mpsc;

pub fn list_ports() -> anyhow::Result<()> {
    let i = MidiInput::new("rmediy")?;
    let o = MidiOutput::new("rmediy")?;
    println!("{}", crate::i18n::t("midi.inputs"));
    for (n, p) in i.ports().iter().enumerate() {
        println!("  {n}: {}", i.port_name(p)?);
    }
    println!("{}", crate::i18n::t("midi.outputs"));
    for (n, p) in o.ports().iter().enumerate() {
        println!("  {n}: {}", o.port_name(p)?);
    }
    Ok(())
}

/// Garde les connexions midir en vie ; `out` reçoit les trames à envoyer.
pub struct Midi {
    pub out: mpsc::UnboundedSender<Vec<u8>>,
    /// Nom du port d'entrée (contient le numéro de série USB).
    pub port_name: String,
    _in_conn: midir::MidiInputConnection<()>,
}

pub fn start(cfg: &Config, state: StateHandle) -> anyhow::Result<Midi> {
    let dev: Device = cfg.device.id;

    let input = MidiInput::new("rmediy-in")?;
    let in_port = input.ports().get(cfg.device.midi_port_in).cloned()
        .ok_or_else(|| anyhow!(crate::i18n::tf("err.port_in", cfg.device.midi_port_in)))?;
    let port_name = input.port_name(&in_port).unwrap_or_default();
    let tx = state.tx.clone();
    let in_conn = input
        .connect(
            &in_port,
            "rmediy-in",
            move |_ts, msg, _| {
                if let Some(triplets) = protocol::parse_incoming(dev, msg) {
                    for (channel, param, value) in triplets {
                        // Callback hors runtime tokio : blocking_send est correct ici.
                        let _ = tx.blocking_send(Cmd::Set { channel, param, value });
                    }
                }
            },
            (),
        )
        .map_err(|e| anyhow!(crate::i18n::tf("err.connect_in", e)))?;

    let output = MidiOutput::new("rmediy-out")?;
    let out_port = output.ports().get(cfg.device.midi_port_out).cloned()
        .ok_or_else(|| anyhow!(crate::i18n::tf("err.port_out", cfg.device.midi_port_out)))?;
    let mut out_conn = output.connect(&out_port, "rmediy-out").map_err(|e| anyhow!(crate::i18n::tf("err.connect_out", e)))?;

    let (out, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    std::thread::spawn(move || {
        while let Some(m) = rx.blocking_recv() {
            let _ = out_conn.send(&m);
        }
    });
    Ok(Midi { out, port_name, _in_conn: in_conn })
}
