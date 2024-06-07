use module::Module;

//#[cfg(target_os = "windows")]
//use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

use crate::*;

pub struct Process {
    pub id: u32,
    pub handle: HANDLE,
}

impl Process {
    pub fn from_name(_process_name: &str) -> Result<Process> {
        todo!()
    }

    pub fn get_module_by_name(_module_name: &str) -> Result<Module> {
        todo!()
    }
}
