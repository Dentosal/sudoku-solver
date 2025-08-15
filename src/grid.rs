use std::{fmt, ops};

/// A 9x9 grid
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
    /// Initialize a grid with the same value in all cells
    pub fn splat(empty: T) -> Self {
        Grid {
            grid: [[empty; 9]; 9],
        }
    }

    /// Apply a function to each cell in the grid, returning a new grid.
    pub fn map<N: Copy, F: Fn(T) -> N>(self, f: F) -> Grid<N> {
        self.try_map(|v| Some(f(v))).unwrap()
    }

    /// Apply a function to each cell in the grid, returning a new grid.
    /// If the function returns None for any cell, the entire operation fails.
    pub fn try_map<N, F>(self, f: F) -> Option<Grid<N>>
    where
        N: Copy,
        F: Fn(T) -> Option<N>,
    {
        let mut res = Grid::splat(f(self.grid[0][0])?);
        for i in 0..9 {
            for j in 0..9 {
                res.grid[i][j] = f(self.grid[i][j])?;
            }
        }
        Some(res)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.grid.iter().flat_map(|row| row.iter())
    }

    /// Apply a binary operation to each cell in two grids,
    /// combining them into a new grid.
    pub fn binop<F: Fn(T, T) -> T>(self, rhs: Self, op: F) -> Self {
        let mut res = self;
        for i in 0..9 {
            for j in 0..9 {
                res.grid[i][j] = op(self.grid[i][j], rhs.grid[i][j]);
            }
        }
        res
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

impl<T> fmt::Display for Grid<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.grid {
            for n in row {
                write!(f, "{} ", n)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
