//! Menu de contexto do Windows 11 (o menu "novo", fora de "Mostrar mais opções").
//!
//! O Explorer carrega esta DLL num processo substituto (dllhost) por causa do pacote
//! registrado pelo app. Ela expõe um comando "QuickEncoder" com um subitem por preset,
//! lidos de %APPDATA%\QuickEncoder\config.json, e apenas dispara o quickencoder.exe
//! que está na mesma pasta.

#![allow(non_snake_case)]

use std::ffi::{OsString, c_void};
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_NOTIMPL, E_POINTER, HINSTANCE, HMODULE, S_FALSE, S_OK,
};
use windows::Win32::System::Com::{CoTaskMemFree, IBindCtx, IClassFactory, IClassFactory_Impl};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::UI::Shell::{
    ECF_DEFAULT, ECF_HASSUBCOMMANDS, ECF_SEPARATORBEFORE, ECS_ENABLED, IEnumExplorerCommand, IEnumExplorerCommand_Impl,
    FOLDERID_RoamingAppData, IExplorerCommand, IExplorerCommand_Impl, IShellItemArray, KF_FLAG_DEFAULT,
    SHGetKnownFolderPath, SHStrDupW, SIGDN_FILESYSPATH,
};
use windows::core::{BOOL, GUID, HRESULT, HSTRING, IUnknown, Interface, PWSTR, Ref, Result, implement};

/// Mesmo CLSID declarado no AppxManifest gerado pelo app (src/shell.rs).
const CLSID_MENU: GUID = GUID::from_u128(0x7b3e5c1a_9d42_4f86_a1e7_2c5d8b946f30);

static MODULE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

#[unsafe(no_mangle)]
extern "system" fn DllMain(module: HINSTANCE, reason: u32, _reserved: *mut c_void) -> BOOL {
    const DLL_PROCESS_ATTACH: u32 = 1;
    if reason == DLL_PROCESS_ATTACH {
        MODULE.store(module.0, Ordering::SeqCst);
    }
    true.into()
}

#[unsafe(no_mangle)]
extern "system" fn DllGetClassObject(clsid: *const GUID, iid: *const GUID, out: *mut *mut c_void) -> HRESULT {
    if out.is_null() || clsid.is_null() || iid.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = std::ptr::null_mut();
        if *clsid != CLSID_MENU {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        let factory: IClassFactory = Factory.into();
        factory.query(iid, out)
    }
}

#[unsafe(no_mangle)]
extern "system" fn DllCanUnloadNow() -> HRESULT {
    // O processo substituto encerra sozinho quando fica ocioso.
    S_FALSE
}

#[implement(IClassFactory)]
struct Factory;

impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(&self, outer: Ref<IUnknown>, iid: *const GUID, out: *mut *mut c_void) -> Result<()> {
        if out.is_null() {
            return Err(E_POINTER.into());
        }
        unsafe { *out = std::ptr::null_mut() };
        if !outer.is_null() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        let root: IExplorerCommand = Command { title: "QuickEncoder".into(), action: Action::Root, separator: false }.into();
        unsafe { root.query(iid, out).ok() }
    }

    fn LockServer(&self, _lock: BOOL) -> Result<()> {
        Ok(())
    }
}

enum Action {
    /// Item de topo, que só abre o submenu.
    Root,
    Preset(String),
    AddToList,
    EditPresets,
}

#[implement(IExplorerCommand)]
struct Command {
    title: String,
    action: Action,
    separator: bool,
}

fn exe_path() -> Option<PathBuf> {
    let mut buf = [0u16; 1024];
    let len = unsafe { GetModuleFileNameW(Some(HMODULE(MODULE.load(Ordering::SeqCst))), &mut buf) } as usize;
    if len == 0 || len >= buf.len() {
        return None;
    }
    let dll = PathBuf::from(OsString::from_wide(&buf[..len]));
    Some(dll.parent()?.join("quickencoder.exe"))
}

fn dup(text: &str) -> Result<PWSTR> {
    unsafe { SHStrDupW(&HSTRING::from(text)) }
}

/// %APPDATA%\QuickEncoder\config.json, o mesmo arquivo que o app grava.
fn config_path() -> Option<PathBuf> {
    let roaming = unsafe { SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, None) }.ok()?;
    let dir = PathBuf::from(OsString::from_wide(unsafe { roaming.as_wide() }));
    unsafe { CoTaskMemFree(Some(roaming.0 as *const c_void)) };
    Some(dir.join("QuickEncoder").join("config.json"))
}

include!("../../shared/default_menu.rs");

/// Presets do config.json, ou o motivo de não ter conseguido lê-los.
fn config_presets() -> std::result::Result<Vec<(String, String)>, String> {
    let path = config_path().ok_or("pasta AppData não encontrada")?;
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let presets = json["presets"].as_array().ok_or_else(|| format!("{}: sem lista de presets", path.display()))?;
    Ok(presets
        .iter()
        .filter(|p| p["in_menu"].as_bool().unwrap_or(true))
        .filter_map(|p| Some((p["id"].as_str()?.to_string(), p["name"].as_str()?.to_string())))
        .collect())
}

/// (id, nome) dos presets do menu e de onde vieram. Sem config.json legível (o app
/// ainda não rodou nesta conta), mostra os presets de fábrica — o app cria os mesmos
/// na primeira execução, então os ids batem.
fn menu_presets() -> (Vec<(String, String)>, String) {
    match config_presets() {
        Ok(presets) => (presets, "presets do config.json".into()),
        Err(why) => {
            let defaults = DEFAULT_MENU.iter().map(|(id, name)| (id.to_string(), name.to_string())).collect();
            (defaults, format!("presets de fábrica ({why})"))
        }
    }
}

