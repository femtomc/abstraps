use crate::core::ir::SupportsVerification;
use std::fmt;
use yansi::Paint;
use {indenter::indented, std::fmt::Write};

use crate::core::diagnostics::LocationInfo;
impl fmt::Display for LocationInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LocationInfo::Unknown => {
                write!(f, "{}", Paint::magenta(" (<unknown location>)").dimmed())
            }
            LocationInfo::FileLineCol(file, line, col) => {
                write!(
                    f,
                    "{}",
                    Paint::magenta(format!("<{} @ {}:{}>", file, line, col)).dimmed()
                )
            }
            _ => Ok(()),
        }
    }
}

use crate::core::ir::Var;
impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "%{}", self.get_id())
    }
}

use crate::core::ir::Intrinsic;
impl fmt::Display for dyn Intrinsic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}.{}",
            Paint::green(self.get_namespace()).underline(),
            Paint::green(self.get_name()).bold()
        )
    }
}

use crate::core::region::Region;
impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Region::Directed(ssacfg) => {
                for ind in 0..ssacfg.get_blocks().len() {
                    write!(f, "{}", Paint::white(format!("{}: ", ind)).bold())?;
                    let b = &ssacfg.get_blocks()[ind];
                    let boperands = &b.get_operands();
                    if !boperands.is_empty() {
                        write!(f, "(")?;
                        let l = boperands.len();
                        for (ind, arg) in boperands.iter().enumerate() {
                            match l - 1 == ind {
                                true => write!(f, "{}", arg)?,
                                _ => write!(f, "{}, ", arg)?,
                            };
                        }
                        write!(f, ")")?;
                    }
                    writeln!(f)?;
                    for (v, op) in self.block_iter(ind) {
                        writeln!(indented(f).with_str("  "), "{} = {}", v, op)?;
                    }
                }
                Ok(())
            }

            Region::Undirected(_) => {
                for (v, op) in self.block_iter(0) {
                    writeln!(indented(f).with_str("  "), "{} = {}", v, op)?;
                }
                Ok(())
            }
        }
    }
}

use crate::core::ir::Operation;
impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_intrinsic())?;
        if !self.get_operands().is_empty() {
            write!(f, "(")?;
            let l = self.get_operands().len();
            for (ind, arg) in self.get_operands().iter().enumerate() {
                match l - 1 == ind {
                    true => write!(f, "{}", arg)?,
                    _ => write!(f, "{}, ", arg)?,
                };
            }
            write!(f, ")")?;
        }
        write!(f, "{}", self.get_location())?;
        let mut fmter = indented(f).with_str(" ");
        if !self.get_attributes().is_empty() {
            write!(fmter, "\n[")?;
            let l = self.get_attributes().len();
            for (ind, attr) in self.get_attributes().iter().enumerate() {
                match l - 1 == ind {
                    true => write!(
                        indented(&mut fmter).with_str(" "),
                        " {}: {}",
                        Paint::magenta(attr.0),
                        attr.1
                    )?,
                    _ => writeln!(
                        indented(&mut fmter).with_str(" "),
                        "{}: {},",
                        Paint::magenta(attr.0),
                        attr.1
                    )?,
                };
            }
            write!(fmter, " ]")?;
        }
        let mut fmter1 = indented(&mut fmter).with_str(" ");
        if !self.get_regions().is_empty() {
            for r in self.get_regions().iter() {
                write!(indented(&mut fmter1).with_str(" "), "\n{}", r)?;
            }
        }
        Ok(())
    }
}

use crate::core::builder::OperationBuilder;
impl fmt::Display for OperationBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_location())?;
        write!(f, "{}", self.get_intrinsic())?;
        if !self.get_operands().is_empty() {
            write!(f, "(")?;
            let l = self.get_operands().len();
            for (ind, arg) in self.get_operands().iter().enumerate() {
                match l - 1 == ind {
                    true => write!(f, "{}", arg)?,
                    _ => write!(f, "{}, ", arg)?,
                };
            }
            write!(f, ")")?;
        }
        if !self.get_attributes().is_empty() {
            write!(f, " {{ ")?;
            let l = self.get_attributes().len();
            for (ind, attr) in self.get_attributes().iter().enumerate() {
                match l - 1 == ind {
                    true => write!(f, "{} = {}", attr.0, attr.1)?,
                    _ => write!(f, "{} = {}, ", attr.0, attr.1)?,
                };
            }
            write!(f, " }}")?;
        }
        if !self.get_regions().is_empty() {
            for r in self.get_regions().iter() {
                writeln!(f, " {{")?;
                write!(indented(f).with_str("  "), "{}", r)?;
                write!(f, "}}")?;
            }
        }
        Ok(())
    }
}

use crate::core::pass_manager::OperationPassManager;
impl<T> fmt::Display for OperationPassManager<T>
where
    T: Intrinsic,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tag = self.get_intrinsic_tag();
        let intr = format!(
            "{}.{}",
            Paint::green(tag.get_namespace()).underline(),
            Paint::green(tag.get_name()).bold()
        );
        writeln!(f, "({})", intr)?;
        for p in self.get_passes().iter() {
            writeln!(
                indented(f).with_str("  "),
                "{}",
                Paint::magenta(format!("{:?}", p))
            )?;
        }
        for pm in self.get_managers().iter() {
            writeln!(indented(f).with_str("  "), "{}", pm)?;
        }
        Ok(())
    }
}
