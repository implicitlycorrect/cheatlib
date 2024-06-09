use std::borrow::Cow;

use crate::*;

#[cfg(windows)]
#[inline]
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

/// # Safety
pub unsafe fn read_null_terminated_string<'a>(address: *const u8) -> Cow<'a, str> {
    let len = (0..500).take_while(|&i| *address.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(address, len);

    String::from_utf8_lossy(slice)
}
