//! Menu contextual do Explorer, registrado só para o usuário atual (sem admin).
//!
//! Menu clássico ("Mostrar mais opções"): cada extensão ganha no registro (HKCU) um
//! verbo `QuickEncoder` (Position=Top) que aponta, via `ExtendedSubCommandsKey`, para
//! um submenu compartilhado com um item por preset.
//!
//! Menu principal do Windows 11: pacote esparso + DLL COM — veja o fim do arquivo.

use crate::config::Config;
use std::io;
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};

const CLASSES: &str = r"Software\Classes";
const VERB: &str = "QuickEncoder";
const MENU_KEY: &str = "QuickEncoder.Menu";
const ECF_SEPARATORBEFORE: u32 = 0x20;

fn classes() -> io::Result<RegKey> {
    Ok(RegKey::predef(HKEY_CURRENT_USER).create_subkey(CLASSES)?.0)
}

fn clean_ext(ext: &str) -> Option<String> {
    let e = ext.trim().trim_start_matches('.').to_lowercase();
    (!e.is_empty() && e.chars().all(|c| c.is_ascii_alphanumeric())).then_some(e)
}

pub fn is_installed() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(format!(r"{CLASSES}\{MENU_KEY}"), KEY_READ)
        .is_ok()
}

pub fn install(cfg: &Config) -> io::Result<()> {
    uninstall()?;
    let exe = std::env::current_exe()?;
    let exe = exe.to_string_lossy();
    let classes = classes()?;

    // Submenu compartilhado: um item por preset marcado para o menu.
    let mut index = 0;
    for p in cfg.presets.iter().filter(|p| p.in_menu) {
        let (item, _) = classes.create_subkey(format!(r"{MENU_KEY}\shell\{index:02}_{}", p.id))?;
        item.set_value("MUIVerb", &p.name)?;
        item.set_value("MultiSelectModel", &"Player")?;
        let (cmd, _) = item.create_subkey("command")?;
        cmd.set_value("", &format!("\"{exe}\" --preset \"{}\" \"%1\"", p.id))?;
        index += 1;
    }
    let (open, _) = classes.create_subkey(format!(r"{MENU_KEY}\shell\98_open"))?;
    open.set_value("MUIVerb", &"Abrir no QuickEncoder…")?;
    open.set_value("MultiSelectModel", &"Player")?;
    if index > 0 {
        open.set_value("CommandFlags", &ECF_SEPARATORBEFORE)?;
    }
    let (cmd, _) = open.create_subkey("command")?;
    cmd.set_value("", &format!("\"{exe}\" --add \"%1\""))?;

    let (edit, _) = classes.create_subkey(format!(r"{MENU_KEY}\shell\99_presets"))?;
    edit.set_value("MUIVerb", &"Editar presets…")?;
    let (cmd, _) = edit.create_subkey("command")?;
    cmd.set_value("", &format!("\"{exe}\" --presets"))?;

    // Entrada no topo do menu de cada extensão.
    for ext in cfg.extensions.iter().filter_map(|e| clean_ext(e)) {
        let (verb, _) = classes.create_subkey(format!(r"SystemFileAssociations\.{ext}\shell\{VERB}"))?;
        verb.set_value("MUIVerb", &"QuickEncoder")?;
        verb.set_value("Icon", &format!("\"{exe}\",0"))?;
        verb.set_value("Position", &"Top")?;
        verb.set_value("MultiSelectModel", &"Player")?;
        verb.set_value("ExtendedSubCommandsKey", &MENU_KEY)?;
    }
    notify_shell();
    Ok(())
}

