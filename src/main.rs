use std::env;
use std::process;

use connectz::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let outcome = connectz::run(config);
    println!("{}", outcome);
}
