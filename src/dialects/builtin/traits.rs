use crate::core::builder::OperationBuilder;
use crate::core::ir::{Attribute, IntrinsicTrait, Operation, SupportsVerification};
use crate::dialects::builtin::attributes::{Symbol, SymbolTable};
use anyhow::bail;

#[derive(Debug, Clone, Copy)]
pub struct ProvidesSymbolTable;

impl IntrinsicTrait for ProvidesSymbolTable {
    fn verify(&self, op: &dyn SupportsVerification) -> anyhow::Result<()> {
        if !op.get_attributes().contains_key("symbols") {
            bail!("Operation attributes map does not contain the `symbols` key.")
        }
        let attr = op.get_attributes().get("symbols").unwrap();
        match attr.downcast_ref::<SymbolTable>() {
            Some(_v) => Ok(()),
            None => bail!("The attribute value indexed by `symbols` is not a `SymbolTable`."),
        }
    }

    fn get_attribute_mut<'a>(
        &self,
        op: &'a mut Operation,
    ) -> anyhow::Result<&'a mut Box<dyn Attribute>> {
        match op.get_attributes_mut().get_mut("symbols") {
            None => bail!("Failed to get `symbols` key in operation attributes map."),
            Some(v) => Ok(v),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProvidesSymbol;

impl IntrinsicTrait for ProvidesSymbol {
    fn verify(&self, op: &dyn SupportsVerification) -> anyhow::Result<()> {
        if !op.get_attributes().contains_key("symbol") {
            bail!("Operation attribute map does not contain the `symbol` key.")
        }
        let attr = op.get_attributes().get("symbol").unwrap();
        match attr.downcast_ref::<Symbol>() {
            Some(_v) => Ok(()),
            None => bail!("The attribute value indexed by `symbol` is not a `Symbol`."),
        }
    }

    fn get_attribute<'a>(&self, op: &'a Operation) -> anyhow::Result<&'a Box<dyn Attribute>> {
        match op.get_attributes().get("symbol") {
            None => bail!("Failed to get `symbol` key in operation attributes map."),
            Some(v) => Ok(v),
        }
    }

    fn get_attribute_mut<'a>(
        &self,
        op: &'a mut Operation,
    ) -> anyhow::Result<&'a mut Box<dyn Attribute>> {
        match op.get_attributes_mut().get_mut("symbol") {
            None => bail!("Failed to get `symbol` key in operation attributes map."),
            Some(v) => Ok(v),
        }
    }
}
