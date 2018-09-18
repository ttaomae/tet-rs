use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt;

use rand::distributions::{Distribution, Standard};
use rand::Rng;

use super::core::{Piece, Playfield, Space, Tetromino};

const GRAVITY: u8 = 20;

/// The main game engine.
pub struct Engine {
    playfield: Playfield,
    current_piece: CurrentPiece,
    tetromino_generator: Box<TetrominoGenerator>,
    hold_piece: Option<Tetromino>,
    is_hold_available: bool,
    ticks_since_drop: u8,
}

/// The current piece on the playfield.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CurrentPiece {
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

    pub fn get_bounding_box(&self) -> [[Space; 4]; 4] {
        self.piece.get_bounding_box()
    }

    pub fn get_row(&self) -> i8 {
        self.row
    }

    pub fn get_col(&self) -> i8 {
        self.col
    }
}

enum DropResult {
    Success,
    Collision,
}

impl Engine {
    /// Creates a new engine with an empty playfield.
    pub fn new() -> Engine {
        let tetromino_generator = Box::new(BagGenerator::new());
        let current_piece = CurrentPiece::new(tetromino_generator.next());
        Engine {
            playfield: Playfield::new(),
            current_piece,
            tetromino_generator,
            hold_piece: Option::None,
            is_hold_available: true,
            ticks_since_drop: 0,
        }
    }

    pub fn get_playfield(&self) -> Playfield {
        self.playfield
    }

    pub fn get_current_piece(&self) -> CurrentPiece {
        self.current_piece
    }

    /* * * * * * * * * *
     * Engine actions. *
     * * * * * * * * * */
    // Actions performed by the engine.

    pub fn tick(&mut self) {
        if self.ticks_since_drop == GRAVITY {
            if let DropResult::Collision = self.drop() {
                self.lock();
                self.clear_rows();
                self.next_piece();
            }
            self.ticks_since_drop = 0;
        } else {
            self.ticks_since_drop += 1;
        }
    }

    /// Sets the next current piece.
    fn next_piece(&mut self) {
        self.current_piece = CurrentPiece::new(self.tetromino_generator.next());
        self.is_hold_available = true;
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
    fn drop(&mut self) -> DropResult {
        self.current_piece.row -= 1;
        if self.has_collision() {
            self.current_piece.row += 1;
            DropResult::Collision
        } else {
            DropResult::Success
        }
    }

    /// Locks the current piece into it's current location.
    fn lock(&mut self) {
        let bounding_box = self.current_piece.piece.get_bounding_box();
        // Iterate through spaces of bounding box.
        for (row_offset, bb_row) in bounding_box.iter().enumerate() {
            for (col_offset, bb_space) in bb_row.iter().enumerate() {
                // Collisions can only occur on blocks.
                if bb_space == &Space::Block {
                    // Calculate position of space in playfield.
                    let row = (self.current_piece.row + row_offset as i8) as u8;
                    let col = (self.current_piece.col + col_offset as i8) as u8;
                    self.playfield.set(row as u8, col as u8);
                }
            }
        }
    }

    /// Clears any rows that are full and drops blocks down.
    fn clear_rows(&mut self) {
        // Construct a list of all row that will NOT be cleared.
        let mut non_full_rows = Vec::with_capacity(Playfield::TOTAL_HEIGHT as usize);
        for row in 1..=Playfield::TOTAL_HEIGHT {
            for col in 1..=Playfield::WIDTH {
                // Any row that has a space will not be cleared.
                if self.playfield.get(row, col) == Space::Empty {
                    non_full_rows.push(row);
                    break;
                }
            }
        }

        // Don't do anything if no rows are full
        if non_full_rows.len() == Playfield::TOTAL_HEIGHT as usize {
            return;
        }

        // Copy non-full rows to next available row. Since full rows are not in the list, this has
        // the effect of overwriting the full rows.
        let mut current_row = 1;
        for row in non_full_rows {
            // Copy non-full row to current row.
            for col in 1..=Playfield::WIDTH {
                match self.playfield.get(row, col) {
                    Space::Empty => self.playfield.clear(current_row, col),
                    Space::Block => self.playfield.set(current_row, col),
                };
            }
            current_row += 1;
        }

        // Clear remaining rows.
        for row in current_row..Playfield::TOTAL_HEIGHT {
            for col in 1..=Playfield::WIDTH {
                self.playfield.clear(row, col);
            }
        }
    }

    /* * * * * * * * * *
     * Player actions. *
     * * * * * * * * * */
    // Actions initiated by the player.

    /// Drops the piece as far as possible without collision, then locks it into place.
    pub fn hard_drop(&mut self) {
        while !self.has_collision() {
            self.current_piece.row -= 1;
        }
        self.current_piece.row += 1;
        self.lock();
    }

    /// Rotates the current piece clockwise, if it does not result in a collision.
    pub fn rotate_piece_cw(&mut self) {
        self.current_piece.rotate_cw();
        if self.has_collision() {
            self.current_piece.rotate_ccw();
        }
    }

    /// Rotates the current piece counter-clockwise, if it does not result in a collision.
    pub fn rotate_piece_ccw(&mut self) {
        self.current_piece.rotate_ccw();
        if self.has_collision() {
            self.current_piece.rotate_cw();
        }
    }

    /// Moves the current piece one column to the left, if it does not result in a collision.
    pub fn move_piece_left(&mut self) {
        self.current_piece.col -= 1;
        if self.has_collision() {
            self.current_piece.col += 1;
        }
    }

    /// Moves the current piece one column to the right, if it does not result in a collision.
    pub fn move_piece_right(&mut self) {
        self.current_piece.col += 1;
        if self.has_collision() {
            self.current_piece.col -= 1;
        }
    }

    // Holds the current piece and replaces it with the existing hold piece or with the next piece
    // if there was no hold piece.
    pub fn hold_piece(&mut self) {
        if self.is_hold_available {
            let current_tetromino = *self.current_piece.piece.get_shape();

            match self.hold_piece {
                Option::Some(piece) => self.current_piece = CurrentPiece::new(piece),
                Option::None => self.next_piece(),
            }
            self.hold_piece = Option::Some(current_tetromino);
            self.is_hold_available = false;
        }
    }
}

trait TetrominoGenerator {
    fn next(&self) -> Tetromino;
}

struct BagGenerator {
    bag: RefCell<VecDeque<Tetromino>>,
}

impl BagGenerator {
    fn new() -> BagGenerator {
        let mut bag = VecDeque::with_capacity(7);
        bag.extend(BagGenerator::new_bag().iter());
        BagGenerator {
            bag: RefCell::from(bag),
        }
    }

