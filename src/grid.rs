use std::ops::{Index, IndexMut};

/// A representation of a 2d grid.
///
/// For indexing operations on this grid, `(0, 0)` is the top left corner.
#[derive(Debug, Clone)]
pub struct Grid<T, const WIDTH: usize, const HEIGHT: usize>(Vec<T>);

impl<T, const WIDTH: usize, const HEIGHT: usize> Grid<T, WIDTH, HEIGHT>
where
    T: Default + Clone,
{
    pub fn new() -> Self {
        Grid(vec![T::default(); WIDTH * HEIGHT])
    }
}

impl<T, const WIDTH: usize, const HEIGHT: usize> Grid<T, WIDTH, HEIGHT> {
    /// Get the internal index where the desired value is stored,
    /// or `None` if it is out of bounds.
    pub fn idx(x: usize, y: usize) -> Option<usize> {
        (x < WIDTH && y < HEIGHT).then_some((y * WIDTH) + x)
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        Self::idx(x, y).map(|idx| &self.0[idx])
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        Self::idx(x, y).map(|idx| &mut self.0[idx])
    }
}

impl<T, const WIDTH: usize, const HEIGHT: usize> Index<(usize, usize)> for Grid<T, WIDTH, HEIGHT> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        self.get(x, y).unwrap()
    }
}

impl<T, const WIDTH: usize, const HEIGHT: usize> IndexMut<(usize, usize)>
    for Grid<T, WIDTH, HEIGHT>
{
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        self.get_mut(x, y).unwrap()
    }
}
