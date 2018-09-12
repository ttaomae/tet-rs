use std::collections::HashSet;

use rand::distributions::{Distribution, Standard};
use rand::Rng;

use super::core::{Piece, Playfield, Rotation, Space, Tetromino};

/// The main game engine.
struct Engine {
    playfield: Playfield,
    current_piece: CurrentPiece,
}

/// The current piece on the playfield.
#[derive(Debug, PartialEq, Eq)]
struct CurrentPiece {
    piece: Piece,
    row: usize,
    col: usize,
}

impl CurrentPiece {
    /// Creates a new piece in spawn position.
    fn new(shape: Tetromino) -> CurrentPiece {
        let (row, col) = match shape {
            Tetromino::I => (21, 4),
            Tetromino::O => (22, 5),
            _ => (22, 4),
        };

        CurrentPiece {
            piece: Piece::new(shape),
            row: row,
            col: col,
        }
    }

    fn rotate_cw(&mut self) {
        self.piece.rotate_cw();
    }

    fn rotate_ccw(&mut self) {
        self.piece.rotate_ccw();
    }
}

impl Engine {
    /// Creates a new engine with an empty playfield.
    fn new() -> Engine {
        Engine {
            playfield: Playfield::new(),
            current_piece: CurrentPiece::new(rand::random::<Tetromino>()),
        }
    }

    /* * * * * * * * * *
     * Engine actions. *
     * * * * * * * * * */
    // Actions performed by the engine.

    /// Sets the next current piece.
    fn next_piece(&mut self) {
        self.current_piece = CurrentPiece::new(rand::random::<Tetromino>());
    }

    /// Drops the current piece by one row if it does not result in a collision.
    fn drop(&mut self) {
        self.current_piece.row -= 1;
        if self.has_collision() {
            self.current_piece.row += 1;
        }
    }

    /// Returns whether or not there is a collision between the current piece and the playfield.
    fn has_collision(&self) -> bool {
        false
    }

    /* * * * * * * * * *
     * Player actions. *
     * * * * * * * * * */
    // Actions initiated by the player.

    /// Rotates the current piece clockwise, if it does not result in a collision.
    fn rotate_piece_cw(&mut self) {
        self.current_piece.rotate_cw();
        if self.has_collision() {
            self.current_piece.rotate_ccw();
        }
    }

    /// Rotates the current piece counter-clockwise, if it does not result in a collision.
    fn rotate_piece_ccw(&mut self) {
        self.current_piece.rotate_ccw();
        if self.has_collision() {
            self.current_piece.rotate_cw();
        }
    }

    /// Moves the current piece one column to the left, if it does not result in a collision.
    fn move_piece_left(&mut self) {
        self.current_piece.col -= 1;
        if self.has_collision() {
            self.current_piece.col += 1;
        }
    }

    /// Moves the current piece one column to the right, if it does not result in a collision.
    fn move_piece_right(&mut self) {
        self.current_piece.col += 1;
        if self.has_collision() {
            self.current_piece.col -= 1;
        }
    }
}

