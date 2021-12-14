use crate::core::builder::OperationBuilder;
use crate::core::ir::{Attribute, IntrinsicTrait, SupportsVerification};
use crate::dialects::builtin::attributes::{Symbol, SymbolTable};
use anyhow::bail;

#[derive(Debug, Clone, Copy)]
pub struct ProvidesSymbolTable;

impl IntrinsicTrait for ProvidesSymbolTable {
    fn verify(&self, op: &dyn SupportsVerification) -> anyhow::Result<()> {
        if !op.get_attributes().contains_key("symbols") {
            bail!("Operation attribute map does not contain the `symbols` key.")
        }
        let attr = op.get_attributes().get("symbols").unwrap();
        match attr.downcast_ref::<SymbolTable>() {
            Some(v) => Ok(()),
            None => bail!("The attribute value indexed by `symbols` is not a `SymbolTable`."),
        }
    }

    fn get_attribute_mut<'a>(
        &self,
        b: &'a mut OperationBuilder,
    ) -> anyhow::Result<&'a mut Box<dyn Attribute>> {
        match b.get_attributes_mut().get_mut("symbols") {
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
            Some(v) => Ok(()),
            None => bail!("The attribute value indexed by `symbol` is not a `Symbol`."),
        }
    }
}
