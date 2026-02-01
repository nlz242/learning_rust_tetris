use crate::game::{Game, WIDTH, HEIGHT};
use crate::tetromino::TetrominoShape;

pub struct ConsoleRenderer;

impl ConsoleRenderer {
    pub fn new() -> Self {
        ConsoleRenderer
    }

    pub fn render(&self, game: &Game) {
        println!("Displaying Game State:\r"); // \r returns carriage to start of line
        
        let stats_start_y = 7;
        let items_count = 7;
        let item_height = 3; // 2 rows icon + 1 separator
        let sidebar_height = stats_start_y + (items_count * item_height);
        
        // Determine total height (Grid is HEIGHT=20, so 0..20, plus bottom border at 21)
        // We iterate y from 0 to max
        let grid_height = HEIGHT; 
        let total_height = if (grid_height + 1) > sidebar_height { grid_height + 1 } else { sidebar_height };
        
        for y in 0..total_height {
            // --- LEFT COLUMN (GRID) ---
            if y < HEIGHT {
                print!("|"); 
                for x in 0..WIDTH {
                    let mut is_piece_part = false;

                    // Check if this map position (x, y) is part of the current falling piece
                    if let Some(ref piece) = game.current_piece {
                        for (local_x, local_y) in piece.cells {
                            let abs_x = piece.x + local_x;
                            let abs_y = piece.y + local_y;

                            if abs_x == x as i32 && abs_y == y as i32 {
                                is_piece_part = true;
                                break;
                            }
                        }
                    }

                    let cell = game.grid[y][x];
                    if is_piece_part {
                        print!(" O ");
                    } else if cell == 0 {
                        print!(" . ");
                    } else {
                        print!(" # ");
                    }
                }
                print!("|  "); // End of grid row + spacing
            } else if y == HEIGHT {
                 // Bottom border
                 print!(" {:-<30}   ", ""); // 3 spaces to align with sidebar
            } else {
                 // Empty space below grid
                 print!("{:34}", " "); // spacer to match grid width
            }

            // --- RIGHT COLUMN (SIDEBAR) ---
            match y {
                0 => print!("Score: {}", game.score),
                2 => print!("Next:"),
                3 => print!("{}", self.get_mini_icon(game.next_piece, 0)),
                4 => print!("{}", self.get_mini_icon(game.next_piece, 1)),
                6 => print!("Stats:"),
                i if i >= stats_start_y => {
                    let stats_row = i - stats_start_y;
                    let shape_idx = stats_row / item_height;
                    let sub_row = stats_row % item_height;

                    if shape_idx < items_count {
                        let shape = TetrominoShape::from_index(shape_idx);
                        match sub_row {
                            0 | 1 => {
                                let icon = self.get_mini_icon(shape, sub_row);
                                print!("{} ", icon);
                                if sub_row == 0 {
                                    let count = game.piece_stats[shape_idx];
                                    let total: u32 = game.piece_stats.iter().sum();
                                    let pct = if total > 0 { (count as f32 / total as f32) * 100.0 } else { 0.0 };
                                    print!(": {} ({:.1}%)", count, pct);
                                }
                            },
                            2 => print!("-------------"), // Separator
                            _ => {}
                        }
                    }
                }
                _ => {}
            }

            println!("\r"); // \r required in raw mode
        }
        
        if game.is_game_over {
            println!("GAME OVER! Press 'q' to quit.\r");
        }
    }

    fn get_mini_icon(&self, shape: TetrominoShape, row: usize) -> &str {
        match (shape, row) {
            // I: 4 wide
            (TetrominoShape::I, 0) => "[][][][]",
            (TetrominoShape::I, 1) => "        ",
            
            // O: 2x2
            (TetrominoShape::O, 0) => "  [][]  ",
            (TetrominoShape::O, 1) => "  [][]  ",
    
            // T: 3 wide top, 1 bottom middle
            (TetrominoShape::T, 0) => " [][][] ",
            (TetrominoShape::T, 1) => "   []   ",
    
            // S: 2 top right, 2 bottom left (offset)
            // S coords: (0,0), (-1,0) [Top Left], (0,1), (1,1) [Bottom Right]
            // Top: [ ][ ] . Bottom:     [ ][ ]
            (TetrominoShape::S, 0) => " [][]   ",  
            (TetrominoShape::S, 1) => "   [][] ",
    
            // Z: 2 top left, 2 bottom right (offset)
            // Z coords: (0,0), (1,0) [Top Right], (0,1), (-1,1) [Bottom Left]
            (TetrominoShape::Z, 0) => "   [][] ",
            (TetrominoShape::Z, 1) => " [][]   ",
    
            // J: 3 top, 1 bottom left
            (TetrominoShape::J, 0) => " [][][] ",
            (TetrominoShape::J, 1) => " []     ",
    
            // L: 3 top, 1 bottom right
            (TetrominoShape::L, 0) => " [][][] ",
            (TetrominoShape::L, 1) => "     [] ",
    
            _ => "        "
        }
    }
}
