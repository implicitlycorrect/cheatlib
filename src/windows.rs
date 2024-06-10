use crate::*;

use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    Module32First, Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE,
};
#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::GetLastError,
    System::{
        Diagnostics::{
            Debug::{ReadProcessMemory, WriteProcessMemory},
            ToolHelp::{
                CreateToolhelp32Snapshot, Process32First, Process32Next,
                CREATE_TOOLHELP_SNAPSHOT_FLAGS, TH32CS_SNAPPROCESS,
            },
        },
        LibraryLoader::GetModuleHandleA,
        Memory::{VirtualAllocEx, VirtualProtect, VirtualQuery, MEM_COMMIT, MEM_RESERVE},
        ProcessStatus::GetModuleInformation,
        Threading::{CreateRemoteThread, CreateThread, GetCurrentProcess},
    },
};

#[cfg(windows)]
pub use windows_sys::Win32::{
    Foundation::{
        CloseHandle, BOOL, FALSE, HANDLE, HINSTANCE, HMODULE, INVALID_HANDLE_VALUE, TRUE,
    },
    System::{
        Diagnostics::ToolHelp::PROCESSENTRY32,
        Memory::{
            MEMORY_BASIC_INFORMATION, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
            PAGE_READONLY, PAGE_READWRITE,
        },
        ProcessStatus::MODULEINFO,
    },
};

#[cfg(windows)]
pub fn create_toolhelp32_snapshot(
    flags: CREATE_TOOLHELP_SNAPSHOT_FLAGS,
    process_id: u32,
) -> Option<HANDLE> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(flags, process_id) };
    if snapshot == INVALID_HANDLE_VALUE {
        return None;
    }
    Some(snapshot)
}

#[cfg(windows)]
pub fn get_process_entry_by_name(name: &str) -> Option<PROCESSENTRY32> {
    let snapshot = create_toolhelp32_snapshot(TH32CS_SNAPPROCESS, 0)?;

    let mut entry = unsafe { mem::zeroed::<PROCESSENTRY32>() };
    entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

    if unsafe { Process32First(snapshot, &mut entry) } <= 0 {
        return None;
    }

    let found = loop {
        let process_name = unsafe { read_null_terminated_string(entry.szExeFile.as_ptr()) };
        if process_name == name {
            break true;
        }

        if unsafe { Process32Next(snapshot, &mut entry) } <= 0 {
            break false;
        }
    };

    close_handle(snapshot);

    if found {
        Some(entry)
    } else {
        None
    }
}

pub fn get_process_modules(process: (HANDLE, u32)) -> Vec<Module> {
    let mut modules = vec![];

    let (process_handle, process_id) = process;
    let Some(snapshot) = create_toolhelp32_snapshot(TH32CS_SNAPMODULE, process_id) else {
        return modules;
    };

    let mut entry = unsafe { mem::zeroed::<MODULEENTRY32>() };
    entry.dwSize = mem::size_of::<MODULEENTRY32>() as u32;

    if unsafe { Module32First(snapshot, &mut entry) } == FALSE {
        return modules;
    }

    loop {
        modules.push(Module {
            name: unsafe { read_null_terminated_string(entry.szModule.as_ptr()) }.to_string(),
            process_handle,
            handle: entry.hModule,
            size: entry.dwSize as usize,
            base_address: entry.modBaseAddr as usize,
        });

        if unsafe { Module32Next(snapshot, &mut entry) } == FALSE {
            break;
        }
    }

    close_handle(snapshot);

    modules
}

#[cfg(windows)]
pub fn create_thread(
    module: HMODULE,
    function: unsafe extern "system" fn(*mut c_void) -> u32,
) -> HANDLE {
    unsafe {
        CreateThread(
            ptr::null_mut(),
            0,
            Some(function),
            module as *const c_void,
            0,
            ptr::null_mut(),
        )
    }
}

#[cfg(windows)]
pub fn close_handle(handle: HANDLE) -> bool {
    (unsafe { CloseHandle(handle) }) > 0
}

#[cfg(windows)]
pub fn read_process_memory<T>(
    process_handle: isize,
    address: usize,
    buffer: *mut T,
    size: usize,
) -> Result<()> {
    let success = unsafe {
        ReadProcessMemory(
            process_handle,
            address as *const c_void,
            buffer as *mut c_void,
            size,
            ptr::null_mut(),
        ) > 0
    };
    if !success {
        let error_code = unsafe { GetLastError() };
        let error_message = format!(
            "ReadProcessMemory failed for target: {:#0x}. Error code: {}. Description: {}",
            address,
            error_code,
            std::io::Error::from_raw_os_error(error_code as i32)
        );
        return Err(anyhow::anyhow!(error_message));
    }
    Ok(())
}

