use std::fmt;

/// The playfield where blocks are placed.
pub struct Playfield {
    grid: [[Space; 10]; 40],
}

/// A space in the playfield.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Space {
    Empty,
    Block,
}

impl Playfield {
    pub const WIDTH: u8 = 10;
    pub const VISIBLE_HEIGHT: u8 = 20;
    pub const TOTAL_HEIGHT: u8 = 40;

    /// Creates a new empty playfield.
    pub fn new() -> Playfield {
        Playfield {
            grid: [[Space::Empty; Playfield::WIDTH as usize]; Playfield::TOTAL_HEIGHT as usize],
        }
    }

    /// Gets the space at the specified row and column.
    pub fn get(&self, row: u8, col: u8) -> Space {
        Playfield::check_index(row, col);
        self.grid[row as usize - 1][col as usize - 1]
    }

    // Sets the space at the specified row and column to a block.
    fn set(&mut self, row: u8, col: u8) {
        Playfield::check_index(row, col);
        self.grid[row as usize - 1][col as usize - 1] = Space::Block;
    }

    // Clears the space at the specified row and column.
    fn clear(&mut self, row: u8, col: u8) {
        Playfield::check_index(row, col);
        self.grid[row as usize - 1][col as usize - 1] = Space::Empty
    }

    /// Panics if row or column are out of bounds.
    fn check_index(row: u8, col: u8) {
        if row < 1 || row > Playfield::TOTAL_HEIGHT {
            panic!("row must be be between 1 and 40.");
        }
        if col < 1 || col > Playfield::WIDTH {
            panic!("col mus tbe between 1 and 10.");
        }
    }
}

/// A shape consisting of four connected squares.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Tetromino {
    /// ```
    /// #
    /// #
    /// #
    /// #
    /// ```
    I,
    /// ```
    /// ##
    /// ##
    /// ```
    O,
    /// ```
    /// ###
    ///  #
    /// ```
    T,
    /// ```
    ///  ##
    /// ##
    /// ```
    S,
    /// ```
    /// ##
    ///  ##
    /// ```
    Z,
    /// ```
    ///  #
    ///  #
    /// ##
    /// ```
    J,
    /// ```
    /// #
    /// #
    /// ##
    /// ```
    L,
}

/// The rotation state of a tetromino.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Rotation {
    /// The default rotation when a piece is spawned.
    Spawn,
    /// 90 degree clockwise rotation from spawn rotation.
    Clockwise,
    /// 180 degree rotation from spawn rotation.
    OneEighty,
    /// 90 degree counter-clockwise rotation from spawn rotation.
    CounterClockwise,
}

impl Rotation {
    fn cw(&self) -> Rotation {
        match self {
            Rotation::Spawn => Rotation::Clockwise,
            Rotation::Clockwise => Rotation::OneEighty,
            Rotation::OneEighty => Rotation::CounterClockwise,
            Rotation::CounterClockwise => Rotation::Spawn,
        }
    }

