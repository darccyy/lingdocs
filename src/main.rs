mod args;

use clap::Parser;

use args::Args;
use lingdocs::run;

fn main() {
    let args = Args::parse();

    run(&args.dir.unwrap_or(".".to_string())).expect("Failed");
}
