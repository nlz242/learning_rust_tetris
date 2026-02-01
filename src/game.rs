// consts are compile-time constants, similar to const in C#
use crate::tetromino::{TetrominoShape, Point};

pub const WIDTH: usize = 10;
pub const HEIGHT: usize = 20;

// Helper struct to group piece data
pub struct ActivePiece {
    pub shape: TetrominoShape,
    pub x: i32,
    pub y: i32,
    pub cells: [Point; 4],
}

impl ActivePiece {
    pub fn new(shape: TetrominoShape) -> Self {
        ActivePiece {
            shape,
            x: (WIDTH / 2) as i32,
            y: 0,
            cells: shape.cells(),
        }
    }
}

// This struct holds the "state" of our game.
// It is comparable to a Class in C# with only fields.
pub struct Game {
    // 2D array: [row][col]
    // u8 is an unsigned 8-bit integer (byte).
    // 0 will represent empty, 1-7 will represent colors/shapes later.
    pub grid: [[u8; WIDTH]; HEIGHT],
    pub current_piece: Option<ActivePiece>, // The piece currently falling
    pub next_piece: TetrominoShape, // The upcoming piece
    pub score: u32,
    pub is_game_over: bool,
    pub piece_stats: [u32; 7],
}

// The 'impl' block is where we define methods for the struct.
impl Game {
    // There are no "constructors" in Rust. 
    // The convention is a static function named `new` that returns Self.
    pub fn new() -> Self {
        let start_piece = TetrominoShape::random();
        let next_piece = TetrominoShape::random();
        
        let mut stats = [0; 7];
        stats[start_piece.to_index()] += 1;

        Game {
            grid: [[0; WIDTH]; HEIGHT], // Initialize entire array with 0
            current_piece: Some(ActivePiece::new(start_piece)),
            next_piece,
            score: 0,
            is_game_over: false,
            piece_stats: stats,
        }
    }

    // New Update method
    // Note the `&mut self`. This tells the compiler:
    // "I need exclusive read-write access to this instance."
    // While this method is running, no other part of the code can read or write to this Game instance.
    pub fn update(&mut self) {
        if self.is_game_over {
            return;
        }

        let mut should_lock = false;
        
        if let Some(ref mut piece) = self.current_piece {
             // Calculate potential new position
             let new_y = piece.y + 1;
             
             // Check validity
             if is_valid_position(&self.grid, &piece.cells, piece.x, new_y) {
                 piece.y = new_y;
             } else {
                 should_lock = true;
             }
        }

        if should_lock {
            self.lock_piece();
        }
    }

    pub fn move_left(&mut self) {
        if self.is_game_over { return; }
        if let Some(ref mut piece) = self.current_piece {
             if is_valid_position(&self.grid, &piece.cells, piece.x - 1, piece.y) {
                 piece.x -= 1;
             }
        }
    }

    pub fn move_right(&mut self) {
        if self.is_game_over { return; }
        if let Some(ref mut piece) = self.current_piece {
             if is_valid_position(&self.grid, &piece.cells, piece.x + 1, piece.y) {
                 piece.x += 1;
             }
        }
    }

    pub fn rotate(&mut self) {
        if self.is_game_over { return; }
        if let Some(ref mut piece) = self.current_piece {
            // Clone current cells to test rotation
            let mut temp_cells = piece.cells;
            
            // Apply rotation math to temp
            for cell in &mut temp_cells {
                let (x, y) = *cell;
                *cell = (-y, x);
            }

            // Check if valid
            if is_valid_position(&self.grid, &temp_cells, piece.x, piece.y) {
                piece.cells = temp_cells; // Commit rotation
            }
        }
    }

    pub fn soft_drop(&mut self) {
        if self.is_game_over { return; }
        if let Some(ref mut piece) = self.current_piece {
            if is_valid_position(&self.grid, &piece.cells, piece.x, piece.y + 1) {
                piece.y += 1;
                self.score += 1; // 1 point per soft drop unit
            }
            // Note: We don't lock here. Soft drop just moves faster. 
        }
    }

