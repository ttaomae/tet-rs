use super::core::Playfield;
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
}

impl StatTracker {
    fn new() -> StatTracker {
        StatTracker {
            score: Cell::new(0),
            last_lock: Cell::new(TSpin::None),
            lines_cleared: Cell::new(0),
        }
    }

    fn get_level(&self) -> u8 {
        let level = 1 + self.lines_cleared.get() / 10;
        std::cmp::min(level, 15) as u8
    }
}

impl BaseEngineObserver for StatTracker {
    fn on_lock(&self, t_spin: TSpin) {
        self.last_lock.set(t_spin);
    }

    fn on_soft_drop(&self, n_rows: u8) {
        self.score.set(self.score.get() + n_rows as u32);
    }

    fn on_hard_drop(&self, n_rows: u8) {
        self.score.set(self.score.get() + 2 * n_rows as u32);
    }

    fn on_line_clear(&self, n_rows: u8) {
        self.lines_cleared.set(self.lines_cleared.get() + n_rows as u32);

        let mut points = match (n_rows, self.last_lock.get()) {
            (1, TSpin::Regular) => 800,
            (2, TSpin::Regular) => 1200,
            (3, TSpin::Regular) => 1600,
            (1, _) => 100,
            (2, _) => 300,
            (3, _) => 500,
            (4, _) => 800,
            (_, _) => 0,
        };
        points *= self.get_level() as u32;
        self.score.set(self.score.get() + points);
    }
}
