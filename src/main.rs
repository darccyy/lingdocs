use std::fs;

use lingdocs::{compile, Config};

fn main() {
    let file = fs::read_to_string("./Lingdocs.toml").expect("Could not read config file");
    let config = Config::from(&file).expect("Could not parse config file");
    drop(file);

    compile(config).expect("Could not compile");
}
