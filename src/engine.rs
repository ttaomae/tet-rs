use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;

use rand::distributions::{Distribution, Standard};
use rand::Rng;

use super::core::{Piece, Playfield, Rotation, Space, Tetromino};

const AUTO_REPEAT_DELAY: u32 = 12;
const AUTO_REPEAT_RATE: u32 = 7;

/// The main game engine.
pub struct Engine {
    playfield: Playfield,
    current_piece: CurrentPiece,
    tetromino_generator: Box<dyn TetrominoGenerator>,
    hold_piece: Option<Tetromino>,
    is_hold_available: bool,
    ticks_since_drop: u8,
    current_tick_inputs: RefCell<HashSet<Action>>,
    current_inputs: HashMap<Action, u32>,
    gravity: Gravity,
    soft_drop_gravity: Gravity,
    next_pieces: VecDeque<Tetromino>,
}

enum Gravity {
    TicksPerRow(u8),
    RowsPerTick(u8),
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
enum Action {
    MoveLeft,
    MoveRight,
    RotateClockwise,
    RotateCounterClockwise,
    SoftDrop,
    HardDrop,
    Hold,
}

const ALL_ACTIONS: [Action; 7] = [
    Action::MoveLeft,
    Action::MoveRight,
    Action::RotateClockwise,
    Action::RotateCounterClockwise,
    Action::SoftDrop,
    Action::HardDrop,
    Action::Hold,
];

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
            row: 19,
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
    /// Creates a new engine with the specified tetromino generator.
    fn with_tetromino_generator(tetromino_generator: Box<dyn TetrominoGenerator>) -> Engine {
        let current_piece = CurrentPiece::new(tetromino_generator.next());
        let mut next_pieces = VecDeque::with_capacity(5);
        for _ in 0..5 {
            next_pieces.push_back(tetromino_generator.next());
        }
        let mut current_inputs = HashMap::new();
        for action in ALL_ACTIONS.iter() {
            current_inputs.insert(*action, 0u32);
        }
        Engine {
            playfield: Playfield::new(),
            current_piece,
            tetromino_generator,
            hold_piece: Option::None,
            is_hold_available: true,
            ticks_since_drop: 0,
            current_tick_inputs: RefCell::new(HashSet::new()),
            current_inputs,
            gravity: Gravity::TicksPerRow(40),
            soft_drop_gravity: Gravity::TicksPerRow(2),
            next_pieces,
        }
    }

    /// Creates a new engine with default settings.
    pub fn new() -> Engine {
        Engine::with_tetromino_generator(Box::new(BagGenerator::new()))
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
        let actions = self.get_actions();
        if !self.apply_hold(&actions) {
            self.apply_piece_move(&actions);
            self.apply_piece_rotation(&actions);
            self.apply_gravity(&actions);
        }
    }

    /// Processes input and returns a list of actions to perform on this tick.
    fn get_actions(&mut self) -> HashSet<Action> {
        // Clear current_tick_inputs and update current_inputs.
        for action in ALL_ACTIONS.iter() {
            if self.current_tick_inputs.borrow_mut().remove(&action) {
                match self.current_inputs.get_mut(&action) {
                    Option::Some(duration) => {
                        *duration += 1;
                    },
                    Option::None => panic!(),
                }
            }
            else {
                match self.current_inputs.get_mut(&action) {
                    Option::Some(duration) => *duration = 0,
                    Option::None => panic!(),
                }
            }
        }

        // Special case: When 'left' and 'right' input are both pressed at the same time, give
        // priority to 'left'. Reset 'right' duration so that when 'left' is released, 'right'
        // starts with duration zero rather than being in the middle of auto-repeat, which would
        // lead to inconsistent behavior.
        if let Option::Some(duration) = self.current_inputs.get(&Action::MoveLeft) {
            if *duration > 0u32 {
                self.current_inputs.insert(Action::MoveRight, 0);
            }
        }

        let mut current_turn_actions = HashSet::new();
        // Iterate through inputs and determine which actions are valid.
        for (action, duration) in self.current_inputs.iter() {
            use self::Action::*;
            match action {
                // These actions are only valid on initial press.
                Hold | RotateClockwise | RotateCounterClockwise | HardDrop => {
                    if *duration == 1 {
                        current_turn_actions.insert(*action);
                    }
                },
                // This is always valid if pressed.
                SoftDrop => {
                    if *duration >= 1 {
                        current_turn_actions.insert(*action);
                    }
                },
                // This is valid on first press, when reaching auto-repeat delay,
                // or on intervals based on the auto-repeat rate.
                MoveLeft | MoveRight => {
                    if *duration == 1 || *duration == AUTO_REPEAT_DELAY
                        || *duration > AUTO_REPEAT_DELAY && (*duration - AUTO_REPEAT_DELAY) % AUTO_REPEAT_RATE == 0
                    {
                        current_turn_actions.insert(*action);
                    }
                },
            }
        }

        current_turn_actions
    }

