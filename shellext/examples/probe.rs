//! Diagnóstico: conversa com a extensão do menu do Windows 11 do mesmo jeito que o
//! Explorer (via COM, fora do processo) e lista o que ela devolve.
//!
//!   cargo run -p mari-encoder-menu --example probe
//!   cargo run -p mari-encoder-menu --example probe -- "WhatsApp" C:\videos\clipe.mp4
//!
//! Com dois argumentos, invoca o item com aquele título para o arquivo dado.

use windows::Win32::System::Com::{
    CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
};
use windows::Win32::UI::Shell::{
    IExplorerCommand, IShellItem, IShellItemArray, SHCreateItemFromParsingName, SHCreateShellItemArrayFromShellItem,
};
use windows::core::{GUID, HSTRING, PWSTR, Result};

use MariEncoderMenu::brand;

fn text(value: Result<PWSTR>) -> String {
    match value {
        Ok(p) => unsafe {
            let s = p.to_string().unwrap_or_default();
            CoTaskMemFree(Some(p.0 as *const _));
            s
        },
        Err(_) => "—".into(),
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        // Tenta a classe da Store e depois a do pacote esparso (versão do GitHub).
        let mut root: Option<IExplorerCommand> = None;
        for (label, id) in [("Store", brand::CLSID_STORE), ("GitHub", brand::CLSID_DEV)] {
            let clsid = GUID::try_from(id).expect("CLSID válido");
            if let Ok(cmd) = CoCreateInstance(&clsid, None, CLSCTX_LOCAL_SERVER) {
                println!("classe registrada: {label} ({id})");
                root = Some(cmd);
                break;
            }
        }
        let Some(root) = root else {
            println!("nenhuma classe do menu registrada (instale pelo app ou pelo pacote da Store)");
            return Ok(());
        };
        println!("raiz: \"{}\"  flags={:#x}  ícone={}", text(root.GetTitle(None)), root.GetFlags()?, text(root.GetIcon(None)));

        println!("origem: {}", text(root.GetToolTip(None)));

        let subs = root.EnumSubCommands()?;
        loop {
            let mut slot = [None];
            let mut fetched = 0u32;
            let _ = subs.Next(&mut slot, Some(&mut fetched));
            let Some(cmd) = slot[0].take().filter(|_| fetched == 1) else { break };
            let title = text(cmd.GetTitle(None));
            println!("  - \"{title}\"  flags={:#x}  estado={:#x}", cmd.GetFlags()?, cmd.GetState(None, false)?);

            if args.len() == 2 && args[0] == title {
                let item: IShellItem = SHCreateItemFromParsingName(&HSTRING::from(&args[1]), None)?;
                let array: IShellItemArray = SHCreateShellItemArrayFromShellItem(&item)?;
                cmd.Invoke(&array, None)?;
                println!("    invocado para {}", args[1]);
            }
        }
    }
    Ok(())
}
