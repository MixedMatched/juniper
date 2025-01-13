# juniper-rs

This is the set of Rust crates representing the Juniper library. The purpose of each of the crates is as follows:
- **lean_parse**: a set of data definitions for core Lean types used for serializing and deserializing Lean type information.
- **juniper_math_expression**: the definition for MathExpression (Juniper's egg [Language](https://docs.rs/egg/latest/egg/trait.Language.html)), JuniperBigRational (a simple wrapper for parsing [num::BigRational](https://docs.rs/num-rational/0.4.2/num_rational/type.BigRational.html)), and ConstantFold (Juniper's egg [Analysis](https://docs.rs/egg/latest/egg/trait.Analysis.html) for eliminating constants).
- **juniper_lean_to_rewrite**: an opinionated LeanExpr to MathExpression transpiler.
- **juniper_lib**: the front-facing API for utilizing Juniper. Exposes an automatically generated list of Rewrites obtained from transpiling JuniperLean results, as well as using a build script to automatically re-elaborate JuniperLean when changes are detected.
- **juniper_repl**: a simple command line tool for evaluating expressions using juniper_lib.

Big picture, the Rust system works by interpreting the json representing the set of Lean equality types, transpiling those types to rewriting rules, and creating an Egg Runner using those rules.

## Transpiling

Going between `LeanExpr`s and `MathExpression`s happens in 2 stages:

- `LeanExpr` to the intermediate representation
- the intermediate representation to `MathExpression`

I chose to use an intermediate representation because `LeanExpr` and `MathExpression` are very different kinds of syntax tree. Expressions from Lean use a new function application for every level of argument (which creates a much more concise language definition), while `MathExpression`s use a single node for every operator and atom (which is much more expressive). The intermediate representation I created is basically a tree with single nodes for operators and atoms, but with 1:1 optional fields for `LeanExpr`, so that the tree can essentially be filled in as-you-go instead of all at once. Then, once the tree is complete, it can be easily converted into a `MathExpression` in a single shot.