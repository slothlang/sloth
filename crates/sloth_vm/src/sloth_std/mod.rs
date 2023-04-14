use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::native::NativeFunction;

pub mod rand;
pub mod stdio;

pub static NATIVE_LIBRARY: Lazy<HashMap<&'static str, NativeFunction>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // rand
    map.insert("rand$gen", rand::GEN_FUNCTION);
    map.insert("rand$gen_range", rand::GEN_RANGE_FUNCTION);

    // stdio
    map.insert("write", stdio::WRITE_FUNCTION);
    map.insert("read", stdio::READ_FUNCTION);

    // filesystem

    map
});
