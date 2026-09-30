//! Table statique des paramètres, portée de `device.go`.

use crate::protocol::{Device, ADDR_DEVICE, ADDR_STATUS};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Raw,
    /// Dixièmes de dB (brut -100 = -10.0 dB).
    Db10,
    /// Demi-dB (brut 14 = 7.0 dB), ex. Bass/Treble Gain du Loudness.
    Db05,
    /// Dixièmes (brut 9 = 0.9), ex. Q des filtres.
    Tenth,
    Hz,
    /// Centièmes (brut 100 = 1.00).
    Percent,
}

#[derive(Debug, Clone, Copy)]
pub struct Param {
    pub index: u8,
    pub name: &'static str,
    pub min: i32,
    pub max: i32,
    pub default: i32,
    pub step: i32,
    pub options: &'static [&'static str],
    pub unit: Unit,
}

impl Param {
    pub fn format(&self, raw: i32) -> String {
        if let Some(o) = usize::try_from(raw).ok().and_then(|i| self.options.get(i)) {
            return (*o).to_string();
        }
        match self.unit {
            Unit::Db10 => format!("{:.1} dB", raw as f32 / 10.0),
            Unit::Tenth => format!("{:.1}", raw as f32 / 10.0),
            Unit::Hz => format!("{raw} Hz"),
            Unit::Db05 => format!("{:.1} dB", raw as f32 / 2.0),
            Unit::Percent => format!("{:.2}", raw as f32 / 100.0),
            Unit::Raw if self.max == 1 && self.min == 0 => (if raw == 0 { "OFF" } else { "ON" }).into(),
            Unit::Raw => raw.to_string(),
        }
    }
}

const fn p(index: u8, name: &'static str, min: i32, max: i32, default: i32, step: i32) -> Param {
    Param { index, name, min, max, default, step, options: &[], unit: Unit::Raw }
}
const fn o(mut x: Param, options: &'static [&'static str]) -> Param {
    x.options = options;
    x
}
const fn u(mut x: Param, unit: Unit) -> Param {
    x.unit = unit;
    x
}

const SOURCES: &[&str] = &["Auto", "AES", "SPDIF", "Analog", "USB 1/2", "USB 3/4"];

pub static DEVICE: &[Param] = &[
    o(p(1, "Mute Line vs.", 0, 3, 1, 1), &["OFF", "vs. Phones", "Toggle Ph/Line", "Toggle plugged"]),
    o(p(2, "Auto Standby", 0, 4, 0, 1), &["OFF", "30min", "1h", "2h", "4h"]),
    p(3, "DSD Detection", 0, 1, 1, 1),
    o(p(4, "DSD Filter", 0, 1, 0, 1), &["50 kHz", "150 kHz"]),
    p(5, "DSD Direct (Line)", 0, 1, 0, 1),
    o(p(6, "Basic Mode", 0, 5, 0, 1), &["Auto", "AD/DA", "USB", "Preamp", "Dig Thru", "DAC"]),
    o(p(7, "Digital Out Source", 0, 1, 0, 1), &["Default", "Main Out"]),
    p(8, "Dual Phones", 0, 1, 0, 0),
    o(p(9, "Bal. TRS Phones Mode", 0, 0, 0, 0), &["OFF", "ON", "Auto"]),
    o(p(10, "Toggle Phones/Line", 0, 0, 0, 0), &["OFF", "Ph 1/2", "Ph 3/4", "1/2+3/4", "all plugged", "Line/Digit."]),
    p(11, "Mute Line vs. PH12", 0, 1, 1, 1),
    p(12, "Mute Line vs. PH 34", 0, 1, 1, 1),
    o(p(15, "Clock Source", 0, 3, 0, 1), &["Auto", "INT", "AES", "SPDIF"]),
    o(p(16, "Sample Rate", 0, 9, 1, 1), &["44.1 kHz", "48 kHz", "88 kHz", "96 kHz", "176.4 kHz", "192 kHz", "352.8 kHz", "384 kHz", "705.6 kHz", "768 kHz"]),
    o(p(21, "IR 5", 0, 2, 2, 1), &["ADI-2/4", "ADI-2 Pro"]),
    o(p(22, "IR 6", 0, 2, 2, 1), &["ADI-2/4", "ADI-2 Pro"]),
    o(p(23, "IR 7", 0, 2, 2, 1), &["ADI-2/4", "ADI-2 Pro"]),
    o(p(25, "Remap Keys", 0, 2, 2, 1), &["OFF", "ON", "IR-Remote"]),
    o(p(32, "Display Mode", 0, 1, 0, 1), &["Default", "Dark"]),
    o(p(33, "Meter Color", 0, 5, 0, 1), &["Green", "Cyan", "Amber", "Monochrome", "Red", "Orange"]),
    o(p(34, "Hor. Meter", 0, 3, 0, 1), &["Post-FX", "Pre-FX", "Dual", "Post-FX dBu"]),
    p(35, "AutoDark Mode", 0, 1, 0, 1),
    p(36, "Show Vol. Screen", 0, 1, 1, 1),
    o(p(37, "Lock UI", 0, 3, 0, 1), &["OFF", "Remote", "Keys", "Keys+Rem."]),
];

