use clang_sys::*;
use std::ffi::CStr;
use std::ffi::{CString, c_void};
use std::path::{Path, PathBuf};

pub fn from_string<S: AsRef<str>>(string: S) -> CString {
    CString::new(string.as_ref()).expect("invalid C string")
}

pub fn from_path<P: AsRef<Path>>(path: P) -> CString {
    from_string(
        path.as_ref()
            .as_os_str()
            .to_str()
            .expect("invalid C string"),
    )
}

// Nullable ______________________________________

/// A type which may be null or otherwise invalid.
pub trait Nullable: Sized {
    fn map<U, F: FnOnce(Self) -> U>(self, f: F) -> Option<U>;
}

impl Nullable for *mut c_void {
    fn map<U, F: FnOnce(*mut c_void) -> U>(self, f: F) -> Option<U> {
        if !self.is_null() { Some(f(self)) } else { None }
    }
}

pub fn to_string(clang: CXString) -> String {
    unsafe {
        let c = CStr::from_ptr(clang_getCString(clang));
        let rust = c.to_str().expect("invalid Rust string").into();
        clang_disposeString(clang);
        rust
    }
}


pub fn to_path(clang: CXString) -> PathBuf {
    let rust_string = unsafe { to_string(clang) };
    PathBuf::from(rust_string)
}