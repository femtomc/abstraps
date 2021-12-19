use crate::core::ir::{Intrinsic, Operation, SupportsVerification};
use crate::core::key::Key;
use anyhow;
use anyhow::bail;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct AnalysisManager {
    cached: HashMap<Box<dyn Key>, Box<dyn AnalysisPass>>,
}

impl AnalysisManager {
    pub fn new() -> AnalysisManager {
        AnalysisManager {
            cached: HashMap::new(),
        }
    }
}

pub trait RequestAnalysis<T> {
    fn ask(&self, op: &Operation, amgr: &mut AnalysisManager) -> anyhow::Result<T>;
}

pub trait AnalysisPass {
    fn apply(&mut self, op: &Operation) -> anyhow::Result<()>;
}

pub trait PassManager {
    fn check(&self, op: &Operation) -> bool;

    /// See the toplevel `Operation` first, and then
    /// moves downwards towards the leaves.
    fn prewalk(&mut self, op: &mut Operation) -> anyhow::Result<()>;

    /// See the leaves of the `Operation` tree first, and then
    /// moves upwards.
    fn postwalk(&mut self, _op: &mut Operation) -> anyhow::Result<()> {
        Ok(())
    }
}

pub trait OperationPass {
    fn target_intrinsic(&self) -> Option<Box<dyn Intrinsic>> {
        return None;
    }

    fn reset(&self) -> Box<dyn OperationPass>;

    fn apply(&self, op: &mut Operation, amgr: &AnalysisManager) -> anyhow::Result<()>;
}

pub struct OperationPassManager<T>
where
    T: Intrinsic,
{
    intrinsic_tag: PhantomData<T>,
    analysis: AnalysisManager,
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
            analysis: AnalysisManager::new(),
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

    fn prewalk(&mut self, op: &mut Operation) -> anyhow::Result<()> {
        Ok(())
    }
}

impl<T> OperationPassManager<T>
where
    T: Intrinsic,
{
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