/// Paramètres communs aux canaux 3 (Line), 6 (Phones 1/2) et 9 (Phones 3/4).
pub static CHANNEL: &[Param] = &[
    o(p(1, "Source", 0, 6, 0, 1), SOURCES),
    o(p(2, "Ref Level", 0, 3, 2, 1), &["+4 dBu", "+13 dBu", "+19 dBu", "+24 dBu"]),
    p(3, "Auto Ref Level", 0, 1, 1, 1),
    o(p(4, "Mono", 0, 2, 0, 1), &["OFF", "ON", "to Left"]),
    u(p(5, "Width", -100, 100, 100, 1), Unit::Percent),
    p(6, "M/S-Proc", 0, 1, 0, 1),
    o(p(7, "Polarity", 0, 3, 0, 1), &["OFF", "Both", "Left", "Right"]),
    p(8, "Crossfeed", 0, 5, 0, 1),
    o(p(9, "DA Filter", 0, 6, 2, 1), &["SD Sharp", "SD Slow", "Sharp", "Slow", "NOS", "SD LD"]),
    o(p(10, "De-Emphasis", 0, 2, 0, 1), &["Auto", "ON", "OFF"]),
    p(11, "Dual EQ", 0, 1, 0, 1),
    u(p(12, "Volume", -1145, 60, -100, 5), Unit::Db10),
    p(13, "Lock Volume", 0, 1, 0, 1),
    u(p(14, "Balance", -100, 100, 0, 1), Unit::Percent),
    p(15, "Mute", 0, 1, 0, 1),
    p(16, "Dim", 0, 1, 0, 1),
    o(p(17, "Loopback to USB", 0, 9, 0, 1), &["OFF", "pre FX to 1/2", "post FX to 1/2", "post 1/2 -6dB", "pre FX to 3/4", "post FX to 3/4", "post 3/4 -6dB", "pre FX to 5/6", "post FX to 5/6", "post 5/6 -6dB"]),
    o(p(18, "Dig. DC Protection", 0, 2, 2, 1), &["OFF", "ON", "Filter"]),
    o(p(19, "Rear TRS Source", 0, 1, 0, 1), &["Line 1/2", "Ph. 3/4"]),
    p(20, "Loudness Enable", 0, 1, 0, 1),
    u(p(21, "Bass Gain", 2, 20, 14, 1), Unit::Db05),
    u(p(22, "Treble Gain", 2, 20, 14, 1), Unit::Db05),
    u(p(23, "Low Vol Ref", -9000, -2000, -3000, 5), Unit::Db10),
];

