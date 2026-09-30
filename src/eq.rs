//! Courbe de réponse de l'EQ paramétrique, calculée localement (le DAC ne l'envoie pas).
//! Filtres bi-quad « Audio EQ Cookbook » (RBJ) ; l'appareil peut différer légèrement.

const FS: f64 = 48_000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Peak,
    LowShelf,
    HighShelf,
    HighPass,
    LowPass,
}

#[derive(Debug, Clone, Copy)]
pub struct Band {
    pub kind: Kind,
    pub gain_db: f64,
    pub freq: f64,
    pub q: f64,
}

impl Band {
    /// Réponse en amplitude (dB) à la fréquence `f`.
    pub fn response_db(&self, f: f64) -> f64 {
        let a = 10f64.powf(self.gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * self.freq / FS;
        let (sn, cs) = w0.sin_cos();
        let alpha = sn / (2.0 * self.q);
        let sa = 2.0 * a.sqrt() * alpha;
        let (b0, b1, b2, a0, a1, a2) = match self.kind {
            Kind::Peak => (1.0 + alpha * a, -2.0 * cs, 1.0 - alpha * a, 1.0 + alpha / a, -2.0 * cs, 1.0 - alpha / a),
            Kind::LowShelf => (
                a * ((a + 1.0) - (a - 1.0) * cs + sa),
                2.0 * a * ((a - 1.0) - (a + 1.0) * cs),
                a * ((a + 1.0) - (a - 1.0) * cs - sa),
                (a + 1.0) + (a - 1.0) * cs + sa,
                -2.0 * ((a - 1.0) + (a + 1.0) * cs),
                (a + 1.0) + (a - 1.0) * cs - sa,
            ),
            Kind::HighShelf => (
                a * ((a + 1.0) + (a - 1.0) * cs + sa),
                -2.0 * a * ((a - 1.0) + (a + 1.0) * cs),
                a * ((a + 1.0) + (a - 1.0) * cs - sa),
                (a + 1.0) - (a - 1.0) * cs + sa,
                2.0 * ((a - 1.0) - (a + 1.0) * cs),
                (a + 1.0) - (a - 1.0) * cs - sa,
            ),
            Kind::LowPass => ((1.0 - cs) / 2.0, 1.0 - cs, (1.0 - cs) / 2.0, 1.0 + alpha, -2.0 * cs, 1.0 - alpha),
            Kind::HighPass => ((1.0 + cs) / 2.0, -(1.0 + cs), (1.0 + cs) / 2.0, 1.0 + alpha, -2.0 * cs, 1.0 - alpha),
        };
        let w = 2.0 * std::f64::consts::PI * f / FS;
        let mag = |c0: f64, c1: f64, c2: f64| {
            let re = c0 + c1 * w.cos() + c2 * (2.0 * w).cos();
            let im = -(c1 * w.sin() + c2 * (2.0 * w).sin());
            (re * re + im * im).sqrt()
        };
        20.0 * (mag(b0, b1, b2) / mag(a0, a1, a2)).log10()
    }
}

/// Construit les bandes actives depuis les valeurs brutes. `eq` : bandes (adresse choisie),
/// `bt` : Bass/Treble (toujours lus sur l'adresse gauche), `None` si B/T désactivé.
pub fn bands(eq: &dyn Fn(u8) -> Option<i32>, bt: Option<&dyn Fn(u8) -> Option<i32>>) -> Vec<Band> {
    let mut v = Vec::new();
    // (index du type, index gain) ; les bandes 2 à 4 n'ont pas de type (Peak fixe).
    let layout: [(Option<u8>, u8); 5] = [(Some(3), 4), (None, 7), (None, 10), (None, 13), (Some(16), 17)];
    for (n, (type_idx, g)) in layout.into_iter().enumerate() {
        let (Some(gain), Some(freq), Some(q)) = (eq(g), eq(g + 1), eq(g + 2)) else { continue };
        let t = type_idx.and_then(eq).unwrap_or(0);
        let kind = match (n, t) {
            (0, 1) => Kind::LowShelf,
            (0, 2) => Kind::HighPass,
            (0, 3) => Kind::LowPass,
            (4, 1) => Kind::HighShelf,
            (4, 2) => Kind::LowPass,
            _ => Kind::Peak,
        };
        let is_cut = matches!(kind, Kind::HighPass | Kind::LowPass);
        if gain != 0 || is_cut {
            v.push(Band { kind, gain_db: gain as f64 / 2.0, freq: freq as f64, q: q as f64 / 10.0 });
        }
    }
    if let Some(bt) = bt {
        if let (Some(g), Some(f), Some(q)) = (bt(21), bt(22), bt(23)) {
            if g != 0 {
                v.push(Band { kind: Kind::LowShelf, gain_db: g as f64 / 2.0, freq: f as f64, q: q as f64 / 10.0 });
            }
        }
        if let (Some(g), Some(f), Some(q)) = (bt(24), bt(25), bt(26)) {
            if g != 0 {
                v.push(Band { kind: Kind::HighShelf, gain_db: g as f64 / 2.0, freq: f as f64, q: q as f64 / 10.0 });
            }
        }
    }
    v
}

/// Points (log10 fréquence, dB) de 20 Hz à 20 kHz.
pub fn curve(bands: &[Band]) -> Vec<(f64, f64)> {
    const N: usize = 160;
    (0..N)
        .map(|i| {
            let lf = 20f64.log10() + (20000f64.log10() - 20f64.log10()) * i as f64 / (N - 1) as f64;
            let f = 10f64.powf(lf);
            (lf, bands.iter().map(|b| b.response_db(f)).sum())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn b(kind: Kind, gain_db: f64, freq: f64, q: f64) -> Band {
        Band { kind, gain_db, freq, q }
    }
    #[test]
    fn peak_gain_at_center() {
        assert!((b(Kind::Peak, 6.0, 1000.0, 1.0).response_db(1000.0) - 6.0).abs() < 0.05);
        assert!(b(Kind::Peak, 6.0, 1000.0, 1.0).response_db(20.0).abs() < 0.2);
    }
    #[test]
    fn shelves() {
        assert!((b(Kind::LowShelf, 6.0, 100.0, 0.7).response_db(20.0) - 6.0).abs() < 0.3);
        assert!(b(Kind::LowShelf, 6.0, 100.0, 0.7).response_db(10000.0).abs() < 0.3);
        assert!((b(Kind::HighShelf, -4.0, 5000.0, 0.7).response_db(18000.0) + 4.0).abs() < 0.5);
    }
    #[test]
    fn cut_is_minus_3db_at_corner() {
        let r = b(Kind::HighPass, 0.0, 200.0, 0.7071).response_db(200.0);
        assert!((r + 3.0).abs() < 0.2, "{r}");
    }
    #[test]
    fn flat_when_no_band() {
        assert!(curve(&[]).iter().all(|&(_, d)| d == 0.0));
    }
}
