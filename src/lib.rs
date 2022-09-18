mod grid;
mod interner;
mod matrix;
mod sequence;

pub use grid::Grid;
pub use interner::{Interned, InternedString, Interner};
pub use matrix::{Active, Matrix};
pub use sequence::Sequence;

/// The Breach Protocol minigame.
///
/// ## The Game
///
/// This game originates in _Cyberpunk 2077_.
///
/// Users are presented a matrix of hex digits, of which one row or column is active.
/// Values in the active set can be selected, if they have not previously been selected.
/// Loops are not possible.
/// When an item is selected, the active set pivots on the selection: if it had previously
/// been vertical, it becomes horizontal, and vice versa.
///
/// Selected values are entered into a buffer. Buffers have a limited size.
///
/// There exist target sequences. Given the list of selected values in the buffer, these
/// sequences are checked against the buffer. If there exists an offset for which the entire
/// sequence is matched in order within the buffer, the sequence is completed.
///
/// The object of the game is to complete as many sequences as possible. Some sequences are more
/// desirable than others, but generally speaking it is desirable to maximize the number of
/// sequences completed.
///
/// The major challenge of the game is not to find some solution to the most desirable sequence.
/// It is finding ways to overlap the sequences such that more than one, and possibly all sequences,
/// can be completed with a buffer shorter than the sum of their lengths.
///
/// ## Implementation Notes
///
/// As Rust doesn't like self-referential structs, and the internals are all built around interned
/// strings which borrow from a central interner, this struct can't encapsulate both the interner
/// and also the interned values. We choose to keep the interner; external code can own the subordinate
/// structures.
///
/// While the game can be challenging for humans, it is sharply bounded in scale. Exhaustive search
/// should easily be fast enough.
pub struct BreachProtocol {
    interner: Interner<String>,
    buffer_size: usize,
}

impl BreachProtocol {
    pub fn solve<'a, const WIDTH: usize, const HEIGHT: usize>(
        &'a self,
        matrix: &'a mut Matrix<'a, WIDTH, HEIGHT>,
        sequences: &'a [Sequence<'a>],
    ) -> Vec<Solution<'a>> {
        let mut solutions = Vec::new();
        self.solve_inner(matrix, sequences, &mut solutions);
        solutions
    }

    fn solve_inner<'a, const WIDTH: usize, const HEIGHT: usize>(
        &'a self,
        matrix: &'a mut Matrix<'a, WIDTH, HEIGHT>,
        sequences: &'a [Sequence<'a>],
        solutions: &'a mut Vec<Solution<'a>>,
    ) {
        let depth = matrix.selected_len();
        if depth < self.buffer_size {
            for (x, y) in matrix.legal_selections() {
                matrix.select(x, y).expect("legal selection remained legal");
                self.solve_inner(matrix, sequences, solutions);
                matrix.deselect();
            }
        } else {
            // only compute which sequences were matched once the buffer is full
            let matches: Vec<_> = sequences
                .iter()
                .enumerate()
                .filter_map(|(idx, sequence)| {
                    sequence.is_matched(matrix.selected_values()).then_some(idx)
                })
                .collect();

            if !matches.is_empty() {
                solutions.push(Solution {
                    buffer: matrix.selected_values().collect(),
                    matches,
                });
            }
        }
    }
}

pub struct Solution<'a> {
    buffer: Vec<InternedString<'a>>,
    matches: Vec<usize>,
}
