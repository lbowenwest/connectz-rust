use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::num::ParseIntError;
use std::{env, error};

use game::Game;

mod game;
mod grid;

#[derive(Debug)]
pub enum ConnectzError {
    Incomplete,
    IllegalContinue,
    IllegalRow,
    IllegalColumn,
    IllegalGame,
    InvalidFile,
    FileNotFound,
    Argument(String),
}

impl fmt::Display for ConnectzError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectzError::Incomplete => write!(f, "{}", 3),
            ConnectzError::IllegalContinue => write!(f, "{}", 4),
            ConnectzError::IllegalRow => write!(f, "{}", 5),
            ConnectzError::IllegalColumn => write!(f, "{}", 6),
            ConnectzError::IllegalGame => write!(f, "{}", 7),
            ConnectzError::InvalidFile => write!(f, "{}", 8),
            ConnectzError::FileNotFound => write!(f, "{}", 9),
            ConnectzError::Argument(v) => write!(f, "{}", v),
        }
    }
}

impl error::Error for ConnectzError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

// Implement the conversion from `ParseIntError` to `DoubleError`.
// This will be automatically called by `?` if a `ParseIntError`
// needs to be converted into a `DoubleError`.
impl From<ParseIntError> for ConnectzError {
    fn from(_err: ParseIntError) -> ConnectzError {
        ConnectzError::InvalidFile
    }
}

impl From<std::io::Error> for ConnectzError {
    fn from(_: Error) -> Self {
        ConnectzError::FileNotFound
    }
}

type Result<T> = std::result::Result<T, ConnectzError>;

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

impl Outcome {
    pub fn as_u8(&self) -> &u8 {
        match self {
            Outcome::Draw => &0,
            Outcome::PlayerWin(player) => player,
            Outcome::Incomplete => &3,
            Outcome::IllegalContinue => &4,
            Outcome::IllegalRow => &5,
            Outcome::IllegalColumn => &6,
            Outcome::IllegalGame => &7,
            Outcome::InvalidFile => &8,
            Outcome::FileNotFound => &9,
        }
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u8())
    }
}

impl ToPyObject for Outcome {
    fn to_object(&self, py: Python) -> PyObject {
        self.as_u8().to_object(py)
    }
}

type Player = u8;

pub struct Config {
    filename: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config> {
        args.next();

        let filename = match args.next() {
            Some(arg) => arg,
            None => {
                return Err(ConnectzError::Argument(String::from(
                    "Provide one input file",
                )))
            }
        };

        Ok(Config { filename })
    }
}

pub fn run(config: Config) -> Result<Outcome> {
    let file = File::open(config.filename)?;
    let mut file = BufReader::new(file);

    let mut header = String::new();
    file.read_line(&mut header)?;

    let mut game = Game::from_string(&header)?;

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
        Ok(game.play(&moves))
    } else {
        Err(ConnectzError::InvalidFile)
    }
}

#[pyfunction]
fn run_file(filename: String) -> PyResult<String> {
    if let Ok(result) = run(Config { filename }) {
        Ok(format!("{}", result))
    } else {
        Ok(String::from("-1"))
    }
}

// create_exception!(connectz, ConnectzError, PyException);

/// A Python module implemented in Rust.
#[pymodule]
fn connectz(_py: Python, m: &PyModule) -> PyResult<()> {
    // m.add("ConnectzError", py.get_type::<ConnectzError>())?;
    m.add_function(wrap_pyfunction!(run_file, m)?)?;
    m.add_class::<Game>()?;

    Ok(())
}
