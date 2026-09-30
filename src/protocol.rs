//! Protocole SysEx RME ADI-2, porté de `sysEx.go` / `bitman.go` / `rme.go`.
//!
//! Trame midir (F0/F7 inclus) : `F0 00 20 0D [device] [cmd] [payload…] F7`.
//! Payload de statut (cmd 1) : triplets de 3 octets `[adresse+index, …, valeur]`.

use serde::Deserialize;

pub const HEADER: [u8; 4] = [0xF0, 0x00, 0x20, 0x0D];
pub const SYSEX_END: u8 = 0xF7;

/// Adresse du device global (paramètres non liés à un canal).
pub const ADDR_DEVICE: u8 = 12;
/// Adresse virtuelle : infos du message de statut (commande 7), en lecture seule.
pub const ADDR_STATUS: u8 = 0x7F;
/// EQ gauche (1, 4, 7, 10 = Input, Line, Phones 1/2, Phones 3/4) et droit (2, 5, 8, 11).
pub const EQ_ADDRESSES: [u8; 8] = [1, 2, 4, 5, 7, 8, 10, 11];

/// Index EQ portant une fréquence : 11 bits + drapeau ×10 (bit 4 de l'octet 2).
fn is_eq_freq(index: u8) -> bool {
    matches!(index, 5 | 8 | 11 | 14 | 18 | 22 | 25)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "u8")]
pub enum Device {
    Dac,
    Pro,
    ProSe,
}

impl TryFrom<u8> for Device {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, String> {
        match v {
            0x71 => Ok(Device::Dac),
            0x72 => Ok(Device::Pro),
            0x73 => Ok(Device::ProSe),
            _ => Err(format!("device id invalide {v:#x} (0x71, 0x72 ou 0x73)")),
        }
    }
}

impl Device {
    pub fn id(self) -> u8 {
        match self {
            Device::Dac => 0x71,
            Device::Pro => 0x72,
            Device::ProSe => 0x73,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Device::Dac => "ADI-2 DAC",
            Device::Pro => "ADI-2 Pro",
            Device::ProSe => "ADI-2/4 Pro SE",
        }
    }
    /// Le canal 9 (Phones 3/4) n'existe pas sur l'ADI-2 DAC de base.
    pub fn has_phones34(self) -> bool {
        !matches!(self, Device::Dac)
    }
}

/// Une valeur décodée : (adresse, index, valeur).
pub type Triplet = (u8, u8, i32);

/// Requête de statut complet.
pub fn status_request(dev: Device) -> Vec<u8> {
    let mut m = HEADER.to_vec();
    m.extend([dev.id(), 0x03, 0x09, SYSEX_END]);
    m
}

/// Commande « set » pour `(adresse, index, valeur)`. `None` si adresse/valeur non supportée.
pub fn set_command(dev: Device, addr: u8, index: u8, value: i32) -> Option<Vec<u8>> {
    let body = match addr {
        ADDR_DEVICE => encode_device(index, value),
        3 | 6 | 9 => encode_channel(addr, index, value)?,
        a if EQ_ADDRESSES.contains(&a) => encode_eq(addr, index, value)?,
        _ => return None,
    };
    let mut m = HEADER.to_vec();
    m.extend([dev.id(), 0x02]);
    m.extend(body);
    m.push(SYSEX_END);
    Some(m)
}

fn encode_channel(addr: u8, index: u8, value: i32) -> Option<[u8; 3]> {
    if addr > 15 || index > 31 || !(-2048..=2047).contains(&value) {
        return None;
    }
    let v = value as i16 as u16; // complément à 2 sur 12 bits, masqué ci-dessous
    Some([
        (addr << 3) | (index >> 2),
        ((index & 0x03) << 5) | ((v >> 7) & 0x1F) as u8,
        (v & 0x7F) as u8,
    ])
}

fn encode_device(index: u8, value: i32) -> [u8; 3] {
    [
        (ADDR_DEVICE << 3) | ((index >> 3) & 0x7),
        ((index & 0x7) << 4) | ((value >> 3) & 0xF) as u8,
        (value & 0x7F) as u8,
    ]
}

/// Décode une trame entrante. Retourne les triplets si c'est un statut (cmd 1) pour `dev`.
pub fn parse_incoming(dev: Device, msg: &[u8]) -> Option<Vec<Triplet>> {
    let inner = msg.strip_prefix(&HEADER)?.strip_suffix(&[SYSEX_END])?;
    let (&id, rest) = inner.split_first()?;
    let (&cmd, payload) = rest.split_first()?;
    if id != dev.id() {
        return None;
    }
    match cmd {
        1 => Some(parse_status(payload)),
        7 => Some(parse_device_status(payload)),
        _ => None,
    }
}

/// Message de statut (cmd 7), numéros d'octets du tableau RME moins 5 :
/// [0] sorties actives/révision, [1..9] IOStatus (non documenté), [9] mode, [10] révision protocole.
/// Exposé comme paramètres virtuels de l'adresse `ADDR_STATUS` (cf. `params::STATUS`).
pub fn parse_device_status(p: &[u8]) -> Vec<Triplet> {
    if p.len() < 11 {
        return vec![];
    }
    let v = |i: u8, x: u8| (ADDR_STATUS, i, x as i32);
    vec![
        v(1, p[0] & 3),
        v(2, (p[0] >> 2) & 3),
        v(3, (p[0] >> 4) & 3),
        v(4, (p[0] >> 6) & 1),
        v(5, p[9] & 7),
        v(6, (p[9] >> 3) & 1),
        v(7, (p[9] >> 4) & 7),
        v(8, p[10]),
    ]
}

