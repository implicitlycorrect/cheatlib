#[cfg(all(target_os = "windows", feature = "minhook"))]
use {crate::*, minhook_sys::MH_OK};

/// Initialize the MinHook library. You must call this function **EXACTLY ONCE** at the end of your program.
#[cfg(all(target_os = "windows", feature = "minhook"))]
pub fn initialize() -> Result<()> {
    let status = unsafe { minhook_sys::MH_Initialize() };
    if status != MH_OK {
        return Err(anyhow!(format!(
            "Error occured when initializing minhook, error code: {:?}",
            status
        )));
    }
    Ok(())
}

/// Uninitialize the MinHook library. You must call this function **EXACTLY ONCE** at the end of your program.
#[cfg(all(target_os = "windows", feature = "minhook"))]
pub fn uninitialize() -> Result<()> {
    let status = unsafe { minhook_sys::MH_Uninitialize() };
    if status != MH_OK {
        return Err(anyhow!(format!(
            "Error occured when uninitializing minhook, error code: {:?}",
            status
        )));
    }
    Ok(())
}

/// Enables all already created hooks.
///
/// This function returns a result indicating wheter the operation finished successfully or not.  
#[cfg(all(target_os = "windows", feature = "minhook"))]
pub fn enable_hooks() -> Result<()> {
    enable_hook(ptr::null_mut())
}

/// Enables an already created hook.
///
/// # Parameters:
///
/// * `target` - A pointer to the target function. If this parameter is null, all created hooks are disabled in one go.
///
/// This function returns a result indicating wheter the operation finished successfully or not.
#[cfg(all(target_os = "windows", feature = "minhook"))]
pub fn enable_hook(target: *mut ()) -> Result<()> {
    let status = unsafe { minhook_sys::MH_EnableHook(target as *mut c_void) };
    if status != MH_OK {
        return Err(anyhow!(format!(
            "Error occured when enabling hook {:#0x}, status code: {:?}",
            target as usize, status
        )));
    }
    Ok(())
}

/// Disables all already created hooks.
///
/// This function returns a result indicating wheter the operation finished successfully or not.  
#[cfg(all(target_os = "windows", feature = "minhook"))]
pub fn disable_hooks() -> Result<()> {
    disable_hook(ptr::null_mut())
}

/// Disables an already created hook.
///
/// # Parameters:
///
/// * `target` - A pointer to the target function. If this parameter is null, all created hooks are disabled in one go.
///
/// This function returns a result indicating wheter the operation finished successfully or not.
#[cfg(all(target_os = "windows", feature = "minhook"))]
pub fn disable_hook(target: *mut ()) -> Result<()> {
    let status = unsafe { minhook_sys::MH_DisableHook(target as *mut c_void) };
    if status != MH_OK {
        return Err(anyhow!(format!(
            "Error occured when disabling hook {:#0x}, status code: {:?}",
            target as usize, status
        )));
    }
    Ok(())
}

/// Creates a hook for the specified API function, in disabled state.
///
/// # Parameters:
///
/// * `target` - A pointer to the target function, which will be overridden by the detour function.
/// * `detour` - A pointer to the detour function, which will override the target function.
///
/// This function returns a pointer to the trampoline function, which will be used to call the original target function. This parameter can be null.      
#[cfg(all(target_os = "windows", feature = "minhook"))]
pub fn create_hook(target: *mut (), detour: *mut ()) -> Result<*mut c_void> {
    let memory_info = virtual_query(target)?;
    let is_executable = memory_info.Protect == PAGE_EXECUTE_READWRITE;
    let mut old_protect = 0;
    if !is_executable {
        virtual_protect(target, PAGE_EXECUTE_READWRITE, &mut old_protect)?;
    }

    let mut original_function = ptr::null_mut();
    let status = unsafe {
        minhook_sys::MH_CreateHook(
            target as *mut c_void,
            detour as *mut c_void,
            &mut original_function,
        )
    };
    if status != MH_OK {
        return Err(anyhow!(format!(
            "Error occured while creating hook for function {:#0x}, status code: {}",
            target as usize, status
        )));
    }

    if !is_executable {
        virtual_protect(target, old_protect, &mut old_protect)?;
    }

    Ok(original_function)
}
