use crate::*;

#[cfg(windows)]
use windows_sys::Win32::{Foundation::FARPROC, System::LibraryLoader::GetProcAddress};

macro_rules! page_operation {
    ($address:expr, $protect:expr, $operation:expr) => {{
        let memory_info = virtual_query($address as *const ())?;

        let mut old_protect = $protect;
        let is_readable = is_page_readable(&memory_info);
        if !is_readable {
            virtual_protect($address as *const (), $protect, &mut old_protect)?;
        }

        let result = $operation;

        if !is_readable {
            virtual_protect($address as *const (), old_protect, &mut old_protect)?;
        }

        Ok(result)
    }};
}

#[derive(Default, Clone, Debug)]
pub struct Module {
    pub name: String,
    pub process_handle: HANDLE,
    pub handle: HMODULE,
    pub size: usize,
    pub base_address: usize,
}

impl Module {
    #[cfg(feature = "external")]
    pub fn get_module_data(&self) -> Result<Vec<u8>> {
        let mut data: Vec<u8> = Vec::with_capacity(self.size);
        self.read(self.base_address, data.as_mut_ptr() as *mut Vec<u8>)?;
        Ok(data)
    }

    /// # Safety
    #[cfg(feature = "internal")]
    pub unsafe fn get_module_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.size);
        let data_pointer = data.as_mut_ptr();
        data.set_len(self.size);
        ptr::copy(self.base_address as *const u8, data_pointer, self.size);
        data
    }
}

#[cfg(windows)]
impl Module {
    #[cfg(feature = "internal")]
    pub fn from_name(name: &str) -> Result<Self> {
        let module_handle = get_module_handle(name)?;
        Self::from_handle(None, module_handle)
    }

    pub fn from_handle(process_handle: Option<HANDLE>, module_handle: HMODULE) -> Result<Self> {
        let module_info = get_module_info(module_handle)?;

        let size = module_info.SizeOfImage as usize;
        let base_address = module_info.lpBaseOfDll as usize;

        Ok(Self {
            name: String::new(),
            process_handle: process_handle.unwrap_or_default(),
            handle: module_handle,
            size,
            base_address,
        })
    }

    #[inline]
    pub fn get_function_address(&self, function_name: &str) -> FARPROC {
        unsafe { GetProcAddress(self.handle, make_lpcstr(function_name)) }
    }

    #[cfg(feature = "external")]
    pub fn read<T>(&self, address: usize, buffer: *mut T) -> Result<()> {
        page_operation!(
            address,
            PAGE_READWRITE,
            read_process_memory(self.process_handle, address, buffer, mem::size_of::<T>())?
        )
    }

    #[cfg(feature = "external")]
    pub fn write<T>(&self, address: usize, value: T) -> Result<()> {
        let mut value = value;
        page_operation!(
            address,
            PAGE_READWRITE,
            write_process_memory(
                self.process_handle,
                address,
                &mut value,
                mem::size_of::<T>()
            )?
        )
    }

    #[cfg(feature = "internal")]
    pub fn read<T>(&self, address: usize) -> Result<*mut T> {
        page_operation!(
            address,
            PAGE_READONLY,
            (self.base_address + address) as *mut T
        )
    }

    #[cfg(feature = "internal")]
    pub fn write<T>(&self, address: usize, value: T) -> Result<()> {
        page_operation!(address, PAGE_READWRITE, unsafe {
            *((self.base_address + address) as *mut T) = value
        })
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        close_handle(self.handle);
    }
}
