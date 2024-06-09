use module::Module;

use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    Module32First, Module32Next, MODULEENTRY32,
};
#[cfg(windows)]
use windows_sys::Win32::System::{
    Diagnostics::ToolHelp::TH32CS_SNAPMODULE,
    Threading::{OpenProcess, PROCESS_ALL_ACCESS},
};

use crate::*;

pub struct Process {
    pub id: usize,
    pub handle: HANDLE,
    pub modules: Option<Vec<Module>>,
}

#[cfg(windows)]
impl Process {
    pub fn from_name(name: &str) -> Result<Process> {
        let Some(process_entry) = get_process_entry_by_name(name) else {
            return Err(anyhow!("process {name} not found"));
        };
        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, process_entry.th32ProcessID) };
        Ok(Self {
            id: process_entry.th32ProcessID as usize,
            handle,
            modules: get_modules(handle, process_entry.th32ProcessID as usize),
        })
    }

    pub fn get_module_by_name(&self, module_name: &str) -> Result<Module> {
        let Some(modules) = &self.modules else {
            return Err(anyhow!("process has no modules"));
        };

        for module in modules {
            if module.name == module_name {
                return Ok(module.clone());
            }
        }
        Err(anyhow!(
            "no module with name {module_name} found in process"
        ))
    }
}

fn get_modules(process_handle: HANDLE, process_id: usize) -> Option<Vec<Module>> {
    let snapshot = create_toolhelp32_snapshot(TH32CS_SNAPMODULE, process_id)?;

    let mut modules = vec![];

    unsafe {
        let mut entry = mem::zeroed::<MODULEENTRY32>();
        entry.dwSize = mem::size_of::<MODULEENTRY32>() as u32;

        if Module32First(snapshot, &mut entry) > 0 {
            modules.push(Module {
                name: String::from_utf8_lossy(&entry.szModule).to_string(),
                process_handle,
                handle: entry.hModule,
                size: entry.dwSize as usize,
                base_address: entry.modBaseAddr as usize,
            })
        }

        while Module32Next(snapshot, &mut entry) > 0 {
            modules.push(Module {
                name: String::from_utf8_lossy(&entry.szModule).to_string(),
                process_handle,
                handle: entry.hModule,
                size: entry.dwSize as usize,
                base_address: entry.modBaseAddr as usize,
            })
        }
    }

    close_handle(snapshot);

    Some(modules)
}
