use crate::*;

#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::GetLastError,
    System::{
        Diagnostics::Debug::ReadProcessMemory,
        LibraryLoader::GetModuleHandleA,
        Memory::{VirtualProtect, VirtualQuery},
        ProcessStatus::GetModuleInformation,
        Threading::GetCurrentProcess,
    },
};

#[cfg(windows)]
pub use windows_sys::Win32::{
    Foundation::{HANDLE, HMODULE},
    System::{
        Memory::{
            MEMORY_BASIC_INFORMATION, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
            PAGE_READONLY, PAGE_READWRITE,
        },
        ProcessStatus::MODULEINFO,
    },
};

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
        )
        .is_positive()
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
    use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;

    let success = unsafe {
        WriteProcessMemory(
            process_handle,
            address as *const c_void,
            buffer as *mut c_void,
            size,
            ptr::null_mut(),
        )
    }
    .is_positive();

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

/// # Safety
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
    }
    .is_negative()
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
pub fn virtual_protect(target: *const (), new_protect: u32, old_protect: &mut u32) -> Result<()> {
    if unsafe {
        VirtualProtect(
            target as *const c_void,
            mem::size_of::<*const c_void>(),
            new_protect,
            old_protect,
        )
    }
    .is_negative()
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
