//! Internacionalização com Mozilla Fluent (mesma abordagem do MariPDF, ADR-004):
//! pt-BR, inglês e espanhol embutidos no binário, troca de idioma em tempo real.
//!
//! Textos ficam em `locales/<idioma>/main.ftl` e são lidos com `t!("chave")` ou
//! `t!("chave", nome => valor)`.

use fluent_bundle::{FluentArgs, FluentResource, FluentValue, concurrent::FluentBundle};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use unic_langid::langid;

pub mod menu {
    include!("../shared/menu_i18n.rs");
}

/// Idioma escolhido em Ajustes. `System` segue o idioma do Windows.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Language {
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "pt-BR")]
    PtBr,
    #[serde(rename = "en")]
    En,
    #[serde(rename = "es")]
    Es,
}

impl Language {
    pub const ALL: [Language; 4] = [Language::System, Language::PtBr, Language::En, Language::Es];

    /// Nome mostrado no seletor (cada idioma no próprio idioma).
    pub fn label(self) -> String {
        match self {
            Language::System => get_args("lang-system", &[]),
            Language::PtBr => "Português (Brasil)".into(),
            Language::En => "English".into(),
            Language::Es => "Español".into(),
        }
    }

    fn code(self) -> &'static str {
        match self {
            Language::System => "system",
            Language::PtBr => "pt-BR",
            Language::En => "en",
            Language::Es => "es",
        }
    }

    /// Índice em `menu::LANG_CODES` (resolve "Sistema" pelo idioma do Windows).
    pub fn index(self) -> usize {
        menu::lang_index(self.code(), windows_ui_langid())
    }
}

/// Idioma da interface do Windows (LANGID).
pub fn windows_ui_langid() -> u16 {
    unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() }
}

static BUNDLE: OnceLock<Mutex<FluentBundle<FluentResource>>> = OnceLock::new();
static CURRENT: Mutex<usize> = Mutex::new(0);

const FTL: [&str; 3] = [
    include_str!("../locales/pt-BR/main.ftl"),
    include_str!("../locales/en/main.ftl"),
    include_str!("../locales/es/main.ftl"),
];

fn build_bundle(index: usize) -> FluentBundle<FluentResource> {
    let locale = match index {
        0 => langid!("pt-BR"),
        2 => langid!("es"),
        _ => langid!("en-US"),
    };
    let resource = FluentResource::try_new(FTL[index].to_owned()).expect("FTL inválido — verifique locales/");
    let mut bundle = FluentBundle::new_concurrent(vec![locale]);
    // Sem os caracteres FSI/PDI ao redor dos argumentos: o egui os desenharia como caixas.
    bundle.set_use_isolating(false);
    bundle.add_resource(resource).expect("chave duplicada no FTL");
    bundle
}

/// Ativa um idioma (pode ser chamado de novo para trocar em tempo real).
pub fn init(lang: Language) {
    let index = lang.index();
    *CURRENT.lock().unwrap() = index;
    let bundle = build_bundle(index);
    match BUNDLE.get() {
        Some(mtx) => *mtx.lock().unwrap() = bundle,
        None => {
            let _ = BUNDLE.set(Mutex::new(bundle));
        }
    }
}

/// Índice do idioma ativo em `menu::LANG_CODES`.
pub fn current_index() -> usize {
    *CURRENT.lock().unwrap()
}

pub fn get_args(key: &str, values: &[(&str, String)]) -> String {
    let Some(mtx) = BUNDLE.get() else { return key.to_string() };
    let bundle = mtx.lock().unwrap();
    let Some(pattern) = bundle.get_message(key).and_then(|m| m.value()) else { return key.to_string() };
    let mut args = FluentArgs::new();
    for (name, value) in values {
        // Números entram como números, para o Fluent escolher o plural certo.
        match value.parse::<f64>() {
            Ok(n) => args.set(*name, FluentValue::from(n)),
            Err(_) => args.set(*name, FluentValue::from(value.clone())),
        }
    }
    let mut errors = vec![];
    bundle.format_pattern(pattern, if values.is_empty() { None } else { Some(&args) }, &mut errors).to_string()
}