const BAND_NAMES: [[&str; 4]; 5] = [
    ["Band 1 Type", "Band 1 Gain", "Band 1 Freq", "Band 1 Q"],
    ["", "Band 2 Gain", "Band 2 Freq", "Band 2 Q"],
    ["", "Band 3 Gain", "Band 3 Freq", "Band 3 Q"],
    ["", "Band 4 Gain", "Band 4 Freq", "Band 4 Q"],
    ["Band 5 Type", "Band 5 Gain", "Band 5 Freq", "Band 5 Q"],
];
const DEFAULT_FREQ: [i32; 5] = [100, 500, 1000, 5000, 10000];

/// Table EQ : 5 bandes (indices 3 à 19) et, côté gauche seulement, Enable, Bass/Treble et « Load B/T ».
/// `bt_gain` = gain Bass/Treble max en demi-dB (DAC : ±6 dB, Pro/SE : ±12 dB).
fn build_eq(left: bool, bt_gain: i32) -> Vec<Param> {
    let mut v = Vec::new();
    if left {
        v.push(p(2, "EQ Enable", 0, 1, 0, 1));
    }
    let bases = [3u8, 7, 10, 13, 16];
    for (n, &base) in bases.iter().enumerate() {
        let [t, g, f, q] = BAND_NAMES[n];
        let mut i = base;
        if n == 0 {
            v.push(o(p(i, t, 0, 3, 1, 1), &["Peak", "Shelf", "Hi Pass", "Hi Cut"]));
            i += 1;
        } else if n == 4 {
            v.push(o(p(i, t, 0, 2, 1, 1), &["Peak", "Shelf", "Hi Cut"]));
            i += 1;
        }
        let (fmin, qmax) = if n >= 3 { (200, 50) } else { (20, 99) };
        v.push(u(p(i, g, -24, 24, 0, 1), Unit::Db05));
        v.push(u(p(i + 1, f, fmin, 20000, DEFAULT_FREQ[n], 1), Unit::Hz));
        v.push(u(p(i + 2, q, 5, qmax, 10, 1), Unit::Tenth));
    }
    if left {
        v.push(p(20, "B/T Enable", 0, 1, 1, 1));
        v.push(u(p(21, "Bass Gain", -bt_gain, bt_gain, 0, 1), Unit::Db05));
        v.push(u(p(22, "Bass Freq", 20, 150, 85, 1), Unit::Hz));
        v.push(u(p(23, "Bass Q", 5, 15, 9, 1), Unit::Tenth));
        v.push(u(p(24, "Treble Gain", -bt_gain, bt_gain, 0, 1), Unit::Db05));
        v.push(u(p(25, "Treble Freq", 3000, 10000, 6500, 100), Unit::Hz));
        v.push(u(p(26, "Treble Q", 5, 15, 7, 1), Unit::Tenth));
        v.push(p(27, "Load B/T w. Preset", 0, 1, 0, 1));
    }
    v
}

/// ADI-2 DAC : Bass/Treble limités à ±6 dB (manuel §8.4). Pro/SE : ±12 dB (tableau MIDI).
fn eq_left(dev: Device) -> &'static [Param] {
    static DAC: OnceLock<Vec<Param>> = OnceLock::new();
    static PRO: OnceLock<Vec<Param>> = OnceLock::new();
    if dev == Device::Dac {
        DAC.get_or_init(|| build_eq(true, 12))
    } else {
        PRO.get_or_init(|| build_eq(true, 24))
    }
}

fn eq_right() -> &'static [Param] {
    static R: OnceLock<Vec<Param>> = OnceLock::new();
    R.get_or_init(|| build_eq(false, 0))
}

/// Adresse EQ droite associée à une adresse EQ gauche (4→5, 7→8, 10→11).
pub fn eq_right_of(left: u8) -> Option<u8> {
    matches!(left, 4 | 7 | 10).then_some(left + 1)
}

