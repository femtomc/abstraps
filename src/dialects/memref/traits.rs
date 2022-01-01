use crate::core::SupportsInterfaceTraits;
use crate::Report;

pub trait AutomaticAllocationScope {
    fn verify(&self, _op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        Ok(())
    }
}
