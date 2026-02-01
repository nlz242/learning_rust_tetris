use crate::graphic_context::{Vertex, TextEntry};
use crate::game::{Game, WIDTH, HEIGHT};
use crate::tetromino::TetrominoShape;

const COLORS: [[f32; 4]; 7] = [
    [0.0, 1.0, 1.0, 1.0], // I - Cyan
    [1.0, 1.0, 0.0, 1.0], // O - Yellow
    [0.5, 0.0, 0.5, 1.0], // T - Purple
    [0.0, 1.0, 0.0, 1.0], // S - Green
    [1.0, 0.0, 0.0, 1.0], // Z - Red
    [0.0, 0.0, 1.0, 1.0], // J - Blue
    [1.0, 0.5, 0.0, 1.0], // L - Orange
];

const UI_COLOR: [f32; 4] = [0.8, 0.8, 0.8, 1.0]; // Light grey for UI elements

pub fn get_color(index: usize) -> [f32; 4] {
    if index < 7 {
        COLORS[index]
    } else {
        [1.0, 1.0, 1.0, 1.0] // Fallback white
    }
}

pub fn build_mesh(game: &Game, window_width: u32, window_height: u32) -> (Vec<Vertex>, Vec<TextEntry>) {
    let mut vertices = Vec::new();
    let mut text_entries = Vec::new();

    // Layout configuration
    // Grid: 10 wide, 20 high.
    // Side panel: starts at x=11, say 6 wide.
    // Total logical area: 28x29 (Widened for Stats).
    
    let logical_width = WIDTH as f32 + 16.0; // 10 + padding/ui space (was +8.0)
    let logical_height = 29.0; // Compacted height to zoom in

    // Determine scale to fit logical area into window while maintaining aspect ratio
    // We want 1 logical unit = N pixels, where N is same for X and Y.
    
    // Scale factor based on height (fit vertical)
    let _scale_h = 2.0 / logical_height; 
    // Normalized Device Coordinates are -1 to 1 (height 2).
    // So if logical height is 20, we want 1 unit = 2/20 = 0.1 NDC height.

    // Correction for aspect ratio
    // NDC Width is 2.0 (-1 to 1).
    // Physical Width / Physical Height = Aspect.
    // To make a square, 1 unit X in NDC = (1 unit Y in NDC) / Aspect.
    let aspect = window_width as f32 / window_height as f32;
    let unit_size_y = 1.9 / logical_height; // Leave a little margin (1.9 instead of 2.0)
    let unit_size_x = unit_size_y / aspect; // Correct for non-square window

    // Center the content
    // Total width in NDC = unit_size_x * logical_width
    let total_ndc_width = unit_size_x * logical_width;
    let start_x = -total_ndc_width / 2.0;
    
    // Total height in NDC = unit_size_y * logical_height
    let total_ndc_height = unit_size_y * logical_height;
    let start_y = total_ndc_height / 2.0;

    let ctx = DrawContext {
        unit_size_x,
        unit_size_y,
        start_x,
        start_y,
    };

    // 1. Render the Grid Background/Border (Optional - can be just empty space)
    // Let's draw a border around the grid
    draw_rect_outline(&mut vertices, ctx, 0.0, 0.0, WIDTH as f32, HEIGHT as f32, [0.3, 0.3, 0.3, 1.0]);

    // 2. Render Existing Grid Blocks
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let cell = game.grid[y][x];
            if cell > 0 {
                let color_idx = (cell - 1) as usize;
                let color = get_color(color_idx);
                add_block(&mut vertices, ctx, x as f32, y as f32, color);
            }
        }
    }
    
    // Ghost Piece
    if let Some(ghost) = game.get_ghost_piece_position() {
        let color_idx = ghost.shape.to_index();
        let base_color = get_color(color_idx);
        let ghost_color = [base_color[0], base_color[1], base_color[2], 0.05]; // low alpha

        for (cx, cy) in ghost.cells.iter() {
            let x = ghost.x + cx;
            let y = ghost.y + cy;

            if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
                add_block(&mut vertices, ctx, x as f32, y as f32, ghost_color);
            }
        }
    }

    // 3. Render Active Piece
    if let Some(ref piece) = game.current_piece {
        let color_idx = piece.shape.to_index();
        let color = get_color(color_idx);
        
        for (cx, cy) in piece.cells.iter() {
            let x = piece.x + cx;
            let y = piece.y + cy;

            if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
                add_block(&mut vertices, ctx, x as f32, y as f32, color);
            }
        }
    }

    // 4. Render UI - Next Piece
    // Valid positions: x=11..
    let ui_start_x = WIDTH as f32 + 2.0;
    
    // Label "NEXT":
    text_entries.push(TextEntry {
        text: "NEXT".to_string(),
        x: ui_start_x,
        y: 0.5,
        color: UI_COLOR,
        scale: 0.8,
    });
    
    let next_piece_y = 2.0;
    let next_color = get_color(game.next_piece.to_index());
    
    for (cx, cy) in game.next_piece.cells().iter() {
         let px = ui_start_x + 2.0 + *cx as f32;
         let py = next_piece_y + 2.0 + *cy as f32;
         add_block(&mut vertices, ctx, px, py, next_color);
    }
    
    // Draw box around next piece area
    draw_rect_outline(&mut vertices, ctx, ui_start_x, next_piece_y, 5.0, 5.0, UI_COLOR);


    // 5. Render Score
    let score_y = 9.0;
    let score_label_y = 8.0; 
    
    text_entries.push(TextEntry {
        text: "SCORE".to_string(),
        x: ui_start_x,
        y: score_label_y,
        color: UI_COLOR,
        scale: 0.8,
    });

    let score_str = game.score.to_string();
    text_entries.push(TextEntry {
        text: score_str,
        x: ui_start_x,
        y: score_y,
        color: [1.0, 1.0, 1.0, 1.0],
        scale: 1.0, 
    });

    // 6. Render Statistics
    // x = ui_start_x
    // start y = 14.0 (Need more space below Score)
    let stats_ptr_y = 12.0;

    text_entries.push(TextEntry {
        text: "STATS".to_string(),
        x: ui_start_x,
        y: stats_ptr_y - 1.2,
        color: UI_COLOR,
        scale: 0.8,
    });
    
    // Calculate total for percentages
    let total_pieces: u32 = game.piece_stats.iter().sum();

    for i in 0..7 {
        // Increase spacing to allow for the visual shape
        let spacing = 2.3; // Shapes are roughly 2 high, plus gap
        let shape_stat_y = stats_ptr_y + (i as f32 * spacing); 
        
        let shape = TetrominoShape::from_index(i);
        let color = get_color(i);

        // 1. Draw Visual Representation (Mini-Shape)
        let mini_scale = 0.6;
        
        // Base position for the shape
        let icon_center_x = ui_start_x + 1.5; 
        let icon_center_y = shape_stat_y + 0.5;

        // Draw the 4 cells
        for (cx, cy) in shape.cells().iter() {
            let mut cell_ctx = ctx;
            cell_ctx.unit_size_x *= mini_scale; 
            cell_ctx.unit_size_y *= mini_scale;
            
            let effective_x = (icon_center_x / mini_scale) + (*cx as f32);
            let effective_y = (icon_center_y / mini_scale) + (*cy as f32);
            
            add_block(&mut vertices, cell_ctx, effective_x, effective_y, color);
        }

        // 2. Draw Count
        let count = game.piece_stats[i];
        let count_str = count.to_string();
        
        let text_start_x = ui_start_x + 3.5;
        let text_y = shape_stat_y + 0.2; // Adjust for font baseline

        text_entries.push(TextEntry {
            text: count_str,
            x: text_start_x,
            y: text_y,
            color: [1.0, 1.0, 1.0, 1.0],
            scale: 0.7,
        });

        // 3. Draw Percentage
        if total_pieces > 0 {
            let pct = (count as f32 / total_pieces as f32) * 100.0;
            let pct_val = pct as u32; // Integer part
            
            // Gap for percentage
            let pct_start_x = text_start_x + 4.0;
            
            // Dash -
            text_entries.push(TextEntry {
                text: "-".to_string(),
                x: text_start_x + 2.5,
                y: text_y,
                color: UI_COLOR,
                scale: 0.7,
            });

            // "XX%"
            let pct_text = format!("{}%", pct_val);
             text_entries.push(TextEntry {
                text: pct_text,
                x: pct_start_x,
                y: text_y,
                color: [1.0, 1.0, 1.0, 1.0],
                scale: 0.7,
            });
        }
    }

    (vertices, text_entries)
}


