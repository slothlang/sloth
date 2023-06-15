use rand::Rng;

use std::ffi::CString;

#[no_mangle]
pub extern "C" fn rand(a: i64, b: i64) -> i64 {
    let value = rand::thread_rng().gen_range(a..b);
    return value;
}

#[no_mangle]
pub unsafe extern "C" fn println(s: *const c_char) {
    let s = unsafe { CStr::from_ptr(s) }.to_str().unwrap();
    println("{s}");
}

#[no_mangle]
pub unsafe extern "C" fn print(s: *const c_char) {
    let s = unsafe { CStr::from_ptr(s) }.to_str().unwrap();
    print("{s}");
}
