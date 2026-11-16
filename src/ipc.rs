//! Instância única: a primeira instância vira dona da fila e escuta um named pipe;
//! as demais (uma por arquivo selecionado no Explorer) só entregam o pedido e saem.

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::windows::io::FromRawHandle;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, ERROR_PIPE_CONNECTED, GetLastError, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_INBOUND;
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows_sys::Win32::System::Threading::CreateMutexW;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Msg {
    /// Id do preset (menu contextual). `None` = só adicionar à lista.
    pub preset: Option<String>,
    pub files: Vec<PathBuf>,
    /// Começar a encodar imediatamente.
    pub start: bool,
    /// Aba a abrir na janela completa: "presets" ou "settings".
    #[serde(default)]
    pub tab: Option<String>,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn pipe_name() -> String {
    let user: String = std::env::var("USERNAME")
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    format!(r"\\.\pipe\QuickEncoder.{user}")
}

/// `true` se esta é a primeira instância na sessão. O mutex vive até o processo sair.
pub fn acquire() -> bool {
    let name = wide("Local\\QuickEncoder.SingleInstance");
    unsafe {
        let handle = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
        !handle.is_null() && GetLastError() != ERROR_ALREADY_EXISTS
    }
}

/// Escuta pedidos de outras instâncias e os repassa pela `tx`. `wake` acorda a UI.
pub fn serve(tx: Sender<Msg>, wake: impl Fn() + Send + 'static) {
    let spawned = std::thread::Builder::new().name("ipc-server".into()).spawn(move || {
        let name = wide(&pipe_name());
        loop {
            let handle = unsafe {
                CreateNamedPipeW(
                    name.as_ptr(),
                    PIPE_ACCESS_INBOUND,
                    PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                    PIPE_UNLIMITED_INSTANCES,
                    0,
                    64 * 1024,
                    0,
                    std::ptr::null(),
                )
            };
            if handle == INVALID_HANDLE_VALUE {
                std::thread::sleep(Duration::from_millis(500));
                continue;
            }
            let connected =
                unsafe { ConnectNamedPipe(handle, std::ptr::null_mut()) != 0 || GetLastError() == ERROR_PIPE_CONNECTED };
            // `File` fecha o handle ao sair do escopo.
            let mut pipe = unsafe { File::from_raw_handle(handle as _) };
            if !connected {
                continue;
            }
            let mut text = String::new();
            if pipe.read_to_string(&mut text).is_ok() {
                if let Ok(msg) = serde_json::from_str::<Msg>(&text) {
                    if tx.send(msg).is_err() {
                        return;
                    }
                    wake();
                }
            }
        }
    });
    let _ = spawned;
}

/// Entrega o pedido à instância principal, insistindo enquanto ela ainda está abrindo.
pub fn send(msg: &Msg) -> bool {
    let Ok(data) = serde_json::to_vec(msg) else { return false };
    let name = pipe_name();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match OpenOptions::new().write(true).open(&name) {
            Ok(mut pipe) => return pipe.write_all(&data).is_ok(),
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(40)),
            Err(_) => return false,
        }
    }
}
