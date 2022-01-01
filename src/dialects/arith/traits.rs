use crate::core::SupportsInterfaceTraits;
use crate::Report;

pub trait Commutative {
    fn verify(&self, _op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        Ok(())
    }
}

pub trait Elementwise {
    fn verify(&self, _op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        Ok(())
    }
}

pub trait Broadcastable {
    fn verify(&self, _op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        Ok(())
    }
}
