use crate::core::diagnostics::LocationInfo;
use crate::core::interfaces::*;
use crate::core::region::Region;
use alloc::string::String;
use alloc::vec::Vec;
use color_eyre::Report;
use downcast_rs::{impl_downcast, Downcast};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Var(usize);

impl Var {
    pub fn new(id: usize) -> Var {
        Var(id)
    }

    pub fn get_id(&self) -> usize {
        self.0
    }
}

pub trait Intrinsic: Downcast + Object + ObjectClone {
    fn get_namespace(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_unique_id(&self) -> String {
        format!("{}.{}", self.get_namespace(), self.get_name())
    }
    fn verify(
        &self,
        boxed: &Box<dyn Intrinsic>,
        op: &dyn SupportsInterfaceTraits,
    ) -> Result<(), Report>;
}
impl_downcast!(Intrinsic);
mopo!(dyn Intrinsic);

#[macro_export]
macro_rules! intrinsic {
    ($struct:ident, $namespace:literal, $name:literal) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
        pub struct $struct;

        impl Intrinsic for $struct {
            fn get_namespace(&self) -> &str {
                return $namespace;
            }

            fn get_name(&self) -> &str {
                return $name;
            }

            fn verify(&self, _boxed: &Box<dyn Intrinsic>, _op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
                Ok(())
            }
        }

        interfaces!($struct: dyn ObjectClone, dyn Intrinsic);
    };

    ($struct:ident, $namespace:literal, $name:literal, $($trait:ident),+) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
        pub struct $struct;

        $(impl $trait for $struct {})*

        impl Intrinsic for $struct {
            fn get_namespace(&self) -> &str {
                return $namespace;
            }

            fn get_name(&self) -> &str {
                return $name;
            }

            fn verify(&self, boxed: &Box<dyn Intrinsic>, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
                $(boxed.query_ref::<dyn $trait>().unwrap().verify(op)?;)*
                Ok(())
            }
        }

        interfaces!($struct: dyn ObjectClone,
            dyn Intrinsic,
            $(dyn $trait),*);
    };
}

pub trait Attribute: Object + std::fmt::Display {}
mopo!(dyn Attribute);

pub trait AttributeValue<T> {
    fn get_value(&self) -> &T;
    fn get_value_mut(&mut self) -> &mut T;
}

pub trait SupportsInterfaceTraits: std::fmt::Display {
    fn get_intrinsic(&self) -> &Box<dyn Intrinsic>;
    fn get_regions(&self) -> &[Region];
    fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>>;
    fn get_attributes_mut(&mut self) -> &mut HashMap<String, Box<dyn Attribute>>;
}

#[derive(Debug)]
pub struct Operation {
    location: LocationInfo,
    intrinsic: Box<dyn Intrinsic>,
    operands: Vec<Var>,
    attributes: HashMap<String, Box<dyn Attribute>>,
    regions: Vec<Region>,
    successors: Vec<BasicBlock>,
}

impl Hash for Operation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
        self.intrinsic.hash(state);
        self.operands.hash(state);
        self.regions.hash(state);
        self.successors.hash(state);
    }
}

impl SupportsInterfaceTraits for Operation {
    fn get_intrinsic(&self) -> &Box<dyn Intrinsic> {
        &self.intrinsic
    }

    fn get_regions(&self) -> &[Region] {
        &self.regions
    }

    fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>> {
        &self.attributes
    }

    fn get_attributes_mut(&mut self) -> &mut HashMap<String, Box<dyn Attribute>> {
        &mut self.attributes
    }
}

impl Operation {
    pub fn new(
        location: LocationInfo,
        intrinsic: Box<dyn Intrinsic>,
        operands: Vec<Var>,
        attributes: HashMap<String, Box<dyn Attribute>>,
        regions: Vec<Region>,
        successors: Vec<BasicBlock>,
    ) -> Operation {
        Operation {
            location,
            intrinsic,
            operands,
            attributes,
            regions,
            successors,
        }
    }

    pub fn get_location(&self) -> &LocationInfo {
        &self.location
    }

    pub fn get_operands(&self) -> Vec<Var> {
        self.operands.to_vec()
    }

    pub fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>> {
        &self.attributes
    }

    pub fn get_attributes_mut(&mut self) -> &mut HashMap<String, Box<dyn Attribute>> {
        &mut self.attributes
    }
}

#[derive(Debug, Hash)]
pub struct BasicBlock {
    operands: Vec<Var>,
    ops: Vec<Operation>,
}

impl Default for BasicBlock {
    fn default() -> BasicBlock {
        BasicBlock {
            ops: Vec::new(),
            operands: Vec::new(),
        }
    }
}

impl BasicBlock {
    pub fn get_ops(&self) -> &[Operation] {
        &self.ops
    }

    pub fn get_ops_mut(&mut self) -> &mut Vec<Operation> {
        &mut self.ops
    }

    pub fn get_operands(&self) -> &[Var] {
        &self.operands
    }

    pub fn get_operands_mut(&mut self) -> &mut Vec<Var> {
        &mut self.operands
    }
}

/////
///// Lowering.
/////

/// Defines the interfaces by which an AST can target
/// and lower to an IR of type `T`.
pub trait Lowering<T> {
    type IRBuilder;
    type Error;
    fn prepare(&self) -> Result<Self::IRBuilder, Self::Error>;
    fn build(&self, b: &mut Self::IRBuilder) -> Result<(), Self::Error>;
    fn lower(&self) -> Result<T, Self::Error>;
}
