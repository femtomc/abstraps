use crate::core::ir::{Intrinsic, Operation, SupportsVerification};
use anyhow;
use anyhow::bail;
use std::marker::PhantomData;

pub trait OperationPass {
    fn target_intrinsic(&self) -> Option<Box<dyn Intrinsic>> {
        return None;
    }

    fn reset(&self) -> Box<dyn OperationPass>;

    fn apply(&self, op: &mut Operation) -> anyhow::Result<()>;
}

pub trait PassManager {
    fn check(&self, op: &Operation) -> bool;

    /// See the toplevel `Operation` first, and then
    /// moves downwards towards the leaves.
    fn prewalk(&mut self, op: &mut Operation) -> anyhow::Result<()> {
        if self.check(op) {
            for pass in self.get_passes_mut().iter() {
                pass.apply(op);
            }
        }
        Ok(())
    }

    /// See the leaves of the `Operation` tree first, and then
    /// moves upwards.
    fn postwalk(&mut self, _op: &mut Operation) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_passes_mut(&mut self) -> &mut Vec<Box<dyn OperationPass>>;
    fn push(&mut self, pass: Box<dyn OperationPass>) -> anyhow::Result<()>;
    fn nest(&mut self, pass: Box<dyn PassManager>) -> anyhow::Result<()>;
}

pub struct OperationPassManager<T>
where
    T: Intrinsic,
{
    intrinsic_tag: PhantomData<T>,
    passes: Vec<Box<dyn OperationPass>>,
    managers: Vec<Box<dyn PassManager>>,
}

impl<T> OperationPassManager<T>
where
    T: Intrinsic,
{
    pub fn new() -> OperationPassManager<T> {
        OperationPassManager {
            intrinsic_tag: PhantomData,
            passes: Vec::new(),
            managers: Vec::new(),
        }
    }
}

impl<T> PassManager for OperationPassManager<T>
where
    T: Intrinsic,
{
    fn check(&self, op: &Operation) -> bool {
        op.get_intrinsic().is::<T>()
    }

    fn get_passes_mut(&mut self) -> &mut Vec<Box<dyn OperationPass>> {
        &mut self.passes
    }

    fn push(&mut self, pass: Box<dyn OperationPass>) -> anyhow::Result<()> {
        let intr = pass.target_intrinsic();
        match intr {
            None => self.passes.push(pass),
            Some(v) => match v.is::<T>() {
                false => bail!("Operation pass must operate on same intrinsic as pass manager."),
                true => self.passes.push(pass),
            },
        };
        Ok(())
    }

    fn nest(&mut self, mgr: Box<dyn PassManager>) -> anyhow::Result<()> {
        self.managers.push(mgr);
        Ok(())
    }
}
