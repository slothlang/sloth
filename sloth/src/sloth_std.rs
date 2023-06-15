use std::ffi::{c_char, CStr};

use rand::Rng;

#[no_mangle]
pub extern "C" fn rand(a: i64, b: i64) -> i64 {
    rand::thread_rng().gen_range(a..b)
}

#[no_mangle]
/// # Safety
///
/// Function is unsafe if passed pointer is not a valid CStr
pub unsafe extern "C" fn println(s: *const c_char) {
    let s = unsafe { CStr::from_ptr(s) }.to_str().unwrap();
    println!("{s}");
}

#[no_mangle]
/// # Safety
///
/// Function is unsafe if passed pointer is not a valid CStr
pub unsafe extern "C" fn print(s: *const c_char) {
    let s = unsafe { CStr::from_ptr(s) }.to_str().unwrap();
    print!("{s}");
}
