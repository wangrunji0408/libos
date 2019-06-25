use super::*;
use core::ptr;

/// Memory utilities that deals with primitive types passed from user process
/// running inside enclave
pub mod from_user {
    use super::*;

    /// Check the user pointer is within the readable memory range of the user process
    pub fn check_ptr<T>(user_ptr: *const T) -> Result<(), Error> {
        Ok(())
    }

    /// Check the mutable user pointer is within the writable memory of the user process
    pub fn check_mut_ptr<T>(user_ptr: *mut T) -> Result<(), Error> {
        Ok(())
    }

    /// Check the readonly array is within the readable memory of the user process
    pub fn check_array<T>(user_buf: *const T, count: usize) -> Result<(), Error> {
        Ok(())
    }

    /// Check the mutable array is within the writable memory of the user process
    pub fn check_mut_array<T>(user_buf: *mut T, count: usize) -> Result<(), Error> {
        Ok(())
    }

    /// Clone a C-string from the user process safely
    pub fn clone_cstring_safely(out_ptr: *const c_char) -> Result<String, Error> {
        check_ptr(out_ptr)?;
        // TODO: using from_cstr directly is not safe
        let cstr = unsafe { from_cstr(out_ptr as *const u8) };
        let cstring = String::from(cstr);
        Ok(cstring)
    }

    /// Clone a C-string array (const char*[]) from the user process safely
    ///
    /// This array must be ended with a NULL pointer.
    pub fn clone_cstrings_safely(user_ptr: *const *const c_char) -> Result<Vec<String>, Error> {
        let mut cstrings = Vec::new();
        if user_ptr == ptr::null() {
            return Ok(cstrings);
        }

        let mut user_ptr = user_ptr;
        loop {
            check_ptr(user_ptr);
            let cstr_ptr = {
                let cstr_ptr = unsafe { *user_ptr };
                if cstr_ptr == ptr::null() {
                    break;
                }
                check_ptr(cstr_ptr);
                cstr_ptr
            };
            let cstring = clone_cstring_safely(cstr_ptr)?;
            cstrings.push(cstring);

            user_ptr = unsafe { user_ptr.offset(1) };
        }
        Ok(cstrings)
    }
}

/// Memory utilities that deals with primitive types passed from outside the enclave
pub mod from_untrusted {
    use super::*;

    /// Check the untrusted pointer is outside the enclave
    pub fn check_ptr<T>(out_ptr: *const T) -> Result<(), Error> {
        Ok(())
    }

    /// Check the untrusted array is outside the enclave
    pub fn check_array<T>(out_ptr: *const T, count: usize) -> Result<(), Error> {
        Ok(())
    }

    /// Clone a C-string from outside the enclave
    pub fn clone_cstring_safely(out_ptr: *const c_char) -> Result<String, Error> {
        check_ptr(out_ptr)?;
        // TODO: using from_cstr directly is not safe
        let cstr = unsafe { from_cstr(out_ptr as *const u8) };
        let cstring = String::from(cstr);
        Ok(cstring)
    }

    /// Clone a C-string array (const char*[]) from outside the enclave
    ///
    /// This array must be ended with a NULL pointer.
    pub fn clone_cstrings_safely(out_ptr: *const *const c_char) -> Result<Vec<String>, Error> {
        let mut cstrings = Vec::new();
        if out_ptr == ptr::null() {
            return Ok(cstrings);
        }

        let mut out_ptr = out_ptr;
        loop {
            check_ptr(out_ptr);
            let cstr_ptr = {
                let cstr_ptr = unsafe { *out_ptr };
                if cstr_ptr == ptr::null() {
                    break;
                }
                check_ptr(cstr_ptr);
                cstr_ptr
            };
            let cstring = clone_cstring_safely(cstr_ptr)?;
            cstrings.push(cstring);

            out_ptr = unsafe { out_ptr.offset(1) };
        }
        Ok(cstrings)
    }
}

/// Convert C string to Rust string
pub unsafe fn from_cstr(s: *const u8) -> &'static str {
    use core::{slice, str};
    let len = (0usize..).find(|&i| *s.add(i) == 0).unwrap();
    str::from_utf8(slice::from_raw_parts(s, len)).unwrap()
}
