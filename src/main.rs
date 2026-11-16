#![windows_subsystem = "windows"]

#[macro_use]
mod i18n;
mod about;
mod app;
#[allow(dead_code)]
mod brand {
    include!("../shared/brand.rs");
}
mod config;
mod ffmpeg;
mod ipc;
mod queue;
mod shell;
mod theme;

use std::path::PathBuf;

/// Uso:
///   MariEncoder                          abre a janela de batch
///   MariEncoder <arquivos>               abre com os arquivos na lista
///   MariEncoder --add <arquivos>         idem (usado pelo menu contextual)
///   MariEncoder --preset <id> <arquivos> encoda direto com o preset
///   MariEncoder --presets | --settings | --about  abre direto na aba indicada
///   MariEncoder --install | --uninstall  registra/remove o menu contextual
///                                        (sem efeito no pacote da Store)
fn main() {
    // Idioma do Windows até ler o config (que pode escolher outro).
    i18n::init(i18n::Language::System);
    let mut msg = ipc::Msg::default();
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--preset") => {
                msg.preset = args.next().map(|s| s.to_string_lossy().into_owned());
                msg.start = true;
            }
            Some("--add") => {}
            Some("--presets") => msg.tab = Some("presets".into()),
            Some("--settings") => msg.tab = Some("settings".into()),
            Some("--about") => msg.tab = Some("about".into()),
            Some("--install") | Some("--uninstall") if shell::is_packaged() => {
                // No pacote da Store o menu vem do manifesto.
                std::process::exit(0);
            }
            Some("--install") => {
                let cfg = config::Config::load();
                i18n::init(cfg.language);
                if !config::Config::path().exists() {
                    // O menu do Windows 11 lê os presets deste arquivo.
                    let _ = cfg.save();
                }
                let classic = shell::install(&cfg).is_ok();
                let modern = !shell::modern_available() || shell::install_modern(&cfg).is_ok();
                // 0 = tudo certo, 1 = falhou o menu clássico, 2 = falhou só o menu do Windows 11
                std::process::exit(if !classic { 1 } else if !modern { 2 } else { 0 });
            }
            Some("--uninstall") => {
                let classic = shell::uninstall().is_ok();
                let modern = shell::uninstall_modern().is_ok();
                std::process::exit(if !classic { 1 } else if !modern { 2 } else { 0 });
            }
            _ => {
                let path = PathBuf::from(arg);
                msg.files.push(std::path::absolute(&path).unwrap_or(path));
            }
        }
    }

    if !ipc::acquire() {
        // Já existe uma janela com a fila: entrega o pedido e sai.
        ipc::send(&msg);
        return;
    }
    app::run(msg);
}
