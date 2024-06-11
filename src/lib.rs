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

/// This macro defines a DLL entry point (`DllMain`) for a Windows DLL with options for 
/// allocating a console and creating a separate thread for the main function.
///
/// # Parameters
/// - `$main`: The main function to be executed when the DLL is attached.
///   It should return a `Result<()>` indicating success or an error.
/// - `$create_thread`: A boolean literal indicating whether to create a separate thread 
///   to run the main function.
///
/// # Usage
/// ```
/// dll_main!(my_main_function, true);
/// ```
/// This will define a `DllMain` function that runs 
/// `my_main_function` in a new thread when the DLL is attached.
#[macro_export]
#[cfg(all(windows, feature = "internal"))]
macro_rules! dll_main {
    ($main:expr, $create_thread:literal) => {
        use cheatlib::{allocate_console, c_void, close_handle, create_thread, deallocate_console, disable_thread_library_calls, BOOL, HINSTANCE};

        unsafe extern "system" fn entry(_module: *mut c_void) -> u32 {
            let result = std::panic::catch_unwind(|| {
                if let Err(error) = $main() {
                    eprintln!("Fatal error occurred: {:?}", error);
                }
            });

            if let Err(error) = result {
                eprintln!("Fatal error occurred: {:?}", error);
            }

            std::thread::sleep(std::time::Duration::from_secs(2));

            0
        }

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
                if $create_thread {
                    let thread_handle = create_thread(dll_module, entry);
                    if thread_handle > 0 {
                        close_handle(thread_handle);
                    }
                } else {
                    unsafe {
                        entry(ptr::null_mut());
                    }
                }
            }
            TRUE
        }
    };
}
