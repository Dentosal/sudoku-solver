#![deny(unused_must_use)]

use sudoku_solver::Sudoku;

fn main() -> Result<(), &'static str> {
    let Some(path) = std::env::args().nth(1) else {
        return Err("usage: solve puzzle.txt");
    };
    let data = std::fs::read_to_string(&path).expect("Failed to read input file");
    let sudoku = Sudoku::parse(&data).ok_or("Invalid sudoku input")?;
    if let Some(solved) = sudoku.solve() {
        print!("{solved}");
        Ok(())
    } else {
        Err("Invalid sudoku, cannot solve")
    }
}
