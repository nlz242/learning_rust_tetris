mod game; // Helper to import the game.rs module
mod tetromino; // Import the tetromino module
mod renderer; // Import the renderer

use game::Game;
use renderer::ConsoleRenderer;
use crossterm::{
    cursor::{Hide, Show, MoveTo},
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io::stdout, time::{Duration, Instant}};

fn main() -> Result<(), Box<dyn Error>> {
    let mut game = Game::new();
    let renderer = ConsoleRenderer::new();
    let mut stdout = stdout();

    // Setup terminal
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(500);

    // Main Loop
    'game_loop: loop {
        // 1. Handle Input
        // Poll for an event for a tiny amount of time
        // This prevents the loop from blocking entirely
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Left => game.move_left(),
                        KeyCode::Right => game.move_right(),
                        KeyCode::Up => game.rotate(),
                        KeyCode::Down => game.soft_drop(),
                        KeyCode::Char(' ') => game.hard_drop(),
                        KeyCode::Esc | KeyCode::Char('q') => {
                            break 'game_loop;
                        } 
                        _ => {}
                    }
                }
            }
        }

        // 2. Update Game State (Gravity)
        if last_tick.elapsed() >= tick_rate {
            game.update();
            last_tick = Instant::now();
        }
        // 3. Render
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        renderer.render(&game); 
    }

    // Cleanup terminal
    execute!(stdout, Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
