mod engine;
mod render;

use std::collections::HashSet;

use piston::{
    input::{Button, ButtonArgs, ButtonState, Event, Input, Key, Loop},
};
use self::engine::{
    base::{Engine, State},
    single::SinglePlayerEngine,
};
use self::render::PistonRender;

fn main() {
    let mut engine = SinglePlayerEngine::new();
    let mut window = engine.create_window();
    let mut pressed_keys = HashSet::new();

    while let Some(event) = window.next() {
        match event {
            Event::Loop(loop_) => match loop_ {
                Loop::Render(_) => {
                    window.draw_2d(&event, |_context, graphics| {
                        engine.render(graphics);
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
