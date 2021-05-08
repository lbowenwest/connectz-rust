use std::ops;

use crate::{Outcome, Player};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Direction(i8, i8);

const HORIZONTAL: Direction = Direction(1, 0);
const VERTICAL: Direction = Direction(0, 1);
const FORWARD_DIAGONAL: Direction = Direction(1, 1);
const BACKWARD_DIAGONAL: Direction = Direction(-1, 1);

pub const ALL_DIRECTIONS: [Direction; 4] =
    [HORIZONTAL, VERTICAL, FORWARD_DIAGONAL, BACKWARD_DIAGONAL];

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Location(u32, u32);

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

pub struct Grid {
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
    fn add_direction_column_error() {
        let location = Location(0, 0);
        let direction = Direction(-1, 0);

        assert_eq!(location + direction, Err("already at first column"));
    }

    #[test]
    fn add_direction_row_error() {
        let location = Location(0, 0);
        let direction = Direction(0, -1);

        assert_eq!(location + direction, Err("already at first row"));
    }

    #[test]
    fn sub_direction() {
        let location = Location(1, 2);
        let direction = Direction(1, -1);

        assert_eq!(location - direction, Ok(Location(0, 3)));
    }

    #[test]
    fn sub_direction_column_error() {
        let location = Location(0, 0);
        let direction = Direction(1, 0);

        assert_eq!(location - direction, Err("already at first column"));
    }

    #[test]
    fn sub_direction_row_error() {
        let location = Location(0, 0);
        let direction = Direction(0, 1);

        assert_eq!(location - direction, Err("already at first row"));
    }

    #[test]
    fn grid_full() {
        let mut grid = Grid::with_dimensions(2, 2);

        assert!(grid.insert_piece(1, 0).is_ok());
        assert!(grid.insert_piece(1, 0).is_ok());
        assert!(grid.insert_piece(1, 1).is_ok());
        assert!(grid.insert_piece(1, 1).is_ok());

        assert!(grid.is_full());
    }

    #[test]
    fn inserting_bad_column() {
        let mut grid = Grid::with_dimensions(2, 2);
        let result = grid.insert_piece(1, 23).err();
        assert_eq!(result, Some(Outcome::IllegalColumn));
    }

    #[test]
    fn inserting_bad_row() {
        let mut grid = Grid::with_dimensions(2, 2);

        assert!(grid.insert_piece(1, 0).is_ok());
        assert!(grid.insert_piece(1, 0).is_ok());

        let result = grid.insert_piece(1, 0).err();
        assert_eq!(result, Some(Outcome::IllegalRow));
    }
}
