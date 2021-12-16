use crate::core::ir::{Intrinsic, Operation};
use anyhow;
use anyhow::bail;
use std::marker::PhantomData;

pub trait OperationPass {
    fn apply(&self, op: &mut Operation) -> anyhow::Result<()>;

    fn check_valid(&self, op: &Operation) -> bool {
        true
    }
}

pub trait IntrinsicPass<T>: OperationPass
where
    T: Intrinsic,
{
    fn get_intrinsic(&self) -> Option<Box<dyn Intrinsic>> {
        return None;
    }

    fn apply(&self, op: &mut Operation) -> anyhow::Result<()>;

    fn check_valid(&self, op: &Operation) -> bool {
        match self.get_intrinsic() {
            None => true,
            Some(v) => v.is::<T>(),
        }
    }
}

pub trait PassManager {
    fn prewalk(&self, op: &mut Operation) -> anyhow::Result<()>;
    fn postwalk(&self, op: &mut Operation) -> anyhow::Result<()>;
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
    fn prewalk(&self, op: &mut Operation) -> anyhow::Result<()> {
        Ok(())
    }

    /// See the leaves of the `Operation` tree first, and then
    /// moves upwards.
    fn postwalk(&self, op: &mut Operation) -> anyhow::Result<()> {
        Ok(())
    }
}
