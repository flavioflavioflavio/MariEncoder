//! Configuração persistida em %APPDATA%\MariEncoder\config.json: presets e ajustes.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

macro_rules! labeled_enum {
    ($(#[$m:meta])* $name:ident { $($var:ident => $label:expr),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($var),+ }
        impl $name {
            /// Nome mostrado na interface. Começando com "@", é uma chave do Fluent.
            pub fn label(self) -> String {
                let label: &str = match self { $(Self::$var => $label),+ };
                match label.strip_prefix('@') {
                    Some(key) => $crate::i18n::get_args(key, &[]),
                    None => label.to_string(),
                }
            }
        }
    };
}

labeled_enum!(Container {
    Mp4 => "MP4",
    Mov => "MOV",
    Mkv => "MKV",
    Webm => "WebM",
    Gif => "@container-gif",
    Mp3 => "@container-mp3",
    M4a => "@container-m4a",
    Wav => "@container-wav",
});

labeled_enum!(VideoCodec {
    H264 => "H.264",
    H265 => "H.265 / HEVC",
    Av1 => "AV1",
    Vp9 => "VP9",
    Prores => "ProRes 422 HQ",
    Copy => "@codec-copy",
});

labeled_enum!(Hw {
    Cpu => "@hw-windows",
    Nvidia => "@hw-nvidia",
    Intel => "@hw-intel",
    Amd => "@hw-amd",
});

labeled_enum!(RateMode {
    Quality => "@rate-quality",
    Bitrate => "@rate-bitrate",
    TargetSize => "@rate-target-size",
});

labeled_enum!(Speed {
    Fast => "@speed-fast",
    Medium => "@speed-medium",
    Slow => "@speed-slow",
});

labeled_enum!(AudioCodec {
    Aac => "AAC",
    Opus => "Opus",
    Mp3 => "MP3",
    Pcm => "@audio-pcm",
    Copy => "@codec-copy",
    None => "@audio-none",
});

impl Container {
    pub const ALL: [Container; 8] = [
        Container::Mp4,
        Container::Mov,
        Container::Mkv,
        Container::Webm,
        Container::Gif,
        Container::Mp3,
        Container::M4a,
        Container::Wav,
    ];

    pub fn ext(self) -> &'static str {
        match self {
            Container::Mp4 => "mp4",
            Container::Mov => "mov",
            Container::Mkv => "mkv",
            Container::Webm => "webm",
            Container::Gif => "gif",
            Container::Mp3 => "mp3",
            Container::M4a => "m4a",
            Container::Wav => "wav",
        }
    }

    pub fn audio_only(self) -> bool {
        matches!(self, Container::Mp3 | Container::M4a | Container::Wav)
    }

    /// Codecs de vídeo que o container aceita (vazio = sem trilha de vídeo configurável).
    pub fn video_codecs(self) -> &'static [VideoCodec] {
        use VideoCodec::*;
        match self {
            Container::Mp4 => &[H264, H265, Av1, Copy],
            Container::Mov => &[H264, H265, Prores, Copy],
            Container::Mkv => &[H264, H265, Av1, Vp9, Prores, Copy],
            Container::Webm => &[Vp9, Av1],
            _ => &[],
        }
    }

    pub fn audio_codecs(self) -> &'static [AudioCodec] {
        use AudioCodec::*;
        match self {
            Container::Mp4 => &[Aac, Mp3, Copy, None],
            Container::Mov => &[Aac, Pcm, Copy, None],
            Container::Mkv => &[Aac, Opus, Mp3, Pcm, Copy, None],
            Container::Webm => &[Opus, None],
            Container::Gif => &[],
            Container::Mp3 => &[Mp3],
            Container::M4a => &[Aac],
            Container::Wav => &[Pcm],
        }
    }
}

impl Hw {
    pub const ALL: [Hw; 4] = [Hw::Cpu, Hw::Nvidia, Hw::Intel, Hw::Amd];
}
impl RateMode {
    pub const ALL: [RateMode; 3] = [RateMode::Quality, RateMode::Bitrate, RateMode::TargetSize];
}
impl Speed {
    pub const ALL: [Speed; 3] = [Speed::Fast, Speed::Medium, Speed::Slow];
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Preset {
    /// Identificador estável usado na linha de comando (`--preset <id>`).
    pub id: String,
    pub name: String,
    pub in_menu: bool,
    pub container: Container,
    pub video: VideoCodec,
    pub hw: Hw,
    pub rate_mode: RateMode,
    /// Valor passado ao encoder no modo Qualidade (menor = melhor).
    pub crf: u32,
    pub video_kbps: u32,
    pub target_mb: u32,
    pub speed: Speed,
    /// Limite do lado menor do quadro (720 = "720p"). 0 = resolução original.
    pub max_short_side: u32,
    /// FPS máximo. 0 = original.
    pub max_fps: u32,
    pub audio: AudioCodec,
    pub audio_kbps: u32,
    /// Sufixo adicionado ao nome do arquivo de saída.
    pub suffix: String,
    /// Vazio = mesma pasta do original; relativo = subpasta; absoluto = pasta fixa.
    pub output_dir: String,
    pub extra_args: String,
}

impl Default for Preset {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: crate::t!("preset-new-name"),
            in_menu: true,
            container: Container::Mp4,
            video: VideoCodec::H264,
            hw: Hw::Cpu,
            rate_mode: RateMode::Quality,
            crf: 23,
            video_kbps: 3000,
            target_mb: 50,
            speed: Speed::Medium,
            max_short_side: 0,
            max_fps: 0,
            audio: AudioCodec::Aac,
            audio_kbps: 128,
            suffix: "_novo".into(),
            output_dir: String::new(),
            extra_args: String::new(),
        }
    }
}

