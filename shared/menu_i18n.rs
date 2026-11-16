// Textos do menu de botão direito e nomes dos presets de fábrica nos três idiomas.
//
// Incluído pelo app (src/i18n.rs) e pela DLL do menu, que não carrega o Fluent:
// ela só precisa destes poucos textos. A ordem dos arrays é [pt-BR, en, es].

/// Idiomas suportados, na ordem dos arrays abaixo.
#[allow(dead_code)]
pub const LANG_CODES: [&str; 3] = ["pt-BR", "en", "es"];

/// Índice do idioma a partir do código salvo no config ("system" ou vazio = idioma
/// do Windows, informado pelo `LANGID` da interface do usuário).
pub fn lang_index(code: &str, windows_ui_langid: u16) -> usize {
    match code {
        "pt-BR" => 0,
        "en" => 1,
        "es" => 2,
        _ => match windows_ui_langid & 0x3ff {
            0x16 => 0, // português
            0x0a => 2, // espanhol
            _ => 1,    // demais idiomas: inglês
        },
    }
}

/// (id, nomes) dos presets de fábrica, na ordem do menu.
pub const DEFAULT_PRESETS: &[(&str, [&str; 3])] = &[
    ("whatsapp", ["WhatsApp", "WhatsApp", "WhatsApp"]),
    ("3mbps", ["3 Mbps", "3 Mbps", "3 Mbps"]),
    ("mp4", ["MP4", "MP4", "MP4"]),
    ("youtube", ["YouTube", "YouTube", "YouTube"]),
    ("hevc", ["HEVC compacto", "Compact HEVC", "HEVC compacto"]),
    ("gif", ["GIF", "GIF", "GIF"]),
    ("mp3", ["Extrair MP3", "Extract MP3", "Extraer MP3"]),
];

/// "Abrir no Mari Encoder…"
pub const MENU_OPEN: [&str; 3] = ["Abrir no Mari Encoder…", "Open in Mari Encoder…", "Abrir en Mari Encoder…"];
/// "Editar presets…"
pub const MENU_EDIT_PRESETS: [&str; 3] = ["Editar presets…", "Edit presets…", "Editar presets…"];
