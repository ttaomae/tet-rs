mod engine;

use std::collections::HashSet;

use graphics::{
    draw_state::DrawState,
    rectangle::{Rectangle, Shape},
    Graphics,
};
use piston::{
    event_loop::EventLoop,
    input::{Button, ButtonArgs, ButtonState, Event, Input, Key, Loop},
    window::WindowSettings,
};
use piston_window::PistonWindow;

use self::engine::{
    core::{Playfield, Space},
    base::{BaseEngine, Engine, State},
};

const SPACE_SIZE: u32 = 20;
// Include room for 1 space border on all sides.
const WIDTH: u32 = 12 * SPACE_SIZE;
const HEIGHT: u32 = 22 * SPACE_SIZE;
fn main() {
    let mut window: PistonWindow = WindowSettings::new("tet-rs", (WIDTH, HEIGHT))
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();
    window.set_max_fps(60);
    window.set_ups(60);

    let mut engine = BaseEngine::new();
    let mut pressed_keys = HashSet::new();

    while let Some(event) = window.next() {
        match event {
            Event::Loop(loop_) => match loop_ {
                Loop::Render(_) => {
                    window.draw_2d(&event, |_context, graphics| {
                        render(&engine, graphics);
                    });
                }
                Loop::Update(_) =>
                {
                    handle_input(&mut engine, &pressed_keys);
                    if let State::TopOut = engine.tick() {
                        break;
                    }
                }
                _ => window.event(&event),
            },
            Event::Input(Input::Button(button_args)) => {
                update_held_keys(&mut pressed_keys, button_args);
            }
            _ => window.event(&event),
        }
    }
}

const GREY_RECTANGLE: Rectangle = Rectangle {
    color: [0.1, 0.1, 0.1, 1.],
    shape: Shape::Square,
    border: Option::None,
};
const RED_RECTANGLE: Rectangle = Rectangle {
    color: [1., 0., 0., 1.],
    shape: Shape::Square,
    border: Option::None,
};
const CYAN_RECTANGLE: Rectangle = Rectangle {
    color: [0., 1., 1., 1.],
    shape: Shape::Square,
    border: Option::None,
};
const DEFAULT_DRAW_STATE: DrawState = DrawState {
    scissor: Option::None,
    stencil: Option::None,
    blend: Option::None,
};
const IDENTITY_TRANSFORMATION_MATRIX: [[f64; 3]; 2] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];

fn render<G: Graphics>(engine: &impl Engine, graphics: &mut G) {
    fn convert_coordinates(x: u32, y: u32, w: u32, h: u32) -> [f64; 4] {
        let width_scale = 2.0 / WIDTH as f64;
        let height_scale = 2.0 / HEIGHT as f64;

        [
            -1.0 + x as f64 * width_scale,
            -1.0 + y as f64 * height_scale,
            w as f64 * width_scale,
            h as f64 * height_scale,
        ]
    }

    graphics.clear_color([0.5, 0.5, 0.5, 1.]);

    GREY_RECTANGLE.draw(
        convert_coordinates(SPACE_SIZE, SPACE_SIZE, 10 * SPACE_SIZE, 20 * SPACE_SIZE),
        &DEFAULT_DRAW_STATE,
        IDENTITY_TRANSFORMATION_MATRIX,
        graphics,
    );

    // playfield: width = 0.9, height = 1.8
    // playfield offset = -0.9, -0.9
    let playfield = engine.get_playfield();
    // Draw playfield
    for row in 1..=Playfield::VISIBLE_HEIGHT {
        for col in 1..=Playfield::WIDTH {
            if playfield.get(row, col) == Space::Block {
                RED_RECTANGLE.draw(
                    // Since row/col uses 1-based index, playfiled will be offset by 1 square space.
                    convert_coordinates(
                        col as u32 * SPACE_SIZE,
                        row as u32 * SPACE_SIZE,
                        SPACE_SIZE,
                        SPACE_SIZE,
                    ),
                    &DEFAULT_DRAW_STATE,
                    IDENTITY_TRANSFORMATION_MATRIX,
                    graphics,
                );
            }
        }
    }

    // Draw current piece
    let current_piece = engine.get_current_piece();
    let bounding_box = current_piece.get_bounding_box();
    for bb_row in 0..4 {
        for bb_col in 0..4 {
            if bounding_box[bb_row][bb_col] == Space::Block {
                let col = (current_piece.get_col() + bb_col as i8) as u32;
                let row = (current_piece.get_row() + bb_row as i8) as u32;
                if row <= 20 {
                    CYAN_RECTANGLE.draw(
                        // Since row/col uses 1-based index, playfiled will be offset by 1 square space.
                        convert_coordinates(
                            col * SPACE_SIZE,
                            row * SPACE_SIZE,
                            SPACE_SIZE,
                            SPACE_SIZE,
                        ),
                        &DEFAULT_DRAW_STATE,
                        IDENTITY_TRANSFORMATION_MATRIX,
                        graphics,
                    );
                }
            }
        }
    }
}

fn update_held_keys(held_keys: &mut HashSet<Key>, button_args: ButtonArgs) {
    if let Button::Keyboard(key) = button_args.button {
        match button_args.state {
            ButtonState::Press => held_keys.insert(key),
            ButtonState::Release => held_keys.remove(&key),
        };
    }
}
fn handle_input(engine: &mut impl Engine, held_keys: &HashSet<Key>) {
    for key in held_keys.iter() {
        match key {
            Key::Left => engine.input_move_left(),
            Key::Right => engine.input_move_right(),
            Key::Space => engine.input_hard_drop(),
            Key::Down => engine.input_soft_drop(),
            Key::Z => engine.input_rotate_ccw(),
            Key::X => engine.input_rotate_cw(),
            Key::C => engine.input_hold(),
            _ => {}
        }
    }
}
