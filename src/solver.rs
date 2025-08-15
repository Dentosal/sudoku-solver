use crate::{Grid, PossibleValues, Sudoku, SudokuSolution};
use std::fmt;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub type SudokuPossibilities = Grid<PossibleValues>;

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
                } else if let Some(det) = cell.determined() {
                    write!(f, "{} ", det)?;
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

    pub fn solved(&self) -> Option<SudokuSolution> {
        assert!(!self.is_broken(), "Cannot operate on a broken sudoku");

        for i in 0..9 {
            for j in 0..8 {
                self.grid[i][j].determined()?;
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
                    let mut copy = *self;
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
            let original = *self;
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
    ) -> Result<SudokuSolution, CannotSolve> {
        self.infer()?;

        if let Some(solution) = self.solved() {
            return Ok(solution);
        } else if depth > limit {
            return Err(CannotSolve::DepthLimit(*self));
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
                        let mut copy = *self;
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

    pub fn solve(mut self) -> Result<SudokuSolution, Broken> {
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

#[cfg(test)]
mod tests {
    use crate::Digit;

    use super::*;
    #[test]
    fn possibilities_broken_simple() {
        let mut sp = SudokuPossibilities::EMPTY;
        assert!(!sp.is_broken());

        sp.grid[0][0] &= PossibleValues::from(Digit::unchecked(2));
        assert!(!sp.is_broken());
        sp.grid[0][1] &= PossibleValues::from(Digit::unchecked(3));
        assert!(!sp.is_broken());
        sp.grid[0][2] &= PossibleValues::from(Digit::unchecked(2));
        assert!(sp.is_broken());
    }

    #[test]
    fn possibilities_broken_cell() {
        let mut sp = SudokuPossibilities::EMPTY;
        assert!(!sp.is_broken());

        sp.grid[4][4] &= PossibleValues::from(Digit::unchecked(2));
        sp.grid[5][5] &= PossibleValues::from(Digit::unchecked(2));
        assert!(sp.is_broken());
    }
}