    /// Attempts to hold the current piece if it is one of the specified actions.
    /// Returns whether or not the the hold was successful.
    fn apply_hold(&mut self, actions: &HashSet<Action>) -> bool {
        if actions.contains(&Action::Hold) {
            if self.is_hold_available {
                self.hold_piece();
                self.is_hold_available = false;
                return true;
            }
        }

        false
    }

    /// Holds the current piece. Swaps with the current hold piece, if it exists, or generates the
    /// next piece if there is no current hold piece.
    fn hold_piece(&mut self) {
        let current_tetromino = *self.current_piece.piece.get_shape();

        match self.hold_piece {
            Option::Some(piece) => self.current_piece = CurrentPiece::new(piece),
            Option::None => self.next_piece(),
        }
        self.hold_piece = Option::Some(current_tetromino);
    }

    /// Applies move if contained in the specified action set.
    /// Left moves are given priority over left moves.
    fn apply_piece_move(&mut self, actions: &HashSet<Action>) {
        if actions.contains(&Action::MoveLeft) {
            self.move_piece(-1);
        } else if actions.contains(&Action::MoveRight) {
            self.move_piece(1);
        }
    }

    /// Applies rotation if contained in the specified action set.
    /// Clockwise rotation is given priority over counter-clockwise rotations.
    fn apply_piece_rotation(&mut self, actions: &HashSet<Action>) {
            if actions.contains(&Action::RotateClockwise) {
                self.rotate_piece_cw();
            } else if actions.contains(&Action::RotateCounterClockwise) {
                self.rotate_piece_ccw();
            }
    }

    /// Applies gravity, given the specified action set.
    fn apply_gravity(&mut self, actions: &HashSet<Action>) {
        // Handle hard drop.
        if actions.contains(&Action::HardDrop) {
            // Do not apply hard drop if piece was held this turn.
            if !actions.contains(&Action::Hold) {
                self.drop(Playfield::TOTAL_HEIGHT);
                self.lock_clear_next();
                self.ticks_since_drop = 0;
            }
            return;
        }

        let gravity = if actions.contains(&Action::SoftDrop) {
            &self.soft_drop_gravity
        } else {
            &self.gravity
        };

        // Handle normal gravity.
        match gravity {
            Gravity::TicksPerRow(tpr) => {
                if self.ticks_since_drop >= *tpr {
                    if let DropResult::Collision = self.drop_one() {
                        self.lock_clear_next();
                    }
                    self.ticks_since_drop = 0;
                }
                else {
                    self.ticks_since_drop += 1;
                }
            },
            _ => unimplemented!(),
        }
    }

    /// Sets the next current piece.
    fn next_piece(&mut self) {
        self.current_piece = match self.next_pieces.pop_front() {
            Option::Some(piece) => CurrentPiece::new(piece),
            Option::None => panic!("This should never happen."),
        };

        self.next_pieces.push_back(self.tetromino_generator.next());
        self.is_hold_available = true;
    }

