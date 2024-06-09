use module::Module;

#[cfg(windows)]
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

use crate::*;

pub struct Process {
    pub id: u32,
    pub handle: HANDLE,
    pub modules: Vec<Module>,
}

#[cfg(windows)]
impl Process {
    pub fn from_name(name: &str) -> Result<Process> {
        let Some(entry) = get_process_entry_by_name(name) else {
            return Err(anyhow!("process {name} not found"));
        };
        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, entry.th32ProcessID) };
        Ok(Self {
            id: entry.th32ProcessID,
            handle,
            modules: get_process_modules((handle, entry.th32ProcessID)),
        })
    }

    pub fn get_module_by_name(&self, module_name: &str) -> Result<Module> {
        for module in self.modules.iter() {
            if module.name == module_name {
                return Ok(module.clone());
            }
        }
        Err(anyhow!(
            "no module with name {module_name} found in process"
        ))
    }
}
