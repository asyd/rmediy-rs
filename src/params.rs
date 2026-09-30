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
    p(1, "Sortie active (DAC)", 0, 3, 0, 0),
    p(2, "Révision appareil", 0, 3, 0, 0),
    p(3, "Sortie ligne active", 0, 3, 0, 0),
    p(4, "Phones 3/4 actif", 0, 1, 0, 0),
    p(5, "Mode courant", 0, 7, 0, 0),
    p(6, "Mode balancé", 0, 1, 0, 0),
    p(7, "Décodage DSD", 0, 7, 0, 0),
    p(8, "Révision protocole (brut)", 0, 127, 0, 0),
];

/// Courte description d'un paramètre (source : manuel ADI-2 DAC v1.8 et tableau MIDI RME).
pub fn describe(addr: u8, name: &str) -> &'static str {
    match (addr, name) {
        (3 | 6 | 9, "Source") => "Signal envoyé aux sorties analogiques. En Auto, un signal SPDIF détecté a priorité sur l'USB. Les choix dépendent du modèle.",
        (3 | 6 | 9, "Ref Level") => "Niveau de sortie pour 0 dBFS. Line (DAC) : -5/+1/+7/+13 dBu. Phones : Lo-Power ou Hi-Power (+15 dB).",
        (3 | 6 | 9, "Auto Ref Level") => "Garde une échelle de volume continue en dB quand le Ref Level change. Recommandé avec le Loudness.",
        (3 | 6 | 9, "Mono") => "OFF, ON, ou « to Left » : la somme L+R n'est envoyée qu'à gauche.",
        (3 | 6 | 9, "Width") => "Largeur stéréo : 1,00 = stéréo complet, 0 = mono, -1,00 = canaux inversés.",
        (3 | 6 | 9, "M/S-Proc") => "Traitement Mid/Side : le contenu mono va à gauche, le stéréo à droite.",
        (3 | 6 | 9, "Polarity") => "Inverse la polarité du signal : Both, Left ou Right.",
        (3 | 6 | 9, "Crossfeed") => "Crossfeed Bauer pour casque : de 1 (discret, -13 dB) à 5 (fort, -3 dB). 0 = désactivé.",
        (3 | 6 | 9, "DA Filter") => "Filtre de suréchantillonnage du DAC. SD Sharp (défaut) : réponse la plus linéaire, latence minimale. NOS : le plus doux, meilleure réponse impulsionnelle, désactive De-Emphasis.",
        (3 | 6 | 9, "De-Emphasis") => "Filtre de dé-emphase du DAC : Auto (défaut), ON ou OFF. Sans effet avec le filtre NOS.",
        (3 | 6 | 9, "Dual EQ") => "Permet de régler l'EQ 5 bandes séparément pour la gauche et la droite.",
        (3 | 6 | 9, "Volume") => "Volume de la sortie, par pas de 0,5 dB (de -96 à +6 dB sur l'appareil).",
        (3 | 6 | 9, "Lock Volume") => "Verrouille le gros bouton de volume. Le volume reste réglable dans les menus.",
        (3 | 6 | 9, "Balance") => "Balance gauche/droite : -1,00 = tout à gauche, +1,00 = tout à droite.",
        (3 | 6 | 9, "Mute") => "Coupe la sortie. Monter le volume désactive le mute immédiatement.",
        (3 | 6 | 9, "Dim") => "Baisse le volume de 20 dB. Monter le volume désactive Dim immédiatement.",
        (3 | 6 | 9, "Loopback to USB") => "Renvoie le signal de la sortie vers l'USB pour l'enregistrer, avant (pre FX) ou après (post FX) traitement.",
        (3 | 6 | 9, "Dig. DC Protection") => "Protection contre les composantes continues du signal numérique (OFF, ON, Filter). Non détaillé dans le manuel.",
        (3 | 6 | 9, "Rear TRS Source") => "Source de la sortie TRS arrière : Line 1/2 ou Phones 3/4 (ADI-2/4 uniquement).",
        (3 | 6 | 9, "Loudness Enable") => "Active le Loudness : ajoute des basses et des aiguës quand le volume est bas.",
        (3 | 6 | 9, "Bass Gain") => "Loudness : gain maximal des basses, atteint sous Low Vol Ref (1 à 10 dB, défaut 7 dB).",
        (3 | 6 | 9, "Treble Gain") => "Loudness : gain maximal des aiguës, atteint sous Low Vol Ref (1 à 10 dB, défaut 7 dB).",
        (3 | 6 | 9, "Low Vol Ref") => "Loudness : volume sous lequel le gain est maximal. Il est nul 20 dB au-dessus. Unité à confirmer sur l'appareil.",

        (4 | 7 | 10, "B/T Enable") => "Active les réglages Bass/Treble de cette sortie.",
        (4 | 7 | 10, "Bass Gain") => "Gain ou atténuation des basses, par pas de 0,5 dB (±6 dB sur le DAC).",
        (4 | 7 | 10, "Bass Freq") => "Fréquence du filtre de basses, de 20 à 150 Hz (défaut 85 Hz).",
        (4 | 7 | 10, "Bass Q") => "Facteur de qualité du filtre de basses, de 0,5 à 1,5 (défaut 0,9).",
        (4 | 7 | 10, "Treble Gain") => "Gain ou atténuation des aiguës, par pas de 0,5 dB (±6 dB sur le DAC).",
        (4 | 7 | 10, "Treble Freq") => "Fréquence du filtre d'aiguës, de 3 à 10 kHz (défaut 6,5 kHz).",
        (4 | 7 | 10, "Treble Q") => "Facteur de qualité du filtre d'aiguës, de 0,5 à 1,5 (défaut 0,7).",

        (4 | 7 | 10, "EQ Enable") => "Active l'EQ paramétrique 5 bandes de cette sortie (courbe grisée quand il est désactivé).",
        (4 | 7 | 10, "Load B/T w. Preset") => "Charge aussi les réglages Bass/Treble quand on charge un preset EQ.",
        (4 | 5 | 7 | 8 | 10 | 11, n) if n.ends_with("Type") => "Type de filtre. Bande 1 : Peak, Shelf, Hi Pass ou Hi Cut. Bande 5 : Peak, Shelf ou Hi Cut. Les bandes 2 à 4 sont fixées en Peak.",
        (4 | 5 | 7 | 8 | 10 | 11, n) if n.starts_with("Band") && n.ends_with("Gain") => "Gain de la bande, de -12 à +12 dB par pas de 0,5 dB. Sans effet sur les filtres Hi Pass et Hi Cut.",
        (4 | 5 | 7 | 8 | 10 | 11, n) if n.starts_with("Band") && n.ends_with("Freq") => "Fréquence centrale (ou de coupure) de la bande. Bandes 1 à 3 : 20 Hz à 20 kHz. Bandes 4 et 5 : 200 Hz à 20 kHz. Les pas sont proportionnels.",
        (4 | 5 | 7 | 8 | 10 | 11, n) if n.starts_with("Band") && n.ends_with('Q') => "Facteur de qualité : plus il est élevé, plus la bande est étroite. Bandes 1 à 3 : 0,5 à 9,9. Bandes 4 et 5 : 0,5 à 5,0.",
        (ADDR_DEVICE, "Mute Line vs.") => "Vs. Phones (défaut) : brancher un casque coupe la sortie Line. Toggle : bascule manuelle avant/arrière par appui long sur Volume.",
        (ADDR_DEVICE, "Auto Standby") => "Mise en veille automatique de l'appareil après une période d'inactivité.",
        (ADDR_DEVICE, "DSD Detection") => "Détection automatique du DSD sur SPDIF et USB (défaut : ON).",
        (ADDR_DEVICE, "DSD Filter") => "Filtre de bruit haute fréquence du mode DSD Direct : 50 kHz pour DSD64, 150 kHz pour DSD128 et 256.",
        (ADDR_DEVICE, "DSD Direct (Line)") => "Lit le DSD sans conversion PCM, sur les sorties arrière. Plus de volume ni de DSP, seul le Ref Level change le niveau. Phones et IEM sont désactivés.",
        (ADDR_DEVICE, "Clock Source") => "Source d'horloge courante, choisie automatiquement par l'appareil.",
        (ADDR_DEVICE, "Sample Rate") => "Fréquence d'échantillonnage courante, en lecture seule.",
        (ADDR_DEVICE, "Remap Keys") => "OFF, ON ou IR-Remote : réassigne les 4 touches de fonction (unité et/ou télécommande).",
        (ADDR_DEVICE, "Display Mode") => "Dark inverse l'affichage : fond noir, texte gris clair.",
        (ADDR_DEVICE, "Meter Color") => "Couleur de l'écran des vumètres en PCM et en DSD.",
        (ADDR_DEVICE, "Hor. Meter") => "Vumètre horizontal sous l'analyseur : Post-FX (après traitement), Pre-FX (avant), ou Dual (les deux).",
        (ADDR_DEVICE, "AutoDark Mode") => "Éteint LEDs et écran après 10 s d'inactivité (3 s avec la télécommande).",
        (ADDR_DEVICE, "Show Vol. Screen") => "Affiche l'écran de volume quand on tourne le bouton Volume.",
        (ADDR_DEVICE, "Lock UI") => "Verrouille l'interface : OFF, Remote, Keys, ou Keys+Rem.",
        (ADDR_STATUS, _) => "Valeur brute du message de statut. La correspondance avec des libellés n'est pas documentée par RME.",
        _ => "",
    }
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
