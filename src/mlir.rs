/*

   This file is part of Abstrap. License is MIT.

*/

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

/*

   The builder design in this module
   supports code generation to MLIR dialects.

   The builder implements a collection of generic (and useful)
   MLIR munging capabilities. It also exports a set of traits
   which are used to setup generic code generation functionality.

*/

use crate::ir::{ExtIR, Instruction, Var};
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub enum BuilderError {
    FailedOperationVerification,
    FailedToCodegenInstruction,
    FailedToGenerateLLVMConstantOperation,
    FailedToConvertTypeToMLIRType,
    FailedToGetOperationResult,
    FailedToLookupTypeForVar,
    NoRuleForIntrinsic,
    Caseless,
}

pub struct MLIRBuilder<G> {
    global: Option<Arc<RwLock<G>>>,
    ctx: MlirContext,
    local_map: HashMap<Var, MlirValue>,
    toplevel: MlirModule,
    blocks: Vec<MlirBlock>,
    insertion: usize,

    // This is very stupid -- but whenever
    // `StringRef` instances are created,
    // they do not own the underlying CString.
    // So we have to keep the CString alive
    // for the unsafe reference.
    //
    // When `finish` is called -- this field is "cleaned"
    // (re-initialized, because then it's safe).
    cstring_keep: Vec<CString>,
}

impl<G> Default for MLIRBuilder<G> {
    fn default() -> MLIRBuilder<G> {
        let ctx = unsafe {
            let ctx = mlirContextCreate();
            let llvm = mlirGetDialectHandle__llvm__();
            mlirDialectHandleRegisterDialect(llvm, ctx);
            mlirDialectHandleLoadDialect(llvm, ctx);
            mlirContextSetAllowUnregisteredDialects(ctx, true);
            ctx
        };
        let module = unsafe {
            let loc = mlirLocationUnknownGet(ctx);
            mlirModuleCreateEmpty(loc)
        };
        return MLIRBuilder {
            global: None,
            ctx: ctx,
            local_map: HashMap::new(),
            toplevel: module,
            blocks: Vec::new(),
            insertion: 0,
            cstring_keep: Vec::new(),
        };
    }
}

impl<G> MLIRBuilder<G> {
    pub fn get_sref(&mut self, s: &str) -> MlirStringRef {
        return unsafe {
            let cstring = CString::new(s).expect("CString::new failed.");
            let sr = mlirStringRefCreateFromCString(cstring.as_ptr());
            self.cstring_keep.push(cstring);
            sr
        };
    }

    pub fn get_id(&mut self, sref: MlirStringRef) -> MlirIdentifier {
        unsafe { mlirIdentifierGet(self.ctx, sref) }
    }

    pub fn parse_type(&mut self, s: &str) -> MlirType {
        return unsafe {
            let sr = self.get_sref(s);
            mlirTypeParseGet(self.ctx, sr)
        };
    }

    pub fn get_unknown_loc(&mut self) -> MlirLocation {
        return unsafe { mlirLocationUnknownGet(self.ctx) };
    }

    pub fn new_module(&mut self, loc: MlirLocation) -> MlirModule {
        return unsafe { mlirModuleCreateEmpty(loc) };
    }

    pub fn new_region(&mut self) -> MlirRegion {
        return unsafe { mlirRegionCreate() };
    }

    pub fn get_blk_arg(&mut self, blk: MlirBlock, pos: isize) -> Option<MlirValue> {
        unsafe {
            let n_args = mlirBlockGetNumArguments(blk);
            if pos <= n_args - 1 {
                return Some(mlirBlockGetArgument(blk, pos));
            } else {
                return None;
            }
        }
    }

    pub fn add_region(&mut self, state: &mut MlirOperationState, reg: &MlirRegion) {
        unsafe { mlirOperationStateAddOwnedRegions(state, 1, reg) }
    }

    pub fn add_blk(&mut self, region: MlirRegion, blk: MlirBlock) {
        unsafe {
            mlirRegionAppendOwnedBlock(region, blk);
        }
    }

    pub fn new_blk(&mut self, args: Vec<MlirType>) -> MlirBlock {
        let n_args = args.len() as isize;
        return unsafe { mlirBlockCreate(n_args, args.as_ptr()) };
    }

    pub fn add_op_to_blk(&mut self, op: MlirOperation, blk: MlirBlock) {
        unsafe { mlirBlockAppendOwnedOperation(blk, op) }
    }

