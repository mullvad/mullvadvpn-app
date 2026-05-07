#![cfg(all(target_arch = "x86", target_os = "windows"))]

#![no_std]
#![allow(unused)]
#![allow(nonstandard_style)]

extern crate alloc;

use core::{
    ffi::{c_int, c_void},
    fmt::Display,
    iter,
    mem::{size_of, size_of_val},
};

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::{
    alloc::{GlobalAlloc, Layout},
    vec::Vec,
};

use windows_sys::Win32::{
    Foundation::GlobalFree,
    Globalization::{lstrcpyW, lstrcpynW},
    System::Memory::{
        GetProcessHeap, GlobalAlloc, HeapAlloc, HeapFree, HeapReAlloc, GPTR, HEAP_ZERO_MEMORY,
    },
};

pub use nsis_fn::nsis_fn;

pub type wchar_t = i32;

#[repr(C)]
#[derive(Debug)]
pub struct stack_t {
    pub next: *mut stack_t,
    pub text: [wchar_t; 1],
}

pub static mut G_STRINGSIZE: c_int = 0;
pub static mut G_VARIABLES: *mut wchar_t = core::ptr::null_mut();
pub static mut G_STACKTOP: *mut *mut stack_t = core::ptr::null_mut();

/// Initis the global variables used by NSIS functions: [`push`], [`pushstr`], [`pushint`], [`pop`], [`popstr`] and [`popint`]
///
/// # Safety
///
/// This function mutates static variables and should only be called in a function
#[inline(always)]
pub unsafe fn exdll_init(string_size: c_int, variables: *mut wchar_t, stacktop: *mut *mut stack_t) {
    G_STRINGSIZE = string_size;
    G_VARIABLES = variables;
    G_STACKTOP = stacktop;
}

pub const ZERO: &[u16; 2] = &[48, 0];
pub const ONE: &[u16; 2] = &[49, 0];
pub const TWO: &[u16; 2] = &[50, 0];
pub const NEGATIVE_ONE: &[u16; 3] = &[45, 49, 0];

#[derive(Debug)]
pub enum Error {
    StackIsNull,
    ParseIntError,
}

impl Error {
    const fn description(&self) -> &str {
        match self {
            Error::StackIsNull => "Stack is null",
            Error::ParseIntError => "Failed to parse integer",
        }
    }
    pub fn push_err(&self) {
        let _ = unsafe { pushstr(self.description()) };
    }
}

/// Pushes some bytes onto the NSIS stack.
///
/// # Safety
///
/// This function reads static variables and should only be called after [`exdll_init`] is called.
pub unsafe fn push(bytes: &[u16]) -> Result<(), Error> {
    if G_STACKTOP.is_null() {
        return Err(Error::StackIsNull);
    }

    let n = size_of::<stack_t>() + G_STRINGSIZE as usize * 2;
    let th = GlobalAlloc(GPTR, n) as *mut stack_t;
    lstrcpyW((*th).text.as_ptr() as _, bytes.as_ptr());
    (*th).next = *G_STACKTOP;
    *G_STACKTOP = th;

    Ok(())
}

/// Pushes a string onto the NSIS stack.
///
/// # Safety
///
/// This function reads static variables and should only be called after [`exdll_init`] is called.
pub unsafe fn pushstr(str: &str) -> Result<(), Error> {
    let bytes = encode_utf16(str);
    push(&bytes)
}

/// Pushes an integer onto the NSIS stack.
///
/// # Safety
///
/// This function reads static variables and should only be called after [`exdll_init`] is called.
pub unsafe fn pushint(int: i32) -> Result<(), Error> {
    let str = int.to_string();
    pushstr(&str)
}

/// Pops bytes from NSIS stack.
///
/// # Safety
///
/// This function reads static variables and should only be called after [`exdll_init`] is called.
pub unsafe fn pop() -> Result<Vec<u16>, Error> {
    if G_STACKTOP.is_null() || (*G_STACKTOP).is_null() {
        return Err(Error::StackIsNull);
    }

    let mut out = vec![0_u16; G_STRINGSIZE as _];

    let th: *mut stack_t = *G_STACKTOP;
    lstrcpyW(out.as_mut_ptr(), (*th).text.as_ptr() as _);
    *G_STACKTOP = (*th).next;
    GlobalFree(th as _);

    Ok(out)
}

/// Pops a string from NSIS stack.
///
/// # Safety
///
/// This function reads static variables and should only be called after [`exdll_init`] is called.
pub unsafe fn popstr() -> Result<String, Error> {
    let bytes = pop()?;
    Ok(decode_utf16_lossy(&bytes))
}

/// Pops an integer from NSIS stack.
///
/// # Safety
///
/// This function reads static variables and should only be called after [`exdll_init`] is called.
pub unsafe fn popint() -> Result<i32, Error> {
    let str = popstr()?;
    str.parse().map_err(|_| Error::ParseIntError)
}

pub fn encode_utf16(str: &str) -> Vec<u16> {
    str.encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>()
}

pub fn decode_utf16_lossy(bytes: &[u16]) -> String {
    let bytes = bytes
        .iter()
        .position(|c| *c == 0)
        .map(|nul| &bytes[..nul])
        .unwrap_or(bytes);
    String::from_utf16_lossy(bytes)
}

#[global_allocator]
static WIN32_ALLOCATOR: Heapalloc = Heapalloc;

pub struct Heapalloc;

unsafe impl GlobalAlloc for Heapalloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HeapAlloc(GetProcessHeap(), 0, layout.size()) as *mut u8
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        HeapFree(GetProcessHeap(), 0, ptr as *mut c_void);
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ptr as *mut c_void,
            new_size,
        ) as *mut u8
    }
}

/// Sets up the needed functions for the NSIS plugin dll,
/// like `main`, `panic` and `mem*` extern functions
#[macro_export]
macro_rules! nsis_plugin {
    () => {
        #[no_mangle]
        extern "system" fn DllMain(
            dll_module: ::windows_sys::Win32::Foundation::HINSTANCE,
            call_reason: u32,
            _: *mut (),
        ) -> bool {
            true
        }

        #[cfg(not(test))]
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            unsafe { ::windows_sys::Win32::System::Threading::ExitProcess(u32::MAX) }
        }

        #[no_mangle]
        pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: isize) -> *mut u8 {
            let mut i = 0;
            while i < n {
                *dest.offset(i) = *src.offset(i);
                i += 1;
            }
            return dest;
        }

        #[no_mangle]
        pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: isize) -> i32 {
            let mut i = 0;
            while i < n {
                let a = *s1.offset(i);
                let b = *s2.offset(i);
                if a != b {
                    return a as i32 - b as i32;
                }
                i += 1;
            }
            return 0;
        }

        #[no_mangle]
        pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: isize) -> *mut u8 {
            let mut i = 0;
            while i < n {
                *s.offset(i) = c as u8;
                i += 1;
            }
            return s;
        }
    };
}
