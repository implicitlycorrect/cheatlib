pub use {
    crate::module::Module,
    crate::process::Process,
    anyhow::anyhow,
    std::ffi::{c_char, c_void, CStr, CString},
    std::mem,
    std::ptr,
    utilities::*,
};

pub type Error = anyhow::Error;
pub type Result<T, E = Error> = anyhow::Result<T, E>;

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

/// This macro defines a DLL entry point (`DllMain`) for a Windows DLL.
///
/// # Parameters
/// - `$main`: The main function to be executed when the DLL is attached.
///
/// # Usage
/// ```
/// dll_main!(my_main_function);
/// ```
/// This will define a `DllMain` function that runs `my_main_function` in a new thread when the DLL is attached.
#[macro_export]
#[cfg(all(windows, feature = "internal"))]
macro_rules! dll_main {
    ($main:expr) => {
        use cheatlib::{c_void, disable_thread_library_calls, BOOL, HINSTANCE};

        #[no_mangle]
        #[allow(non_snake_case, unused_variables)]
        extern "system" fn DllMain(
            dll_module: HINSTANCE,
            call_reason: u32,
            _reserved: *mut c_void,
        ) -> BOOL {
            const DLL_PROCESS_ATTACH: u32 = 1;
            if call_reason == DLL_PROCESS_ATTACH {
                disable_thread_library_calls(dll_module);
                $main();
            }
            TRUE
        }
    };
}
