mod compile;
mod config;
mod convert;
mod utils;

use std::{error::Error, fmt};

pub use crate::{compile::compile, config::Config};

#[derive(Debug)]
pub struct MyError(String);
impl Error for MyError {}
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