    pub fn get_module_body(&mut self, mo: MlirModule) -> MlirBlock {
        unsafe { mlirModuleGetBody(mo) }
    }

    pub fn module_append(&mut self, mo: MlirModule, op: MlirOperation) {
        unsafe {
            let body = mlirModuleGetBody(mo);
            mlirBlockAppendOwnedOperation(body, op);
        }
    }

    pub fn new_state(&mut self, name: &str, loc: MlirLocation) -> MlirOperationState {
        return unsafe {
            let s = self.get_sref(name);
            mlirOperationStateGet(s, loc)
        };
    }

    pub fn get_ptr_type(&mut self, pointee: MlirType, address_space: u32) -> MlirType {
        unsafe { mlirLLVMPointerTypeGet(pointee, address_space) }
    }

    pub fn get_func_type(
        &mut self,
        rtype: MlirType,
        argtypes: Vec<MlirType>,
        is_vararg: bool,
    ) -> MlirType {
        let l = argtypes.len() as isize;
        unsafe { mlirLLVMFunctionTypeGet(rtype, l, argtypes.as_ptr(), is_vararg) }
    }

    pub fn get_type_attr(&mut self, t: MlirType) -> MlirAttribute {
        unsafe { mlirTypeAttrGet(t) }
    }

    pub fn get_symref_attr(&mut self, r: &str) -> MlirAttribute {
        unsafe {
            let s = self.get_sref(r);
            mlirFlatSymbolRefAttrGet(self.ctx, s)
        }
    }

    pub fn get_flatsymref_attr(&mut self, r: &str) -> MlirAttribute {
        unsafe {
            let s = self.get_sref(r);
            mlirFlatSymbolRefAttrGet(self.ctx, s)
        }
    }

    pub fn get_str_attr(&mut self, s: &str) -> MlirAttribute {
        unsafe {
            let r = self.get_sref(s);
            mlirStringAttrGet(self.ctx, r)
        }
    }

    pub fn get_nattr(&mut self, name: &str, attr: MlirAttribute) -> MlirNamedAttribute {
        let s = self.get_sref(name);
        let id = self.get_id(s);
        unsafe { mlirNamedAttributeGet(id, attr) }
    }

    pub fn add_nattrs(&mut self, state: &mut MlirOperationState, nattrs: Vec<MlirNamedAttribute>) {
        unsafe {
            let l = nattrs.len() as isize;
            mlirOperationStateAddAttributes(state, l, nattrs.as_ptr())
        }
    }

    pub fn get_integer_attr(&mut self, rt: MlirType, v: i64) -> MlirAttribute {
        unsafe { mlirIntegerAttrGet(rt, v) }
    }

    pub fn add_results(&mut self, state: &mut MlirOperationState, res: Vec<MlirType>) {
        let l = res.len() as isize;
        unsafe { mlirOperationStateAddResults(state, l, res.as_ptr()) }
    }

    pub fn add_operands(&mut self, state: &mut MlirOperationState, operands: Vec<MlirValue>) {
        let l = operands.len() as isize;
        unsafe { mlirOperationStateAddOperands(state, l, operands.as_ptr()) }
    }

    pub fn get_result(
        &mut self,
        operation: MlirOperation,
        pos: isize,
    ) -> Result<MlirValue, BuilderError> {
        unsafe {
            let l = mlirOperationGetNumResults(operation);
            if pos <= l {
                Ok(mlirOperationGetResult(operation, pos))
            } else {
                Err(BuilderError::FailedToGetOperationResult)
            }
        }
    }

    pub fn finish_no_verify(&mut self, state: &mut MlirOperationState) -> MlirOperation {
        return unsafe { mlirOperationCreate(state) };
    }

    pub fn finish(
        &mut self,
        state: &mut MlirOperationState,
    ) -> Result<MlirOperation, BuilderError> {
        return unsafe {
            let op = mlirOperationCreate(state);
            match mlirOperationVerify(op) {
                true => Ok(op),
                false => Err(BuilderError::FailedOperationVerification),
            }
        };
    }

    pub fn module_get_op_no_verify(&mut self, mo: MlirModule) -> MlirOperation {
        return unsafe { mlirModuleGetOperation(mo) };
    }

    pub fn module_get_op(&mut self, mo: MlirModule) -> Option<MlirOperation> {
        return unsafe {
            let op = mlirModuleGetOperation(mo);
            if mlirOperationVerify(op) {
                Some(op)
            } else {
                None
            }
        };
    }

    pub fn dump_op_no_verify(&mut self, op: MlirOperation) {
        unsafe { mlirOperationDump(op) }
    }

