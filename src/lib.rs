mod case;
mod compile;
mod config;
mod convert;
mod utils;

use std::{error::Error, fmt, fs};

pub use crate::{compile::compile, config::Config};

#[derive(Debug)]
pub struct MyError(String);
impl Error for MyError {}
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn run(dir: &str) -> Result<(), Box<dyn Error>> {
    let file =
        fs::read_to_string(format!("{}/Lingdocs.toml", dir)).expect("Could not read config file");

    let mut config = Config::from(&file).expect("Could not parse config file");

    config.files.source = format!("{}/{}", dir, config.files.source);
    config.files.build = format!("{}/{}", dir, config.files.build);

    compile(config).expect("Could not compile");

    Ok(())
}
