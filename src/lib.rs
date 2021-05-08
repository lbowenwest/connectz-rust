use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind};

use game::Game;

mod game;
mod grid;

#[derive(PartialEq, Debug)]
pub enum Outcome {
    Draw,
    PlayerWin(Player),
    Incomplete,
    IllegalContinue,
    IllegalRow,
    IllegalColumn,
    IllegalGame,
    InvalidFile,
    FileNotFound,
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Draw => write!(f, "0"),
            Outcome::PlayerWin(player) => write!(f, "{}", player),
            Outcome::Incomplete => write!(f, "3"),
            Outcome::IllegalContinue => write!(f, "4"),
            Outcome::IllegalRow => write!(f, "5"),
            Outcome::IllegalColumn => write!(f, "6"),
            Outcome::IllegalGame => write!(f, "7"),
            Outcome::InvalidFile => write!(f, "8"),
            Outcome::FileNotFound => write!(f, "9"),
        }
    }
}

type Player = u8;

pub struct Config {
    filename: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next();

        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("Provide one input file"),
        };

        Ok(Config { filename })
    }
}

pub fn run(config: Config) -> Outcome {
    let file = match File::open(config.filename) {
        Ok(f) => f,
        Err(_) => return Outcome::FileNotFound,
    };
    let mut file = BufReader::new(file);

    let mut header = String::new();
    file.read_line(&mut header).expect("couldn't read file");

    let mut game = match Game::from_string(&header) {
        Ok(g) => g,
        Err(outcome) => return outcome,
    };

    if let Ok(moves) = file
        .lines()
        .map(|line| {
            line.and_then(|v| {
                v.parse::<u32>()
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))
                    .map(|v| v - 1)
            })
        })
        .collect()
    {
        game.play(&moves)
    } else {
        Outcome::InvalidFile
    }
}

#[pyfunction]
fn run_file(filename: String) -> PyResult<String> {
    Ok(format!("{}", run(Config { filename })))
}

/// A Python module implemented in Rust.
#[pymodule]
fn connectz(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_file, m)?)?;

    Ok(())
}