impl Preset {
    /// Corrige combinações que o container não aceita (ex.: WebM com H.264).
    pub fn normalize(&mut self) {
        let vcs = self.container.video_codecs();
        if !vcs.is_empty() && !vcs.contains(&self.video) {
            self.video = vcs[0];
        }
        let acs = self.container.audio_codecs();
        if !acs.is_empty() && !acs.contains(&self.audio) {
            self.audio = acs[0];
        }
    }

    pub fn has_video_settings(&self) -> bool {
        !self.container.audio_only() && self.container != Container::Gif
    }

    /// Resumo curto mostrado nas listas.
    pub fn summary(&self) -> String {
        let mut parts: Vec<String> = vec![self.container.ext().to_uppercase()];
        if self.has_video_settings() {
            parts.push(self.video.label().split(' ').next().unwrap_or("").to_string());
            if self.video != VideoCodec::Copy {
                match self.rate_mode {
                    RateMode::Quality => parts.push(format!("CRF {}", self.crf)),
                    RateMode::Bitrate => parts.push(fmt_kbps(self.video_kbps)),
                    RateMode::TargetSize => parts.push(format!("≈{} MB", self.target_mb)),
                }
            }
        }
        if !self.container.audio_only() && self.max_short_side > 0 {
            parts.push(format!("{}p", self.max_short_side));
        }
        if !self.container.audio_only() && self.max_fps > 0 {
            parts.push(format!("{} fps", self.max_fps));
        }
        if self.container.audio_only() {
            parts.push(format!("{} kbps", self.audio_kbps));
        }
        parts.join(" · ")
    }
}

pub fn fmt_kbps(kbps: u32) -> String {
    if kbps >= 1000 {
        let m = kbps as f32 / 1000.0;
        if (m - m.round()).abs() < 0.05 {
            format!("{} Mbps", m.round() as u32)
        } else {
            format!("{m:.1} Mbps")
        }
    } else {
        format!("{kbps} kbps")
    }
}

/// Nome de fábrica de um preset no idioma ativo (mesma tabela usada pela DLL do menu).
fn factory_name(id: &str) -> String {
    let lang = crate::i18n::current_index();
    crate::i18n::menu::DEFAULT_PRESETS
        .iter()
        .find(|(preset_id, _)| *preset_id == id)
        .map(|(_, names)| names[lang].to_string())
        .unwrap_or_else(|| id.to_string())
}

fn default_presets() -> Vec<Preset> {
    let base = Preset::default();
    vec![
        Preset {
            id: "whatsapp".into(),
            name: factory_name("whatsapp"),
            crf: 27,
            max_short_side: 720,
            max_fps: 30,
            audio_kbps: 96,
            suffix: "_whatsapp".into(),
            ..base.clone()
        },
        Preset {
            id: "3mbps".into(),
            name: factory_name("3mbps"),
            rate_mode: RateMode::Bitrate,
            video_kbps: 3000,
            suffix: "_3mbps".into(),
            ..base.clone()
        },
        Preset {
            id: "mp4".into(),
            name: factory_name("mp4"),
            crf: 21,
            audio_kbps: 160,
            suffix: "_mp4".into(),
            ..base.clone()
        },
        Preset {
            id: "youtube".into(),
            name: factory_name("youtube"),
            crf: 17,
            speed: Speed::Slow,
            audio_kbps: 320,
            suffix: "_youtube".into(),
            ..base.clone()
        },
        Preset {
            id: "hevc".into(),
            name: factory_name("hevc"),
            video: VideoCodec::H265,
            crf: 26,
            suffix: "_hevc".into(),
            ..base.clone()
        },
        Preset {
            id: "gif".into(),
            name: factory_name("gif"),
            container: Container::Gif,
            max_short_side: 480,
            max_fps: 12,
            suffix: "_gif".into(),
            ..base.clone()
        },
        Preset {
            id: "mp3".into(),
            name: factory_name("mp3"),
            container: Container::Mp3,
            audio: AudioCodec::Mp3,
            audio_kbps: 192,
            suffix: String::new(),
            ..base
        },
    ]
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Config {
    pub presets: Vec<Preset>,
    /// Extensões (sem ponto) que recebem o menu contextual.
    pub extensions: Vec<String>,
    /// Pasta com ffmpeg.exe. Vazio = procurar ao lado do app e no PATH.
    pub ffmpeg_dir: String,
    /// Fecha a janela rápida (menu contextual) quando tudo termina sem erro.
    pub close_when_done: bool,
    /// Preset pré-selecionado para arquivos adicionados pela janela principal.
    pub default_preset: String,
    /// Idioma da interface ("system" segue o Windows).
    pub language: crate::i18n::Language,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            presets: default_presets(),
            // Mesma lista que o pacote da Store declara no manifesto (tools/package_msix.ps1).
            extensions: include_str!("../shared/video_extensions.txt").split_whitespace().map(String::from).collect(),
            ffmpeg_dir: String::new(),
            close_when_done: true,
            default_preset: "whatsapp".into(),
            language: crate::i18n::Language::System,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::i18n::menu::DEFAULT_PRESETS;

    /// A DLL do menu mostra `DEFAULT_PRESETS` antes de existir um config.json: os ids
    /// e a ordem precisam bater com os presets que o app cria.
    #[test]
    fn default_menu_matches_default_presets() {
        let presets = super::default_presets();
        let ids: Vec<&str> = presets.iter().map(|p| p.id.as_str()).collect();
        let menu_ids: Vec<&str> = DEFAULT_PRESETS.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, menu_ids);
        assert!(presets.iter().all(|p| p.in_menu));
    }
}

