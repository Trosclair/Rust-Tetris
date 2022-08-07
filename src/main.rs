use ggez::{event, graphics, Context, GameResult, GameError};
use std::{env, path};
use ggez::event::{Axis, Button, ErrorOrigin, GamepadId, MouseButton};
use ggez::event::winit_event::TouchPhase;
use ggez::input::keyboard::{KeyCode, KeyInput};
use glam::{const_uvec4, const_vec4};
use rand::Rng;
use std;
use std::process::exit;
use ggez::graphics::{Canvas, Color};
use ggez::mint::Vector2;
use ggez::winit::dpi::Size;

// Here we define the size of our game board in terms of how many grid
// cells it will take up. We choose to make a 30 x 20 game board.
const GRID_SIZE: (i16, i16) = (10, 20);
// Now we define the pixel size of each tile, which we make 32x32 pixels.
const GRID_CELL_SIZE: (i16, i16) = (30, 30);

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
    Purple,
    Black
}

struct GameState {
    current_piece: Piece,
    next_piece: Piece,
    hold_piece: Option<Piece>,
    rows_cleared_count: i16,
    score: i32,
    has_held_a_piece: bool,
    board: [[Option<PieceColor>; 20]; 10]
}

impl GameState {
    pub fn new(board: [[Option<PieceColor>; 20]; 10]) -> Self {
        GameState {
            rows_cleared_count: 0,
            score: 0,
            has_held_a_piece: false,
            current_piece: Piece::get_piece(),
            next_piece: Piece::get_piece(),
            hold_piece: None,
            board
        }
    }

    pub fn move_direction(&mut self, direction: GameInput) -> bool {
        let mut x: i8 = self.current_piece.x;
        let mut y: i8 = self.current_piece.y;

        if direction == GameInput::Left {
            x = x - 1;
        }
        else if direction == GameInput::Right {
            x = x + 1;
        }
        else if direction == GameInput::Down {
            y = y + 1;
        }

        if !GameState::check_collision(self, x, y) {
            self.current_piece.x = x;
            self.current_piece.y = y;
            return true;
        }
        return false;
    }

    pub fn drop(&mut self, is_holding_down: bool) {
        if is_holding_down {
            self.score = self.score + 10
        }

        if !self.move_direction(GameInput::Down) {
            self.after_drop();
        }
    }

    pub fn hard_drop(&mut self) -> bool {
        while self.move_direction(GameInput::Down) {
            self.score = self.score + 10
        }
        self.score = self.score + 10;
        self.after_drop();

        return true;
    }

    pub fn after_drop(&mut self) {
        self.drop_piece();
        self.remove_lines();
        self.current_piece = self.next_piece;
        self.next_piece = Piece::get_piece();
        self.has_held_a_piece = false;
        self.check_collision(self.current_piece.x, self.current_piece.y);
    }

    fn remove_lines(&mut self) {
        let mut n = 0;
        let mut y = 19;

        while y > 0 {
            let mut is_line_complete = true;
            let mut x = 0;
            while x < 10 {
                is_line_complete = is_line_complete & !self.board[x][y].is_none();
                x = x + 1;
            }

            if is_line_complete {
                self.remove_line(y as i8);
                y = y + 1;
                n = n + 1;
            }

            y = y - 1;
        }
    }

    fn remove_line(&mut self, mut n: i8) {

        loop {
            let mut x = 0;
            while x < 10 {
                self.board[x as usize][n as usize] = if n == 0 { None } else { self.board[x as usize][(n - 1) as usize] };
                x = x + 1
            }
            if n == 0 {
                break;
            }
            n = n - 1;
        }
    }

    pub fn drop_piece(&mut self) {
        let mut iterations = 0;
        let mut func = |x: i8, y: i8| {
            self.board[x as usize][y as usize] = Option::from(self.current_piece.piece_color);
        };
        while iterations < 16
        {
            if (self.current_piece.get_rotation_state() & (0x8000 >> iterations)) > 0 {
                func(self.current_piece.x + (iterations % 4), self.current_piece.y + (iterations / 4));
            }
            iterations = iterations + 1;
        }
    }

    pub fn hold(&mut self) -> bool {
        if !self.has_held_a_piece {
            self.has_held_a_piece = true;

            if self.hold_piece.is_some() {
                let temp = match self.hold_piece { None => Piece::get_piece(), Some(temp) => temp };
                self.hold_piece = Option::from(self.current_piece);
                self.current_piece = temp;
            }
            else {
                self.hold_piece = Option::from(self.current_piece);
                self.current_piece = self.next_piece;
                self.next_piece = Piece::get_piece();
            }
            let mut hold_piece_as_some = match self.hold_piece { None => Piece::get_piece(), Some(temp) => temp };
            hold_piece_as_some.x = 4;
            hold_piece_as_some.y = 0;
        }
        return true;
    }

    pub fn rotate(&mut self, direction: GameInput) -> bool {
        let old_rotation_state = self.current_piece.rotation_state;
        self.current_piece.rotation_state = match direction {
            GameInput::RotateLeft => (self.current_piece.rotation_state + 3) % 4,
            GameInput::RotateRight => (self.current_piece.rotation_state + 1) % 4,
            _ => self.current_piece.rotation_state
        };

        if GameState::check_collision(self, self.current_piece.x, self.current_piece.y) {
            self.current_piece.rotation_state = old_rotation_state;
        }
        return true;
    }