pub fn uninstall() -> io::Result<()> {
    let classes = classes()?;
    if classes.open_subkey_with_flags(MENU_KEY, KEY_READ).is_ok() {
        classes.delete_subkey_all(MENU_KEY)?;
    }
    if let Ok(assoc) = classes.open_subkey("SystemFileAssociations") {
        let exts: Vec<String> = assoc.enum_keys().filter_map(Result::ok).collect();
        for ext in exts {
            let verb = format!(r"{ext}\shell\{VERB}");
            if assoc.open_subkey_with_flags(&verb, KEY_READ).is_ok() {
                assoc.delete_subkey_all(&verb)?;
                // Remove as chaves-pai só se ficaram vazias (delete_subkey falha se houver filhos).
                let _ = assoc.delete_subkey(format!(r"{ext}\shell"));
                let _ = assoc.delete_subkey(&ext);
            }
        }
    }
    notify_shell();
    Ok(())
}

// ---------- Menu principal do Windows 11 ----------
//
// O menu novo só aceita extensões COM (IExplorerCommand) que venham de um pacote com
// identidade. Registramos um pacote "esparso": só o manifesto, apontando para a pasta
// onde estão o quickencoder.exe e a quickencoder_menu.dll. Sem assinatura, o Windows
// só aceita esse registro com o Modo de Desenvolvedor ligado.
//
// O manifesto desliga a virtualização de arquivos e registro: o app aberto pelo menu
// herda o contexto do pacote e precisa ler/gravar o config.json e o HKCU de verdade,
// os mesmos que ele usa quando é aberto direto pelo Explorer.

const PACKAGE: &str = "QuickEncoder.ContextMenu";
/// Mesmo CLSID de `CLSID_MENU` em shellext/src/lib.rs.
const CLSID: &str = "7B3E5C1A-9D42-4F86-A1E7-2C5D8B946F30";
const MENU_DLL: &str = "quickencoder_menu.dll";
const MANIFEST: &str = "AppxManifest.xml";
const LOGO: &str = "QuickEncoder.Logo.png";

fn exe_dir() -> io::Result<std::path::PathBuf> {
    let exe = std::env::current_exe()?;
    exe.parent().map(|p| p.to_path_buf()).ok_or_else(|| io::Error::other("pasta do executável não encontrada"))
}

/// Windows 11 e a DLL do menu ao lado do executável.
pub fn modern_available() -> bool {
    let build: u32 = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion", KEY_READ)
        .and_then(|k| k.get_value::<String, _>("CurrentBuildNumber"))
        .ok()
        .and_then(|b| b.parse().ok())
        .unwrap_or(0);
    build >= 22000 && exe_dir().is_ok_and(|d| d.join(MENU_DLL).is_file())
}

pub fn modern_installed() -> bool {
    let repo = r"Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppModel\Repository\Packages";
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(repo, KEY_READ)
        .is_ok_and(|k| k.enum_keys().filter_map(Result::ok).any(|name| name.starts_with(&format!("{PACKAGE}_"))))
}

