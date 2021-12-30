#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(unused_must_use)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod mlir_tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_context() {
        unsafe {
            let ctx = mlirContextCreate();
            assert!(mlirContextEqual(ctx, ctx));
            mlirContextDestroy(ctx);
        }
    }

    #[test]
    fn test_stringref() {
        unsafe {
            let c_to_print = CString::new("Hello, world!").expect("CString::new failed");
            let _r = mlirStringRefCreateFromCString(c_to_print.as_ptr());
        }
    }

    #[test]
    fn test_location() {
        unsafe {
            let ctx = mlirContextCreate();
            mlirRegisterAllDialects(ctx);
            let loc = mlirLocationUnknownGet(ctx);
            let c_to_print = CString::new("newmod").expect("CString::new failed");
            let r = mlirStringRefCreateFromCString(c_to_print.as_ptr());
            let _opstate = mlirOperationStateGet(r, loc);
        }
    }

    #[test]
    fn test_get_inttype() {
        unsafe {
            let ctx = mlirContextCreate();
            let inttype = mlirIntegerTypeGet(ctx, 32);
            let bitwidth = mlirIntegerTypeGetWidth(inttype);
            let signless = mlirIntegerTypeIsSignless(inttype);
            mlirContextDestroy(ctx);
        }
    }

    #[test]
    fn test_get_voidtype() {
        unsafe {
            let ctx = mlirContextCreate();
            let _inttype = mlirIntegerTypeGet(ctx, 32);
            let _voidtype = mlirLLVMVoidTypeGet(ctx);
            mlirContextDestroy(ctx);
        }
    }

    #[test]
    fn test_try_getcontext() {
        unsafe {
            let ctx = mlirContextCreate();
            let llvm = mlirGetDialectHandle__llvm__();
            mlirDialectHandleRegisterDialect(llvm, ctx);
            mlirDialectHandleLoadDialect(llvm, ctx);
            let num_dialects = mlirContextGetNumRegisteredDialects(ctx);
            mlirContextDestroy(ctx);
        }
    }

    #[test]
    fn test_llvm_pointertype() {
        unsafe {
            let ctx = mlirContextCreate();
            let llvm = mlirGetDialectHandle__llvm__();
            mlirDialectHandleRegisterDialect(llvm, ctx);
            mlirDialectHandleLoadDialect(llvm, ctx);
            let inttype = mlirIntegerTypeGet(ctx, 32);
            let _pointertype = mlirLLVMPointerTypeGet(inttype, 32);
            mlirContextDestroy(ctx);
        }
    }

    #[test]
    fn test_llvm_functype() {
        unsafe {
            let ctx = mlirContextCreate();
            let llvm = mlirGetDialectHandle__llvm__();
            mlirDialectHandleRegisterDialect(llvm, ctx);
            mlirDialectHandleLoadDialect(llvm, ctx);
            let inttype = mlirIntegerTypeGet(ctx, 32);
            let _functype = mlirLLVMFunctionTypeGet(inttype, 2, [inttype, inttype].as_ptr(), false);
            mlirContextDestroy(ctx);
        }
    }

    #[test]
    fn test_create_op() {
        unsafe {
            let ctx = mlirContextCreate();
            mlirContextSetAllowUnregisteredDialects(ctx, true);
            let llvm = mlirGetDialectHandle__llvm__();
            mlirDialectHandleRegisterDialect(llvm, ctx);
            mlirDialectHandleLoadDialect(llvm, ctx);
            let cstring = CString::new("new_func").expect("CString::new failed");
            let r = mlirStringRefCreateFromCString(cstring.as_ptr());
            let loc = mlirLocationUnknownGet(ctx);
            let mut state = mlirOperationStateGet(r, loc);
            let inttype = mlirIntegerTypeGet(ctx, 32);
            mlirOperationStateAddResults(&mut state, 1, [inttype].as_ptr());
            let _op = mlirOperationCreate(&mut state);
        }
    }
}