    pub fn get_drop_shadow_y(&mut self) -> i8 {
        let mut y: i8 = self.current_piece.y;
        while !GameState::check_collision(self, self.current_piece.x, y) {
            y = y + 1;
        }
        return y - 1;
    }

    pub fn check_collision(&self, x: i8, y: i8) -> bool {
        let mut b: bool = false;
        let mut iterations = 0;
        let mut func = |x: i8, y: i8| {

            if (x < 0) || (x >= 10) || (y < 0) || (y >= 20) {
                b = b | true;
            }
            else {
                b = b | !self.board[x as usize][y as usize].is_none();
            }
        };
        while iterations < 16
        {
            if (self.current_piece.rotation[self.current_piece.rotation_state as usize] & (0x8000 >> iterations)) > 0 {
                func(x + (iterations % 4), y + (iterations / 4));
            }
            iterations = iterations + 1;
        }
        return b;
    }

    pub fn apply_function_to_each_block_in_apiece(rotation: u32, x: i8, y: i8, function: fn(i8, i8)) {
        let mut iterations = 0;
        while iterations < 16
        {
            if (rotation & (0x8000 >> iterations)) > 0 {
                function(x + (iterations % 4), y + (iterations / 4));
            }
            iterations = iterations + 1;
        }
    }

    pub fn draw_board(&self, mut canvas: &mut Canvas) {
        let rect = graphics::Rect::new(0.0,0.0,450.0,900.0);
        canvas.draw(&graphics::Quad, graphics::DrawParam::new().dest(rect.point()).scale(rect.size()).color(Color::BLACK));


        let mut y: i8 = 0;

        while y < 20 {
            let mut x: i8 = 0;
            while x < 10 {
                let piece_color = match self.board[x as usize][y as usize] { None => PieceColor::Black, Some(temp) => temp};
                let print_color = match piece_color {
                    PieceColor::Red => Color::RED,
                    PieceColor::Purple => Color::MAGENTA,
                    PieceColor::Green => Color::GREEN,
                    PieceColor::Blue => Color::BLUE,
                    PieceColor::Cyan => Color::CYAN,
                    PieceColor::Orange => Color::new(255.0, 140.0, 50.0, 100.0),
                    PieceColor::Yellow => Color::YELLOW,
                    PieceColor::Black => Color::BLACK
                };
                let rect = graphics::Rect::new((x as f32) * 30.0, (y as f32) * 30.0,30.0,30.0);
                canvas.draw(&graphics::Quad, graphics::DrawParam::new().dest(rect.point()).scale(rect.size()).color(print_color));
                x = x + 1;
            }
            y = y + 1;
        }

        self.draw_piece(&mut canvas, self.current_piece.rotation[self.current_piece.rotation_state as usize], self.current_piece.x, self.current_piece.y, self.current_piece.piece_color);
    }

    fn draw_piece(&self, canvas: &mut Canvas, rotation: u32, x: i8, y: i8, color: PieceColor) {
        let mut iterations = 0;
        let mut func = |x: i8, y: i8| {

            let print_color = match color {
                PieceColor::Red => Color::RED,
                PieceColor::Purple => Color::MAGENTA,
                PieceColor::Green => Color::GREEN,
                PieceColor::Blue => Color::BLUE,
                PieceColor::Cyan => Color::CYAN,
                PieceColor::Orange => Color::new(255.0,140.0, 50.0, 100.0),
                PieceColor::Yellow => Color::YELLOW,
                PieceColor::Black => Color::BLACK
            };
            let rect = graphics::Rect::new((x as f32) * 30.0, (y as f32) * 30.0,30.0,30.0);
            canvas.draw(&graphics::Quad, graphics::DrawParam::new().dest(rect.point()).scale(rect.size()).color(print_color));
        };
        while iterations < 16
        {
            if (rotation & (0x8000 >> iterations)) > 0 {
                func(x + (iterations % 4), y + (iterations / 4));
            }
            iterations = iterations + 1;
        }
    }
}

#[derive(Copy, Clone)]
struct Piece {
    piece_size: u8,
    rotation_state: i8,
    x: i8,
    y: i8,
    rotation: [u32; 4],
    piece_color: PieceColor
}

impl Piece {
    fn new(size: u8, piece_color: PieceColor, rotation: [u32; 4]) -> Self {
        Piece {
            piece_size: size,
            rotation_state: 0,
            x: 4,
            y: 0,
            rotation,
            piece_color
        }
    }

    fn get_rotation_state(&self) -> u32 { return self.rotation[self.rotation_state as usize]; }

    pub fn get_piece() -> Piece {
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

        self.draw_board(&mut canvas);

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if let Some(dir) = input.keycode.and_then(GameInput::from_keycode){
            let is_successful_move = match dir {
                GameInput::Down => self.move_direction(dir),
                GameInput::Left => self.move_direction(dir),
                GameInput::Right => self.move_direction(dir),
                GameInput::RotateRight => self.rotate(dir),
                GameInput::RotateLeft => self.rotate(dir),
                GameInput::HardDrop => self.hard_drop(),
                GameInput::Hold => self.hold(),
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

    let mut board: [[Option<PieceColor>; 20]; 10] = [[None; 20]; 10];
    let state = GameState::new(board);
    event::run(ctx, events_loop, state)
}
