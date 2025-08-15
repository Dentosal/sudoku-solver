#![forbid(unsafe_code)]
#![deny(unused_must_use)]
#![allow(clippy::result_large_err)]

mod bitset;
mod digit;
mod grid;
mod solver;

use crate::solver::SudokuPossibilities;
pub use crate::{bitset::PossibleValues, digit::Digit, grid::Grid};

pub type Sudoku = Grid<Option<Digit>>;
pub type SudokuSolution = Grid<Digit>;

impl Sudoku {
    pub fn parse(data: &str) -> Option<Self> {
        let mut grid = [[None; 9]; 9];
        for (ri, row) in data.split('\n').enumerate() {
            for (ci, cell) in row.chars().enumerate() {
                if cell == '.' || cell.is_whitespace() {
                    continue;
                }

                let Some(n) = cell.to_digit(10) else {
                    panic!("Invalid character in sudoku input: {}", cell);
                };
                grid[ri][ci] = Some(Digit::new(n as u8)?);
            }
        }

        Some(Self { grid })
    }

    pub fn solve(&self) -> Option<Grid<Digit>> {
        SudokuPossibilities::from(*self).solve().ok()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn solve_examples() {
        for entry in fs::read_dir("puzzles").expect("Failed to read puzzles directory") {
            let entry = entry.expect("Failed to read entry");
            if entry
                .path()
                .extension()
                .map(|ext| ext.to_str() != Some("txt"))
                .unwrap_or(true)
            {
                continue;
            }
            let Ok(data) = std::fs::read_to_string(&entry.path()) else {
                panic!("Failed to read file: {}", entry.path().display());
            };
            let Some(sudoku) = Sudoku::parse(&data) else {
                panic!(
                    "Failed to parse sudoku from file: {}",
                    entry.path().display()
                );
            };
            if sudoku.solve().is_none() {
                panic!(
                    "Failed to solve sudoku from file: {}",
                    entry.path().display()
                );
            }
        }
    }
}
