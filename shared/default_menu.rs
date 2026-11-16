/// (id, nome) dos presets de fábrica, na ordem do menu.
///
/// Incluído pela DLL do menu do Windows 11, que mostra esta lista enquanto o
/// config.json ainda não existe. Um teste em src/config.rs garante que ela bate
/// com `default_presets()`.
pub const DEFAULT_MENU: &[(&str, &str)] = &[
    ("whatsapp", "WhatsApp"),
    ("3mbps", "3 Mbps"),
    ("mp4", "MP4"),
    ("youtube", "YouTube"),
    ("hevc", "HEVC compacto"),
    ("gif", "GIF"),
    ("mp3", "Extrair MP3"),
];
