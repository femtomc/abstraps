use crate::core::SupportsInterfaceTraits;
use crate::{bail, Report};
use yansi::Paint;

pub trait AutomaticAllocationScope {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        Ok(())
    }
}
