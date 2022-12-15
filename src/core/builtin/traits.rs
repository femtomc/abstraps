use crate::core::{AttributeValue, Region, SupportsInterfaceTraits, Var};
use crate::{bail, Report};
use std::collections::HashMap;
use yansi::Paint;

pub trait Terminator {
    fn verify(&self, _op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        Ok(())
    }
}

pub trait RequiresTerminators {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        for r in op.get_regions().iter() {
            for (ind, _) in r.get_blocks().iter().enumerate() {
                match r.get_block_iter(ind).last() {
                    None => bail!(format!("Block {} is empty in {}.", ind, op.get_primitive())),
                    Some((v, term)) => match term.get_primitive().query_ref::<dyn Terminator>() {
                        None => bail!(format!(
                            "{} is not {} traited, so is not a valid terminator.\n\n{}\n=> In {} at ({}, {}).",
                            term.get_primitive(),
                            Paint::magenta("Terminator").bold(),
                            op,
                            op.get_primitive(),
                            Paint::white(format!("{}", ind)).bold(),
                            v
                        )),
                        Some(_) => (),
                    },
                };
            }
        }
        Ok(())
    }
}

pub trait FunctionLike: ProvidesSymbolAttr {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        if op.get_regions().len() != 1 {
            bail!(format!(
                "{} has multiple regions, which is illegal for {} trait holders.",
                op.get_primitive(),
                Paint::magenta("FunctionLike").bold()
            ))
        }
        match op.get_regions()[0] {
            Region::Directed(_) => Ok(()),
            _ => bail!(format!(
                "For {} trait holders, the region type must be {}",
                Paint::magenta("FunctionLike").bold(),
                Paint::magenta("SSACFG")
            )),
        }
    }
}

// This is an example of an "extern" interface which requires
// a user-defined method (here: `verify`) when defining
// their primitives.
pub trait NonVariadic {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report>;
}
