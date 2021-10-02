use std::{ffi::OsStr, iter, os::windows::prelude::OsStrExt};

pub(crate) fn str_to_c16<T>(string: &T) -> Vec<u16>
where
    T: std::convert::AsRef<std::ffi::OsStr> + ?Sized,
{
    OsStr::new(string)
        .encode_wide()
        .chain(iter::once(0))
        .collect::<Vec<u16>>()
}
pub(crate) fn win32_to_hresult(code: u32) -> i32 {
    if code as i32 <= 0 {
        code as i32
    } else {
        ((code & 0x0000FFFF) | (7 << 16) | 0x80000000) as i32
    }
}
#[macro_export]
macro_rules! offset_of {
    ($Struct:path, $field:tt) => {{
        let s1 = MaybeUninit::<&$Struct>::uninit();
        let s = s1.as_ptr();
        let f = unsafe { std::ptr::addr_of!((*(s as *const $Struct)).$field) };
        (f as usize) - (s as usize)
    }};
}
#[macro_export]
macro_rules! release {
    ($($tt:tt)*) => {
        unsafe { (&*($($tt)* as *mut _ as *mut IUnknown)).Release() }
    };
}
