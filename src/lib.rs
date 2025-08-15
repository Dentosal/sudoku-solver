#![deny(unused_must_use)]

mod bitset;

use core::fmt;
use std::ops;

use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::bitset::PossibleValues;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Number(u8);

impl Number {
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(9);

    pub fn index(self) -> u8 {
        self.0 - 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid<T> {
    pub grid: [[T; 9]; 9],
}
impl<T> From<[[T; 9]; 9]> for Grid<T> {
    fn from(grid: [[T; 9]; 9]) -> Self {
        Self { grid }
    }
}

impl<T: Copy> Grid<T> {
    pub fn binop<F: Fn(T, T) -> T>(self, rhs: Self, op: F) -> Self {
        let mut res = self.clone();
        for i in 0..9 {
            for j in 0..9 {
                res.grid[i][j] = op(self.grid[i][j], rhs.grid[i][j]);
            }
        }
        res
    }

    fn splat(empty: T) -> Self {
        Grid {
            grid: [[empty; 9]; 9],
        }
    }
}

impl<T> ops::BitAnd for Grid<T>
where
    T: Copy + ops::BitAnd<Output = T>,
{
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.binop(rhs, |a, b| a & b)
    }
}
impl<T> ops::BitAndAssign for Grid<T>
where
    T: Copy + ops::BitAnd<Output = T>,
{
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.binop(rhs, |a, b| a & b);
    }
}
impl<T> ops::BitOr for Grid<T>
where
    T: Copy + ops::BitOr<Output = T>,
{
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.binop(rhs, |a, b| a | b)
    }
}
impl<T> ops::BitOrAssign for Grid<T>
where
    T: Copy + ops::BitOr<Output = T>,
{
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.binop(rhs, |a, b| a | b);
    }
}

