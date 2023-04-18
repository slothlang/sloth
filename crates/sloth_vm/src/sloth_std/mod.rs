use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::native::NativeFunction;

pub mod file;
pub mod misc;
pub mod rand;
pub mod stdio;
pub mod term;
pub mod time;

pub static NATIVE_LIBRARY: Lazy<HashMap<&'static str, NativeFunction>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // rand
    map.insert("rand$gen", rand::GEN_FUNCTION);
    map.insert("rand$gen_range", rand::GEN_RANGE_FUNCTION);

    // stdio
    map.insert("write", stdio::WRITE_FUNCTION);
    map.insert("writeln", stdio::WRITELN_FUNCTION);
    map.insert("read", stdio::READ_FUNCTION);

    // term
    map.insert("term$clear", term::TERM_CLEAR);
    map.insert("term$setpos", term::TERM_SETPOS);

    // filesystem
    // TODO: Make the files commands work by making a global file variable with
    // certain permissions created by 'file.open' instead of just reading the file.
    map.insert("file$read", file::FILE_READ);
    map.insert("file$write", file::FILE_WRITE);

    // time
    map.insert("wait", time::WAIT);

    map
});