fn manifest_xml(cfg: &Config) -> String {
    let mut exts: Vec<String> = cfg.extensions.iter().filter_map(|e| clean_ext(e)).collect();
    exts.sort();
    exts.dedup();
    let item_types: String = exts
        .iter()
        .map(|e| {
            format!(
                "            <desktop5:ItemType Type=\".{e}\"><desktop5:Verb Id=\"QuickEncoder\" Clsid=\"{CLSID}\" /></desktop5:ItemType>\n"
            )
        })
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:uap10="http://schemas.microsoft.com/appx/manifest/uap/windows10/10"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  xmlns:desktop4="http://schemas.microsoft.com/appx/manifest/desktop/windows10/4"
  xmlns:desktop5="http://schemas.microsoft.com/appx/manifest/desktop/windows10/5"
  xmlns:desktop6="http://schemas.microsoft.com/appx/manifest/desktop/windows10/6"
  xmlns:com="http://schemas.microsoft.com/appx/manifest/com/windows10"
  IgnorableNamespaces="uap uap10 rescap desktop4 desktop5 desktop6 com">
  <Identity Name="{PACKAGE}" Publisher="CN=QuickEncoder" Version="1.0.0.0" ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>QuickEncoder</DisplayName>
    <PublisherDisplayName>QuickEncoder</PublisherDisplayName>
    <Logo>{LOGO}</Logo>
    <uap10:AllowExternalContent>true</uap10:AllowExternalContent>
    <desktop6:FileSystemWriteVirtualization>disabled</desktop6:FileSystemWriteVirtualization>
    <desktop6:RegistryWriteVirtualization>disabled</desktop6:RegistryWriteVirtualization>
  </Properties>
  <Resources>
    <Resource Language="pt-BR" />
  </Resources>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.19041.0" MaxVersionTested="10.0.26100.0" />
  </Dependencies>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
    <rescap:Capability Name="unvirtualizedResources" />
  </Capabilities>
  <Applications>
    <Application Id="QuickEncoder" Executable="quickencoder.exe" uap10:TrustLevel="mediumIL" uap10:RuntimeBehavior="win32App">
      <uap:VisualElements DisplayName="QuickEncoder" Description="QuickEncoder" BackgroundColor="transparent" Square150x150Logo="{LOGO}" Square44x44Logo="{LOGO}" AppListEntry="none" />
      <Extensions>
        <desktop4:Extension Category="windows.fileExplorerContextMenus">
          <desktop4:FileExplorerContextMenus>
{item_types}          </desktop4:FileExplorerContextMenus>
        </desktop4:Extension>
        <com:Extension Category="windows.comServer">
          <com:ComServer>
            <com:SurrogateServer DisplayName="QuickEncoder">
              <com:Class Id="{CLSID}" Path="{MENU_DLL}" ThreadingModel="STA" />
            </com:SurrogateServer>
          </com:ComServer>
        </com:Extension>
      </Extensions>
    </Application>
  </Applications>
</Package>
"#
    )
}

/// Roda um script no Windows PowerShell (que traz o módulo Appx) sem abrir janela.
fn powershell(script: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    let out = std::process::Command::new(format!(r"{root}\System32\WindowsPowerShell\v1.0\powershell.exe"))
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .stdin(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("não foi possível rodar o PowerShell: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    let first = err.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("erro desconhecido");
    Err(first.to_string())
}

fn ps_quote(path: &std::path::Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "''"))
}

/// Registra (ou atualiza) o pacote do menu do Windows 11. Não faz nada se já está em dia.
pub fn install_modern(cfg: &Config) -> Result<(), String> {
    let dir = exe_dir().map_err(|e| e.to_string())?;
    if !dir.join(MENU_DLL).is_file() {
        return Err(format!("{MENU_DLL} não está na pasta do executável"));
    }
    let manifest = manifest_xml(cfg);
    let path = dir.join(MANIFEST);
    if modern_installed() && std::fs::read_to_string(&path).is_ok_and(|current| current == manifest) {
        return Ok(());
    }
    std::fs::write(dir.join(LOGO), include_bytes!("../assets/icon.png")).map_err(|e| e.to_string())?;
    std::fs::write(&path, manifest).map_err(|e| e.to_string())?;
    powershell(&format!(
        "$ErrorActionPreference = 'Stop'; Get-AppxPackage -Name '{PACKAGE}' | Remove-AppxPackage; \
         Add-AppxPackage -Register {} -ExternalLocation {}",
        ps_quote(&path),
        ps_quote(&dir)
    ))
    .map_err(|e| format!("{e} (sem assinatura, o registro exige o Modo de Desenvolvedor do Windows)"))?;
    notify_shell();
    Ok(())
}

pub fn uninstall_modern() -> Result<(), String> {
    if modern_installed() {
        powershell(&format!("$ErrorActionPreference = 'Stop'; Get-AppxPackage -Name '{PACKAGE}' | Remove-AppxPackage"))?;
    }
    if let Ok(dir) = exe_dir() {
        let _ = std::fs::remove_file(dir.join(MANIFEST));
        let _ = std::fs::remove_file(dir.join(LOGO));
    }
    notify_shell();
    Ok(())
}

fn notify_shell() {
    use windows_sys::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED as _, SHCNF_IDLIST as _, std::ptr::null(), std::ptr::null()) };
}
