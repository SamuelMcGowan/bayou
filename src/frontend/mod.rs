#[cfg(test)]
mod tests;

// FIXME: hide internals
pub mod ast;
mod lexer;
mod parser;

pub use parser::Parser;