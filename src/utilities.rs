use crate::*;

#[cfg(windows)]
pub fn is_page_readable(memory_info: &MEMORY_BASIC_INFORMATION) -> bool {
    use windows_sys::Win32::System::Memory::{MEM_COMMIT, PAGE_NOACCESS};

    !(memory_info.State != MEM_COMMIT
        || memory_info.Protect == 0x0
        || memory_info.Protect == PAGE_NOACCESS)
}

#[inline]
pub fn make_lpcstr(text: &str) -> *const u8 {
    format!("{}{}", text, "\0").as_ptr()
}
