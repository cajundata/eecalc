# eecalc

A small command-line calculator for common electronics bench calculations, written in Rust.

`eecalc` is project 1 of a progressive Rust and embedded-development learning pathway. It is deliberately software-only. The goal is to learn idiomatic Rust on a problem you already understand, without an MCU toolchain in the way. This document is the planning specification for v0.1. It does not contain implementation code.

## 1. Purpose

- Produce a genuinely useful CLI for everyday bench math.
- Exercise the Rust fundamentals that every later project in the pathway depends on.
- Establish habits (module layout, error types, tests) that carry into the embedded projects unchanged.

## 2. Learning objectives

| Feature                           | Rust concept exercised                                            |
| --------------------------------- | ----------------------------------------------------------------- |
| Formula functions                 | Functions, `f64`, return values, doc comments                     |
| Command dispatch                  | `enum` with data, `match`, exhaustiveness                         |
| Ohm's law "given two of three"    | Enum variants modeling valid input combinations                   |
| Argument parsing                  | `std::env::args`, iterators, `str::parse`, `Option`               |
| SI prefix parsing (`10k`, `4.7u`) | String slicing, `char` handling, `Result`                         |
| Error reporting                   | Custom error `enum`, `Display`, `std::error::Error`, `?` operator |
| Code organization                 | Modules, `pub` visibility, `lib.rs` + `main.rs` split             |
| Correctness                       | Unit tests, integration tests, float tolerance comparisons        |
| Ownership basics                  | Borrowing `&str` and `&[f64]` rather than cloning                 |

Not objectives for v0.1: traits, generics, lifetimes beyond elision, `unsafe`, `no_std`. These arrive in projects 2 and 3.

## 3. v0.1 scope

Five commands, each a pure function over `f64` inputs:

1. `ohm`: given any two of voltage, current, resistance, compute the third.
2. `power`: given any two of voltage, current, resistance, compute power.
3. `series` / `parallel`: equivalent resistance of two or more resistors.
4. `divider`: output voltage of a two-resistor divider from `vin`, `r1`, `r2`.
5. `rc`: time constant and cutoff frequency from resistance and capacitance.

Inputs accept plain decimals or an SI-prefixed value. Outputs are printed in engineering notation with a unit.

## 4. Example commands and expected output

```
$ eecalc ohm --voltage 12 --resistance 470
I = 25.53 mA

$ eecalc ohm --current 20m --resistance 330
V = 6.6 V

$ eecalc power --voltage 5 --resistance 100
P = 250 mW

$ eecalc series 100 220 330
R = 650 Ω

$ eecalc parallel 1k 1k 2.2k
R = 407.4 Ω

$ eecalc divider --vin 5 --r1 10k --r2 2.2k
Vout = 901.6 mV

$ eecalc rc --resistance 10k --capacitance 1u
tau = 10 ms
fc  = 15.92 Hz
```

Error cases:

```
$ eecalc ohm --voltage 12
error: ohm requires exactly two of --voltage, --current, --resistance
(exit code 2)

$ eecalc ohm --voltage 12 --resistance 0
error: resistance must be greater than zero
(exit code 1)

$ eecalc rc --resistance 10x --capacitance 1u
error: invalid value '10x': unknown SI prefix 'x'
(exit code 2)
```

## 5. Command structure

```
eecalc <command> [arguments]
eecalc help | --help
eecalc <command> --help
```

| Command    | Arguments                                                        | Output               |
| ---------- | ---------------------------------------------------------------- | -------------------- |
| `ohm`      | exactly two of `--voltage/-v`, `--current/-i`, `--resistance/-r` | the missing quantity |
| `power`    | exactly two of `--voltage/-v`, `--current/-i`, `--resistance/-r` | `P`                  |
| `series`   | two or more positional resistances                               | `R`                  |
| `parallel` | two or more positional resistances                               | `R`                  |
| `divider`  | `--vin`, `--r1`, `--r2` (all required)                           | `Vout`               |
| `rc`       | `--resistance/-r`, `--capacitance/-c` (both required)            | `tau`, `fc`          |

Value syntax: a decimal number optionally followed by one SI prefix from `p n u m k M`. Examples: `470`, `4.7k`, `100n`, `2.2M`. Whitespace between number and prefix is not allowed.

Exit codes: `0` success, `1` calculation error (e.g., division by zero), `2` usage or parse error. Errors go to stderr; results go to stdout.

## 6. Suggested project structure

