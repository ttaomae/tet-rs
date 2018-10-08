
use graphics::{
    draw_state::DrawState,
    rectangle::{Rectangle, Shape},
    Graphics,
};
use piston::{
    event_loop::EventLoop,
    window::WindowSettings,
};
use piston_window::PistonWindow;

use crate::engine::{
    core::{Playfield, Space, Piece},
    base::{Engine},
    single::SinglePlayerEngine
};


const SPACE_SIZE: u32 = 20;
// Playfield is 10 x 20.
// Include room for 1 space border on all sides,
// plus 5 spaces to draw hold and next pieces.
const WIDTH: u32 = 17 * SPACE_SIZE;
const HEIGHT: u32 = 22 * SPACE_SIZE;

const GREY_RECTANGLE: Rectangle = Rectangle {
    color: [0.1, 0.1, 0.1, 1.],
    shape: Shape::Square,
    border: Option::None,
};
const GREEN_RECTANGLE: Rectangle = Rectangle {
    color: [0., 1., 0., 1.],
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
const BLUE_RECTANGLE: Rectangle = Rectangle {
    color: [0., 0., 1., 1.],
    shape: Shape::Square,
    border: Option::None,
};
const DEFAULT_DRAW_STATE: DrawState = DrawState {
    scissor: Option::None,
    stencil: Option::None,
    blend: Option::None,
};
const IDENTITY_TRANSFORMATION_MATRIX: [[f64; 3]; 2] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];

pub trait PistonRender {
    fn create_window(&self) -> Box<PistonWindow>;
    fn render<G: Graphics>(&self, graphics: &mut G);
}

impl PistonRender for SinglePlayerEngine {
    fn create_window(&self) -> Box<PistonWindow> {
        let mut window: PistonWindow = WindowSettings::new("tet-rs", (WIDTH, HEIGHT))
            .exit_on_esc(true)
            .resizable(false)
            .build()
            .unwrap();
        window.set_max_fps(60);
        window.set_ups(60);

        Box::new(window)
    }

    fn render<G: Graphics>(&self, graphics: &mut G) {
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

        fn draw_block<G: Graphics>(row: u32, col: u32, rectangle: Rectangle, graphics: &mut G) {
            rectangle.draw(
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

        fn draw_bounding_box<G: Graphics>(bounding_box: [[Space; 4]; 4],
            row_offset: i8, col_offset: i8,
            rectangle: Rectangle, graphics: &mut G) {

            for bb_row in 0..4 {
                for bb_col in 0..4 {
                    if bounding_box[bb_row][bb_col] == Space::Block {
                        let col = (col_offset + bb_col as i8) as u32;
                        let row = (row_offset + bb_row as i8) as u32;
                        if row <= 20 {
                            draw_block(row, col, rectangle, graphics);
                        }
                    }
                }
            }
        }


        graphics.clear_color([0.5, 0.5, 0.5, 1.]);

        GREY_RECTANGLE.draw(
            convert_coordinates(SPACE_SIZE, SPACE_SIZE, 10 * SPACE_SIZE, 20 * SPACE_SIZE),
            &DEFAULT_DRAW_STATE,
            IDENTITY_TRANSFORMATION_MATRIX,
            graphics,
        );

        let playfield = self.get_playfield();
        // Draw playfield.
        for row in 1..=Playfield::VISIBLE_HEIGHT {
            for col in 1..=Playfield::WIDTH {
                if playfield.get(row, col) == Space::Block {
                    draw_block(row as u32, col as u32, RED_RECTANGLE, graphics);
                }
            }
        }

        // Draw current piece.
        let current_piece = self.get_current_piece();
        let bounding_box = current_piece.get_bounding_box();
        draw_bounding_box(bounding_box, current_piece.get_row(), current_piece.get_col(),
            CYAN_RECTANGLE, graphics);


        // Draw hold piece at upper right corner.
        if let Option::Some(hold_piece) = self.get_hold_piece() {
            let bounding_box = Piece::new(hold_piece).get_bounding_box();
            draw_bounding_box(bounding_box, 17, 12, GREEN_RECTANGLE, graphics);
        }

        // Draw next pieces to right of playfield.
        for (i, next_piece) in self.get_next_pieces().iter().enumerate() {
            let bounding_box = Piece::new(*next_piece).get_bounding_box();
            let col_offset = 14 - (3 * i as i8);
            draw_bounding_box(bounding_box, col_offset, 12, BLUE_RECTANGLE, graphics);
        }
    }
}
