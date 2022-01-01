use crate::core::*;
use crate::*;

intrinsic! {
    Alloc: ["memref", "alloc"],
    [],
    extern: []
}

intrinsic! {
    Alloca: ["memref", "alloca"],
    [],
    extern: []
}

intrinsic! {
    Copyto: ["memref", "copyto"],
    [],
    extern: []
}

intrinsic! {
    Dealloc: ["memref", "dealloc"],
    [],
    extern: []
}