    fn new_bag() -> [Tetromino; 7] {
        let mut bag = [
            Tetromino::I,
            Tetromino::O,
            Tetromino::T,
            Tetromino::S,
            Tetromino::Z,
            Tetromino::J,
            Tetromino::L,
        ];
        rand::thread_rng().shuffle(&mut bag);
        bag
    }
}

impl TetrominoGenerator for BagGenerator {
    fn next(&self) -> Tetromino {
        if self.bag.borrow().is_empty() {
            self.bag.borrow_mut().extend(BagGenerator::new_bag().iter());
        }

        // Since we fill the bag if it is empty, pop_front should always return Option::Some.
        self.bag.borrow_mut().pop_front().unwrap()
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

        for _ in 0..10 {
            let mut piece = engine.current_piece.piece;
            assert_eq!(piece.get_rotation(), &Rotation::Spawn);
            // Rotate the piece and verify that next piece is in spawn rotation on next iteration.
            piece.rotate_cw();

            engine.next_piece()
        }
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
        // Futher attempts to drop will fail since it would collide with the obstacle.
        engine.drop();
        assert_eq!(engine.current_piece.row, start_row - 6);
        engine.drop();
        assert_eq!(engine.current_piece.row, start_row - 6);
    }

    #[test]
    fn test_engine_lock() {
        let mut engine = Engine::new();

        // Drop piece to bottom then lock into place.
        for _ in 0..21 {
            engine.drop();
        }
        // Column 5 is guaranteed to be occupied for all pieces in spawn position.
        // Check that it is empty before locking and occupied after locking.
        assert_eq!(engine.playfield.get(1, 5), Space::Empty);
        engine.lock();
        assert_eq!(engine.playfield.get(1, 5), Space::Block);

        // Move piece to far left, drop piece to bottom, then lock into place.
        engine.next_piece();
        for _ in 0..Playfield::WIDTH {
            engine.move_piece_left();
        }
        for _ in 0..21 {
            engine.drop();
        }
        // Column 2 is guaranteed to be occupied for all pieces in far left.
        // Check that it is empty before locking and occupied after locking.
        assert_eq!(engine.playfield.get(1, 2), Space::Empty);
        engine.lock();
        assert_eq!(engine.playfield.get(1, 2), Space::Block);

        // Move piece to far right, drop piece to bottom, then lock into place.
        engine.next_piece();
        for _ in 0..Playfield::WIDTH {
            engine.move_piece_right();
        }
        for _ in 0..21 {
            engine.drop();
        }
        // Column 9 is guaranteed to be occupied for all pieces in far right.
        // Check that it is empty before locking and occupied after locking.
        assert_eq!(engine.playfield.get(1, 9), Space::Empty);
        engine.lock();
        assert_eq!(engine.playfield.get(1, 9), Space::Block);
    }