    pub fn hard_drop(&mut self) {
        if self.is_game_over { return; }
        let mut dropped = false;
        while let Some(ref mut piece) = self.current_piece {
            if is_valid_position(&self.grid, &piece.cells, piece.x, piece.y + 1) {
                piece.y += 1;
                self.score += 2; // 2 points per hard drop unit
                dropped = true;
            } else {
                break;
            }
        }
        
        if dropped || self.current_piece.is_some() {
             self.lock_piece();
        }
    }

    pub fn get_ghost_piece_position(&self) -> Option<ActivePiece> {
         if let Some(ref piece) = self.current_piece {
            let mut ghost = ActivePiece {
                shape: piece.shape,
                x: piece.x,
                y: piece.y,
                cells: piece.cells,
            };

            while is_valid_position(&self.grid, &ghost.cells, ghost.x, ghost.y + 1) {
                ghost.y += 1;
            }
            return Some(ghost);
        }
        None
    }

    fn lock_piece(&mut self) {
        if let Some(ref piece) = self.current_piece {
            for (local_x, local_y) in piece.cells {
                let abs_x = piece.x + local_x;
                let abs_y = piece.y + local_y;

                // Write to grid if within bounds
                if abs_x >= 0 && abs_x < WIDTH as i32 && abs_y >= 0 && abs_y < HEIGHT as i32 {
                    self.grid[abs_y as usize][abs_x as usize] = piece.shape.to_index() as u8 + 1; // Mark with shape index (1-7)
                }
            }
        }

        self.check_lines();

        // Respawn a new piece from the 'next' queue
        let next_shape = self.next_piece;
        
        // Generate a new next piece
        self.next_piece = TetrominoShape::random();

        // Update stats for the piece that just entered the board
        self.piece_stats[next_shape.to_index()] += 1;

        let new_piece = ActivePiece::new(next_shape);
        
        // Game Over Check: Is the spawn position valid?
        if !is_valid_position(&self.grid, &new_piece.cells, new_piece.x, new_piece.y) {
            self.is_game_over = true;
        }
        
        self.current_piece = Some(new_piece);
    }

    fn check_lines(&mut self) {
        let mut new_grid = [[0u8; WIDTH]; HEIGHT];
        let mut new_y = HEIGHT - 1; // Start from bottom of new grid
        let mut lines_cleared = 0;

        // Iterate old grid from bottom to top
        for y in (0..HEIGHT).rev() {
            let is_full = self.grid[y].iter().all(|&cell| cell != 0);

            if !is_full {
                // Copy this row to new_grid
                if new_y <= HEIGHT - 1 { // Bounds check though loop handles it
                    new_grid[new_y] = self.grid[y];
                }
                if new_y > 0 {
                    new_y -= 1;
                }
            } else {
                lines_cleared += 1;
            }
        }
        
        self.grid = new_grid;

        // Simple scoring: 100 * 2^(lines-1)
        if lines_cleared > 0 {
            self.score += match lines_cleared {
                1 => 100,
                2 => 300,
                3 => 500,
                4 => 800, // Tetris!
                _ => 100,
            };
        }
    }
}

// Helper function, separated from struct to avoid borrowing issues
fn is_valid_position(grid: &[[u8; WIDTH]; HEIGHT], cells: &[Point; 4], x: i32, y: i32) -> bool {
    for (local_x, local_y) in cells {
        let abs_x = x + local_x;
        let abs_y = y + local_y;

        // Check boundaries
        // Left/Right walls && Floor
        if abs_x < 0 || abs_x >= WIDTH as i32 || abs_y >= HEIGHT as i32 {
            return false;
        }

        // Check against existing blocks in the grid
        // (We assume y >= 0 for array indexing, though technically pieces can exist above board)
        if abs_y >= 0 {
            if grid[abs_y as usize][abs_x as usize] != 0 {
                return false;
            }
        }
    }
    true
}