fn selected_paths(items: Ref<IShellItemArray>) -> Vec<OsString> {
    let Some(items) = items.as_ref() else { return Vec::new() };
    let mut paths = Vec::new();
    unsafe {
        for i in 0..items.GetCount().unwrap_or(0) {
            let Ok(item) = items.GetItemAt(i) else { continue };
            if let Ok(name) = item.GetDisplayName(SIGDN_FILESYSPATH) {
                paths.push(OsString::from_wide(name.as_wide()));
                CoTaskMemFree(Some(name.0 as *const c_void));
            }
        }
    }
    paths
}

/// Dispara o app com todos os arquivos selecionados, quebrando em lotes para não
/// estourar o limite de tamanho da linha de comando do Windows.
fn launch(head: &[&str], files: Vec<OsString>) {
    const MAX_CHARS: usize = 24_000;
    let Some(exe) = exe_path() else { return };
    let mut batch: Vec<OsString> = Vec::new();
    let mut size = 0;
    let spawn = |batch: &mut Vec<OsString>| {
        let _ = std::process::Command::new(&exe).args(head).args(batch.drain(..)).spawn();
    };
    for file in files {
        let len = file.len() + 3;
        if !batch.is_empty() && size + len > MAX_CHARS {
            spawn(&mut batch);
            size = 0;
        }
        size += len;
        batch.push(file);
    }
    spawn(&mut batch);
}

impl IExplorerCommand_Impl for Command_Impl {
    fn GetTitle(&self, _items: Ref<IShellItemArray>) -> Result<PWSTR> {
        dup(&self.title)
    }

    fn GetIcon(&self, _items: Ref<IShellItemArray>) -> Result<PWSTR> {
        match (&self.action, exe_path()) {
            (Action::Root, Some(exe)) => dup(&exe.to_string_lossy()),
            _ => Err(E_NOTIMPL.into()),
        }
    }

    fn GetToolTip(&self, _items: Ref<IShellItemArray>) -> Result<PWSTR> {
        // O menu do Windows 11 não exibe dicas; no item raiz ela serve de diagnóstico
        // (examples/probe.rs mostra de onde os presets estão vindo).
        match self.action {
            Action::Root => dup(&menu_presets().1),
            _ => Err(E_NOTIMPL.into()),
        }
    }

    fn GetCanonicalName(&self) -> Result<GUID> {
        Ok(GUID::zeroed())
    }

    fn GetState(&self, _items: Ref<IShellItemArray>, _ok_to_be_slow: BOOL) -> Result<u32> {
        Ok(ECS_ENABLED.0 as u32)
    }

    fn Invoke(&self, items: Ref<IShellItemArray>, _ctx: Ref<IBindCtx>) -> Result<()> {
        match &self.action {
            Action::Root => {}
            Action::Preset(id) => {
                let files = selected_paths(items);
                if !files.is_empty() {
                    launch(&["--preset", id], files);
                }
            }
            Action::AddToList => launch(&["--add"], selected_paths(items)),
            Action::EditPresets => launch(&["--presets"], Vec::new()),
        }
        Ok(())
    }

    fn GetFlags(&self) -> Result<u32> {
        let mut flags = match self.action {
            Action::Root => ECF_HASSUBCOMMANDS.0,
            _ => ECF_DEFAULT.0,
        };
        if self.separator {
            flags |= ECF_SEPARATORBEFORE.0;
        }
        Ok(flags as u32)
    }

    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        if !matches!(self.action, Action::Root) {
            return Err(E_NOTIMPL.into());
        }
        let mut items: Vec<IExplorerCommand> = menu_presets()
            .0
            .into_iter()
            .map(|(id, name)| Command { title: name, action: Action::Preset(id), separator: false }.into())
            .collect();
        let separator = !items.is_empty();
        items.push(Command { title: "Abrir no QuickEncoder…".into(), action: Action::AddToList, separator }.into());
        items.push(Command { title: "Editar presets…".into(), action: Action::EditPresets, separator: false }.into());
        Ok(SubCommands { items, pos: AtomicUsize::new(0) }.into())
    }
}

#[implement(IEnumExplorerCommand)]
struct SubCommands {
    items: Vec<IExplorerCommand>,
    pos: AtomicUsize,
}

impl IEnumExplorerCommand_Impl for SubCommands_Impl {
    fn Next(&self, count: u32, out: *mut Option<IExplorerCommand>, fetched: *mut u32) -> HRESULT {
        if out.is_null() {
            return E_POINTER;
        }
        let mut n = 0u32;
        let mut pos = self.pos.load(Ordering::SeqCst);
        while n < count && pos < self.items.len() {
            unsafe { out.add(n as usize).write(Some(self.items[pos].clone())) };
            n += 1;
            pos += 1;
        }
        self.pos.store(pos, Ordering::SeqCst);
        if !fetched.is_null() {
            unsafe { *fetched = n };
        }
        if n == count { S_OK } else { S_FALSE }
    }

    fn Skip(&self, count: u32) -> Result<()> {
        let pos = self.pos.load(Ordering::SeqCst).saturating_add(count as usize).min(self.items.len());
        self.pos.store(pos, Ordering::SeqCst);
        Ok(())
    }

    fn Reset(&self) -> Result<()> {
        self.pos.store(0, Ordering::SeqCst);
        Ok(())
    }

    fn Clone(&self) -> Result<IEnumExplorerCommand> {
        Ok(SubCommands { items: self.items.clone(), pos: AtomicUsize::new(self.pos.load(Ordering::SeqCst)) }.into())
    }
}
