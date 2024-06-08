pub use {
    crate::module::Module,
    crate::process::Process,
    anyhow::anyhow,
    std::ffi::{c_char, c_void, CStr, CString},
    std::mem,
    std::ptr,
    utilities::*,
};

pub type Result<T, E = anyhow::Error> = anyhow::Result<T, E>;

#[cfg(windows)]
pub use windows::*;

pub mod module;

#[cfg(feature = "patternscan")]
pub mod patternscan;

#[cfg(all(windows, feature = "minhook"))]
pub mod minhook;

#[cfg(windows)]
pub mod windows;

pub mod keyboard;

pub mod process;

pub mod utilities;

#[cfg(all(windows, feature = "internal"))]
pub fn allocate_console() -> bool {
    unsafe { windows_sys::Win32::System::Console::AllocConsole().is_positive() }
}

#[cfg(all(windows, feature = "internal"))]
pub fn set_console_title(title: &str) -> bool {
    unsafe {
        windows_sys::Win32::System::Console::SetConsoleTitleA(make_lpcstr(title)).is_positive()
    }
}

#[cfg(all(windows, feature = "internal"))]
pub fn deallocate_console() -> bool {
    unsafe { windows_sys::Win32::System::Console::FreeConsole().is_positive() }
}

#[cfg(any(feature = "console", debug_assertions))]
pub static ALLOCATE_CONSOLE: bool = true;
#[cfg(not(any(feature = "console", debug_assertions)))]
pub static ALLOCATE_CONSOLE: bool = false;

#[macro_export]
#[cfg(all(windows, feature = "internal"))]
macro_rules! dll_main {
    ($main:expr) => {
        use cheatlib::*;

        #[no_mangle]
        #[allow(non_snake_case, unused_variables)]
        extern "system" fn DllMain(
            dll_module: HINSTANCE,
            call_reason: u32,
            _reserved: *mut c_void,
        ) -> BOOL {
            const DLL_PROCESS_ATTACH: u32 = 1;
            if call_reason == DLL_PROCESS_ATTACH {
                std::thread::spawn(|| unsafe {
                    if ALLOCATE_CONSOLE {
                        allocate_console();
                    }

                    match $main() {
                        Ok(()) => println!("exiting"),
                        Err(error) => eprintln!("error: {error}"),
                    }

                    std::thread::sleep(std::time::Duration::from_secs(1));

                    if ALLOCATE_CONSOLE {
                        deallocate_console();
                    }
                });
            }
            1
        }
    };
}
