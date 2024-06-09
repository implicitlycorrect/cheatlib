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

pub mod patternscan;

#[cfg(all(windows, feature = "minhook"))]
pub mod minhook;

#[cfg(windows)]
pub mod windows;

pub mod keyboard;

pub mod process;

pub mod utilities;

#[cfg(all(windows, feature = "internal"))]
pub fn disable_thread_library_calls(module: HMODULE) -> bool {
    use windows_sys::Win32::System::LibraryLoader::DisableThreadLibraryCalls;

    unsafe { DisableThreadLibraryCalls(module) > 0 }
}

#[cfg(all(windows, feature = "internal"))]
pub fn allocate_console() -> bool {
    use windows_sys::Win32::System::Console::AllocConsole;
    unsafe { AllocConsole() > 0 }
}

#[cfg(all(windows, feature = "internal"))]
pub fn set_console_title(title: &str) -> bool {
    use windows_sys::Win32::System::Console::SetConsoleTitleA;
    unsafe { SetConsoleTitleA(make_lpcstr(title)) > 0 }
}

#[cfg(all(windows, feature = "internal"))]
pub fn deallocate_console() -> bool {
    use windows_sys::Win32::System::Console::FreeConsole;
    unsafe { FreeConsole() > 0 }
}

pub static ALLOCATE_CONSOLE: bool = cfg!(debug_assertions);

#[macro_export]
#[cfg(all(windows, feature = "internal"))]
macro_rules! dll_main {
    ($main:expr) => {
        use cheatlib::*;

        unsafe extern "system" fn entry(_module: *mut c_void) -> u32 {
            let console_allocated = ALLOCATE_CONSOLE && allocate_console();

            let result = std::panic::catch_unwind(|| unsafe {
                if let Err(error) = $main() {
                    eprintln!("Fatal error occured: {error:?}");
                }
            });

            if let Err(error) = result {
                eprintln!("Fatal error occured: {error:?}");
            }

            std::thread::sleep(std::time::Duration::from_secs(2));

            if console_allocated {
                deallocate_console();
            }

            0
        }

        static mut THREAD_HANDLE: HANDLE = 0;

        #[no_mangle]
        #[allow(non_snake_case, unused_variables)]
        extern "system" fn DllMain(
            dll_module: HINSTANCE,
            call_reason: u32,
            _reserved: *mut c_void,
        ) -> BOOL {
            const DLL_PROCESS_ATTACH: u32 = 1;
            const DLL_PROCESS_DETACH: u32 = 0;

            disable_thread_library_calls(dll_module);

            match call_reason {
                DLL_PROCESS_ATTACH => unsafe { THREAD_HANDLE = create_thread(dll_module, entry) },
                DLL_PROCESS_DETACH => unsafe {
                    if THREAD_HANDLE > 0 {
                        close_handle(THREAD_HANDLE);
                    }
                },
                _ => {}
            }
            1
        }
    };
}