    #[test]
    fn test_clear_rows() {
        let mut engine = Engine::new();

        // Fill first, second, and fourth row.
        for col in 1..=Playfield::WIDTH {
            engine.playfield.set(1, col);
            engine.playfield.set(2, col);
            engine.playfield.set(4, col);
        }
        // Fill miscellaneous spaces in other rows.
        engine.playfield.set(3, 3);
        engine.playfield.set(3, 6);
        engine.playfield.set(5, 1);
        engine.playfield.set(6, 4);
        engine.playfield.set(6, 10);
        engine.playfield.set(7, 2);
        engine.playfield.set(7, 5);
        engine.playfield.set(7, 7);
        engine.playfield.set(8, 9);

        // Playfield should now look like this (ignoring empty rows).
        // 8 --------#-
        // 7 -#--#-#---
        // 6 ---#-----#
        // 5 #---------
        // 4 ##########
        // 3 --#--#----
        // 2 ##########
        // 1 ##########
        //   1234567890

        engine.clear_rows();
        // Playfield should now look like this (ignoring empty rows).
        // 5 --------#-
        // 4 -#--#-#---
        // 3 ---#-----#
        // 2 #---------
        // 1 --#--#----
        //   1234567890
        assert_eq!(engine.playfield.get(1, 3), Space::Block);
        assert_eq!(engine.playfield.get(1, 6), Space::Block);
        assert_eq!(engine.playfield.get(2, 1), Space::Block);
        assert_eq!(engine.playfield.get(3, 4), Space::Block);
        assert_eq!(engine.playfield.get(3, 10), Space::Block);
        assert_eq!(engine.playfield.get(4, 2), Space::Block);
        assert_eq!(engine.playfield.get(4, 5), Space::Block);
        assert_eq!(engine.playfield.get(4, 7), Space::Block);
        assert_eq!(engine.playfield.get(5, 9), Space::Block);

        // Rows above should be empty.
        for row in 6..=8 {
            for col in 1..Playfield::WIDTH {
                assert_eq!(engine.playfield.get(row, col), Space::Empty);
            }
        }
    }

    #[test]
    fn test_engine_hard_drop() {
        let mut engine = Engine::new();

        // Column 5 is guaranteed to be occupied for all pieces in spawn position.
        engine.hard_drop();
        assert_eq!(engine.playfield.get(1, 5), Space::Block);

        // Move piece to far left, then hard drop.
        engine.next_piece();
        for _ in 0..Playfield::WIDTH {
            engine.move_piece_left();
        }
        // Column 2 is guaranteed to be occupied for all pieces in far left.
        engine.hard_drop();
        assert_eq!(engine.playfield.get(1, 2), Space::Block);

        // Move piece to far right, then hard drop.
        engine.next_piece();
        for _ in 0..Playfield::WIDTH {
            engine.move_piece_right();
        }
        // Column 9 is guaranteed to be occupied for all pieces in far right.
        engine.hard_drop();
        assert_eq!(engine.playfield.get(1, 9), Space::Block);
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

    #[test]
    fn test_engine_hold_piece() {
        let mut engine = Engine::new();

        //
        assert!(engine.hold_piece.is_none());

        let current_piece = engine.current_piece.piece.get_shape().clone();
        engine.hold_piece();

        let hold_piece = engine.hold_piece.unwrap();
        assert_eq!(hold_piece, current_piece);

        // Hold is only allowed once per piece.
        let current_piece = engine.current_piece.piece.get_shape().clone();
        engine.hold_piece();
        assert_eq!(engine.hold_piece.unwrap(), hold_piece);
        assert_eq!(engine.current_piece.piece.get_shape(), &current_piece);

        engine.next_piece();

        // Hold piece should be the same.
        assert_eq!(engine.hold_piece.unwrap(), hold_piece);
        // Since we are on the next piece hold should work again.
        let current_piece = engine.current_piece.piece.get_shape().clone();
        engine.hold_piece();
        assert_eq!(engine.hold_piece.unwrap(), current_piece);
        assert_eq!(engine.current_piece.piece.get_shape(), &hold_piece);
    }

    #[test]
    fn test_bag_generator() {
        let bag_generator = BagGenerator::new();

        // The bag generator should always generate tetrominos in sets containing one of each.
        for _ in 0..5 {
            let mut tetrominos = HashSet::new();
            for _ in 0..7 {
                tetrominos.insert(bag_generator.next());
            }
            assert_eq!(tetrominos.len(), 7);
        }
    }
}
