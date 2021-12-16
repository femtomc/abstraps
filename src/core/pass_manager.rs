use crate::core::ir::{Intrinsic, Operation, SupportsVerification};
use anyhow;

use std::marker::PhantomData;

pub trait OperationPass {
    fn apply(&self, op: &mut Operation) -> anyhow::Result<()>;

    fn check_valid(&self, _op: &Operation) -> bool {
        true
    }
}

pub trait IntrinsicPass<T>: OperationPass
where
    T: Intrinsic,
{
    fn check_valid(&self, op: &Operation) -> bool {
        return op.get_intrinsic().is::<T>();
    }
}

pub trait PassManager {
    fn prewalk(&mut self, op: &mut Operation) -> anyhow::Result<()>;
    fn postwalk(&mut self, op: &mut Operation) -> anyhow::Result<()>;
    fn push(&mut self, pass: Box<dyn OperationPass>) -> anyhow::Result<()>;
    fn nest(&mut self, pass: Box<dyn PassManager>) -> anyhow::Result<()>;
}

struct OperationPassManager<T>
where
    T: Intrinsic,
{
    passes: Vec<Box<dyn OperationPass>>,
    managers: Vec<Box<dyn PassManager>>,
    intrinsic_type: PhantomData<T>,
}

impl<T> OperationPassManager<T>
where
    T: Intrinsic,
{
    fn new() -> OperationPassManager<T> {
        OperationPassManager {
            passes: Vec::new(),
            managers: Vec::new(),
            intrinsic_type: PhantomData,
        }
    }
}

impl<T> PassManager for OperationPassManager<T>
where
    T: Intrinsic,
{
    /// See the toplevel `Operation` first, and then
    /// moves downwards towards the leaves.
    fn prewalk(&mut self, _op: &mut Operation) -> anyhow::Result<()> {
        Ok(())
    }

    /// See the leaves of the `Operation` tree first, and then
    /// moves upwards.
    fn postwalk(&mut self, _op: &mut Operation) -> anyhow::Result<()> {
        Ok(())
    }

    fn push(&mut self, pass: Box<dyn OperationPass>) -> anyhow::Result<()> {
        Ok(())
    }

    fn nest(&mut self, pass: Box<dyn PassManager>) -> anyhow::Result<()> {
        Ok(())
    }
}
