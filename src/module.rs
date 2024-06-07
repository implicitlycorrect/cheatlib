use crate::*;

#[cfg(feature = "internal")]
use {std::string::FromUtf8Error, utilities::read_null_terminated_string};

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

#[derive(Default, Debug)]
pub struct Module {
    pub handle: HMODULE,
    pub size: usize,
    pub base_address: usize,
}

impl Module {
    /// # Safety
    #[cfg(feature = "external")]
    pub unsafe fn get_module_data(&self) -> Vec<u8> {
        todo!()
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
        Self::from_handle(module_handle)
    }

    pub fn from_handle(module_handle: HMODULE) -> Result<Self> {
        let module_info = get_module_info(module_handle)?;

        let size = module_info.SizeOfImage as usize;
        let base_address = module_info.lpBaseOfDll as usize;

        Ok(Self {
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
    pub fn read<T>(&self, process_handle: HANDLE, address: usize, buffer: *mut T) -> Result<()> {
        page_operation!(
            address,
            PAGE_READWRITE,
            read_process_memory(process_handle, address, buffer, mem::size_of::<T>())?
        )
    }

    #[cfg(feature = "external")]
    pub fn write<T>(&self, process_handle: HANDLE, address: usize, value: T) -> Result<()> {
        let mut value = value;
        page_operation!(
            address,
            PAGE_READWRITE,
            write_process_memory(process_handle, address, &mut value, mem::size_of::<T>())?
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

    #[cfg(feature = "external")]
    pub fn read_string(&self, address: usize) -> Result<std::string::String, FromUtf8Error> {
        todo!()
    }

    #[cfg(feature = "internal")]
    pub fn read_string(&self, address: usize) -> Result<std::string::String, FromUtf8Error> {
        unsafe { read_null_terminated_string(self.handle as usize + address) }
    }
}
