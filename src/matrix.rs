use crate::{grid::Grid, interner::Interned};

#[derive(PartialEq, Eq)]
pub enum Active {
    Row(usize),
    Column(usize),
}

impl Default for Active {
    fn default() -> Self {
        Self::Row(0)
    }
}

pub struct Matrix<'a, const WIDTH: usize, const HEIGHT: usize> {
    values: Grid<Interned<'a, String>, WIDTH, HEIGHT>,
    chosen: Grid<bool, WIDTH, HEIGHT>,
    selections: Vec<Interned<'a, String>>,
}