    pub fn dump_op(&mut self, op: MlirOperation) {
        unsafe {
            if mlirOperationVerify(op) {
                mlirOperationDump(op)
            }
        }
    }

    pub fn new_execution_engine(
        &mut self,
        op: MlirModule,
        opt_level: i32,
        shared_lib_paths: Vec<MlirStringRef>,
    ) -> MlirExecutionEngine {
        let l = shared_lib_paths.len() as i32;
        unsafe { mlirExecutionEngineCreate(op, opt_level, l, shared_lib_paths.as_ptr()) }
    }

    pub fn dump_execution_engine(&mut self, ee: MlirExecutionEngine, path: MlirStringRef) {
        unsafe { mlirExecutionEngineDumpToObjectFile(ee, path) }
    }
}

/////
///// Codegen interfaces for custom intrinsics/attributes.
/////

/*

   These interfaces support generic code generation
   implementations.

*/

// `Convert` allows conversion from a type lattice to `MlirType` instances.
pub trait Convert {
    fn convert<G>(builder: &mut MLIRBuilder<G>, t: &Self) -> Result<MlirType, BuilderError>;
}

// `Codegen` is implemented for intrinsics, and requires
// specification of the attribute set `A` and the lattice
// type set `T`.
pub trait Codegen<A, T>
where
    Self: Sized,
    T: Convert + Sized,
{
    fn codegen_instr<G>(
        b: &mut MLIRBuilder<G>,
        intr: &Instruction<Self, A>,
        rettype: &T,
    ) -> Result<Option<MlirValue>, BuilderError>;
}

impl<G> MLIRBuilder<G> {
    pub fn codegen_function<T, K, Ty>(
        &mut self,
        name: &str,
        ir: &ExtIR<T, K>,
        argtypes: &Vec<Ty>,
        analysis: &Vec<Ty>,
        rettype: &Ty,
    ) -> Result<MlirOperation, BuilderError>
    where
        Ty: Convert,
        T: Codegen<K, Ty>,
    {
        let loc = self.get_unknown_loc();
        let mut state = self.new_state("llvm.func", loc);
        let region = self.new_region();
        let args = argtypes
            .iter()
            .map(|x| Convert::convert(self, x))
            .collect::<Result<Vec<_>, _>>()?;
        for blk in 0..ir.blocks.len() {
            match blk {
                0 => {
                    let entry = self.new_blk(args.clone());
                    for (ind, v) in ir.blocks[0].args.iter().enumerate() {
                        let blkind = ind as isize;
                        let arg = self.get_blk_arg(entry, blkind).unwrap();
                        self.local_map.insert(v.clone(), arg);
                    }
                    for (v, instr) in ir.block_iter(0) {
                        let res = Codegen::codegen_instr(self, instr, rettype)?;
                        match res {
                            None => (),
                            Some(r) => {
                                self.local_map.insert(v.clone(), r);
                                ()
                            }
                        };
                    }
                    self.add_blk(region, entry);
                }
                _ => {
                    let args = ir.blocks[blk]
                        .args
                        .iter()
                        .map(|v| {
                            let t = match analysis.get(v.id) {
                                None => Err(BuilderError::FailedToLookupTypeForVar),
                                Some(t) => Ok(t),
                            }?;
                            Convert::convert(self, t)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let mlirblk = self.new_blk(args);
                    for (ind, v) in ir.blocks[0].args.iter().enumerate() {
                        let blkind = ind as isize;
                        let arg = self.get_blk_arg(mlirblk, blkind).unwrap();
                        self.local_map.insert(v.clone(), arg);
                    }
                    for (v, instr) in ir.block_iter(0) {
                        let res = Codegen::codegen_instr(self, instr, rettype)?;
                        match res {
                            None => (),
                            Some(r) => {
                                self.local_map.insert(v.clone(), r);
                                ()
                            }
                        };
                    }
                    self.add_blk(region, mlirblk);
                }
            }
        }
        let rettype = Convert::convert(self, rettype)?;
        let functype = self.get_func_type(rettype, args, false);
        let funcattr = self.get_type_attr(functype);
        let funcnattr = self.get_nattr("type", funcattr);
        let strattr = self.get_str_attr(name);
        let nstrattr = self.get_nattr("sym_name", strattr);
        self.add_nattrs(&mut state, vec![nstrattr, funcnattr]);
        self.add_region(&mut state, &region);
        Ok(self.finish_no_verify(&mut state))
    }
}
