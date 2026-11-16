#![windows_subsystem = "windows"]

mod app;
mod config;
mod ffmpeg;
mod ipc;
mod queue;
mod shell;
mod theme;

use std::path::PathBuf;

/// Uso:
///   quickencoder                         abre a janela de batch
///   quickencoder <arquivos>              abre com os arquivos na lista
///   quickencoder --add <arquivos>        idem (usado pelo menu contextual)
///   quickencoder --preset <id> <arquivos> encoda direto com o preset
///   quickencoder --presets | --settings  abre direto na aba indicada
///   quickencoder --install | --uninstall registra/remove o menu contextual
fn main() {
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
            Some("--install") => {
                let cfg = config::Config::load();
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
