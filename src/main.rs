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

#[derive(Copy, Clone)]
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
    board: [[Option<PieceColor>; 10]; 20]
}

impl GameState {
    pub fn new(board: [[Option<PieceColor>; 10]; 20]) -> Self {
        GameState {
            rowsClearedCount: 0,
            score: 0,
            hasHeldAPiece: false,
            currentPiece: Piece::GetPiece(),
            nextPiece: Piece::GetPiece(),
            holdPiece: None,
            board
        }
    }

    pub fn moveDirection(&mut self, direction: GameInput) -> bool {
        let mut x: usize = self.currentPiece.X;
        let mut y: usize = self.currentPiece.Y;

        match direction {
            GameInput::Left => x = x - 1,
            GameInput::Right => x = x + 1,
            GameInput::Down => y = y - 1,
            _ => ()
        }

        if !GameState::CheckCollision(self, x, y) {
            self.currentPiece.X = x;
            self.currentPiece.Y = y;
            return true;
        }
        return false;
    }

    pub fn drop(&mut self, isHoldingDown: bool) {
        if isHoldingDown {
            self.score = self.score + 10
        }

        if !self.moveDirection(GameInput::Down) {
            self.after_drop();
        }
    }

    pub fn hard_drop(&mut self) -> bool {
        while self.moveDirection(GameInput::Down) {
            self.score = self.score + 10
        }
        self.score = self.score + 10;
        self.after_drop();

        return true;
    }

    pub fn after_drop(&mut self) {
        self.dropPiece();
    }

    pub fn hold(&mut self) -> bool {

        return true;
    }

    pub fn dropPiece(&mut self) {
        let mut iterations = 0;
        let mut func = |x: usize, y: usize| {
            self.board[x][y] = Option::from(self.currentPiece.PieceColor);
        };
        while iterations < 16
        {
            if (self.currentPiece.getRotationState() & (0x8000 >> iterations)) > 0 {
                func(self.currentPiece.X + (iterations % 4), self.currentPiece.Y + (iterations / 4));
            }
            iterations = iterations + 1;
        }
    }

    pub fn rotate(&mut self, direction: GameInput) -> bool {
        let newRotationState = match direction {
            GameInput::RotateLeft => (self.currentPiece.RotationState + 3) % 4,
            GameInput::RotateRight => (self.currentPiece.RotationState + 1) % 4,
            _ => self.currentPiece.RotationState
        };

        if !GameState::CheckCollision(self, self.currentPiece.X, self.currentPiece.Y) {
            self.currentPiece.RotationState = newRotationState;
        }
        return true;
    }

    pub fn getDropShadowY(&mut self) -> usize {
        let mut y: usize = self.currentPiece.Y;
        while !GameState::CheckCollision(self, self.currentPiece.X, y) {
            y = y + 1;
        }
        return y - 1;
    }

    pub fn CheckCollision(&self, x: usize, y: usize) -> bool {
        let mut b: bool = false;
        let mut iterations = 0;
        let mut func = |x: usize, y: usize| {
            b = b | ((x < 0) || (x >= 10) || (y < 0) || (y >= 20) || (!self.board[x][y].is_none()));
        };
        while iterations < 16
        {
            if (self.currentPiece.Rotation[self.currentPiece.RotationState] & (0x8000 >> iterations)) > 0 {
                func(x + (iterations % 4), y + (iterations / 4));
            }
            iterations = iterations + 1;
        }
        return b;
    }

    pub fn applyFunctionToEachBlockInAPiece(rotation: u32, x: usize, y: usize, function: fn(usize, usize)) {
        let mut iterations = 0;
        while iterations < 16
        {
            if (rotation & (0x8000 >> iterations)) > 0 {
                function(x + (iterations % 4), y + (iterations / 4));
            }
            iterations = iterations + 1;
        }
    }
}

#[derive(Copy, Clone)]
struct Piece {
    PieceSize: u8,
    RotationState: usize,
    X: usize,
    Y: usize,
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
            let is_successful_move = match dir {
                GameInput::Down => self.moveDirection(dir),
                GameInput::Left => self.moveDirection(dir),
                GameInput::Right => self.moveDirection(dir),
                GameInput::RotateRight => self.rotate(dir),
                GameInput::RotateLeft => self.rotate(dir),
                GameInput::HardDrop => self.hard_drop(),
                GameInput::Hold => self.hold(),
                _ => false
            };
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
        .window_setup(ggez::conf::WindowSetup::default().title("Tetris!"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .build()?;

    let mut board: [[Option<PieceColor>; 10]; 20] = [[None; 10]; 20];
    let state = GameState::new(board);
    event::run(ctx, events_loop, state)
}
