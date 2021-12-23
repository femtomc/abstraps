use crate::core::{AttributeValue, SupportsInterfaceTraits, Var};
use crate::{bail, Report};
use std::collections::HashMap;

pub trait ProvidesSymbolTable {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        if !op.get_attributes().contains_key("symbols") {
            bail!("Operation attributes map does not contain the `symbols` key.")
        }
        let obj = op.get_attributes().get("symbols").unwrap();
        match obj.query_ref::<dyn AttributeValue<HashMap<String, Var>>>() {
            None => bail!("The attribute value indexed by `symbols` is not a `SymbolTable`."),
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

pub trait Terminator {}
pub trait RequiresTerminators {
    fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
        for r in op.get_regions().iter() {
            for (ind, blk) in r.get_blocks().iter().enumerate() {
                match blk.get_ops().last() {
                    None => bail!(format!("Block {} is empty.", ind)),
                    Some(v) => match v.get_intrinsic().query_ref::<dyn Terminator>() {
                        None => bail!(format!(
                            "The intrinsic {} is not `Terminator` traited.",
                            v.get_intrinsic()
                        )),
                        Some(_) => (),
                    },
                };
            }
        }
        Ok(())
    }
}