impl fmt::Display for Grid<Number> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.grid {
            for n in row {
                write!(f, "{} ", n.0)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub type Sudoku = Grid<Option<Number>>;

impl Sudoku {
    pub fn parse(data: &str) -> Self {
        let mut grid = [[None; 9]; 9];
        for (ri, row) in data.split('\n').enumerate() {
            for (ci, cell) in row.chars().enumerate() {
                if cell == '.' || cell.is_whitespace() {
                    continue;
                }

                let Some(n) = cell.to_digit(10) else {
                    panic!("Invalid character in sudoku input: {}", cell);
                };
                if n < 1 || n > 9 {
                    panic!("Sudoku numbers must be between 1 and 9, found: {}", n);
                }
                grid[ri][ci] = Some(Number(n as u8));
            }
        }

        Self { grid }
    }
}

impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.grid {
            for cell in row {
                match cell {
                    Some(n) => write!(f, "{} ", n.0)?,
                    None => write!(f, ". ")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

type SudokuPossibilities = Grid<PossibleValues>;

impl From<Sudoku> for SudokuPossibilities {
    fn from(sudoku: Sudoku) -> Self {
        let mut grid = [[PossibleValues::ANY; 9]; 9];
        for (ri, row) in sudoku.grid.iter().enumerate() {
            for (ci, cell) in row.iter().enumerate() {
                grid[ri][ci] = PossibleValues::initial_state(*cell)
            }
        }
        Self { grid }
    }
}

impl fmt::Display for SudokuPossibilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.grid {
            for cell in row {
                if cell.is_broken() {
                    write!(f, "X ")?;
                } else if cell.count() == 1 {
                    write!(f, "{} ", cell.options()[0].0)?;
                } else {
                    write!(f, "- ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl SudokuPossibilities {
    pub const EMPTY: Self = SudokuPossibilities {
        grid: [[PossibleValues::ANY; 9]; 9],
    };

    pub fn solved(&self) -> Option<Grid<Number>> {
        assert!(!self.is_broken(), "Cannot operate on a broken sudoku");

        for i in 0..9 {
            for j in 0..8 {
                if self.grid[i][j].determined().is_none() {
                    return None;
                }
            }
        }

        Some(
            self.grid
                .map(|row| row.map(|cell| cell.determined().unwrap()))
                .into(),
        )
    }

    pub fn is_broken(&self) -> bool {
        for i in 0..9 {
            for j in 0..9 {
                if self.grid[i][j].is_broken() {
                    return true;
                }
            }
        }

        for i in 0..9 {
            for j in 0..8 {
                for k in j + 1..9 {
                    // row
                    let a = self.grid[i][j].determined();
                    let b = self.grid[i][k].determined();
                    if a.is_some() && a == b {
                        return true;
                    }

                    // col
                    let a = self.grid[j][i].determined();
                    let b = self.grid[k][i].determined();
                    if a.is_some() && a == b {
                        return true;
                    }

                    // cell
                    let ax = (i / 3) * 3 + (j / 3);
                    let ay = (i % 3) * 3 + (j % 3);

                    let bx = (i / 3) * 3 + (k / 3);
                    let by = (i % 3) * 3 + (k % 3);

                    let a = self.grid[ax][ay].determined();
                    let b = self.grid[bx][by].determined();
                    if a.is_some() && a == b {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Do a full round of inference
    pub fn infer_step(&mut self) -> Result<(), Broken> {
        if self.is_broken() {
            return Err(Broken);
        }

        for i in 0..9 {
            for j in 0..9 {
                if self.grid[i][j].determined().is_some() {
                    continue;
                }

                for opt in self.grid[i][j].options() {
                    let mut copy = self.clone();
                    copy.grid[i][j] = PossibleValues::from(opt);
                    if copy.is_broken() {
                        self.grid[i][j].remove(opt);
                    }
                }
            }
        }

        if self.is_broken() {
            return Err(Broken);
        }

        Ok(())
    }

    pub fn infer(&mut self) -> Result<(), Broken> {
        loop {
            let original = self.clone();
            self.infer_step()?;
            if *self == original {
                break Ok(());
            }
        }
    }

    pub fn recursive_hypothetical(
        &mut self,
        depth: usize,
        limit: usize,
    ) -> Result<Grid<Number>, CannotSolve> {
        self.infer()?;

        if let Some(solution) = self.solved() {
            return Ok(solution);
        } else if depth > limit {
            return Err(CannotSolve::DepthLimit(self.clone()));
        }

        for i in 0..9 {
            for j in 0..9 {
                if self.grid[i][j].determined().is_some() {
                    continue;
                }

                let mut alts = Vec::new();
                for opt in self.grid[i][j]
                    .options()
                    .into_par_iter()
                    .map(|opt| {
                        let mut copy = self.clone();
                        copy.grid[i][j] = PossibleValues::from(opt);
                        copy.recursive_hypothetical(depth + 1, limit)
                    })
                    .collect::<Vec<_>>()
                {
                    match opt {
                        Ok(solved) => return Ok(solved),
                        Err(CannotSolve::Broken) => {}
                        Err(CannotSolve::DepthLimit(alt)) => alts.push(alt),
                    }
                }
                let mut combined = alts.pop().unwrap_or(Grid::splat(PossibleValues::EMPTY));
                while let Some(a) = alts.pop() {
                    combined |= a;
                }
                self.grid = combined.grid;
            }
        }

        Err(CannotSolve::DepthLimit(*self))
    }

    pub fn solve(mut self) -> Result<Grid<Number>, Broken> {
        let mut limit = 1;
        loop {
            match self.recursive_hypothetical(1, limit) {
                Ok(solved) => return Ok(solved),
                Err(CannotSolve::Broken) => return Err(Broken),
                Err(CannotSolve::DepthLimit(_)) => {
                    limit += 1;
                }
            }
        }
    }
}

#[must_use]
pub struct Broken;

pub enum CannotSolve {
    Broken,
    DepthLimit(SudokuPossibilities),
}
impl From<Broken> for CannotSolve {
    fn from(_: Broken) -> Self {
        Self::Broken
    }
}

impl Sudoku {
    pub fn solve(&self) -> Option<Grid<Number>> {
        SudokuPossibilities::from(*self).solve().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn possibilities_broken_simple() {
        let mut sp = SudokuPossibilities::EMPTY;
        assert!(!sp.is_broken());

        sp.grid[0][0] &= PossibleValues::from(Number(2));
        assert!(!sp.is_broken());
        sp.grid[0][1] &= PossibleValues::from(Number(3));
        assert!(!sp.is_broken());
        sp.grid[0][2] &= PossibleValues::from(Number(2));
        assert!(sp.is_broken());
    }

    #[test]
    fn possibilities_broken_cell() {
        let mut sp = SudokuPossibilities::EMPTY;
        assert!(!sp.is_broken());

        sp.grid[4][4] &= PossibleValues::from(Number(2));
        sp.grid[5][5] &= PossibleValues::from(Number(2));
        assert!(sp.is_broken());
    }
}
