#![cfg(target_os = "windows")]

use std::sync::{mpsc, OnceLock};

use neon::prelude::*;
use windows::core::Interface;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED, STGM_READ,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink, SLGP_UNCPRIORITY};

static THREAD_SENDER: OnceLock<mpsc::Sender<Message>> = OnceLock::new();

/// Messages that can be sent to the thread
enum Message {
    ResolveShortcut {
        path: String,
        result_tx: mpsc::Sender<Result<Option<String>, Error>>,
    },
}

#[derive(thiserror::Error, Debug)]
enum Error {
    /// The handler thread is down
    #[error("The handler thread is down")]
    ThreadDown,

    /// CoCreateInstance failed to create an IShellLinkW instance
    #[error("CoCreateInstance failed to create an IShellLinkW instance")]
    CreateInstance(#[source] windows::core::Error),

    /// Failed to cast shortcut to IPersistFile
    #[error("Failed to cast IShellLinkW")]
    CastShortcut(#[source] windows::core::Error),

    /// Failed to load shortcut
    #[error("Failed to load shortcut .lnk")]
    LoadShortcut(#[source] windows::core::Error),

    /// Failed to retrieve IShellLinkW path
    #[error("Failed to retrieve IShellLinkW link")]
    GetPath(#[source] windows::core::Error),
}

/// Maximum path length of shortcut
/// 32 KiB: https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation?tabs=registry
const MAX_PATH_LEN: usize = 0x7fff;

#[neon::main]
fn main(mut cx: ModuleContext<'_>) -> NeonResult<()> {
    cx.export_function("readShortcut", read_shortcut)?;

    Ok(())
}

fn read_shortcut(mut cx: FunctionContext<'_>) -> JsResult<'_, JsValue> {
    let link_path = cx.argument::<JsString>(0)?.value(&mut cx);

    match read_shortcut_inner(link_path) {
        Ok(Some(path)) => Ok(cx.string(path).as_value(&mut cx)),
        Ok(None) => Ok(cx.null().as_value(&mut cx)),
        Err(err) => cx.throw_error(format!("Failed to read shortcut: {err}")),
    }
}

fn read_shortcut_inner(link_path: String) -> Result<Option<String>, Error> {
    let tx = get_com_thread();

    let (result_tx, result_rx) = mpsc::channel();
    tx.send(Message::ResolveShortcut {
        path: link_path,
        result_tx,
    })
    .map_err(|_err| Error::ThreadDown)?;

    result_rx.recv().map_err(|_err| Error::ThreadDown)?
}

/// Retrieve shortcut .lnk to its target path
fn get_shortcut_path(path: &str) -> Result<Option<String>, Error> {
    let shell_link_result: windows::core::Result<IShellLinkW> =
        unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) };
    let shell_link = shell_link_result.map_err(Error::CreateInstance)?;

    // Load the .lnk using IPersistFile
    let path = HSTRING::from(path);
    let persist_file_result: windows::core::Result<IPersistFile> = shell_link.cast();
    let persist_file = persist_file_result.map_err(Error::CastShortcut)?;
    unsafe {
        persist_file
            .Load(PCWSTR(path.as_ptr()), STGM_READ)
            .map_err(Error::LoadShortcut)?;
    }

    let mut target_buffer = [0u16; MAX_PATH_LEN];
    unsafe {
        shell_link
            .GetPath(
                &mut target_buffer,
                std::ptr::null_mut(),
                SLGP_UNCPRIORITY.0 as u32,
            )
            .map_err(Error::GetPath)?;
    }

    Ok(Some(strip_null_terminator(&target_buffer)))
}

fn strip_null_terminator(slice: &[u16]) -> String {
    let s = slice.split(|&c| c == 0).next().unwrap_or(slice);
    String::from_utf16_lossy(s)
}

/// Struct for safely handling initialization and deinitialization of the Windows COM library.
/// A call to CoInitializeEx _needs_ to be accompanied by a call to CoUninitialize, which is
/// taken care by the drop implementation on [ComContext]. It is up to the consumer of [ComContext]
/// to every only call [ComContext::new] once per thread before calling drop.
struct ComContext {}

impl ComContext {
    fn new() -> Result<Self, windows::core::Error> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        };
        Ok(Self {})
    }
}

impl Drop for ComContext {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

/// Retrieve a channel for communicating with the thread responsible for handling
/// COM library safely.
/// We spawn a thread in case the caller may have already initialized COM in an
/// incompatible way.
fn get_com_thread() -> mpsc::Sender<Message> {
    THREAD_SENDER
        .get_or_init(move || {
            let (tx, rx) = mpsc::channel();

            std::thread::spawn(move || {
                let _com = ComContext::new().expect("failed to initialize COM");

                while let Ok(msg) = rx.recv() {
                    match msg {
                        Message::ResolveShortcut { path, result_tx } => {
                            let _ = result_tx.send(get_shortcut_path(&path));
                        }
                    }
                }
            });

            tx
        })
        .clone()
}
