# Rust Tetris (WGPU)

A classic Tetris clone built with **Rust** and **wgpu** (WebGPU for native). This project started as a terminal-based game and has evolved into a hardware-accelerated 2D application.

## Features

- **Core Gameplay**: Complete Tetris logic including collision handling, line clearing, and loose gravity.
- **Hardware Acceleration**: Uses `wgpu` to render graphics efficiently via Vulkan, Metal, DX12, or OpenGL.
- **UI & Statistics**:
  - Real-time score tracking.
  - "Next Piece" preview.
  - Piece statistics table showing the count and percentage of shapes received.
- **Visuals**:
  - Color-coded shapes (Classic 7-color palette).
  - Custom drawn text and icons (rendering logic handled manually in `vertex_data.rs`).

## Controls

| Key | Action |
| --- | --- |
| **Left Arrow** | Move Piece Left |
| **Right Arrow** | Move Piece Right |
| **Up Arrow** | Rotate Piece |
| **Down Arrow** | Soft Drop (Accelerate Fall) |
| **Space** | Hard Drop (Instant Place) |
| **Esc** | Exit Game |

## How to Run

1. **Install Rust**: Ensure you have Rust and Cargo installed via [rustup.rs](https://rustup.rs).
2. **Clone & Run**:
   ```bash
   git clone https://github.com/nlz242/learning_rust_tetris.git
   cd learning_rust_tetris
   cargo run
   ```

## Technical Details

- **Winit**: Handles window creation and input events.
- **WGPU**: Handles the graphics pipeline, shaders, and draw calls.
- **Buffers**: The game logic is decoupled from the renderer; `vertex_data.rs` converts the game state (grid, pieces, stats) into a single vertex buffer every frame.

## License

This project is for educational purposes.