#[cfg(windows)]
pub fn write_process_memory<T>(
    process_handle: isize,
    address: usize,
    buffer: *mut T,
    size: usize,
) -> Result<()> {
    let success = unsafe {
        WriteProcessMemory(
            process_handle,
            address as *const c_void,
            buffer as *mut c_void,
            size,
            ptr::null_mut(),
        )
    } > 0;

    if !success {
        let error_code = unsafe { GetLastError() };
        let error_message = format!(
            "WriteProcessMemory failed for target: {:#0x}. Error code: {}. Description: {}",
            address,
            error_code,
            std::io::Error::from_raw_os_error(error_code as i32)
        );
        return Err(anyhow::anyhow!(error_message));
    }
    Ok(())
}

#[cfg(windows)]
pub fn get_module_handle(name: &str) -> Result<HMODULE> {
    let module_name = make_lpcstr(name);
    let module_handle = unsafe { GetModuleHandleA(module_name) };
    if module_handle <= 0 {
        let error_code = unsafe { GetLastError() };
        let error_message = format!(
            "GetModuleHandleA failed. Error code: {}. Description: {}",
            error_code,
            std::io::Error::from_raw_os_error(error_code as i32)
        );
        return Err(anyhow::anyhow!(error_message));
    }
    Ok(module_handle)
}

#[cfg(windows)]
pub fn get_module_info(module: HMODULE) -> Result<MODULEINFO> {
    let mut module_info = unsafe { mem::zeroed::<MODULEINFO>() };
    if unsafe {
        GetModuleInformation(
            GetCurrentProcess(),
            module as HMODULE,
            &mut module_info,
            mem::size_of::<MODULEINFO>() as u32,
        )
    } == FALSE
    {
        let error_code = unsafe { GetLastError() };
        let error_message = format!(
            "GetModuleInformation failed. Error code: {}. Description: {}",
            error_code,
            std::io::Error::from_raw_os_error(error_code as i32)
        );
        return Err(anyhow::anyhow!(error_message));
    }
    Ok(module_info)
}

/// # Safety
#[cfg(windows)]
pub unsafe fn create_remote_thread(
    process_handle: HANDLE,
    start_address: *const c_void,
) -> Result<HANDLE> {
    type StartRoutine =
        unsafe extern "system" fn(lpthreadparameter: *mut ::core::ffi::c_void) -> u32;

    let thread_handle = CreateRemoteThread(
        process_handle,
        ptr::null_mut(),
        0,
        Some(mem::transmute::<*const c_void, StartRoutine>(start_address)),
        ptr::null_mut(),
        0,
        ptr::null_mut(),
    );
    if thread_handle <= 0 {
        return Err(anyhow!("Failed to create remote thread"));
    }
    Ok(thread_handle)
}

/// # Safety
#[cfg(windows)]
pub unsafe fn get_module_data(module_info: MODULEINFO) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(module_info.SizeOfImage as usize);
    let data_ptr = data.as_mut_ptr();
    data.set_len(0);
    ptr::copy_nonoverlapping(
        module_info.lpBaseOfDll as *const u8,
        data_ptr,
        module_info.SizeOfImage as usize,
    );
    data.set_len(module_info.SizeOfImage as usize);
    data
}

#[cfg(windows)]
pub fn virtual_alloc_ex(process_handle: HANDLE, size: usize) -> Result<*mut c_void> {
    let remote_memory = unsafe {
        VirtualAllocEx(
            process_handle,
            ptr::null_mut(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };

    if remote_memory.is_null() {
        return Err(anyhow!("Failed to allocate memory in remote process"));
    }
    Ok(remote_memory)
}

#[cfg(windows)]
pub fn virtual_protect(target: *const (), new_protect: u32, old_protect: &mut u32) -> Result<()> {
    if unsafe {
        VirtualProtect(
            target as *const c_void,
            mem::size_of::<*const c_void>(),
            new_protect,
            old_protect,
        )
    } == FALSE
    {
        let error_code = unsafe { GetLastError() };
        let error_message = format!(
            "VirtualProtect failed. Error code: {}. Description: {}",
            error_code,
            std::io::Error::from_raw_os_error(error_code as i32)
        );
        return Err(anyhow::anyhow!(error_message));
    }
    Ok(())
}

#[cfg(windows)]
pub fn virtual_query(target: *const ()) -> Result<MEMORY_BASIC_INFORMATION> {
    let mut memory_info: MEMORY_BASIC_INFORMATION =
        unsafe { mem::zeroed::<MEMORY_BASIC_INFORMATION>() };
    let result = unsafe {
        VirtualQuery(
            target as *const c_void,
            &mut memory_info,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    if result == 0 {
        let error_code = unsafe { GetLastError() };
        let error_message = format!(
            "VirtualQuery failed for target: {:p}. Error code: {}. Description: {}",
            target,
            error_code,
            std::io::Error::from_raw_os_error(error_code as i32)
        );
        return Err(anyhow::anyhow!(error_message));
    }
    Ok(memory_info)
}
