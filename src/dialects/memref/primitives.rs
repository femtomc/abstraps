use crate::core::*;
use crate::*;

primitive! {
    Alloc: ["memref", "alloc"],
    [],
    extern: []
}

primitive! {
    Alloca: ["memref", "alloca"],
    [],
    extern: []
}

primitive! {
    Copyto: ["memref", "copyto"],
    [],
    extern: []
}

primitive! {
    Dealloc: ["memref", "dealloc"],
    [],
    extern: []
}
