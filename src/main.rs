use std::collections::HashMap;
use std::fmt;
use std::ops;

enum Outcome {
    Draw,
    PlayerWin(Player),
    Incomplete,
    IllegalContinue,
    IllegalRow,
    IllegalColumn,
    IllegalGame,
    // InvalidFile,
    // FileNotFound,
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
            // Outcome::InvalidFile => write!(f, "8"),
            // Outcome::FileNotFound => write!(f, "9"),
        }
    }
}

type Player = u8;

#[derive(Clone, Copy, Debug)]
struct Direction(i8, i8);

const HORIZONTAL: Direction = Direction(1, 0);
const VERTICAL: Direction = Direction(0, 1);
const FORWARD_DIAGONAL: Direction = Direction(1, 1);
const BACKWARD_DIAGONAL: Direction = Direction(-1, 1);

const ALL_DIRECTIONS: [Direction; 4] = [HORIZONTAL, VERTICAL, FORWARD_DIAGONAL, BACKWARD_DIAGONAL];

#[derive(Clone, Copy, Debug)]
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

struct Game {
    win_length: u32,
    grid: Grid,
    moves_made: HashMap<Player, u32>,
    last_move: Option<Location>,
}

impl Game {
    fn new(width: u32, height: u32, win_length: u32) -> Game {
        if win_length > width && win_length > height {
            exit_with_outcome(Outcome::IllegalGame);
        }
        Game {
            win_length,
            grid: Grid::with_dimensions(width as usize, height as usize),
            moves_made: HashMap::new(),
            last_move: None,
        }
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

    pub fn could_win(&self, player: Player) -> bool {
        if let Some(val) = self.moves_made.get(&player) {
            *val >= self.win_length
        } else {
            false
        }
    }
}

fn exit_with_outcome(outcome: Outcome) {
    println!("{}", outcome);
    std::process::exit(0);
}

fn main() {
    let moves = [1, 2, 1, 2, 1];
    let mut game = Game::new(3, 3, 3);
    let mut player = 1;

    for (idx, &column) in moves.iter().enumerate() {
        let result = game.make_move(player, column);
        match result {
            Some(outcome @ Outcome::PlayerWin(_)) => {
                if moves.len() > idx + 1 {
                    exit_with_outcome(Outcome::IllegalContinue);
                } else {
                    exit_with_outcome(outcome);
                }
            }
            Some(outcome) => exit_with_outcome(outcome),
            None => match player {
                1 => player = 2,
                2 => player = 1,
                _ => (),
            },
        }
    }

    exit_with_outcome(Outcome::Incomplete);
}