impl Config {
    pub fn dir() -> PathBuf {
        // Aberto pelo menu do Windows 11 o app herda o ambiente de um processo do
        // sistema; se APPDATA faltar, monta o caminho a partir do perfil do usuário.
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("AppData").join("Roaming")))
            .unwrap_or_else(|| PathBuf::from("."));
        base.join(crate::brand::DATA_DIR)
    }

    /// config.json das versões anteriores à troca de nome.
    fn legacy_path() -> PathBuf {
        Self::dir().with_file_name(crate::brand::LEGACY_DATA_DIR).join("config.json")
    }

    pub fn path() -> PathBuf {
        Self::dir().join("config.json")
    }

    pub fn load() -> Self {
        let mut cfg: Config = std::fs::read_to_string(Self::path())
            .or_else(|_| std::fs::read_to_string(Self::legacy_path()))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        for i in 0..cfg.presets.len() {
            cfg.presets[i].normalize();
            if cfg.presets[i].id.is_empty() {
                let name = cfg.presets[i].name.clone();
                cfg.presets[i].id = cfg.unique_id(&name);
            }
        }
        cfg
    }

    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(Self::dir())?;
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        let tmp = Self::path().with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(tmp, Self::path())
    }

    /// Depois de trocar o idioma: renomeia os presets de fábrica que ainda têm o nome
    /// original (em qualquer idioma). Nomes que o usuário mudou ficam como estão.
    pub fn relocalize_factory_names(&mut self) {
        let lang = crate::i18n::current_index();
        for p in &mut self.presets {
            let factory = crate::i18n::menu::DEFAULT_PRESETS.iter().find(|(id, _)| *id == p.id);
            if let Some((_, names)) = factory {
                if names.contains(&p.name.as_str()) {
                    p.name = names[lang].to_string();
                }
            }
        }
    }

    pub fn preset(&self, id: &str) -> Option<&Preset> {
        self.presets.iter().find(|p| p.id == id)
    }

    /// Preset padrão, ou o primeiro da lista se o padrão não existir mais.
    pub fn fallback_preset(&self) -> Option<&Preset> {
        self.preset(&self.default_preset).or(self.presets.first())
    }

    /// Gera um id (slug) que ainda não existe a partir de um nome.
    pub fn unique_id(&self, name: &str) -> String {
        let mut slug: String = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        while slug.contains("--") {
            slug = slug.replace("--", "-");
        }
        let slug = slug.trim_matches('-');
        let slug = if slug.is_empty() { "preset" } else { slug };
        let mut id = slug.to_string();
        let mut n = 2;
        while self.presets.iter().any(|p| p.id == id) {
            id = format!("{slug}-{n}");
            n += 1;
        }
        id
    }
}

#[cfg(test)]
mod relocalize_tests {
    use super::*;

    #[test]
    fn factory_names_follow_language_but_custom_names_stay() {
        let mut cfg = Config::default();
        let mp3 = cfg.presets.iter().position(|p| p.id == "mp3").unwrap();
        let hevc = cfg.presets.iter().position(|p| p.id == "hevc").unwrap();
        cfg.presets[mp3].name = "Extrair MP3".into();
        cfg.presets[hevc].name = "Meu HEVC".into();
        crate::i18n::init(crate::i18n::Language::En);
        cfg.relocalize_factory_names();
        assert_eq!(cfg.presets[mp3].name, "Extract MP3");
        assert_eq!(cfg.presets[hevc].name, "Meu HEVC");
    }
}
