mod game;
mod tetromino;
mod renderer; // Keep for reference, but unused
mod graphic_context;
mod vertex_data;

use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use game::Game;
use graphic_context::GraphicContext;

struct App {
    window: Option<Arc<Window>>,
    game: Game,
    graphics: Option<GraphicContext>,
    last_gravity_update: Instant,
    gravity_interval: Duration,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            game: Game::new(),
            graphics: None,
            last_gravity_update: Instant::now(),
            gravity_interval: Duration::from_millis(500),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title("Rust Tetris (WGPU)")
                .with_inner_size(winit::dpi::LogicalSize::new(600.0, 800.0));
            
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(window.clone());

            let mut graphics = pollster::block_on(GraphicContext::new(window.clone()));
            
            // Initial mesh build
            let vertices = vertex_data::build_mesh(&self.game, graphics.size.width, graphics.size.height);
            graphics.update_buffers(&vertices);
            
            self.graphics = Some(graphics);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(physical_size) => {
                if let Some(graphics) = &mut self.graphics {
                    graphics.resize(physical_size);
                }
            },
            WindowEvent::RedrawRequested => {
                if let Some(graphics) = &mut self.graphics {
                    
                    // Game Loop Logic (Update)
                    let now = Instant::now();
                    if now.duration_since(self.last_gravity_update) > self.gravity_interval {
                        self.game.update();
                        self.last_gravity_update = now;
                        
                        if self.game.is_game_over {
                             // Handle game over? Reset?
                             // For now, auto-restart
                             self.game = Game::new();
                        }
                    }

                    // Rebuild Mesh
                    let vertices = vertex_data::build_mesh(&self.game, graphics.size.width, graphics.size.height);
                    graphics.update_buffers(&vertices);

                    // Render
                    match graphics.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => graphics.resize(graphics.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                
                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            },
            WindowEvent::KeyboardInput {
                event: key_event,
                ..
            } => {
                if key_event.state == ElementState::Pressed && !key_event.repeat {
                    if let PhysicalKey::Code(keycode) = key_event.physical_key {
                        match keycode {
                            KeyCode::ArrowLeft => self.game.move_left(),
                            KeyCode::ArrowRight => self.game.move_right(),
                            KeyCode::ArrowDown => self.game.soft_drop(), // Soft drop
                            KeyCode::ArrowUp => self.game.rotate(),
                            KeyCode::Space => self.game.hard_drop(),
                            KeyCode::Escape => event_loop.exit(),
                            _ => {}
                        }
                        // Request immediate redraw on input for responsiveness
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }
            },
            _ => (),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll); // Poll allows continuous updates for game loop

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
