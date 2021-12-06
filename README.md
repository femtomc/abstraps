# abstraps

[![crates.io](https://img.shields.io/crates/v/abstraps.svg)](https://crates.io/crates/abstraps)
[![CI](https://github.com/femtomc/abstraps/workflows/CI/badge.svg)](https://github.com/femtomc/abstraps/actions?query=workflow%3ACI)

> Extensible compiler middle layer with abstract interpreters.

This project started off as an experiment in compiler design. Specifically, the author was motivated by studying [the Julia language and compiler](https://julialang.org/) inference system to understand language design with extensible abstract interpretation as a compiler (and potentially language!) feature.

It turns out that there are quite a few of languages (e.g. Crystal, Nim, Julia) experimenting with abstract interpretation as part of their type systems. In the long term, the goal of the project might be to provide a stable substrate to experiment on compiler design _cons_ abstract interpreters.

---

The framework provides:

1. A fixed intermediate representation with concepts isomorphic to a subset of MLIR.
2. Interfaces for defining interpreters which act as abstract virtual machines on the IR, as well as reference (usable!) interpreters for standard design patterns.
3. Builder interfaces which support code generation to MLIR.

"Extensibility" means that there are no fixed intrinsics built into the IR (or any of the interfaces). The same is true for abstract lattices -- the interpreter patterns describe (certain instances) of abstract interpreters will traverse the IR, but the user must provide the semantics by specifying intrinsics and how the interpreter should interpret the intrinsics on a user-defined lattice.

<sup>
Started by <a href="https://femtomc.github.io/">McCoy R. Becker</a> during <a href="https://pl-design-seminar.seas.harvard.edu/">Harvard CS 252r</a>. All code is licensed under the <a href="LICENSE">MIT License</a>.
</sup>
