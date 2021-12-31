use crate::core::{AttributeValue, Region, SupportsInterfaceTraits, Var};
use crate::dialects::builtin::*;
use crate::{bail, Report};
use std::collections::HashMap;
use yansi::Paint;

pub trait ConstantLike {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        if !op.get_attributes().contains_key("value") {
            bail!(format!(
                "{} must provide a {} key to satisfy the {} trait.",
                op.get_intrinsic(),
                Paint::magenta("value"),
                Paint::magenta("ConstantLike").bold()
            ))
        }
        let obj = op.get_attributes().get("value").unwrap();
        match obj.query_ref::<dyn AttributeValue<ConstantAttr>>() {
            None => bail!(format!("The attribute indexed by {} does not provide valid (can be coerced to) {}, which is required to be a valid {}.",
                    Paint::magenta("value"),
                    Paint::magenta("ConstantAttr").bold(),
                    Paint::magenta("ConstantLike").bold(),
            )),
            Some(_v) => Ok(()),
        }
    }
}

pub trait ProvidesSymbolTable {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        if !op.get_attributes().contains_key("symbols") {
            bail!(format!(
                "{} must provide a {} key to satisfy the {} trait.",
                op.get_intrinsic(),
                Paint::magenta("symbols"),
                Paint::magenta("ProvidesSymbolTable").bold()
            ))
        }
        let obj = op.get_attributes().get("symbols").unwrap();
        match obj.query_ref::<dyn AttributeValue<HashMap<String, Var>>>() {
            None => bail!(format!("The attribute indexed by {} does not provide a `HashMap<String, Var>` value, which is required to be a valid {}.",
                    Paint::magenta("symbols"),
                    Paint::magenta("SymbolTable").bold(),
            )),
            Some(_v) => Ok(()),
        }
    }

    fn get_value<'a>(&self, op: &'a dyn SupportsInterfaceTraits) -> &'a HashMap<String, Var> {
        let obj = op.get_attributes().get("symbols").unwrap();
        let attr_val = obj
            .query_ref::<dyn AttributeValue<HashMap<String, Var>>>()
            .unwrap();
        attr_val.get_value()
    }

    fn get_value_mut<'a>(
        &self,
        op: &'a mut dyn SupportsInterfaceTraits,
    ) -> &'a mut HashMap<String, Var> {
        let obj = op.get_attributes_mut().get_mut("symbols").unwrap();
        let attr_val = obj
            .query_mut::<dyn AttributeValue<HashMap<String, Var>>>()
            .unwrap();
        attr_val.get_value_mut()
    }
}

pub trait ProvidesSymbol {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        if !op.get_attributes().contains_key("symbol") {
            bail!("Operation attribute map does not contain the `symbol` key.")
        }
        let obj = op.get_attributes().get("symbol").unwrap();
        match obj.query_ref::<dyn AttributeValue<String>>() {
            None => bail!("The attribute value indexed by `symbol` is not a `Symbol`."),
            Some(_v) => Ok(()),
        }
    }

    fn get_value<'a>(&self, op: &'a dyn SupportsInterfaceTraits) -> &'a String {
        let obj = op.get_attributes().get("symbol").unwrap();
        let attr_val = obj.query_ref::<dyn AttributeValue<String>>().unwrap();
        attr_val.get_value()
    }

    fn get_value_mut<'a>(&self, op: &'a mut dyn SupportsInterfaceTraits) -> &'a mut String {
        let obj = op.get_attributes_mut().get_mut("symbol").unwrap();
        let attr_val = obj.query_mut::<dyn AttributeValue<String>>().unwrap();
        attr_val.get_value_mut()
    }
}

pub trait Terminator {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        Ok(())
    }
}

pub trait RequiresTerminators {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        for r in op.get_regions().iter() {
            for (ind, blk) in r.get_blocks().iter().enumerate() {
                match r.get_block_iter(ind).last() {
                    None => bail!(format!("Block {} is empty in {}.", ind, op.get_intrinsic())),
                    Some((v, term)) => match term.get_intrinsic().query_ref::<dyn Terminator>() {
                        None => bail!(format!(
                            "{} is not {} traited, so is not a valid terminator.\n\n{}\n=> In {} at ({}, {}).",
                            term.get_intrinsic(),
                            Paint::magenta("Terminator").bold(),
                            op,
                            op.get_intrinsic(),
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

pub trait ProvidesLinkage {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        if !op.get_attributes().contains_key("linkage") {
            bail!("Operation attribute map does not contain the `linkage` key.")
        }
        let obj = op.get_attributes().get("linkage").unwrap();
        match obj.query_ref::<dyn AttributeValue<LinkageAttr>>() {
            None => bail!("The attribute value indexed by `linkage` is not a `LinkageAttr`."),
            Some(_v) => Ok(()),
        }
    }

    fn get_value<'a>(&self, op: &'a dyn SupportsInterfaceTraits) -> &'a LinkageAttr {
        let obj = op.get_attributes().get("linkage").unwrap();
        let attr_val = obj.query_ref::<dyn AttributeValue<LinkageAttr>>().unwrap();
        attr_val.get_value()
    }
}

pub trait FunctionLike: ProvidesSymbol {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        if op.get_regions().len() != 1 {
            bail!(format!(
                "{} has multiple regions, which is illegal for {} trait holders.",
                op.get_intrinsic(),
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
// their intrinsics.
pub trait NonVariadic {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report>;
}
