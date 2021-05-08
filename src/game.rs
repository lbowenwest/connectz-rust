use std::collections::HashMap;

use itertools::Itertools;

use crate::grid::{Grid, Location, ALL_DIRECTIONS};
use crate::{Outcome, Player};

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