#[derive(Clone, Copy)]
struct DrawContext {
    unit_size_x: f32,
    unit_size_y: f32,
    start_x: f32,
    start_y: f32,
}

fn add_block(vertices: &mut Vec<Vertex>, ctx: DrawContext, x: f32, y: f32, color: [f32; 4]) {
    // Bevel logic for 3D effect
    let margin = 0.05;
    let block_size = 1.0 - (margin * 2.0);
    
    let sx = ctx.start_x + ((x + margin) * ctx.unit_size_x);
    let sy = ctx.start_y - ((y + margin) * ctx.unit_size_y);
    
    let w = block_size * ctx.unit_size_x;
    let h = block_size * ctx.unit_size_y;

    // Bevel effect colors
    let r = color[0];
    let g = color[1];
    let b = color[2];
    let a = color[3];

    // Lighter color for top/left
    let light = [
        (r + 0.3).min(1.0), 
        (g + 0.3).min(1.0), 
        (b + 0.3).min(1.0), 
        a
    ];
    
    // Darker color for bottom/right
    let dark = [
        (r * 0.6), 
        (g * 0.6), 
        (b * 0.6), 
        a
    ];

    let center_color = color;
    
    // Size of the bevel border (percentage of the block width/height)
    let bevel_ratio = 0.15;
    let bevel_size_x = w * bevel_ratio;
    let bevel_size_y = h * bevel_ratio;

    let left = sx;
    let right = sx + w;
    let top = sy;
    let bottom = sy - h;

    let inner_left = left + bevel_size_x;
    let inner_right = right - bevel_size_x;
    let inner_top = top - bevel_size_y;
    let inner_bottom = bottom + bevel_size_y;

    // 1. Center Rectangle (Original Color)
    draw_quad_absolute(vertices, inner_left, inner_right, inner_top, inner_bottom, center_color);

    // 2. Top Trapezoid (Light)
    // TL, InnerTL, InnerTR, TR
    vertices.push(Vertex { position: [left, top, 0.0], color: light }); 
    vertices.push(Vertex { position: [inner_left, inner_top, 0.0], color: light });
    vertices.push(Vertex { position: [inner_right, inner_top, 0.0], color: light });
    vertices.push(Vertex { position: [left, top, 0.0], color: light }); 
    vertices.push(Vertex { position: [inner_right, inner_top, 0.0], color: light });
    vertices.push(Vertex { position: [right, top, 0.0], color: light });

    // 3. Left Trapezoid (Light)
    // TL, BL, InnerBL, InnerTL
    vertices.push(Vertex { position: [left, top, 0.0], color: light }); 
    vertices.push(Vertex { position: [left, bottom, 0.0], color: light });
    vertices.push(Vertex { position: [inner_left, inner_bottom, 0.0], color: light });
    vertices.push(Vertex { position: [left, top, 0.0], color: light });
    vertices.push(Vertex { position: [inner_left, inner_bottom, 0.0], color: light });
    vertices.push(Vertex { position: [inner_left, inner_top, 0.0], color: light });

    // 4. Right Trapezoid (Dark)
    // TR, InnerTR, InnerBR, BR
    vertices.push(Vertex { position: [right, top, 0.0], color: dark });
    vertices.push(Vertex { position: [inner_right, inner_top, 0.0], color: dark });
    vertices.push(Vertex { position: [inner_right, inner_bottom, 0.0], color: dark });
    vertices.push(Vertex { position: [right, top, 0.0], color: dark });
    vertices.push(Vertex { position: [inner_right, inner_bottom, 0.0], color: dark });
    vertices.push(Vertex { position: [right, bottom, 0.0], color: dark });

    // 5. Bottom Trapezoid (Dark)
    // BL, InnerBL, InnerBR, BR
    vertices.push(Vertex { position: [left, bottom, 0.0], color: dark });
    vertices.push(Vertex { position: [inner_left, inner_bottom, 0.0], color: dark });
    vertices.push(Vertex { position: [inner_right, inner_bottom, 0.0], color: dark });
    vertices.push(Vertex { position: [left, bottom, 0.0], color: dark });
    vertices.push(Vertex { position: [inner_right, inner_bottom, 0.0], color: dark });
    vertices.push(Vertex { position: [right, bottom, 0.0], color: dark });
}

