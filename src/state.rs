//! Device state owned by a single task (actor pattern):
//! no mutex, no possible race (cf. the Go version's "concurrent map writes" crash).

use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc, oneshot};

pub type Channel = u8;
pub type Param = u8;
pub type Snapshot = HashMap<Channel, HashMap<Param, i32>>;

pub enum Cmd {
    Set { channel: Channel, param: Param, value: i32 },
    Snapshot(oneshot::Sender<Snapshot>),
}

#[derive(Debug, Clone)]
pub struct Change {
    pub channel: Channel,
    pub param: Param,
    pub value: i32,
}

#[derive(Clone)]
pub struct StateHandle {
    pub tx: mpsc::Sender<Cmd>,
    pub changes: broadcast::Sender<Change>,
}

impl StateHandle {
    pub async fn snapshot(&self) -> Option<Snapshot> {
        let (t, r) = oneshot::channel();
        self.tx.send(Cmd::Snapshot(t)).await.ok()?;
        r.await.ok()
    }
}

pub fn spawn() -> StateHandle {
    let (tx, mut rx) = mpsc::channel(256);
    let (changes, _) = broadcast::channel(256);
    let bc = changes.clone();
    tokio::spawn(async move {
        let mut state = Snapshot::new();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Cmd::Set { channel, param, value } => {
                    let slot = state.entry(channel).or_default().entry(param).or_insert(i32::MIN);
                    if *slot != value {
                        *slot = value;
                        let _ = bc.send(Change { channel, param, value });
                    }
                }
                Cmd::Snapshot(reply) => {
                    let _ = reply.send(state.clone());
                }
            }
        }
    });
    StateHandle { tx, changes }
}
