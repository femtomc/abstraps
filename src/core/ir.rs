use crate::core::diagnostics::LocationInfo;
use crate::core::interfaces::*;
use crate::core::region::Region;
use color_eyre::Report;
use downcast_rs::{impl_downcast, Downcast};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// A primitive SSA register.
///
/// Defines an index into operations at any nesting level.
#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Var(usize);

impl Var {
    /// Create a new `Var` instance.
    pub fn new(id: usize) -> Var {
        Var(id)
    }

    /// Get the internal `usize` index of a `Var` instance.
    pub fn get_id(&self) -> usize {
        self.0
    }
}

/// The core means of IR extension - defines
/// a computational unit with a particular (user-defined) semantics.
/// Practically, an `Intrinsic` is always instanced inside of an [`Operation`] (or as part of
/// [`crate::core::OperationBuilder`] IR construction)
/// and specifies the interface identity of that [`Operation`] in the IR.
///
/// `Intrinsic` instances are placed inside of [`Operation`] instances
/// (in the IR) and support customized (trait object) interfaces.
///
/// Users of the crate should likely use the [`intrinsic!`] declarative
/// macro to define new intrinsics.
pub trait Intrinsic: Downcast + Object + ObjectClone {
    fn get_namespace(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_unique_id(&self) -> String {
        format!("{}.{}", self.get_namespace(), self.get_name())
    }

    /// `verify` provides a mechanism by which an `Intrinsic`
    /// can defer the definition of verification of ad hoc traits/interfaces (e.g. trait objects)
    /// on [`Operation`]'s until the concrete implementor of `Intrinsic`
    /// is defined.
    ///
    /// Mostly, the user will never be required to define this interface
    /// directly, and should use the declarative [`intrinsic!`] macro
    /// (which will handle defining this method).
    fn verify(
        &self,
        boxed: &Box<dyn Intrinsic>,
        op: &dyn SupportsInterfaceTraits,
    ) -> Result<(), Report>;
}
impl_downcast!(Intrinsic);
mopo!(dyn Intrinsic);

/// A declarative interface for defining new [`Intrinsic`] implementors.
///
/// The syntax looks like the following:
/// ```ignore
/// intrinsic!(Foo: ["namespace", "name"],
///     [AnyTraitsWithAutoImplementations, ...],
///     extern: [AnyTraitsWhichRequireUserProvidedImplementations, ...]
///     )
/// ```
///
/// The trait declaration functionality (and usage of traits for verifying ad hoc properties of an
/// [`Operation`] containing an [`Intrinsic`]) follows the trait/interface
/// specification in MLIR:
/// <https://mlir.llvm.org/docs/Traits/>
///
/// Here, these are just Rust traits implemented on the [`Intrinsic`].
#[macro_export]
macro_rules! intrinsic {
    ($(#[$attr:meta])* $struct:ident:
     [$namespace:literal, $name:literal],
     [$($trait:ident),*],
     extern: [$($extr:ident),*]) => {
        $(#[$attr])*
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

            #[allow(unused_variables)]
            fn verify(&self, boxed: &Box<dyn Intrinsic>, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
                $($trait::verify(boxed.query_ref::<dyn $trait>().unwrap(), op)?;)*
                $($extr::verify(boxed.query_ref::<dyn $extr>().unwrap(), op)?;)*
                Ok(())
            }
        }

        interfaces!($struct: dyn ObjectClone,
            dyn Intrinsic
            $(,dyn $trait)*
            $(,dyn $extr)*);
    };
}

/// Constant metadata which can be attached to [`Operation`] instances.
pub trait Attribute: Object + std::fmt::Display {}
mopo!(dyn Attribute);

pub trait AttributeValue<T> {
    fn get_value(&self) -> &T;
    fn get_value_mut(&mut self) -> &mut T;
}

#[macro_export]
macro_rules! attribute {
    ($struct:ident:
     $key:literal,
     trait: $trt:ident) => {
        impl Attribute for $struct {}

        impl AttributeValue<$struct> for $struct {
            fn get_value(&self) -> &$struct {
                self
            }

            fn get_value_mut(&mut self) -> &mut $struct {
                self
            }
        }

        pub trait $trt {
            fn verify(&self, op: &dyn SupportsInterfaceTraits) -> Result<(), Report> {
                if !op.get_attributes().contains_key($key) {
                    bail!(format!(
                            "{} must provide a {} key for {} trait.",
                            op.get_intrinsic(),
                            Paint::magenta($key),
                            Paint::magenta(stringify!($trt)).bold()
                    ))
                }
                let obj = op.get_attributes().get($key).unwrap();
                match obj.query_ref::<dyn AttributeValue<$struct>>() {
                    None => bail!(format!("{}:\nThe attribute indexed by {} does not provide a {} value, which is required to support the {} trait.",
                            op.get_intrinsic(),
                            Paint::magenta($key),
                            Paint::magenta(stringify!($struct)).bold(),
                            Paint::magenta(stringify!($trt)).bold(),
                    )),
                    Some(_v) => Ok(()),
                }
            }

            fn get_value<'a>(&self, op: &'a dyn SupportsInterfaceTraits) -> &'a $struct {
                let obj = op.get_attributes().get($key).unwrap();
                let attr_val = obj
                    .query_ref::<dyn AttributeValue<$struct>>()
                    .unwrap();
                attr_val.get_value()
            }

            fn get_value_mut<'a>(
                &self,
                op: &'a mut dyn SupportsInterfaceTraits,
            ) -> &'a mut $struct {
                let obj = op.get_attributes_mut().get_mut($key).unwrap();
                let attr_val = obj
                    .query_mut::<dyn AttributeValue<$struct>>()
                    .unwrap();
                attr_val.get_value_mut()
            }
        }

        interfaces!($struct: dyn Attribute,
            dyn std::fmt::Display,
            dyn std::fmt::Debug,
            dyn AttributeValue<$struct>);
    }
}

/// A trait which provides non-mutating accessors for use in checking
/// intrinsic/operation verification conditions.
pub trait SupportsInterfaceTraits: std::fmt::Display {
    fn get_intrinsic(&self) -> &Box<dyn Intrinsic>;
    fn get_operands(&self) -> &[Var];
    fn get_regions(&self) -> &[Region];
    fn get_attributes(&self) -> &HashMap<String, Box<dyn Attribute>>;
    fn get_attributes_mut(&mut self) -> &mut HashMap<String, Box<dyn Attribute>>;
}

/// The main IR container - supports intrinsic and interface extension
/// mechanisms using trait object dispatch.
///
/// Owns:
/// 1. A set of `operands` (parameters provided to the `Operation`).
/// 2. An `attributes` map - representing attached constant metadata.
/// 3. A set of `regions` - which handle scoping.
/// 4. A set of `successors` - blocks which the operation can transfer control to.
///
/// [`Operation`] instances are almost always created through the builder interface
/// ([`crate::core::OperationBuilder`]).
#[derive(Debug)]
pub struct Operation {
    location: LocationInfo,
    intrinsic: Box<dyn Intrinsic>,
    operands: Vec<Var>,
    attributes: HashMap<String, Box<dyn Attribute>>,
    regions: Vec<Region>,
    successors: Vec<usize>,
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

    fn get_operands(&self) -> &[Var] {
        &self.operands
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
        successors: Vec<usize>,
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