/// Infos du message de statut (lecture seule), adresse virtuelle `ADDR_STATUS`.
/// Valeurs brutes : la correspondance avec les libellés n'est pas documentée par RME.
pub static STATUS: &[Param] = &[
    p(1, "Active output (DAC)", 0, 3, 0, 0),
    p(2, "Device revision", 0, 3, 0, 0),
    p(3, "Active line output", 0, 3, 0, 0),
    p(4, "Phones 3/4 active", 0, 1, 0, 0),
    p(5, "Current mode", 0, 7, 0, 0),
    p(6, "Balanced mode", 0, 1, 0, 0),
    p(7, "DSD decode", 0, 7, 0, 0),
    p(8, "Protocol revision (raw)", 0, 127, 0, 0),
];

/// Courte description d'un paramètre (source : manuel ADI-2 DAC v1.8 et tableau MIDI RME).
pub fn describe(addr: u8, name: &str) -> &'static str {
    crate::i18n::t(match (addr, name) {
        (3 | 6 | 9, "Source") => "desc.ch.source",
        (3 | 6 | 9, "Ref Level") => "desc.ch.ref_level",
        (3 | 6 | 9, "Auto Ref Level") => "desc.ch.auto_ref_level",
        (3 | 6 | 9, "Mono") => "desc.ch.mono",
        (3 | 6 | 9, "Width") => "desc.ch.width",
        (3 | 6 | 9, "M/S-Proc") => "desc.ch.m_s_proc",
        (3 | 6 | 9, "Polarity") => "desc.ch.polarity",
        (3 | 6 | 9, "Crossfeed") => "desc.ch.crossfeed",
        (3 | 6 | 9, "DA Filter") => "desc.ch.da_filter",
        (3 | 6 | 9, "De-Emphasis") => "desc.ch.de_emphasis",
        (3 | 6 | 9, "Dual EQ") => "desc.ch.dual_eq",
        (3 | 6 | 9, "Volume") => "desc.ch.volume",
        (3 | 6 | 9, "Lock Volume") => "desc.ch.lock_volume",
        (3 | 6 | 9, "Balance") => "desc.ch.balance",
        (3 | 6 | 9, "Mute") => "desc.ch.mute",
        (3 | 6 | 9, "Dim") => "desc.ch.dim",
        (3 | 6 | 9, "Loopback to USB") => "desc.ch.loopback_to_usb",
        (3 | 6 | 9, "Dig. DC Protection") => "desc.ch.dig_dc_protection",
        (3 | 6 | 9, "Rear TRS Source") => "desc.ch.rear_trs_source",
        (3 | 6 | 9, "Loudness Enable") => "desc.ch.loudness_enable",
        (3 | 6 | 9, "Bass Gain") => "desc.ch.bass_gain",
        (3 | 6 | 9, "Treble Gain") => "desc.ch.treble_gain",
        (3 | 6 | 9, "Low Vol Ref") => "desc.ch.low_vol_ref",

        (4 | 7 | 10, "B/T Enable") => "desc.tone.b_t_enable",
        (4 | 7 | 10, "Bass Gain") => "desc.tone.bass_gain",
        (4 | 7 | 10, "Bass Freq") => "desc.tone.bass_freq",
        (4 | 7 | 10, "Bass Q") => "desc.tone.bass_q",
        (4 | 7 | 10, "Treble Gain") => "desc.tone.treble_gain",
        (4 | 7 | 10, "Treble Freq") => "desc.tone.treble_freq",
        (4 | 7 | 10, "Treble Q") => "desc.tone.treble_q",

        (4 | 7 | 10, "EQ Enable") => "desc.tone.eq_enable",
        (4 | 7 | 10, "Load B/T w. Preset") => "desc.tone.load_b_t_w_preset",
        (4 | 5 | 7 | 8 | 10 | 11, n) if n.ends_with("Type") => "desc.eq.band_type",
        (4 | 5 | 7 | 8 | 10 | 11, n) if n.starts_with("Band") && n.ends_with("Gain") => "desc.eq.band_gain",
        (4 | 5 | 7 | 8 | 10 | 11, n) if n.starts_with("Band") && n.ends_with("Freq") => "desc.eq.band_freq",
        (4 | 5 | 7 | 8 | 10 | 11, n) if n.starts_with("Band") && n.ends_with('Q') => "desc.eq.band_q",
        (ADDR_DEVICE, "Mute Line vs.") => "desc.dev.mute_line_vs",
        (ADDR_DEVICE, "Auto Standby") => "desc.dev.auto_standby",
        (ADDR_DEVICE, "DSD Detection") => "desc.dev.dsd_detection",
        (ADDR_DEVICE, "DSD Filter") => "desc.dev.dsd_filter",
        (ADDR_DEVICE, "DSD Direct (Line)") => "desc.dev.dsd_direct_line",
        (ADDR_DEVICE, "Clock Source") => "desc.dev.clock_source",
        (ADDR_DEVICE, "Sample Rate") => "desc.dev.sample_rate",
        (ADDR_DEVICE, "Remap Keys") => "desc.dev.remap_keys",
        (ADDR_DEVICE, "Display Mode") => "desc.dev.display_mode",
        (ADDR_DEVICE, "Meter Color") => "desc.dev.meter_color",
        (ADDR_DEVICE, "Hor. Meter") => "desc.dev.hor_meter",
        (ADDR_DEVICE, "AutoDark Mode") => "desc.dev.autodark_mode",
        (ADDR_DEVICE, "Show Vol. Screen") => "desc.dev.show_vol_screen",
        (ADDR_DEVICE, "Lock UI") => "desc.dev.lock_ui",
        (ADDR_STATUS, _) => "desc.status.all",
        _ => "",
    })
}

