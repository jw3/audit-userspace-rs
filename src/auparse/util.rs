use auparse_sys::{auparse_find_field, auparse_get_field_int, auparse_state_t};
use std::ffi::CString;

pub unsafe fn auparse_get_int(au: *mut auparse_state_t, field: &str) -> i32 {
    let str = CString::new(field).expect("CString");
    let tpid = auparse_find_field(au, str.as_ptr());
    if !tpid.is_null() {
        auparse_get_field_int(au) as i32
    } else {
        -1
    }
}
