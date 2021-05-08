use std::env;
use std::process;

use connectz_rust::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let outcome = connectz_rust::run(config);
    println!("{}", outcome);
}
