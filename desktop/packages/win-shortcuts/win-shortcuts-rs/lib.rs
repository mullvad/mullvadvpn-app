#![cfg(target_os = "windows")]

use std::marker::PhantomData;
use std::string::FromUtf16Error;
use std::sync::{mpsc, OnceLock};

use neon::prelude::*;
use windows::core::{Interface, HSTRING, PCWSTR};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED, STGM_READ,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink, SLGP_UNCPRIORITY};

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

    /// Path is not valid UTF-16
    #[error("Path is not valid UTF-16")]
    Utf16ToString(#[source] FromUtf16Error),
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
        // SAFETY: We're passing a valid GUID pointer.
        unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) };
    let shell_link = shell_link_result.map_err(Error::CreateInstance)?;

    // Load the .lnk using IPersistFile
    let path = HSTRING::from(path);
    let persist_file_result: windows::core::Result<IPersistFile> = shell_link.cast();
    let persist_file = persist_file_result.map_err(Error::CastShortcut)?;

    // SAFETY: HSTRING::from will ensure that path is a valid utf16 null-terminated string.
    unsafe { persist_file.Load(PCWSTR(path.as_ptr()), STGM_READ) }.map_err(Error::LoadShortcut)?;

    let mut target_buffer = [0u16; MAX_PATH_LEN];

    // SAFETY: This function is trivially safe to call.
    unsafe {
        shell_link.GetPath(
            &mut target_buffer,
            std::ptr::null_mut(),
            SLGP_UNCPRIORITY.0 as u32,
        )
    }
    .map_err(Error::GetPath)?;

    let utf16_slice = split_at_null(&target_buffer);
    let s = String::from_utf16(utf16_slice).map_err(Error::Utf16ToString)?;
    Ok(Some(s))
}

fn split_at_null(slice: &[u16]) -> &[u16] {
    slice.split(|&c| c == 0).next().unwrap_or(slice)
}

/// Struct for safely handling initialization and deinitialization of the Windows COM library.
/// A successful call to [CoInitializeEx] _needs_ to be accompanied by a call to [CoUninitialize],
/// which is taken care by the drop implementation on [ComContext].
///
/// [CoInitializeEx] sets up thread-local state. Thus this type is `!Send` to stop it being moved
/// to another thread.
struct ComContext {
    // HACK: until negative impls are stable, this how we stop `Send` from being impld
    _do_not_impl_send: PhantomData<*mut ()>,
}

impl ComContext {
    /// Create a new [ComContext].
    ///
    /// This will call [CoInitializeEx] now, and [CoUninitialize] when dropped.
    ///
    /// May return an error if [CoInitializeEx] was previously called with different arguments on
    /// the same thread.
    fn new() -> Result<Self, windows::core::Error> {
        // SAFETY: This is paired with CoUninitialize in impl Drop
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.ok()?;

        Ok(Self {
            _do_not_impl_send: PhantomData,
        })
    }
}

impl Drop for ComContext {
    fn drop(&mut self) {
        // SAFETY: CoInitializeEx was called when this struct was created,
        // and it was called on the same thread since ComContext is !Send.
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
    static THREAD_SENDER: OnceLock<mpsc::Sender<Message>> = OnceLock::new();
    THREAD_SENDER
        .get_or_init(move || {
            let (tx, rx) = mpsc::channel();

            std::thread::spawn(move || {
                let com = match ComContext::new() {
                    Ok(com) => com,
                    Err(e) => {
                        eprintln!("Failed to initialize ComContext: {e}");
                        return;
                    }
                };

                while let Ok(msg) = rx.recv() {
                    match msg {
                        Message::ResolveShortcut { path, result_tx } => {
                            let _ = result_tx.send(get_shortcut_path(&path));
                        }
                    }
                }

                drop(com);
            });

            tx
        })
        .clone()
}
