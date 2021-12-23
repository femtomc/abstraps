# abstraps

[![CI](https://img.shields.io/github/workflow/status/femtomc/abstraps/CI?style=for-the-badge)](https://github.com/femtomc/abstraps/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/abstraps?style=for-the-badge)](https://crates.io/crates/abstraps)
[![docs.rs](https://img.shields.io/docsrs/abstraps?style=for-the-badge)](https://docs.rs/abstraps/latest/abstraps/)

> Fun with abstract interpreters.

This project started off as an experiment in compiler design. Specifically, the author was motivated by studying [the Julia language and compiler](https://julialang.org/) inference system to understand language design with extensible abstract interpretation as a compiler (and potentially language!) feature.

It turns out that there are quite a few of languages (e.g. Crystal, Nim, Julia) experimenting with abstract interpretation as part of their type systems. In the long term, the goal of the project might be to provide a stable substrate to experiment on compiler design _cons_ abstract interpreters on an IR framework which supports MLIR-like concepts. 

The original author would have liked to just do this in MLIR proper -- but given time constraints and engineering ability (or lack thereof in C++), this project is implemented in Rust. In fact, for those interested in MLIR -- you may find "another implementation viewpoint" useful on the design concepts -- the repository has been kept as clean as possible, or otherwise documented with respect to non-trivial implementation decisions and their motivation from MLIR.

<div align="center">
<b><i><a href="https://mitpress.mit.edu/books/little-schemer-fourth-edition">Bon appetit!</a></i></b>
</div>

#### Claims

The framework provides:

1. An intermediate representation with concepts isomorphic to MLIR.
2. Interfaces for defining interpreters which act as abstract virtual machines on the IR, as well as reference (usable!) interpreters for standard design patterns.
3. Builder interfaces which support code generation to MLIR.

"Extensibility" means that there are no fixed intrinsics built into the IR (or any of the interfaces). The same is true for abstract lattices -- the interpreter patterns describe how (certain instances) of abstract interpreters will traverse the IR, but the user must provide the semantics by specifying intrinsics and how the interpreter should interpret the intrinsics on a user-defined lattice.

---

<div align="center">
<sup>
Started by <a href="https://femtomc.github.io/">McCoy R. Becker</a> during <a href="https://pl-design-seminar.seas.harvard.edu/">Harvard CS 252r</a>. All code is licensed under the <a href="LICENSE">MIT License</a>.
</sup>
</div>