/// `t!("chave")` ou `t!("chave", nome => valor, ...)` → texto no idioma ativo.
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::get_args($key, &[])
    };
    ($key:expr, $($arg:ident => $val:expr),+ $(,)?) => {
        $crate::i18n::get_args($key, &[$((stringify!($arg), ($val).to_string())),+])
    };
}

#[cfg(test)]
mod tests {
    use super::FTL;
    use std::collections::BTreeSet;

    fn keys(ftl: &str) -> BTreeSet<&str> {
        ftl.lines()
            .filter(|line| line.starts_with(|c: char| c.is_ascii_lowercase()))
            .filter_map(|line| line.split_once('=').map(|(key, _)| key.trim()))
            .collect()
    }

    #[test]
    fn locales_expose_the_same_keys() {
        let pt = keys(FTL[0]);
        for (name, ftl) in [("en", FTL[1]), ("es", FTL[2])] {
            let other = keys(ftl);
            let missing: Vec<_> = pt.difference(&other).collect();
            let extra: Vec<_> = other.difference(&pt).collect();
            assert!(missing.is_empty() && extra.is_empty(), "{name}: faltando {missing:?}; sobrando {extra:?}");
        }
    }

    #[test]
    fn windows_language_picks_the_right_locale() {
        use super::menu::lang_index;
        assert_eq!(lang_index("system", 0x0416), 0); // pt-BR
        assert_eq!(lang_index("system", 0x0816), 0); // pt-PT
        assert_eq!(lang_index("system", 0x0c0a), 2); // es-ES
        assert_eq!(lang_index("system", 0x080a), 2); // es-MX
        assert_eq!(lang_index("system", 0x0409), 1); // en-US
        assert_eq!(lang_index("system", 0x0407), 1); // alemão → inglês
        assert_eq!(lang_index("es", 0x0416), 2); // escolha manual vence
    }

    #[test]
    fn every_ftl_parses() {
        for index in 0..FTL.len() {
            let _ = super::build_bundle(index);
        }
    }

    #[test]
    fn every_key_used_in_code_exists() {
        let defined = keys(FTL[0]);
        let mut missing = Vec::new();
        for (path, source) in rust_sources() {
            for key in literal_keys(&source) {
                if !defined.contains(key.as_str()) {
                    missing.push(format!("{path}: {key}"));
                }
            }
        }
        assert!(missing.is_empty(), "chaves sem tradução: {missing:#?}");
    }

    /// Texto visível deve passar pelo Fluent: pega literais com palavras em widgets.
    #[test]
    fn ui_text_is_not_hardcoded() {
        const SINKS: [&str; 8] = [
            "RichText::new(\"",
            "on_hover_text(\"",
            "hint_text(\"",
            "set_title(\"",
            "ui.label(\"",
            "ui.button(\"",
            "hint(ui, \"",
            "section(ui, \"",
        ];
        const ALLOWED: [&str; 4] = ["Mari Encoder", "Flávio Takemoto", "English", "Español"];
        let mut offenders = Vec::new();
        for (path, source) in rust_sources() {
            for (number, line) in source.lines().enumerate() {
                for sink in SINKS {
                    let Some(start) = line.find(sink) else { continue };
                    let literal = line[start + sink.len()..].split('"').next().unwrap_or_default();
                    let has_word = literal.split(|c: char| !c.is_alphabetic()).any(|w| w.chars().count() >= 3);
                    if has_word && !ALLOWED.contains(&literal) {
                        offenders.push(format!("{path}:{}: {literal}", number + 1));
                    }
                }
            }
        }
        assert!(offenders.is_empty(), "texto fixo na UI: {offenders:#?}");
    }

    fn rust_sources() -> Vec<(String, String)> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        std::fs::read_dir(dir)
            .expect("ler src/")
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "rs") && !p.ends_with("i18n.rs"))
            .map(|p| (p.display().to_string(), std::fs::read_to_string(&p).expect("ler fonte")))
            .collect()
    }

    /// Chaves literais em `t!("...")`.
    fn literal_keys(source: &str) -> Vec<String> {
        source
            .match_indices("t!(\"")
            .filter_map(|(index, mat)| {
                let key = source[index + mat.len()..].split('"').next()?;
                key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-').then(|| key.to_string())
            })
            .collect()
    }
}
