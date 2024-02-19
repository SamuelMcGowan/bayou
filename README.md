# Bayou

A systems programming language, written as a hobby project in Rust.

Uses [Cranelift](https://cranelift.dev/) for code generation.

## Features

Features marked as completed are working for the tiny  language subset currently implemented but are likely to need much more work.

### Minimal Viable Product

- [x] Expressions (no logical expressions yet)
- [x] Local variables
- [ ] Function calls
- [ ] Basic Control flow (while, for, if, else)
- [ ] Modules
- [x] Static type checking
- [x] Linking (`gcc` is the only option at the moment)
- [ ] Custom data types (in particular structs)
- [ ] C FFI
- [x] Diagnostics and recoverable parsing

### Other wished-for features

Don't cross your fingers for these...

- [ ] Type inference
- [ ] Unions and tagged unions
- [ ] Generic types and functions
- [ ] Affine types and borrow checking
- [ ] Type classes