impl Distribution<Tetromino> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Tetromino {
        let rand = rng.gen_range(0, 7);
        match rand {
            0 => Tetromino::I,
            1 => Tetromino::O,
            2 => Tetromino::T,
            3 => Tetromino::S,
            4 => Tetromino::Z,
            5 => Tetromino::J,
            6 => Tetromino::L,
            _ => panic!("This should be impossible."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let engine = Engine::new();

        // Playfield should start empty.
        for row in 1..=40 {
            for col in 1..=10 {
                assert_eq!(engine.playfield.get(row, col), Space::Empty);
            }
        }

        // Current piece should be in spawn rotation.
        assert_eq!(engine.current_piece.piece.get_rotation(), &Rotation::Spawn);
    }

    #[test]
    fn test_current_piece_new() {
        // I is 4 spaces wide; starts cenetered, one row above visible field.
        assert_current_piece_new(CurrentPiece::new(Tetromino::I), Tetromino::I, 21, 4);
        // O is 2 spaces wide; starts centered, two columns above visible field.
        assert_current_piece_new(CurrentPiece::new(Tetromino::O), Tetromino::O, 22, 5);
        // Others are 3 spaces wide; start left of center, two columns above visible filed.
        assert_current_piece_new(CurrentPiece::new(Tetromino::T), Tetromino::T, 22, 4);
        assert_current_piece_new(CurrentPiece::new(Tetromino::S), Tetromino::S, 22, 4);
        assert_current_piece_new(CurrentPiece::new(Tetromino::Z), Tetromino::Z, 22, 4);
        assert_current_piece_new(CurrentPiece::new(Tetromino::J), Tetromino::J, 22, 4);
        assert_current_piece_new(CurrentPiece::new(Tetromino::L), Tetromino::L, 22, 4);
    }

    fn assert_current_piece_new(
        piece: CurrentPiece,
        expected_shape: Tetromino,
        expected_row: usize,
        expected_col: usize,
    ) {
        assert_eq!(piece.piece.get_rotation(), &Rotation::Spawn);
        assert_eq!(piece.piece.get_shape(), &expected_shape);
        assert_eq!(piece.row, expected_row);
        assert_eq!(piece.col, expected_col);
    }

    #[test]
    fn test_engine_next_piece() {
        let mut engine = Engine::new();
        let mut unique_shapes = HashSet::new();

        for _ in 0..10 {
            let piece = engine.current_piece.piece;
            assert_eq!(piece.get_rotation(), &Rotation::Spawn);
            unique_shapes.insert(piece.get_shape().clone());
            engine.next_piece();
        }

        // Test that different pieces are being generated. Since pieces are selected randomly, we
        // can't make any guarantees. However, after inserting 10 random pieces, the probability of
        // only receiving the same shape every single time is (1/7)^10 == 3.5e-9. This should be
        // sufficiently low that this test will rarely, if ever, fail.
        assert!(unique_shapes.len() > 1);
    }

    #[test]
    fn test_engine_rotate() {
        let mut engine = Engine::new();

        // Rotate clockwise.
        assert_eq!(engine.current_piece.piece.get_rotation(), &Rotation::Spawn);
        engine.rotate_piece_cw();
        assert_eq!(
            engine.current_piece.piece.get_rotation(),
            &Rotation::Clockwise
        );
        engine.rotate_piece_cw();
        assert_eq!(
            engine.current_piece.piece.get_rotation(),
            &Rotation::OneEighty
        );
        engine.rotate_piece_cw();
        assert_eq!(
            engine.current_piece.piece.get_rotation(),
            &Rotation::CounterClockwise
        );
        engine.rotate_piece_cw();
        assert_eq!(engine.current_piece.piece.get_rotation(), &Rotation::Spawn);

        // Rotate counter-clockwise.
        assert_eq!(engine.current_piece.piece.get_rotation(), &Rotation::Spawn);
        engine.rotate_piece_ccw();
        assert_eq!(
            engine.current_piece.piece.get_rotation(),
            &Rotation::CounterClockwise
        );
        engine.rotate_piece_ccw();
        assert_eq!(
            engine.current_piece.piece.get_rotation(),
            &Rotation::OneEighty
        );
        engine.rotate_piece_ccw();
        assert_eq!(
            engine.current_piece.piece.get_rotation(),
            &Rotation::Clockwise
        );
        engine.rotate_piece_ccw();
        assert_eq!(engine.current_piece.piece.get_rotation(), &Rotation::Spawn);
    }

    #[test]
    fn test_engine_drop() {
        let mut engine = Engine::new();
        let start_row = engine.current_piece.row;

        engine.drop();
        assert_eq!(engine.current_piece.row, start_row - 1);
        engine.drop();
        assert_eq!(engine.current_piece.row, start_row - 2);
        engine.drop();
        assert_eq!(engine.current_piece.row, start_row - 3);
        engine.drop();
        assert_eq!(engine.current_piece.row, start_row - 4);
        engine.drop();
        assert_eq!(engine.current_piece.row, start_row - 5);

        // New pieces should spawn back at the top.
        engine.next_piece();
        assert!(engine.current_piece.row > 20);
    }

    #[test]
    fn test_engine_move_piece() {
        let mut engine = Engine::new();

        let start_col = engine.current_piece.col;
        engine.move_piece_left();
        assert_eq!(engine.current_piece.col, start_col - 1);
        engine.move_piece_left();
        assert_eq!(engine.current_piece.col, start_col - 2);
        engine.move_piece_left();
        assert_eq!(engine.current_piece.col, start_col - 3);
        engine.move_piece_right();
        assert_eq!(engine.current_piece.col, start_col - 2);

        let start_col = engine.current_piece.col;
        engine.move_piece_right();
        assert_eq!(engine.current_piece.col, start_col + 1);
        engine.move_piece_right();
        assert_eq!(engine.current_piece.col, start_col + 2);
        engine.move_piece_right();
        assert_eq!(engine.current_piece.col, start_col + 3);
        engine.move_piece_left();
        assert_eq!(engine.current_piece.col, start_col + 2);
    }
}
