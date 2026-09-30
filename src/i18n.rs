//! Textes de l'interface : deux fichiers de langue (`locales/*.yml`) intégrés au binaire.
//! Langue : `language:` du config.yaml, sinon `LC_ALL` / `LC_MESSAGES` / `LANG`, sinon anglais.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

type Table = HashMap<String, String>;

static EN: OnceLock<Table> = OnceLock::new();
static FR: OnceLock<Table> = OnceLock::new();
static FRENCH: AtomicBool = AtomicBool::new(false);

fn load(src: &str) -> Table {
    serde_yaml::from_str(src).expect("fichier de langue invalide")
}
fn en() -> &'static Table {
    EN.get_or_init(|| load(include_str!("../locales/en.yml")))
}
fn fr() -> &'static Table {
    FR.get_or_init(|| load(include_str!("../locales/fr.yml")))
}

/// Choisit la langue. `explicit` (ex. `"fr"`) l'emporte sur l'environnement.
pub fn init(explicit: Option<&str>) {
    let lang = explicit.map(str::to_owned).or_else(|| {
        ["LC_ALL", "LC_MESSAGES", "LANG"].iter().find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
    });
    FRENCH.store(lang.is_some_and(|l| l.to_lowercase().starts_with("fr")), Ordering::Relaxed);
}

/// Texte traduit ; repli sur l'anglais, puis sur la clé elle-même. La chaîne vide reste vide.
pub fn t(key: &'static str) -> &'static str {
    if key.is_empty() {
        return "";
    }
    let own = if FRENCH.load(Ordering::Relaxed) { fr().get(key) } else { None };
    own.or_else(|| en().get(key)).map(String::as_str).unwrap_or(key)
}

/// Comme `t`, en remplaçant `{}` par `arg`.
pub fn tf(key: &'static str, arg: impl std::fmt::Display) -> String {
    t(key).replace("{}", &arg.to_string())
}

/// Libellé d'une info de statut (adresse virtuelle `ADDR_STATUS`).
pub fn status_label(index: u8) -> &'static str {
    match index {
        1 => t("status.1"),
        2 => t("status.2"),
        3 => t("status.3"),
        4 => t("status.4"),
        5 => t("status.5"),
        6 => t("status.6"),
        7 => t("status.7"),
        8 => t("status.8"),
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn languages_have_the_same_keys() {
        let mut a: Vec<_> = en().keys().collect();
        let mut b: Vec<_> = fr().keys().collect();
        a.sort();
        b.sort();
        assert_eq!(a, b);
        assert!(en().values().chain(fr().values()).all(|v| !v.is_empty()));
    }

    #[test]
    fn status_labels_exist() {
        for i in 1..=8 {
            assert_ne!(status_label(i), "?");
            assert!(en().contains_key(&format!("status.{i}")));
        }
    }

    #[test]
    fn french_overrides_english() {
        init(Some("fr_FR.UTF-8"));
        let fr_text = t("ui.description");
        let fr_help = t("ui.help");
        init(Some("en_US.UTF-8"));
        assert_ne!(fr_help, t("ui.help"));
        assert_eq!(fr_text, "Description");
    }

    #[test]
    fn every_key_used_in_code_exists() {
        for (file, src) in [
            ("params.rs", include_str!("params.rs")),
            ("tui.rs", include_str!("tui.rs")),
            ("midi.rs", include_str!("midi.rs")),
            ("config.rs", include_str!("config.rs")),
            ("protocol.rs", include_str!("protocol.rs")),
        ] {
            for prefix in ["\"desc.", "\"ui.", "\"err.", "\"midi."] {
                for part in src.split(prefix).skip(1) {
                    let key = format!("{}{}", &prefix[1..], part.split('"').next().unwrap());
                    assert!(en().contains_key(&key), "clé manquante dans {file}: {key}");
                }
            }
        }
    }
}