    /// Returns whether or not there is a collision between the playfield and the current piece.
    fn has_collision(&self) -> bool {
        self.has_collision_with_piece(self.current_piece)
    }

    /// Returns whether or not there would be a collision
    /// between the playfield and the specified piece.
    fn has_collision_with_piece(&self, piece: CurrentPiece) -> bool {
        let bounding_box = piece.piece.get_bounding_box();
        // Iterate through spaces of bounding box.
        for (row_offset, bb_row) in bounding_box.iter().enumerate() {
            for (col_offset, bb_space) in bb_row.iter().enumerate() {
                // Calculate position of space in playfield.
                let row = piece.row + row_offset as i8;
                let col = piece.col + col_offset as i8;

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
    fn drop_one(&mut self) -> DropResult {
        self.drop(1)
    }

    /// Drops the current piece by up to the specified number of row, or until there is a collision.
    fn drop(&mut self, n_rows: u8) -> DropResult {
        for _ in 0..n_rows {
            self.current_piece.row -= 1;
            if self.has_collision() {
                self.current_piece.row += 1;
                return DropResult::Collision;
            }
        }

        DropResult::Success
    }

    /// Locks the current piece into place, clears rows if necessary, then generates next the piece.
    fn lock_clear_next(&mut self) {
        self.lock();
        self.clear_rows();
        self.next_piece();
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

    /// Moves the current piece horizontally by up to the specified amount.
    fn move_piece(&mut self, col_offset: i8) {
        for _ in 0..col_offset.abs() {
            self.current_piece.col += col_offset.signum();
            if self.has_collision() {
                self.current_piece.col -= col_offset.signum();
            }
        }
    }

    /// Rotates the current piece clockwise.
    fn rotate_piece_cw(&mut self) {
        self.rotate_piece(|p| p.rotate_cw());
    }

    /// Rotates the current piece counter-clockwise.
    fn rotate_piece_ccw(&mut self) {
        self.rotate_piece(|p| p.rotate_ccw());
    }

    /// Rotates the current piece and applies wall kick, if possible. Otherwise, does nothing.
    fn rotate_piece<F>(&mut self, mut rotate: F) where F: FnMut(&mut CurrentPiece)
    {
        let initial = self.current_piece.piece.get_rotation().clone();
        let mut updated_piece = self.current_piece.clone();
        rotate(&mut updated_piece);
        let rotated = updated_piece.piece.get_rotation().clone();

        if let Option::Some((col_offset, row_offset)) =
            self.check_rotation(&mut updated_piece, initial, rotated)
        {
            self.current_piece.col += col_offset;
            self.current_piece.row += row_offset;
            rotate(&mut self.current_piece);
        }
    }

    /// Checks whether or not the specified piece would collide with the playfield.
    /// If it does, attempts to perform a wall kick based on the specified rotation.
    /// Returns the offset which resulted in no collision as (col_offset, row_offset)
    /// or `Option::None` if the rotation is not possible.
    fn check_rotation(
        &self,
        piece: &mut CurrentPiece,
        initial: Rotation,
        rotated: Rotation,
    ) -> Option<(i8, i8)> {
        if !self.has_collision_with_piece(*piece) {
            return Option::Some((0, 0));
        }

        use super::core::Rotation::*;
        // A list of (col, row) offsets for the given piece and rotation.
        let wall_kick_offsets = match piece.piece.get_shape() {
            // O rotations are identical. Since the piece does not move between rotations,
            // it cannot collide and should have passed the test above.
            Tetromino::O => panic!("This should be impossible"),
            // I has separate different wall kick rules.
            Tetromino::I => match (initial, rotated) {
                (Spawn, Clockwise) => vec![(-2, 0), (1, 0), (-2, -1), (1, 2)],
                (Clockwise, Spawn) => vec![(2, 0), (-1, 0), (2, 1), (-1, -2)],
                (Clockwise, OneEighty) => vec![(-1, 0), (2, 0), (-1, 2), (2, -1)],
                (OneEighty, Clockwise) => vec![(1, 0), (-2, 0), (1, -2), (-2, 1)],
                (OneEighty, CounterClockwise) => vec![(2, 0), (-1, 0), (2, 1), (-1, -2)],
                (CounterClockwise, OneEighty) => vec![(-2, 0), (1, 0), (-2, -1), (1, 2)],
                (CounterClockwise, Spawn) => vec![(1, 0), (-2, 0), (1, -2), (-2, 1)],
                (Spawn, CounterClockwise) => vec![(-1, 0), (2, 0), (-1, 2), (2, -1)],
                // The only cases left are 180 rotations, which are not supported.
                _ => panic!("This should be impossible"),
            },
            // All other pieces follow the same rules.
            _ => match (initial, rotated) {
                (Spawn, Clockwise) => vec![(-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (Clockwise, Spawn) => vec![(1, 0), (1, -1), (0, 2), (1, 2)],
                (Clockwise, OneEighty) => vec![(1, 0), (1, -1), (0, 2), (1, 2)],
                (OneEighty, Clockwise) => vec![(-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (OneEighty, CounterClockwise) => vec![(1, 0), (1, 1), (0, -2), (1, -2)],
                (CounterClockwise, OneEighty) => vec![(-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (CounterClockwise, Spawn) => vec![(-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (Spawn, CounterClockwise) => vec![(1, 0), (1, 1), (0, -2), (1, -2)],
                // The only cases left are 180 rotations, which are not supported.
                _ => panic!("This should be impossible"),
            },
        };

        // Check each offset.
        for offset in wall_kick_offsets {
            piece.col += offset.0;
            piece.row += offset.1;
            // Return if there was no collision.
            if !self.has_collision_with_piece(*piece) {
                return Option::Some(offset);
            }
            // Reset position for next test.
            piece.col -= offset.0;
            piece.row -= offset.1;
        }

        // Could not find a valid wall kick.
        Option::None
    }

    /* * * * * * * * * *
     * Player inputs. *
     * * * * * * * * * */
    // Methods to indicate inputs for the current tick.

    fn input_action(&self, action: Action) {
        self.current_tick_inputs.borrow_mut().insert(action);
    }

    pub fn input_move_left(&self) {
        self.input_action(Action::MoveLeft);
    }

    pub fn input_move_right(&self) {
        self.input_action(Action::MoveRight);
    }

    pub fn input_rotate_cw(&self) {
        self.input_action(Action::RotateClockwise);
    }

    pub fn input_rotate_ccw(&self) {
        self.input_action(Action::RotateCounterClockwise);
    }

    pub fn input_soft_drop(&self) {
        self.input_action(Action::SoftDrop);
    }

    pub fn input_hard_drop(&self) {
        self.input_action(Action::HardDrop);
    }

    pub fn input_hold(&self) {
        self.input_action(Action::Hold);
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

    enum SingleTetrominoGenerator {
        I,
        O,
        T,
        S,
        Z,
        J,
        L,
    }

    /// Always generate the same tetromino.
    impl TetrominoGenerator for SingleTetrominoGenerator {
        fn next(&self) -> Tetromino {
            match self {
                SingleTetrominoGenerator::I => Tetromino::I,
                SingleTetrominoGenerator::O => Tetromino::O,
                SingleTetrominoGenerator::T => Tetromino::T,
                SingleTetrominoGenerator::S => Tetromino::S,
                SingleTetrominoGenerator::Z => Tetromino::Z,
                SingleTetrominoGenerator::J => Tetromino::J,
                SingleTetrominoGenerator::L => Tetromino::L,
            }
        }
    }

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
        assert_eq!(piece.row, 19);
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
        engine.playfield.set(21, 5);
        assert!(engine.has_collision());
    }

    #[test]
    fn test_engine_drop() {
        let mut engine = Engine::new();
        let start_row = engine.current_piece.row;

        engine.drop_one();
        assert_eq!(engine.current_piece.row, start_row - 1);
        engine.drop_one();
        assert_eq!(engine.current_piece.row, start_row - 2);
        engine.drop_one();
        assert_eq!(engine.current_piece.row, start_row - 3);
        engine.drop_one();
        assert_eq!(engine.current_piece.row, start_row - 4);
        engine.drop_one();
        assert_eq!(engine.current_piece.row, start_row - 5);
        engine.drop(2);
        assert_eq!(engine.current_piece.row, start_row - 7);
        engine.drop(4);
        assert_eq!(engine.current_piece.row, start_row - 11);
    }

    #[test]
    fn test_engine_drop_collision() {
        let mut engine = Engine::new();
        let start_row = engine.current_piece.row;

        // Bottom of tetromino should start just above visible playfield, so we should be able to
        // drop the entire height of the playfield.
        for drop in 1..=Playfield::VISIBLE_HEIGHT as i8 {
            engine.drop_one();
            assert_eq!(engine.current_piece.row, start_row - drop);
        }

        // The tetromino should be at the bottom of the playfield
        // so dropping again should have no effect.
        engine.drop_one();
        assert_eq!(engine.current_piece.row, start_row - Playfield::VISIBLE_HEIGHT as i8);
        engine.drop_one();
        assert_eq!(engine.current_piece.row, start_row - Playfield::VISIBLE_HEIGHT as i8);

        // Perform same test with drop().
        engine.next_piece();
        engine.drop(25);
        assert_eq!(engine.current_piece.row, start_row - Playfield::VISIBLE_HEIGHT as i8);


        // Add an obstacle, then test that piece cannot drop past it.
        engine.next_piece();
        engine.playfield.set(15, 5);

        // We should be able to drop 5 rows before hitting the obstacle.
        for drop in 1..=5 {
            engine.drop_one();
            assert_eq!(engine.current_piece.row, start_row - drop);
        }
        // Futher attempts to drop will fail since it would collide with the obstacle.
        engine.drop_one();
        assert_eq!(engine.current_piece.row, start_row - 5);
        engine.drop(4);
        assert_eq!(engine.current_piece.row, start_row - 5);

        // Perform same test with drop().
        engine.next_piece();
        engine.drop(10);
        assert_eq!(engine.current_piece.row, start_row - 5);
    }

    #[test]
    fn test_engine_lock() {
        let mut engine = Engine::with_tetromino_generator(Box::new(SingleTetrominoGenerator::S));

        // Drop and lock three S tetrominos in spawn position, far left, and far right.
        // Check before and after locking that expected pieces are empty/occupied.
        // -##-##--##
        // ##-##--##-
        // 1234567890

        // Spawn position.
        engine.next_piece();
        engine.drop(Playfield::VISIBLE_HEIGHT);
        assert_eq!(engine.playfield.get(1, 4), Space::Empty);
        assert_eq!(engine.playfield.get(1, 5), Space::Empty);
        engine.lock();
        assert_eq!(engine.playfield.get(1, 4), Space::Block);
        assert_eq!(engine.playfield.get(1, 5), Space::Block);

        // Far left.
        engine.next_piece();
        engine.move_piece(-10);
        engine.drop(Playfield::VISIBLE_HEIGHT);
        assert_eq!(engine.playfield.get(1, 1), Space::Empty);
        assert_eq!(engine.playfield.get(1, 2), Space::Empty);
        engine.lock();
        assert_eq!(engine.playfield.get(1, 1), Space::Block);
        assert_eq!(engine.playfield.get(1, 2), Space::Block);

        // Far right.
        engine.next_piece();
        engine.move_piece(10);
        engine.drop(Playfield::VISIBLE_HEIGHT);
        assert_eq!(engine.playfield.get(1, 8), Space::Empty);
        assert_eq!(engine.playfield.get(1, 9), Space::Empty);
        engine.lock();
        assert_eq!(engine.playfield.get(1, 8), Space::Block);
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
        let mut engine = Engine::with_tetromino_generator(Box::new(SingleTetrominoGenerator::I));
        engine.next_piece();

        // Surround above and below to prevent rotation.
        for col in 4..=7 {
            engine.playfield.set(20, col);
            engine.playfield.set(22, col);
        }

        // attempt rotate
        engine.rotate_piece_cw();
        assert_eq!(engine.current_piece.piece.get_rotation(), &Rotation::Spawn);

        engine.rotate_piece_ccw();
        assert_eq!(engine.current_piece.piece.get_rotation(), &Rotation::Spawn);
    }

    #[test]
    fn test_engine_rotate_piece_wall_kick() {
        let mut engine = Engine::with_tetromino_generator(Box::new(SingleTetrominoGenerator::T));
        engine.next_piece();

        // Setup wall kick
        // T---------
        // TT#-------
        // T--#------
        engine.playfield.set(1, 4);
        engine.playfield.set(2, 3);
        engine.rotate_piece_cw();
        engine.move_piece(-10);
        engine.drop(Playfield::VISIBLE_HEIGHT);

        // Perform wall kick and lock into place.
        // ----------
        // -T#-------
        // TTT#------
        engine.rotate_piece_ccw();
        engine.lock();

        // Check that piece in expected position.
        assert_eq!(engine.playfield.get(1, 1), Space::Block);
        assert_eq!(engine.playfield.get(1, 2), Space::Block);
        assert_eq!(engine.playfield.get(1, 3), Space::Block);
        assert_eq!(engine.playfield.get(2, 2), Space::Block);
    }

    #[test]
    fn test_engine_move_piece() {
        let mut engine = Engine::new();

        // Test move left.
        let start_col = engine.current_piece.col;
        engine.move_piece(-1);
        assert_eq!(engine.current_piece.col, start_col - 1);
        engine.move_piece(-1);
        assert_eq!(engine.current_piece.col, start_col - 2);
        engine.move_piece(-1);
        assert_eq!(engine.current_piece.col, start_col - 3);

        // Spawn a new piece then test move right.
        let start_col = engine.current_piece.col;
        engine.move_piece(1);
        assert_eq!(engine.current_piece.col, start_col + 1);
        engine.move_piece(1);
        assert_eq!(engine.current_piece.col, start_col + 2);
        engine.move_piece(1);
        assert_eq!(engine.current_piece.col, start_col + 3);
    }

    #[test]
    fn test_engine_move_piece_collision() {
        let mut engine = Engine::new();

        // Spawn new piece then move to far left.
        engine.move_piece(-10);
        // Moving further left should have no effect.
        let far_left_col = engine.current_piece.col;
        engine.move_piece(-1);
        assert_eq!(engine.current_piece.col, far_left_col);
        engine.move_piece(-1);
        assert_eq!(engine.current_piece.col, far_left_col);

        // Spawn new piece then do same as above, but move right.
        engine.next_piece();
        engine.move_piece(10);
        let far_right_col = engine.current_piece.col;
        engine.move_piece(1);
        assert_eq!(engine.current_piece.col, far_right_col);
        engine.move_piece(1);
        assert_eq!(engine.current_piece.col, far_right_col);
    }

    #[test]
    fn test_engine_hold_piece() {
        let mut engine = Engine::new();

        assert!(engine.hold_piece.is_none());

        let current_piece = engine.current_piece.piece.get_shape().clone();
        engine.hold_piece();

        let hold_piece = engine.hold_piece.unwrap();
        assert_eq!(hold_piece, current_piece);
    }

    #[test]
    fn test_engine_next_pieces() {
        let mut engine = Engine::new();

        for _ in 0..10 {
            let next_piece = engine.next_pieces[0];
            engine.next_piece();
            assert_eq!(engine.current_piece.piece.get_shape(), &next_piece);
        }
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
