# Juniper CAS

Juniper is a formally-specified computer algebra system written in Rust and formalized in Lean 4.

## What?

A CAS, or Computer Algebra System, is a library or program which can symbolically manipulate mathematical statements. For example, simplifying the statement `(x * x * y) / x + (3 + 2) * x` to `x * y + 5 * x` is a problem CASs are built to solve. Juniper is a CAS library written in Rust, which uses formal definitions from Lean 4 to understand which mathematical rules are applicable to a given statement.

## How?

A high-level overview: 

Proven theorems in the Lean 4 project are exported as JSON and imported into Rust. Then, those theorems are converted into rewriting rules by extracting `Forall`s as variables. Finally, the Egg e-graph equality saturation library is used to find the simplest form for a given statement using the rewriting rules provided by Lean.

For a more detailed explanation, visit lean/README.md and rs/README.md.

## (Un)Soundness

The point of this project is not to create a perfectly sound CAS using Lean proofs[^1], more to demonstrate how results in Lean can be automatically leveraged and utilized for computer algebra (or other similar rule-rewriting systems). A few things result from this distinction:

1. The conceptions of certain mathematical concepts are not the same in Lean as they are in Juniper. For a myriad of reasons, many mathematical operators in Lean are defined in ways which are unusual for the uninitiated. The constant folding in Juniper does not (currently) line up with Lean's understanding of those operators.
2. Constant folding in Juniper is also not formally specified. Accomplishing that would essentially require an entire secondary conversion process, but with more complex conversion for computable definitions of functions (which is quite far out of the scope of this project).

## TODO
- [x] create `LeanExpr` -> `egg::Pattern` infrastructure
- [x] create JuniperLean attribute to automatically turn marked theorems into JSON, and export them to a given file
- [x] import JSON in Rust, turn it into a set of rewrites, and use that for the `egg::Runner`
- [ ] create build.rs to track Lean files and rerun some Lean command to capture JSON
- [ ] split juniper_bin into juniper_repl and juniper_lib
- [ ] write readmes and documentation for what this is and how it operates
- [ ] add license(s)?
- [ ] add automatic conditionals (e.g. encoding `x` as a rewrite condition for `x → a / 2 = a * (1 / 2)`) to `lean_to_rewrite`
- [ ] more expansive `lean_to_rewrite` architecture
- [ ] write the actual set of theorems for conversion into the CAS
- [ ] create system to turn `egg::Explanation` into Lean proofs (either textually (lol) or with a certificate)

[^1]: Although, with a little effort, a very similar project could accomplish that!