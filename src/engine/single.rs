use super::core::{Playfield, Tetromino};
use super::base::{BaseEngine, Engine, Gravity, BaseEngineObserver, State, CurrentPiece, TSpin};
use std::cell::*;
use std::rc::Rc;

const GRAVITY: [Gravity;15] = [
    Gravity::TicksPerRow(60),
    Gravity::TicksPerRow(48),
    Gravity::TicksPerRow(37),
    Gravity::TicksPerRow(28),
    Gravity::TicksPerRow(21),
    Gravity::TicksPerRow(16),
    Gravity::TicksPerRow(11),
    Gravity::TicksPerRow(8),
    Gravity::TicksPerRow(6),
    Gravity::TicksPerRow(4),
    Gravity::TicksPerRow(3),
    Gravity::TicksPerRow(2),
    Gravity::TicksPerRow(1),
    Gravity::RowsPerTick(2),
    Gravity::RowsPerTick(3),
];

pub struct SinglePlayerEngine {
    base_engine: BaseEngine,
    stat_tracker: Rc<StatTracker>,
}

impl Engine for SinglePlayerEngine {
    fn tick(&mut self) -> State {
        let state = self.base_engine.tick();

        if let State::Spawn = state {
            self.base_engine.set_gravity(GRAVITY[self.stat_tracker.get_level() as usize - 1]);
        }

        state
    }

    fn get_playfield(&self) -> Playfield {
        self.base_engine.get_playfield()
    }

    fn get_current_piece(&self) -> CurrentPiece {
        self.base_engine.get_current_piece()
    }

    fn get_hold_piece(&self) -> Option<Tetromino> {
        self.base_engine.get_hold_piece()
    }

    fn get_next_pieces(&self) -> Vec<Tetromino> {
        self.base_engine.get_next_pieces()
    }

    fn input_move_left(&self) {
        self.base_engine.input_move_left();
    }

    fn input_move_right(&self) {
        self.base_engine.input_move_right();
    }

    fn input_hard_drop(&self) {
        self.base_engine.input_hard_drop();
    }

    fn input_soft_drop(&self) {
        self.base_engine.input_soft_drop();
    }

    fn input_rotate_cw(&self) {
        self.base_engine.input_rotate_cw();
    }

    fn input_rotate_ccw(&self) {
        self.base_engine.input_rotate_ccw();
    }

    fn input_hold(&self) {
        self.base_engine.input_hold();
    }
}

impl SinglePlayerEngine {
    pub fn new() -> SinglePlayerEngine {
        let mut base_engine = BaseEngine::new();
        base_engine.set_gravity(GRAVITY[0]);
        let stat_tracker = Rc::new(StatTracker::new());

        base_engine.add_observer(stat_tracker.clone());

        let engine = SinglePlayerEngine {
            base_engine,
            stat_tracker,
        };

        engine
    }

    fn get_score(&self) -> u32 {
        self.stat_tracker.score.get()
    }

}

struct StatTracker {
    score: Cell<u32>,
    last_lock: Cell<TSpin>,
    lines_cleared: Cell<u32>,
    combo_status: Cell<ComboStatus>,
    current_combo: Cell<u8>,
    back_to_back: Cell<bool>,
}

#[derive(Copy, Clone)]
enum ComboStatus {
    /// There is not active combo. The last piece locked did not result in a line clear.
    Inactive,
    /// There might be an active combo. A piece was just locked but it is not yet known whether or
    /// not it will result in a line clear.
    Maybe,
    /// There is an active combo. The last piece lock resulted in a line clear.
    Active,
}

impl StatTracker {
    fn new() -> StatTracker {
        StatTracker {
            score: Cell::new(0),
            last_lock: Cell::new(TSpin::None),
            lines_cleared: Cell::new(0),
            combo_status: Cell::new(ComboStatus::Inactive),
            current_combo: Cell::new(0),
            back_to_back: Cell::new(false),
        }
    }

    fn get_level(&self) -> u8 {
        let level = 1 + self.lines_cleared.get() / 10;
        std::cmp::min(level, 15) as u8
    }
}

impl BaseEngineObserver for StatTracker {
    fn on_lock(&self, t_spin: TSpin) {
        // Downgrade combo status. It should be reset to Active on line clear.
        let combo_status = match self.combo_status.get() {
            ComboStatus::Active => ComboStatus::Maybe,
            _ => {
                self.current_combo.set(0);
                ComboStatus::Inactive
            },
        };
        self.combo_status.set(combo_status);

        self.last_lock.set(t_spin);
    }

    fn on_soft_drop(&self, n_rows: u8) {
        self.score.set(self.score.get() + n_rows as u32);
    }

    fn on_hard_drop(&self, n_rows: u8) {
        self.score.set(self.score.get() + 2 * n_rows as u32);
    }

    fn on_line_clear(&self, n_rows: u8) {
        // Increment combo
        self.combo_status.set(ComboStatus::Active);
        self.current_combo.set(self.current_combo.get() + 1);

        let (mut points, back_to_back) = match (n_rows, self.last_lock.get()) {
            (1, TSpin::None) => (100, false),
            (2, TSpin::None) => (300, false),
            (3, TSpin::None) => (500, false),
            (4, TSpin::None) => (800, true),
            (1, TSpin::Mini) => (200, true),
            (1, TSpin::Regular) => (800, true),
            (2, TSpin::Regular) => (1200, true),
            (3, TSpin::Regular) => (1600, true),
            (_, _) => panic!("This should be impossible."),
        };

        // If we were already in the middle of a back-to-back,
        // and the current line clear is also a back-to-back ...
        if self.back_to_back.get() && back_to_back {
            // Multiply by 1.5. Use * 3 / 2 to avoid casting to f64 then back to u32.
            // Given the possible values of `points`, this will not result in any truncation.
            points = points * 3 / 2;
        }

        self.back_to_back.set(back_to_back);

        // 50 points per combo. 1-combo == 2-in-a-row.
        points += 50 * (self.current_combo.get() - 1) as u32;

        self.score.set(self.score.get() + points * self.get_level() as u32);

        // Do not update lines cleared until after final score is computed so that level is based on
        // lines cleared before this current action.
        self.lines_cleared.set(self.lines_cleared.get() + n_rows as u32);
    }
}
