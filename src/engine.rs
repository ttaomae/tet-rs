use std::fmt;

use rand::distributions::{Distribution, Standard};
use rand::Rng;

use super::core::{Piece, Playfield, Space, Tetromino};

/// The main game engine.
struct Engine {
    playfield: Playfield,
    current_piece: CurrentPiece,
}

/// The current piece on the playfield.
#[derive(Debug, PartialEq, Eq)]
struct CurrentPiece {
    piece: Piece,
    // Position of lower-left corner of bounding box.
    row: i8,
    col: i8,
}

impl CurrentPiece {
    /// Creates a new piece in spawn position.
    fn new(shape: Tetromino) -> CurrentPiece {
        CurrentPiece {
            piece: Piece::new(shape),
            row: 20,
            col: 4,
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

    /// Returns whether or not there is a collision between the current piece and the playfield.
    fn has_collision(&self) -> bool {
        let bounding_box = self.current_piece.piece.get_bounding_box();
        // Iterate through spaces of bounding box.
        for (row_offset, bb_row) in bounding_box.iter().enumerate() {
            for (col_offset, bb_space) in bb_row.iter().enumerate() {
                // Calculate position of space in playfield.
                let row = self.current_piece.row + row_offset as i8;
                let col = self.current_piece.col + col_offset as i8;

                // Collisions can only occur on blocks.
                if bb_space == &Space::Block
                    // Collision occurs if block is outside playfield.
                    && ((row < 1 || col < 1 || col > Playfield::WIDTH as i8)
                    // Or if block is inside playfield ...
                    || (row  >= 1 && col >= 1
                        // ... and there is already a block in that position.
                        && self.playfield.get(row as u8, col as u8) == Space::Block))
                {
                    return true;
                }
            }
        }
        false
    }

    /// Drops the current piece by one row if it does not result in a collision.
    fn drop(&mut self) {
        self.current_piece.row -= 1;
        if self.has_collision() {
            self.current_piece.row += 1;
        }
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

impl fmt::Debug for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut playfield = self.playfield;

        let bounding_box = self.current_piece.piece.get_bounding_box();
        for (row_offset, bb_row) in bounding_box.iter().enumerate() {
            for (col_offset, bb_space) in bb_row.iter().enumerate() {
                // Calculate position of space in playfield.
                let row = self.current_piece.row + row_offset as i8;
                let col = self.current_piece.col + col_offset as i8;
                if bb_space == &Space::Block {
                    playfield.set(row as u8, col as u8);
                }
            }
        }

        write!(f, "{:?}", playfield)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use std::collections::HashSet;

    #[test]
    fn test_engine_new() {
        let engine = Engine::new();

        // Playfield should start empty.
        for row in 1..=Playfield::TOTAL_HEIGHT {
            for col in 1..=Playfield::WIDTH {
                assert_eq!(engine.playfield.get(row, col), Space::Empty);
            }
        }

        // Current piece should be in spawn rotation.
        assert_eq!(engine.current_piece.piece.get_rotation(), &Rotation::Spawn);
    }

    #[test]
    fn test_current_piece_new() {
        assert_current_piece_new(CurrentPiece::new(Tetromino::I), Tetromino::I);
        assert_current_piece_new(CurrentPiece::new(Tetromino::O), Tetromino::O);
        assert_current_piece_new(CurrentPiece::new(Tetromino::T), Tetromino::T);
        assert_current_piece_new(CurrentPiece::new(Tetromino::S), Tetromino::S);
        assert_current_piece_new(CurrentPiece::new(Tetromino::Z), Tetromino::Z);
        assert_current_piece_new(CurrentPiece::new(Tetromino::J), Tetromino::J);
        assert_current_piece_new(CurrentPiece::new(Tetromino::L), Tetromino::L);
    }

    fn assert_current_piece_new(piece: CurrentPiece, expected_shape: Tetromino) {
        assert_eq!(piece.piece.get_rotation(), &Rotation::Spawn);
        assert_eq!(piece.piece.get_shape(), &expected_shape);
        assert_eq!(piece.row, 20);
        assert_eq!(piece.col, 4);
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
    fn test_engine_has_collision() {
        let mut engine = Engine::new();
        assert!(!engine.has_collision());

        // The spawn location should always overlap with this space.
        engine.playfield.set(22, 5);
        assert!(engine.has_collision());
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
    }

    #[test]
    fn test_engine_drop_collision() {
        let mut engine = Engine::new();
        let start_row = engine.current_piece.row;

        // Bottom of tetromino should start on row 22, so we should be able to drop 21 rows.
        for drop in 1..=21 {
            engine.drop();
            assert_eq!(engine.current_piece.row, start_row - drop);
        }

        // The tetromino should be at the bottom of the playfield
        // so dropping again should have no effect.
        engine.drop();
        assert_eq!(engine.current_piece.row, start_row - 21);
        engine.drop();
        assert_eq!(engine.current_piece.row, start_row - 21);

        // Add an obstacle, then test that piece cannot drop past it.
        engine.next_piece();
        engine.playfield.set(15, 5);

        // We should be able to drop 6 rows before hitting the obstacle.
        for drop in 1..=6 {
            engine.drop();
            assert_eq!(engine.current_piece.row, start_row - drop);
        }
        // Futher attempts to drop will fail since would collide witht he obstacle.
        engine.drop();
        assert_eq!(engine.current_piece.row, start_row - 6);
        engine.drop();
        assert_eq!(engine.current_piece.row, start_row - 6);
    }

    #[test]
    fn test_engine_rotate_piece() {
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
    fn test_engine_rotate_piece_collision() {
        let mut engine = Engine::new();

        // Create obstacle directly below spawn location.
        engine.playfield.set(21, 4);
        engine.playfield.set(21, 5);
        engine.playfield.set(21, 6);
        engine.playfield.set(21, 7);

        // O piece will not collide, so get a new piece
        while engine.current_piece.piece.get_shape() == &Tetromino::O {
            engine.next_piece();
        }

        // attempt rotate
        engine.rotate_piece_cw();
        assert_eq!(engine.current_piece.piece.get_rotation(), &Rotation::Spawn);

        engine.rotate_piece_ccw();
        assert_eq!(engine.current_piece.piece.get_rotation(), &Rotation::Spawn);
        // assert rotation == spawn
    }

    #[test]
    fn test_engine_move_piece() {
        let mut engine = Engine::new();

        // Test move left.
        let start_col = engine.current_piece.col;
        engine.move_piece_left();
        assert_eq!(engine.current_piece.col, start_col - 1);
        engine.move_piece_left();
        assert_eq!(engine.current_piece.col, start_col - 2);
        engine.move_piece_left();
        assert_eq!(engine.current_piece.col, start_col - 3);

        // Spawn a new piece then test move right.
        let start_col = engine.current_piece.col;
        engine.move_piece_right();
        assert_eq!(engine.current_piece.col, start_col + 1);
        engine.move_piece_right();
        assert_eq!(engine.current_piece.col, start_col + 2);
        engine.move_piece_right();
        assert_eq!(engine.current_piece.col, start_col + 3);
    }

    #[test]
    fn test_engine_move_piece_collision() {
        let mut engine = Engine::new();

        // Spawn new piece then move to far left.
        for _ in 0..Playfield::WIDTH {
            engine.move_piece_left();
        }
        // Moving further left should have no effect.
        let far_left_col = engine.current_piece.col;
        engine.move_piece_left();
        assert_eq!(engine.current_piece.col, far_left_col);
        engine.move_piece_left();
        assert_eq!(engine.current_piece.col, far_left_col);

        // Spawn new piece then do same as above, but move right.
        engine.next_piece();
        for _ in 0..Playfield::WIDTH {
            engine.move_piece_right();
        }
        let far_right_col = engine.current_piece.col;
        engine.move_piece_right();
        assert_eq!(engine.current_piece.col, far_right_col);
        engine.move_piece_right();
        assert_eq!(engine.current_piece.col, far_right_col);
    }
}
