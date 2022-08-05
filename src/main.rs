use ggez::{event, graphics, Context, GameResult, GameError};
use std::{env, path};
use ggez::event::{Axis, Button, ErrorOrigin, GamepadId, MouseButton};
use ggez::event::winit_event::TouchPhase;
use ggez::input::keyboard::{KeyCode, KeyInput};
use glam::{const_uvec4, const_vec4};
use rand::Rng;
use std;

// Here we define the size of our game board in terms of how many grid
// cells it will take up. We choose to make a 30 x 20 game board.
const GRID_SIZE: (i16, i16) = (20, 10);
// Now we define the pixel size of each tile, which we make 32x32 pixels.
const GRID_CELL_SIZE: (i16, i16) = (45, 45);

// Next we define how large we want our actual window to be by multiplying
// the components of our grid size by its corresponding pixel size.
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);

enum PieceColor {
    Red,
    Yellow,
    Blue,
    Cyan,
    Orange,
    Green,
    Purple
}

struct GameState {
    currentPiece: Piece,
    nextPiece: Piece,
    holdPiece: Option<Piece>,
    rowsClearedCount: i16,
    score: i32,
    hasHeldAPiece: bool,
    board: Vec<Vec<i32>>
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            rowsClearedCount: 0,
            score: 0,
            hasHeldAPiece: false,
            currentPiece: Piece::GetPiece(),
            nextPiece: Piece::GetPiece(),
            holdPiece: None,
            board: vec![vec![]]
        }
    }

    pub fn CheckCollision(rotation: [u32; 4], x: u8, y: u8) {
        GameState::applyFunctionToEachBlockInAPiece(rotation, x, y, (x, y) ,16)
    }

    fn IsCollision(x: u8, y: u8) {
        if (x < 0) || (x >= 10) || (y < 0) || (y >= 20)
    }

    pub fn applyFunctionToEachBlockInAPiece(rotation: [u32; 4], x: u8, y: u8, function:  fn(u8, u8), iterations: u8) {
        if iterations > 0
        {
            if (rotation & (0x8000 >> iterations)) > 0 {
                function(x + (iterations % 4), y + (iterations / 4));
            }
            GameState::applyFunctionToEachBlockInAPiece(rotation, x, y, function,iterations - 1);
        }
    }
}

struct Piece {
    PieceSize: u8,
    RotationState: usize,
    X: u8,
    Y: u8,
    Rotation: [u32; 4],
    PieceColor: PieceColor
}

impl Piece {
    fn new(size: u8, pieceColor: PieceColor, rotation: [u32; 4]) -> Self {
        Piece {
            PieceSize: size,
            RotationState: 0,
            X: 4,
            Y: 0,
            Rotation: rotation,
            PieceColor: pieceColor
        }
    }

    fn getRotationState(&self) -> u32 { return self.Rotation[self.RotationState]; }

    pub fn GetPiece() -> Piece {
        match rand::thread_rng().gen_range(0..7)
        {
            0 => Piece::new(4, PieceColor::Cyan, [0x00F0, 0x2222, 0x00F0, 0x2222]),
            1 => Piece::new(3, PieceColor::Blue, [0x44C0, 0x8E00, 0x6440, 0x0E20]),
            2 => Piece::new(3, PieceColor::Orange, [0x4460, 0x0E80, 0xC440, 0x2E00]),
            3 => Piece::new(2, PieceColor::Yellow, [0xCC00, 0xCC00, 0xCC00, 0xCC00]),
            4 => Piece::new(3, PieceColor::Green, [0x06C0, 0x4620, 0x06C0, 0x4620]),
            5 => Piece::new(3, PieceColor::Purple, [0x0E40, 0x4C40, 0x4E00, 0x4640]),
            6 => Piece::new(3, PieceColor::Red, [0x0C60, 0x2640, 0x0C60, 0x2640]),
            _ => Piece::new(4, PieceColor::Cyan, [0x00F0, 0x2222, 0x00F0, 0x2222])
        }
    }
}

/// Next we create an enum that will represent all the possible
/// directions that our snake could move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameInput {
    Down,
    Left,
    Right,
    HardDrop,
    RotateRight,
    RotateLeft,
    Hold
}

impl GameInput {
    /// We also create a helper function that will let us convert between a
    /// `ggez` `Keycode` and the `Direction` that it represents. Of course,
    /// not every keycode represents a direction, so we return `None` if this
    /// is the case.
    pub fn from_keycode(key: KeyCode) -> Option<GameInput> {
        match key {
            KeyCode::D => Some(GameInput::Right),
            KeyCode::A => Some(GameInput::Left),
            KeyCode::S => Some(GameInput::Down),
            KeyCode::W => Some(GameInput::HardDrop),
            KeyCode::E => Some(GameInput::Hold),
            KeyCode::J => Some(GameInput::RotateLeft),
            KeyCode::K => Some(GameInput::RotateRight),
            _ => None,
        }
    }
}

// Then we implement the `ggez:event::EventHandler` trait on it, which
// requires callbacks for updating and drawing the game state each frame.
//
// The `EventHandler` trait also contains callbacks for event handling
// that you can override if you wish, but the defaults are fine.
impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::CanvasLoadOp::Clear([0.1, 0.2, 0.3, 1.0].into()),
        );

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(dir) = input.keycode.and_then(GameInput::from_keycode){

        }
        Ok(())
    }
}

// Now our main function, which does three things:
//
// * First, create a new `ggez::ContextBuilder`
// object which contains configuration info on things such
// as screen resolution and window title.
// * Second, create a `ggez::game::Game` object which will
// do the work of creating our MainState and running our game.
// * Then, just call `game.run()` which runs the `Game` mainloop.
pub fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let (mut ctx, events_loop) = ggez::ContextBuilder::new("Tetris", "Payton Trosclair")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title("Snake!"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .build()?;


    let state = GameState::new();
    event::run(ctx, events_loop, state)
}