pub fn parse_status(payload: &[u8]) -> Vec<Triplet> {
    payload
        .chunks_exact(3)
        .filter_map(|c| {
            let addr = (c[0] >> 3) & 0x0F;
            let generic_index = ((c[0] & 0x07) << 3) | ((c[1] >> 4) & 0x07);
            if generic_index == 0 {
                return None; // le Go ignore aussi ce cas
            }
            match addr {
                3 | 6 | 9 => Some(decode_channel(c)),
                ADDR_DEVICE => {
                    // Décodage repris à l'identique du Go (parseParameterBytes).
                    let v = (((c[1] & 0x0F) as i32) << 4) | c[2] as i32;
                    Some((addr, generic_index, v))
                }
                a if EQ_ADDRESSES.contains(&a) => Some(decode_eq(c)),
                _ => None,
            }
        })
        .collect()
}

fn decode_channel(c: &[u8]) -> Triplet {
    let addr = (c[0] >> 3) & 0x0F;
    let index = ((c[0] & 0x07) << 2) | ((c[1] >> 5) & 0x03);
    let mut v = (((c[1] & 0x1F) as i32) << 7) | c[2] as i32;
    if v > 2047 {
        v -= 4096;
    }
    (addr, index, v)
}

fn decode_eq(c: &[u8]) -> Triplet {
    let addr = (c[0] >> 3) & 0x0F;
    let index = ((c[0] & 0x07) << 2) | ((c[1] >> 5) & 0x03);
    if is_eq_freq(index) {
        let mut v = (((c[1] & 0x0F) as i32) << 7) | c[2] as i32;
        if c[1] & 0x10 != 0 {
            v *= 10;
        }
        (addr, index, v)
    } else {
        // Gains (demi-dB), Q (dixièmes), types : 12 bits signés, bit 4 = bit de poids fort.
        let mut v = (((c[1] & 0x1F) as i32) << 7) | c[2] as i32;
        if v > 2047 {
            v -= 4096;
        }
        (addr, index, v)
    }
}

fn encode_eq(addr: u8, index: u8, value: i32) -> Option<[u8; 3]> {
    if index > 31 {
        return None;
    }
    let (flag10, raw) = if is_eq_freq(index) {
        if !(0..=20470).contains(&value) {
            return None;
        }
        if value > 2047 { (0x10u8, value / 10) } else { (0, value) }
    } else {
        if !(-2048..=2047).contains(&value) {
            return None;
        }
        (0, value & 0xFFF)
    };
    let hi = if is_eq_freq(index) { ((raw >> 7) & 0x0F) as u8 } else { ((raw >> 7) & 0x1F) as u8 };
    Some([(addr << 3) | (index >> 2), ((index & 0x03) << 5) | flag10 | hi, (raw & 0x7F) as u8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_roundtrip() {
        for v in [-1145, -100, 0, 60, 2047, -2048] {
            let b = encode_channel(3, 12, v).unwrap();
            assert_eq!(decode_channel(&b), (3, 12, v), "v={v}");
        }
    }

    #[test]
    fn eq_official_frames() {
        // Trames d'exemple RME (ADI-2/4 Pro SE), adresse 1 = Input EQ gauche.
        assert_eq!(decode_eq(&[0x09, 0x20, 0x64]), (1, 5, 100)); // Band 1 Freq
        assert_eq!(decode_eq(&[0x09, 0x40, 0x63]), (1, 6, 99)); // Band 1 Q = 9.9
        assert_eq!(decode_eq(&[0x0A, 0x03, 0x74]), (1, 8, 500)); // Band 2 Freq
    }

    #[test]
    fn eq_roundtrip() {
        for (idx, v) in [(21, -12), (21, 12), (22, 85), (25, 6500), (25, 3000), (23, 9), (24, -12)] {
            let b = encode_eq(4, idx, v).unwrap();
            assert_eq!(decode_eq(&b), (4, idx, v), "idx={idx} v={v}");
        }
    }

    #[test]
    fn device_status_official_frame() {
        // Statut d'exemple RME : 11 30 7E 08 00 00 00 02 04 01 11 00
        let f = [0xF0, 0, 0x20, 0x0D, 0x73, 7, 0x11, 0x30, 0x7E, 8, 0, 0, 0, 2, 4, 1, 0x11, 0, 0xF7];
        let t = parse_incoming(Device::ProSe, &f).unwrap();
        assert!(t.contains(&(ADDR_STATUS, 1, 1))); // sortie active
        assert!(t.contains(&(ADDR_STATUS, 5, 1))); // mode
        assert!(t.contains(&(ADDR_STATUS, 8, 0x11))); // révision protocole (octet brut)
    }

    #[test]
    fn set_volume_frame() {
        let m = set_command(Device::Dac, 3, 12, -100).unwrap();
        assert_eq!(&m[..6], &[0xF0, 0, 0x20, 0x0D, 0x71, 0x02]);
        assert_eq!(*m.last().unwrap(), 0xF7);
        assert_eq!(m.len(), 10);
    }

    #[test]
    fn status_request_bytes() {
        assert_eq!(
            status_request(Device::Pro),
            [0xF0, 0, 0x20, 0x0D, 0x72, 0x03, 0x09, 0xF7]
        );
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(set_command(Device::Dac, 3, 12, 5000).is_none());
        assert!(set_command(Device::Dac, 13, 1, 0).is_none());
    }

    #[test]
    fn parse_wrong_device() {
        let f = [0xF0, 0, 0x20, 0x0D, 0x72, 1, 0xF7];
        assert!(parse_incoming(Device::Dac, &f).is_none());
        assert!(parse_incoming(Device::Pro, &f).is_some());
    }
}