pub fn channel_name(addr: u8) -> &'static str {
    match addr {
        3 => "Line Out",
        6 => "Phones 1/2",
        9 => "Phones 3/4",
        4 | 5 => "Line EQ",
        7 | 8 => "Phones 1/2 EQ",
        10 | 11 => "Phones 3/4 EQ",
        ADDR_DEVICE => "Device",
        _ => "?",
    }
}

/// Paramètres éditables d'une adresse ; `None` si absente sur ce modèle.
pub fn for_address(dev: Device, addr: u8) -> Option<&'static [Param]> {
    match addr {
        ADDR_DEVICE => Some(DEVICE),
        3 | 6 => Some(CHANNEL),
        9 if dev.has_phones34() => Some(CHANNEL),
        4 | 7 => Some(eq_left(dev)),
        10 if dev.has_phones34() => Some(eq_left(dev)),
        5 | 8 => Some(eq_right()),
        11 if dev.has_phones34() => Some(eq_right()),
        _ => None,
    }
}

/// Adresses affichables pour un modèle, dans l'ordre d'affichage.
pub fn addresses(dev: Device) -> Vec<u8> {
    [3u8, 4, 6, 7, 9, 10, ADDR_DEVICE].into_iter().filter(|&a| for_address(dev, a).is_some()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dac_has_no_phones34() {
        assert_eq!(addresses(Device::Dac), [3, 4, 6, 7, 12]);
        assert_eq!(addresses(Device::Pro), [3, 4, 6, 7, 9, 10, 12]);
    }
    #[test]
    fn bass_treble_gain_is_half_db() {
        // Trame officielle RME : `1D 20 0E` = canal 3, index 21, valeur 14 = +7 dB.
        let (a, i, v) = crate::protocol::parse_status(&[0x1D, 0x20, 0x0E])[0];
        assert_eq!((a, i, v), (3, 21, 14));
        let g = CHANNEL.iter().find(|p| p.index == 21).unwrap();
        assert_eq!(g.format(v), "7.0 dB");
        assert_eq!((g.min, g.max, g.default), (2, 20, 14));
    }

    #[test]
    fn volume_is_converted() {
        let v = CHANNEL.iter().find(|p| p.name == "Volume").unwrap();
        assert_eq!(v.format(-100), "-10.0 dB");
    }
}
