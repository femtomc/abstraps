use crate::core::interfaces::*;
use crate::core::ir::{Intrinsic, Operation, SupportsInterfaceTraits};
use color_eyre::{eyre::bail, Report};
use downcast_rs::{impl_downcast, Downcast};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;

pub trait AnalysisKey: Downcast + Object {
    fn to_pass(&self, op: &Operation) -> Box<dyn AnalysisPass>;
}
mopo!(dyn AnalysisKey);
impl_downcast!(AnalysisKey);

pub trait AnalysisPass: Downcast + Object {
    fn apply(&mut self, op: &Operation) -> Result<(), Report>;
}
mopo!(dyn AnalysisPass);
impl_downcast!(AnalysisPass);

/// `AnalysisManager` is a type which manages
/// static analyses of operations, often required
/// for `OperationPass` application.
///
/// Analyses can be computed lazily (on demand) by operation
/// passes owned by a `PassManager`.
///
/// During `apply` calls, all operations passes are provided
/// with a `Sender` channel (to place requests for analyses),
/// as well as a read-write locked `AnalysisManager`,
/// which the pass can use to ask for the result
pub struct AnalysisManager {
    cached: HashMap<Box<dyn AnalysisKey>, Box<dyn AnalysisPass>>,
}

impl AnalysisManager {
    pub fn new() -> AnalysisManager {
        AnalysisManager {
            cached: HashMap::new(),
        }
    }

    pub fn get_cached(&self) -> &HashMap<Box<dyn AnalysisKey>, Box<dyn AnalysisPass>> {
        &self.cached
    }

    pub fn analyze<T>(&mut self, key: T, op: &Operation) -> Result<(), Report>
    where
        T: 'static + Eq + Hash + AnalysisKey,
    {
        let mut pass = key.to_pass(op);
        pass.apply(op)?;
        self.cached.insert(Box::new(key), pass);
        Ok(())
    }

    pub fn ask(&self, key: Box<dyn AnalysisKey>) -> Option<&Box<dyn AnalysisPass>> {
        if !self.cached.contains_key(&key) {
            return None;
        }
        return Some(self.cached.get(&key).unwrap());
    }
}

pub trait PassManager
where
    Self: std::fmt::Display,
{
    /// Check if the pass manager can apply passes to operations
    /// of a specific intrinsic type.
    fn check(&self, op: &Operation) -> bool;

    /// See the toplevel `Operation` first, and then
    /// moves downwards towards the leaves.
    fn prewalk(self, op: Operation) -> Result<Operation, Report>;
}

pub trait OperationPass: Send + Sync + std::fmt::Debug {
    fn target_intrinsic(&self) -> Option<Box<dyn Intrinsic>> {
        None
    }

    fn reset(&self) -> Box<dyn OperationPass>;

    /// Check if the `OperationPass` can be applied to this `Operation`.
    fn check(&self, op: &RwLock<Operation>) -> Result<(), Report>;

    /// Apply the `OperationPass` to the operation. The semantics
    /// of this function can generally include mutating the operation.
    ///
    /// Access to the operation is provided by a `RwLock<Operation>`,
    /// the `OperationPass` can also access the `AnalysisManager`
    /// through another `RwLock`.
    fn apply(
        &self,
        op: &RwLock<Operation>,
        analysis_manager: &RwLock<AnalysisManager>,
    ) -> Result<(), Report>;
}

pub struct OperationPassManager<T>
where
    T: Intrinsic,
{
    intrinsic_tag: T,
    passes: Vec<Box<dyn OperationPass>>,
    managers: Vec<Box<dyn PassManager>>,
    analysis_manager: Option<AnalysisManager>,
}

impl<T> OperationPassManager<T>
where
    T: Intrinsic,
{
    pub fn new(intr: T) -> OperationPassManager<T> {
        let analysis_manager = AnalysisManager::new();
        OperationPassManager {
            intrinsic_tag: intr,
            passes: Vec::new(),
            managers: Vec::new(),
            analysis_manager: Some(analysis_manager),
        }
    }

    pub fn get_intrinsic(&self) -> &T {
        &self.intrinsic_tag
    }
}

impl<T> PassManager for OperationPassManager<T>
where
    T: Intrinsic,
{
    fn check(&self, op: &Operation) -> bool {
        op.get_intrinsic().is::<T>()
    }

    fn prewalk(mut self, op: Operation) -> Result<Operation, Report> {
        if !self.check(&op) {
            bail!("Operation intrinsic type is not the same as pass manager.".to_string())
        }
        let analysis_manager = self.analysis_manager.take().unwrap();
        let analysis_lock = RwLock::new(analysis_manager);
        let op_lock = RwLock::new(op);
        for pass in self.get_passes().iter() {
            pass.apply(&op_lock, &analysis_lock)?;
        }
        Ok(op_lock.into_inner().unwrap())
    }
}

impl<T> OperationPassManager<T>
where
    T: Intrinsic,
{
    pub fn get_managers(&self) -> &[Box<dyn PassManager>] {
        &self.managers
    }

    pub fn get_passes(&self) -> &[Box<dyn OperationPass>] {
        &self.passes
    }

    pub fn get_passes_mut(&mut self) -> &mut Vec<Box<dyn OperationPass>> {
        &mut self.passes
    }

    pub fn push(&mut self, pass: Box<dyn OperationPass>) -> Result<(), Report> {
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

    pub fn nest(&mut self, mgr: Box<dyn PassManager>) -> Result<(), Report> {
        self.managers.push(mgr);
        Ok(())
    }
}
