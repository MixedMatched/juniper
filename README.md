# Juniper CAS

Juniper is a formally-specified computer algebra system written in Rust and formalized in Lean 4. This is meant to be a toy project to demonstrate how these systems can work together, and not really a ready-made library.

## What?

A CAS, or [Computer Algebra System](https://en.wikipedia.org/wiki/Computer_algebra_system), is a library or program which can symbolically manipulate algebraic statements. For example, simplifying the statement `(x * x * y) / x + (3 + 2) * x` to `x * y + 5 * x` is a problem CASs are built to solve. Juniper is a CAS library, written in Rust, which uses formal definitions from Lean 4 to understand which mathematical rules are applicable to a given statement.

## How?

A high-level overview: 

Proven theorems in the Lean 4 project are exported as JSON and imported into Rust. Then, those theorems are converted into [rewriting rules](https://en.wikipedia.org/wiki/Rewriting) by extracting `Forall`s as variables. Finally, [egg](https://egraphs-good.github.io/) is used to find the simplest form for a given statement using the rewriting rules obtained from Lean.

For a more detailed explanation, see the [Lean README](lean/README.md) and the [Rust README](rs/README.md).

## Mathematical details

### Domain

While the basic datatype of numbers in this project are Rationals, the operating domain of mathematical rules is not. All numbers and expressions should be expected to be in the Reals, but only the Rational subset of the Reals are represented by constants. Approximation is also done in the Reals (but represented by floats).

### (Un)Soundness

The point of this project is not to create a perfectly sound CAS using Lean proofs[^1], more to demonstrate how results in Lean can be automatically leveraged and utilized for computer algebra (or other similar rule-rewriting systems). A few things result from this distinction:

1. The conceptions of certain mathematical concepts are not the same in Lean as they are in Juniper. For a myriad of reasons, many mathematical operators in Lean are defined in ways which are unusual for the uninitiated. The constant folding in Juniper does not (currently) line up with Lean's understanding of those operators.
2. Constant folding in Juniper is also not formally specified. Accomplishing that would essentially require an entire secondary conversion process, but with more complex conversion for computable definitions of functions (which is quite far out of the scope of this project).
3. Equality saturation is also not formalized (mostly because it comes from egg).
4. Parsing/printing is not verified beyond unit tests.

## TODO
- [x] create `LeanExpr` -> `egg::Pattern` infrastructure
- [x] create JuniperLean attribute to automatically turn marked theorems into JSON, and export them to a given file
- [x] import JSON in Rust, turn it into a set of rewrites, and use that for the `egg::Runner`
- [x] create build.rs to track Lean files and rerun some Lean command to capture JSON
- [x] add scientific number parsing to JuniperBigRational
- [x] split juniper_bin into juniper_repl and juniper_lib
- [x] write readmes and documentation for what this is and how it operates
- [x] add license(s)?
- [x] more expansive `lean_to_rewrite` architecture
- [x] add support for automatic conditionals (e.g. encoding `x` as a rewrite condition for `x → a / 2 = a * (1 / 2)`) to `lean_to_rewrite`
- [ ] add environment support (being able to assign variables, e.g. `(= x (* 5 y))` and `x / 5` → `y`)
- [ ] write the actual set of theorems for conversion into the CAS
- [ ] create system to turn `egg::Explanation` into Lean proofs (either textually (lol) or with a proof certificate)

[^1]: Although, with a little effort, a very similar project could accomplish that!