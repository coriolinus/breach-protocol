use crate::{grid::Grid, interner::Interned};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Active {
    Row(usize),
    Column(usize),
}

impl Default for Active {
    fn default() -> Self {
        Self::Row(0)
    }
}

impl Active {
    /// Return the new active set if the specified point is valid, or `Error::NotActive` otherwise.
    pub fn toggle(self, x: usize, y: usize) -> Result<Self, Error> {
        let err = Err(Error::NotActive { x, y, active: self });
        match self {
            Active::Row(row) => {
                if y != row {
                    err
                } else {
                    Ok(Active::Column(x))
                }
            }
            Active::Column(column) => {
                if x != column {
                    err
                } else {
                    Ok(Active::Row(y))
                }
            }
        }
    }
}

pub struct Matrix<'a, const WIDTH: usize, const HEIGHT: usize> {
    values: Grid<Interned<'a, String>, WIDTH, HEIGHT>,
    chosen: Grid<bool, WIDTH, HEIGHT>,
    selections: Vec<(usize, usize)>,
    active: Active,
}

impl<'a, const WIDTH: usize, const HEIGHT: usize> Matrix<'a, WIDTH, HEIGHT> {
    fn check_bounds(x: usize, y: usize) -> Result<(), Error> {
        if x < WIDTH && y < HEIGHT {
            Ok(())
        } else {
            Err(Error::OutOfBounds {
                x,
                y,
                width: WIDTH,
                height: HEIGHT,
            })
        }
    }

    /// Select the point at the given coordinates if it is legal to do so.
    pub fn select(&mut self, x: usize, y: usize) -> Result<(), Error> {
        Self::check_bounds(x, y)?;
        if self.chosen[(x, y)] {
            return Err(Error::AlreadySelected { x, y });
        }
        // the following line modifies self, so we can't fail past that point
        self.active = self.active.toggle(x, y)?;
        self.chosen[(x, y)] = true;

        self.selections.push((x, y));

        Ok(())
    }

    /// Deselect the most recent point selected.
    ///
    /// If the selection queue is empty, silently do nothing.
    pub fn deselect(&mut self) {
        if let Some((x, y)) = self.selections.pop() {
            debug_assert!(self.chosen[(x, y)], "point must already have been selected");
            self.chosen[(x, y)] = false;
            self.active
                .toggle(x, y)
                .expect("toggle must be valid at this point");
        }
    }

    /// Iterate over the selected values
    pub fn selected_values(&self) -> impl Iterator<Item = Interned<'_, String>> {
        self.selections
            .iter()
            .copied()
            .map(|(x, y)| self.values[(x, y)])
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the point 1({x}, {y})` is out of bounds. max: `({width}, {height})`")]
    OutOfBounds {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    },
    #[error("the point `({x}, {y})` is not a member of the active set: {active:?}")]
    NotActive { x: usize, y: usize, active: Active },
    #[error("the point `({x}, {y})` has already been selected")]
    AlreadySelected { x: usize, y: usize },
}
