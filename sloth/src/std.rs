use rand::Rng;

use std::ffi::CString;
use std::{thread, time};

// Random
#[no_mangle]
pub extern "C" fn rand(a: i64, b: i64) -> i64 {
    let value = rand::thread_rng().gen_range(a..b);
    return value;
}


// Print
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


// Terminal
#[no_mangle]
pub extern "C" fn termpos(x: i32, y: i32) {
    print!("\x1b[{x};{y}H");
}

#[no_mangle]
pub extern "C" fn termclear() {
    print!("\x1b[2J\x1b[H");
}


// Time
#[no_mangle]
pub extern "C" fn wait(t: i64) {
    thread::sleep(time::Duration::from_millis(t));
}


// RP2040
#[no_mangle]

