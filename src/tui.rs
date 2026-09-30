use crate::params::{self, Param};
use crate::protocol::{self, Device};
use crate::state::{Snapshot, StateHandle};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::Line;
use ratatui::style::Color;
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType, List, ListItem, ListState, Paragraph, Tabs, Wrap};
use crate::eq;
use ratatui::DefaultTerminal;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

pub async fn run(
    mut term: DefaultTerminal,
    dev: Device,
    state: StateHandle,
    out: UnboundedSender<Vec<u8>>,
    interval: Duration,
    port_name: String,
) -> anyhow::Result<()> {
    let addrs = params::addresses(dev);
    let mut tab = 0usize;
    let mut right = false; // EQ : côté droit (Dual EQ)
    let mut list = ListState::default().with_selected(Some(0));
    let mut snap = Snapshot::new();
    let mut changes = state.changes.subscribe();
    let mut tick = tokio::time::interval(Duration::from_millis(50));
    let mut sync = tokio::time::interval(interval);

    loop {
        while changes.try_recv().is_ok() {
            if let Some(s) = state.snapshot().await {
                snap = s;
            }
        }
        let base = addrs[tab];
        let is_eq = params::eq_right_of(base).is_some();
        let addr = if is_eq && right { base + 1 } else { base };
        let plist = params::for_address(dev, addr).unwrap_or(&[]);
        term.draw(|f| {
            let [top, body, desc, help] =
                Layout::vertical([Constraint::Length(3), Constraint::Min(0), Constraint::Length(4), Constraint::Length(1)]).areas(f.area());
            let titles: Vec<_> = addrs
                .iter()
                .map(|&a| if a == base && is_eq && right { format!("{} (R)", params::channel_name(a)) } else { params::channel_name(a).to_string() })
                .collect();
            f.render_widget(
                Tabs::new(titles).select(tab).block(Block::bordered().title(dev.name())).highlight_style(Style::new().add_modifier(Modifier::REVERSED)),
                top,
            );
            let mut items: Vec<ListItem> = plist
                .iter()
                .map(|p| {
                    let v = snap.get(&addr).and_then(|m| m.get(&p.index)).map(|&v| p.format(v)).unwrap_or_else(|| "—".into());
                    ListItem::new(format!("{:<24} {v}", p.name))
                })
                .collect();
            if addr == protocol::ADDR_DEVICE {
                items.push(ListItem::new("── Infos (lecture seule) ──"));
                items.push(ListItem::new(format!("{:<26} {port_name}", "Port MIDI")));
                for sp in params::STATUS {
                    let v = snap.get(&protocol::ADDR_STATUS).and_then(|m| m.get(&sp.index)).map(|v| v.to_string()).unwrap_or_else(|| "—".into());
                    items.push(ListItem::new(format!("{:<26} {v}", sp.name)));
                }
                items.push(ListItem::new("Firmware FPGA/DSP : absent du protocole MIDI"));
                items.push(ListItem::new("  -> menu SETUP > Options > SW Version"));
            }
            let body = if is_eq {
                let [chart_area, rest] = Layout::vertical([Constraint::Length(13), Constraint::Min(0)]).areas(body);
                let snap_ref = &snap;
                let get = |a: u8| move |i: u8| snap_ref.get(&a).and_then(|m| m.get(&i)).copied();
                let enabled = get(base)(2).unwrap_or(0) != 0;
                let bt_on = get(base)(20).unwrap_or(0) != 0;
                let bt = get(base);
                let bands = eq::bands(&get(addr), if bt_on { Some(&bt) } else { None });
                let pts = eq::curve(&bands);
                let color = if enabled { Color::Green } else { Color::DarkGray };
                let ds = Dataset::default().marker(Marker::Braille).graph_type(GraphType::Line).style(color).data(&pts);
                let xl = ["20", "200", "2k", "20k"].map(Line::from);
                let yl = ["-15", "-10", "-5", "0", "+5", "+10", "+15"].map(Line::from);
                f.render_widget(
                    Chart::new(vec![ds])
                        .block(Block::bordered().title(if enabled { "Réponse EQ (calculée)" } else { "Réponse EQ (EQ désactivé)" }))
                        .x_axis(Axis::default().bounds([20f64.log10(), 20000f64.log10()]).labels(xl))
                        .y_axis(Axis::default().bounds([-15.0, 15.0]).labels(yl)),
                    chart_area,
                );
                rest
            } else {
                body
            };
            f.render_stateful_widget(
                List::new(items).block(Block::bordered()).highlight_style(Style::new().add_modifier(Modifier::REVERSED)),
                body,
                &mut list,
            );
            let text = plist.get(list.selected().unwrap_or(0)).map(|p| params::describe(addr, p.name)).unwrap_or("");
            f.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }).block(Block::bordered().title("Description")), desc);
            f.render_widget("←/→ valeur  ↑/↓ param  Tab onglet  s côté L/R (EQ)  r rafraîchir  q quitter", help);
        })?;

        tokio::select! {
            _ = sync.tick() => { let _ = out.send(protocol::status_request(dev)); }
            _ = tick.tick() => {
                while event::poll(Duration::ZERO)? {
                    let Event::Key(k) = event::read()? else { continue };
                    if k.kind != KeyEventKind::Press { continue }
                    let sel = list.selected().unwrap_or(0);
                    match k.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('r') => { let _ = out.send(protocol::status_request(dev)); }
                        KeyCode::Char('s') if is_eq => { right = !right; list.select(Some(0)); }
                        KeyCode::Tab => { tab = (tab + 1) % addrs.len(); right = false; list.select(Some(0)); }
                        KeyCode::Down => list.select(Some((sel + 1).min(plist.len().saturating_sub(1)))),
                        KeyCode::Up => list.select(Some(sel.saturating_sub(1))),
                        KeyCode::Left | KeyCode::Right => {
                            let dir = if k.code == KeyCode::Right { 1 } else { -1 };
                            if let Some(p) = plist.get(sel) {
                                adjust(dev, addr, p, snap.get(&addr).and_then(|m| m.get(&p.index)).copied(), dir, &out);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn adjust(dev: Device, addr: u8, p: &Param, cur: Option<i32>, dir: i32, out: &UnboundedSender<Vec<u8>>) {
    let Some(cur) = cur else { return };
    let log = p.unit == params::Unit::Hz && p.min >= 20 && p.max / p.min >= 10;
    let step = if log { (cur / 25).max(1) } else { p.step.max(1) };
    let mut new = cur + dir * step;
    if p.unit == params::Unit::Hz && new > 2047 {
        new = (new + 5) / 10 * 10; // le protocole n'envoie que des multiples de 10 au-delà de 2047 Hz
    }
    let new = new.clamp(p.min, p.max);
    if new != cur {
        if let Some(m) = protocol::set_command(dev, addr, p.index, new) {
            let _ = out.send(m);
            let _ = out.send(protocol::status_request(dev)); // le device répond avec le nouvel état
        }
    }
}
