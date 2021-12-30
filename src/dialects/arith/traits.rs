use crate::core::{AttributeValue, SupportsInterfaceTraits, Var};
use crate::dialects::builtin::*;
use crate::{bail, Report};
use std::collections::HashMap;
use yansi::Paint;

pub trait Commutative {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        Ok(())
    }
}

pub trait Elementwise {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        Ok(())
    }
}

pub trait Broadcastable {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        Ok(())
    }
}
