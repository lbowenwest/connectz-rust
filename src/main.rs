use std::env;
use std::process;

use connectz::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    match connectz::run(config) {
        Ok(outcome) => println!("{}", outcome),
        Err(err) => println!("{}", err),
    }
}