```
eecalc/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs        # parse args, call run(), map errors to exit codes
│   ├── lib.rs         # pub mod declarations, pub fn run(args) -> Result<String, EecalcError>
│   ├── cli.rs         # args -> Command enum
│   ├── units.rs       # parse_value(&str) -> Result<f64>, format_eng(f64, unit) -> String
│   ├── error.rs       # EecalcError enum, Display, std::error::Error
│   └── calc/
│       ├── mod.rs
│       ├── ohm.rs
│       ├── power.rs
│       ├── resistance.rs
│       ├── divider.rs
│       └── rc.rs
└── tests/
    └── cli.rs         # runs the built binary, checks stdout/stderr/exit code
```

The `lib.rs` / `main.rs` split exists so every calculation and parser is testable without spawning a process. `main.rs` should stay under about 30 lines.

### External crates

| Crate        | Value                                        | Decision for v0.1                                                                                       |
| ------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `clap`       | Robust argument parsing, auto-generated help | Not used. Hand-rolled parsing is a stated learning objective and the surface is small. Revisit in v0.2. |
| `assert_cmd` | Ergonomic integration tests for binaries     | Not used. `std::process::Command` is sufficient and keeps the dependency count at zero.                 |

v0.1 target: zero external dependencies.

## 7. Functional requirements

- FR1: `ohm` computes `V = I * R`, `I = V / R`, or `R = V / I` depending on which two inputs are supplied.
- FR2: `power` computes `P = V * I`, `P = I² * R`, or `P = V² / R`.
- FR3: `series` computes the sum of all resistances.
- FR4: `parallel` computes `1 / Σ(1 / Rn)`.
- FR5: `divider` computes `Vout = Vin * R2 / (R1 + R2)`.
- FR6: `rc` computes `tau = R * C` and `fc = 1 / (2π * R * C)`.
- FR7: All numeric inputs accept SI prefixes per section 5.
- FR8: Outputs use engineering notation: four significant figures, trailing zeros trimmed, prefix chosen from `p n u m (none) k M`.
- FR9: `help` and `--help` print usage for all commands; `<command> --help` prints usage for one.
- FR10: All formulas are pure functions with no I/O, callable from tests.

## 8. Input validation and error handling

- All errors are variants of a single `EecalcError` enum in `error.rs`. Suggested variants: `UnknownCommand`, `MissingArgument`, `UnexpectedArgument`, `InvalidNumber`, `WrongArgumentCount` (for the "exactly two of three" rule), `NonPositiveValue`, `DivisionByZero`.
- `EecalcError` implements `Display` with a user-facing message and `std::error::Error`.
- No `panic!`, `unwrap`, or `expect` in library code. Tests may use them.
- Resistance and capacitance must be greater than zero. Voltage and current may be zero or negative where physically meaningful (a zero-volt Ohm's law query is valid; dividing by zero current is not).
- Unknown flags, duplicated flags, and missing values after a flag are usage errors (exit 2).
- `series` and `parallel` require at least two values.
- `main.rs` is the only place errors are printed and mapped to exit codes.

## 9. Testing requirements

- Unit tests in every `calc/*.rs` module against hand-verified values, using a tolerance helper (e.g., relative error below `1e-9`).
- Unit tests in `units.rs` covering every prefix, plain decimals, and at least five malformed inputs.
- Unit tests in `cli.rs` covering each command's valid form and each error variant it can produce.
- Integration tests in `tests/cli.rs` that build and run the binary for at least one success and one failure per command, asserting stdout, stderr, and exit code.
- `cargo test` passes with zero warnings; `cargo clippy` produces no warnings on default lints.

## 10. Definition of Done

- [ ] All five commands implemented per section 7 and produce the outputs in section 4 exactly.
- [ ] Every error case in section 8 is handled and tested.
- [ ] `cargo build --release`, `cargo test`, `cargo clippy`, and `cargo fmt --check` all pass clean.
- [ ] Zero external dependencies in `Cargo.toml`.
- [ ] `eecalc help` documents every command and the SI prefix syntax.
- [ ] This README's section 4 has been verified by copy-pasting the commands.
- [ ] Tagged `v0.1.0`.

## 11. Out of scope (later versions)

- AC and complex impedance, RL and LC circuits, reactance
- LED series-resistor calculator, capacitor charge/energy, wire gauge and trace width
- Rounding results to E-series (E12/E24/E96) standard values
- Three-or-more-resistor networks beyond pure series/parallel
- Interactive REPL mode, config files, unit-system preferences
- Migration to `clap`; colored output; shell completions
- `no_std` compatibility (this becomes a requirement in project 3, not here)
- GUI, database, networking, async, or any external service
