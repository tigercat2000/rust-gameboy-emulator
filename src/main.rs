pub mod error;
pub use error::{Error, Result};
pub mod instructions;
pub mod rom;
#[cfg(test)]
pub mod unit_tests;

fn main() {
    println!("Hello, world!");
}