    fn ccw(&self) -> Rotation {
        match self {
            Rotation::Spawn => Rotation::CounterClockwise,
            Rotation::Clockwise => Rotation::Spawn,
            Rotation::OneEighty => Rotation::Clockwise,
            Rotation::CounterClockwise => Rotation::OneEighty,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Piece {
    shape: Tetromino,
    rotation: Rotation,
}

impl Piece {
    pub fn new(shape: Tetromino) -> Piece {
        Piece {
            shape: shape,
            rotation: Rotation::Spawn,
        }
    }

    pub fn get_shape(&self) -> &Tetromino {
        &self.shape
    }

    pub fn get_rotation(&self) -> &Rotation {
        &self.rotation
    }

    pub fn rotate_cw(&mut self) {
        self.rotation = self.rotation.cw();
    }

    pub fn rotate_ccw(&mut self) {
        self.rotation = self.rotation.ccw();
    }
}

impl fmt::Debug for Space {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Space::Empty => write!(f, "."),
            Space::Block => write!(f, "#"),
        }
    }
}

impl fmt::Debug for Playfield {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // only display first 20 rows
        for row in (0..Playfield::VISIBLE_HEIGHT as usize).rev() {
            for col in &self.grid[row] {
                write!(f, "{:?}", col)?
            }
            write!(f, "\n")?
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playfield_new() {
        let playfield = Playfield::new();
        for row in 1..=Playfield::TOTAL_HEIGHT {
            for col in 1..=Playfield::WIDTH {
                assert_eq!(playfield.get(row, col), Space::Empty);
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_playfield_get_row_less() {
        let playfield = Playfield::new();
        playfield.get(0, 1);
    }

    #[test]
    #[should_panic]
    fn test_playfield_get_row_greater() {
        let playfield = Playfield::new();
        playfield.get(Playfield::TOTAL_HEIGHT + 1, 1);
    }

    #[test]
    #[should_panic]
    fn test_playfield_get_col_less() {
        let playfield = Playfield::new();
        playfield.get(1, 0);
    }

    #[test]
    #[should_panic]
    fn test_playfield_get_col_greater() {
        let playfield = Playfield::new();
        playfield.get(1, Playfield::WIDTH + 1);
    }

    #[test]
    fn test_playfield_set_and_clear() {
        let mut playfield = Playfield::new();
        for row in 1..=40 {
            for col in 1..=10 {
                playfield.set(row, col);
                assert_eq!(playfield.get(row, col), Space::Block);
                playfield.clear(row, col);
                assert_eq!(playfield.get(row, col), Space::Empty);
            }
        }
    }

    #[test]
    fn test_rotation_cw() {
        let r = Rotation::Spawn;
        assert_eq!(r.cw(), Rotation::Clockwise);
        assert_eq!(r.cw().cw(), Rotation::OneEighty);
        assert_eq!(r.cw().cw().cw(), Rotation::CounterClockwise);
        assert_eq!(r.cw().cw().cw().cw(), Rotation::Spawn);
    }

    #[test]
    fn test_rotation_ccw() {
        let r = Rotation::Spawn;
        assert_eq!(r.ccw(), Rotation::CounterClockwise);
        assert_eq!(r.ccw().ccw(), Rotation::OneEighty);
        assert_eq!(r.ccw().ccw().ccw(), Rotation::Clockwise);
        assert_eq!(r.ccw().ccw().ccw().ccw(), Rotation::Spawn);
    }

    #[test]
    fn test_piece_new() {
        assert_piece_new(Piece::new(Tetromino::I), Tetromino::I);
        assert_piece_new(Piece::new(Tetromino::O), Tetromino::O);
        assert_piece_new(Piece::new(Tetromino::T), Tetromino::T);
        assert_piece_new(Piece::new(Tetromino::S), Tetromino::S);
        assert_piece_new(Piece::new(Tetromino::Z), Tetromino::Z);
        assert_piece_new(Piece::new(Tetromino::J), Tetromino::J);
        assert_piece_new(Piece::new(Tetromino::L), Tetromino::L);
    }

    fn assert_piece_new(piece: Piece, expected_shape: Tetromino) {
        assert_eq!(piece.get_shape(), &expected_shape);
        assert_eq!(piece.get_rotation(), &Rotation::Spawn);
    }

    #[test]
    fn test_piece_rotate_cw() {
        assert_piece_rotate_cw(&mut Piece::new(Tetromino::I));
        assert_piece_rotate_cw(&mut Piece::new(Tetromino::O));
        assert_piece_rotate_cw(&mut Piece::new(Tetromino::T));
        assert_piece_rotate_cw(&mut Piece::new(Tetromino::S));
        assert_piece_rotate_cw(&mut Piece::new(Tetromino::Z));
        assert_piece_rotate_cw(&mut Piece::new(Tetromino::J));
        assert_piece_rotate_cw(&mut Piece::new(Tetromino::L));
    }

    fn assert_piece_rotate_cw(piece: &mut Piece) {
        assert_eq!(piece.get_rotation(), &Rotation::Spawn);
        piece.rotate_cw();
        assert_eq!(piece.get_rotation(), &Rotation::Clockwise);
        piece.rotate_cw();
        assert_eq!(piece.get_rotation(), &Rotation::OneEighty);
        piece.rotate_cw();
        assert_eq!(piece.get_rotation(), &Rotation::CounterClockwise);
        piece.rotate_cw();
        assert_eq!(piece.get_rotation(), &Rotation::Spawn);
    }

    #[test]
    fn test_piece_rotate_ccw() {
        assert_piece_rotate_ccw(&mut Piece::new(Tetromino::I));
        assert_piece_rotate_ccw(&mut Piece::new(Tetromino::O));
        assert_piece_rotate_ccw(&mut Piece::new(Tetromino::T));
        assert_piece_rotate_ccw(&mut Piece::new(Tetromino::S));
        assert_piece_rotate_ccw(&mut Piece::new(Tetromino::Z));
        assert_piece_rotate_ccw(&mut Piece::new(Tetromino::J));
        assert_piece_rotate_ccw(&mut Piece::new(Tetromino::L));
    }

    fn assert_piece_rotate_ccw(piece: &mut Piece) {
        assert_eq!(piece.get_rotation(), &Rotation::Spawn);
        piece.rotate_ccw();
        assert_eq!(piece.get_rotation(), &Rotation::CounterClockwise);
        piece.rotate_ccw();
        assert_eq!(piece.get_rotation(), &Rotation::OneEighty);
        piece.rotate_ccw();
        assert_eq!(piece.get_rotation(), &Rotation::Clockwise);
        piece.rotate_ccw();
        assert_eq!(piece.get_rotation(), &Rotation::Spawn);
    }
}