fn draw_rect_outline(vertices: &mut Vec<Vertex>, ctx: DrawContext, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
    // Simple 4 lines using thin quads
    let ndc_x = ctx.start_x + (x * ctx.unit_size_x);
    let ndc_y = ctx.start_y - (y * ctx.unit_size_y);
    let ndc_w = w * ctx.unit_size_x;
    let ndc_h = h * ctx.unit_size_y;
    
    let t_x = 0.05 * ctx.unit_size_x; // thickness
    let t_y = 0.05 * ctx.unit_size_y;

    // Top
    draw_quad(vertices, ndc_x, ndc_x + ndc_w, ndc_y, ndc_y - t_y, color);
    // Bottom
    draw_quad(vertices, ndc_x, ndc_x + ndc_w, ndc_y - ndc_h + t_y, ndc_y - ndc_h, color);
    // Left
    draw_quad(vertices, ndc_x, ndc_x + t_x, ndc_y, ndc_y - ndc_h, color);
    // Right
    draw_quad(vertices, ndc_x + ndc_w - t_x, ndc_x + ndc_w, ndc_y, ndc_y - ndc_h, color);
}

fn draw_quad(vertices: &mut Vec<Vertex>, left: f32, right: f32, top: f32, bottom: f32, color: [f32; 4]) {
    draw_quad_absolute(vertices, left, right, top, bottom, color);
}

fn draw_quad_absolute(vertices: &mut Vec<Vertex>, left: f32, right: f32, top: f32, bottom: f32, color: [f32; 4]) {
    vertices.push(Vertex { position: [left, top, 0.0], color });
    vertices.push(Vertex { position: [left, bottom, 0.0], color });
    vertices.push(Vertex { position: [right, bottom, 0.0], color });

    vertices.push(Vertex { position: [left, top, 0.0], color });
    vertices.push(Vertex { position: [right, bottom, 0.0], color });
    vertices.push(Vertex { position: [right, top, 0.0], color });
}


// draw_digit removed
