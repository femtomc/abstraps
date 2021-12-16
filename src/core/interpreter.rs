use crate::core::ir::{Operation};



use anyhow;
use downcast_rs::{impl_downcast, Downcast};

pub trait LatticeElement: Downcast
where
    Self: std::fmt::Debug,
{
    fn get_namespace(&self) -> &str;
}
impl_downcast!(LatticeElement);

pub trait LatticeSemantics<I, L>
where
    L: LatticeElement,
{
    fn propagate(&self, op: Operation) -> anyhow::Result<L>;
}

pub trait AbstractInterpreter<I, L, R>
where
    Self: LatticeSemantics<I, L>,
    L: LatticeElement,
{
    type InterpreterFrame;
    type InterpreterState;
    fn prepare(frame: Self::InterpreterFrame) -> Self;
    fn step(&mut self) -> anyhow::Result<Self::InterpreterState>;
    fn finish(self) -> anyhow::Result<R>;
}
