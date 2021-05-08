use itertools::Itertools;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::{fmt, ops};

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

#[derive(PartialEq, Clone, Copy, Debug)]
struct Direction(i8, i8);

const HORIZONTAL: Direction = Direction(1, 0);
const VERTICAL: Direction = Direction(0, 1);
const FORWARD_DIAGONAL: Direction = Direction(1, 1);
const BACKWARD_DIAGONAL: Direction = Direction(-1, 1);

const ALL_DIRECTIONS: [Direction; 4] = [HORIZONTAL, VERTICAL, FORWARD_DIAGONAL, BACKWARD_DIAGONAL];

#[derive(PartialEq, Clone, Copy, Debug)]
struct Location(u32, u32);

impl ops::Add<Direction> for Location {
    type Output = Result<Location, &'static str>;

    fn add(self, rhs: Direction) -> Self::Output {
        if self.0 == 0 && rhs.0 < 0 {
            return Err("already at first column");
        } else if self.1 == 0 && rhs.1 < 0 {
            return Err("already at first row");
        } else {
            Ok(Location(
                (self.0 as i64 + rhs.0 as i64) as u32,
                (self.1 as i64 + rhs.1 as i64) as u32,
            ))
        }
    }
}

impl ops::Sub<Direction> for Location {
    type Output = Result<Location, &'static str>;

    fn sub(self, rhs: Direction) -> Self::Output {
        if self.0 == 0 && rhs.0 > 0 {
            return Err("already at first column");
        } else if self.1 == 0 && rhs.1 > 0 {
            return Err("already at first row");
        } else {
            Ok(Location(
                (self.0 as i64 - rhs.0 as i64) as u32,
                (self.1 as i64 - rhs.1 as i64) as u32,
            ))
        }
    }
}

struct Grid {
    values: Vec<Vec<Player>>,
    max_height: usize,
}

impl Grid {
    pub fn with_dimensions(width: usize, height: usize) -> Grid {
        Grid {
            values: vec![Vec::with_capacity(height); width],
            max_height: height,
        }
    }

    pub fn at(&self, loc: Location) -> Option<&Player> {
        if let Some(col) = self.values.get(loc.0 as usize) {
            col.get(loc.1 as usize)
        } else {
            None
        }
    }

    pub fn is_full(&self) -> bool {
        self.values.iter().all(|col| col.len() == self.max_height)
    }

    pub fn insert_piece(&mut self, player: Player, column: u32) -> Result<Location, Outcome> {
        let col = match self.values.get_mut(column as usize) {
            Some(col) => col,
            None => return Err(Outcome::IllegalColumn),
        };
        let length = col.len();
        if length >= self.max_height {
            return Err(Outcome::IllegalRow);
        }
        col.push(player);

        Ok(Location(column, length as u32))
    }

    pub fn get_streak(&self, start: Location, direction: Direction) -> u32 {
        let player = match self.at(start) {
            Some(player) => player,
            None => return 0,
        };

        let mut streak: u32 = 1;
        let mut position = start;

        while let Ok(pos) = position + direction {
            position = pos;
            if let Some(new_player) = self.at(pos) {
                if new_player == player {
                    streak += 1
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        position = start;
        while let Ok(pos) = position - direction {
            position = pos;
            if let Some(new_player) = self.at(pos) {
                if new_player == player {
                    streak += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        streak
    }
}

pub struct Game {
    win_length: u32,
    grid: Grid,
    moves_made: HashMap<Player, u32>,
    last_move: Option<Location>,
}

impl Game {
    pub fn from_string(desc: &str) -> Result<Game, Outcome> {
        if let Some((width, height, win_length)) = desc
            .split_ascii_whitespace()
            .map(|v| v.parse::<u32>().expect("a number"))
            .collect_tuple()
        {
            Game::new(width, height, win_length)
        } else {
            Err(Outcome::InvalidFile)
        }
    }

    pub fn new(width: u32, height: u32, win_length: u32) -> Result<Game, Outcome> {
        if win_length > width && win_length > height {
            Err(Outcome::IllegalGame)
        } else {
            Ok(Game {
                win_length,
                grid: Grid::with_dimensions(width as usize, height as usize),
                moves_made: HashMap::new(),
                last_move: None,
            })
        }
    }

    pub fn play(&mut self, moves: &Vec<u32>) -> Outcome {
        let mut player = 1;

        for (idx, &column) in moves.iter().enumerate() {
            let result = self.make_move(player, column);
            match result {
                Some(outcome @ Outcome::PlayerWin(_)) => {
                    return if moves.len() > idx + 1 {
                        Outcome::IllegalContinue
                    } else {
                        outcome
                    }
                }
                Some(outcome) => return outcome,
                None => match player {
                    1 => player = 2,
                    2 => player = 1,
                    _ => (),
                },
            }
        }

        Outcome::Incomplete
    }

    fn make_move(&mut self, player: Player, column: u32) -> Option<Outcome> {
        match self.grid.insert_piece(player, column) {
            Ok(location) => {
                self.last_move = Some(location);
                self.moves_made
                    .entry(player)
                    .and_modify(|value| *value += 1)
                    .or_insert(1);
            }
            Err(outcome) => return Some(outcome),
        };

        if self.could_win(player) {
            for direction in ALL_DIRECTIONS.iter() {
                let streak = self.grid.get_streak(self.last_move.expect(""), *direction);
                if streak >= self.win_length {
                    return Some(Outcome::PlayerWin(player));
                }
            }
        }

        if self.grid.is_full() {
            Some(Outcome::Draw)
        } else {
            None
        }
    }

    fn could_win(&self, player: Player) -> bool {
        if let Some(val) = self.moves_made.get(&player) {
            *val >= self.win_length
        } else {
            false
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_direction() {
        let location = Location(1, 2);
        let direction = Direction(1, -1);

        assert_eq!(location + direction, Ok(Location(2, 1)));
    }

    #[test]
    fn sub_direction() {
        let location = Location(1, 2);
        let direction = Direction(1, -1);

        assert_eq!(location - direction, Ok(Location(0, 3)));
    }
}
