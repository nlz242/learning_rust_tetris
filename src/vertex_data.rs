use crate::graphic_context::Vertex;
use crate::game::{Game, WIDTH, HEIGHT};
use crate::tetromino::TetrominoShape;

const COLORS: [[f32; 3]; 7] = [
    [0.0, 1.0, 1.0], // I - Cyan
    [1.0, 1.0, 0.0], // O - Yellow
    [0.5, 0.0, 0.5], // T - Purple
    [0.0, 1.0, 0.0], // S - Green
    [1.0, 0.0, 0.0], // Z - Red
    [0.0, 0.0, 1.0], // J - Blue
    [1.0, 0.5, 0.0], // L - Orange
];

const UI_COLOR: [f32; 3] = [0.8, 0.8, 0.8]; // Light grey for UI elements

pub fn get_color(index: usize) -> [f32; 3] {
    if index < 7 {
        COLORS[index]
    } else {
        [1.0, 1.0, 1.0] // Fallback white
    }
}

pub fn build_mesh(game: &Game, window_width: u32, window_height: u32) -> Vec<Vertex> {
    let mut vertices = Vec::new();

    // Layout configuration
    // Grid: 10 wide, 20 high.
    // Side panel: starts at x=11, say 6 wide.
    // Total logical area: 18x20.
    
    let logical_width = WIDTH as f32 + 8.0; // 10 + padding/ui
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
    draw_rect_outline(&mut vertices, ctx, 0.0, 0.0, WIDTH as f32, HEIGHT as f32, [0.3, 0.3, 0.3]);

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
    // Draw letters roughly? Maybe too hard.
    // Just draw the piece.
    let next_piece_y = 2.0;
    let next_color = get_color(game.next_piece.to_index());
    
    for (cx, cy) in game.next_piece.cells().iter() {
         // Center the piece locally in the UI box
         // Standard piece coords are around 0,0.
         // Let's shift it to ui_start_x + 2, next_piece_y + 2
         let px = ui_start_x + 2.0 + *cx as f32;
         let py = next_piece_y + 2.0 + *cy as f32;
         add_block(&mut vertices, ctx, px, py, next_color);
    }
    
    // Draw box around next piece area
    draw_rect_outline(&mut vertices, ctx, ui_start_x, next_piece_y, 5.0, 5.0, UI_COLOR);


    // 5. Render Score
    let score_y = 9.0;
    let score_label_y = 8.0; // "SC"
    // Ideally we would label "SCORE"
    
    // Draw digits
    let score_str = game.score.to_string();
    for (i, char) in score_str.chars().enumerate() {
        if let Some(digit) = char.to_digit(10) {
            let dx = ui_start_x + (i as f32 * 1.5); // Spacing
            draw_digit(&mut vertices, ctx, dx, score_y, digit as u8);
        }
    }

    // 6. Render Statistics
    // x = ui_start_x
    // start y = 14.0 (Need more space below Score)
    let stats_ptr_y = 12.0;
    
    // Calculate total for percentages
    let total_pieces: u32 = game.piece_stats.iter().sum();

    for i in 0..7 {
        // Increase spacing to allow for the visual shape
        let spacing = 2.3; // Shapes are roughly 2 high, plus gap
        let shape_stat_y = stats_ptr_y + (i as f32 * spacing); 
        
        let shape = TetrominoShape::from_index(i);
        let color = get_color(i);

        // 1. Draw Visual Representation (Mini-Shape)
        // We'll scale it down to 0.7 size to fit nicely
        let mini_scale = 0.6;
        let mut mini_ctx = ctx;
        mini_ctx.unit_size_x *= mini_scale;
        mini_ctx.unit_size_y *= mini_scale;

        // Base position for the shape (accounting for scale influence on coordinates)
        // We want absolute position (ui_start_x + 1.0, shape_stat_y)
        // Since add_block uses `start_x + x * unit_size`, we need to adjust x input.
        // x_input = absolute_target / unit_size (roughly)
        // But add_block is relative to DrawContext setup.
        // The simplest way is to manually call draw_quad with calculated NDC coordinates 
        // OR adjust the DrawContext start_x/y temporarily.
        // Let's rely on add_block but adjust the input x/y to compensate for the smaller unit_size
        // relative to the "logical" grid the context was built for.
        
        // Actually, easier strategy: Just pass standard coordinates to `add_block` but apply a local offset
        // relative to the shape center, multiplied by scale factor? 
        // No, `add_block` does `x * unit_size`. 
        // If we want a "small block", we must provide a context with small unit_size.
        // To place that small block at Position P, we need `start_x + x_input * small_unit = P`.
        // So `x_input = (P - start_x) / small_unit`.
        
        // Let's calculate the logical position where we want the shape center
        let icon_center_x = ui_start_x + 1.5; 
        let icon_center_y = shape_stat_y + 0.5;

        // Draw the 4 cells
        for (cx, cy) in shape.cells().iter() {
            // Apply scale locally to the offsets
            // We can trick add_block by keeping the main context (standard unit size) 
            // but just passing fractional coordinates for the small blocks?
            // e.g. x = icon_center_x + (cx * 0.6)
            // But `add_block` draws a full logical block (size ~0.9). 
            // If we want visually smaller blocks, we MUST change unit_size in context OR modify add_block.
            
            // Let's modify the passed context's unit_size
            let mut cell_ctx = ctx;
            cell_ctx.unit_size_x *= mini_scale; 
            cell_ctx.unit_size_y *= mini_scale;
            
            // Re-calculate effective x/y inputs to place them at correct screen spot
            // Target NDC X = main_start_x + (icon_center_x + cx*scale) * main_unit_x
            // Current Formula = main_start_x + input * (main_unit_x * scale)
            // So input = (icon_center_x/scale) + cx
            
            let effective_x = (icon_center_x / mini_scale) + (*cx as f32);
            let effective_y = (icon_center_y / mini_scale) + (*cy as f32);
            
            add_block(&mut vertices, cell_ctx, effective_x, effective_y, color);
        }

        // 2. Draw Count
        let count = game.piece_stats[i];
        let count_str = count.to_string();
        
        // Position text to the right of the icon
        let text_start_x = ui_start_x + 3.5;
        let text_y = shape_stat_y + 0.5; // Vertical alignment

        for (j, char) in count_str.chars().enumerate() {
            if let Some(digit) = char.to_digit(10) {
                 let dx = text_start_x + (j as f32 * 1.5);
                 // Use small digits (0.6 scale)
                 let mut text_ctx = ctx;
                 text_ctx.unit_size_x *= 0.6;
                 text_ctx.unit_size_y *= 0.6;
                 let effective_x = dx / 0.6;
                 let effective_y = text_y / 0.6;
                 
                 draw_digit(&mut vertices, text_ctx, effective_x, effective_y, digit as u8);
            }
        }

        // 3. Draw Percentage
        if total_pieces > 0 {
            let pct = (count as f32 / total_pieces as f32) * 100.0;
            let pct_val = pct as u32; // Integer part
            let pct_str = pct_val.to_string();
            
            // Gap for percentage
            let pct_start_x = text_start_x + 4.0;
            
            // Draw dashes for separation? "-"
            // Approximate "-" with horizontal segment
            let mut dash_ctx = ctx;
            dash_ctx.unit_size_x *= 0.6;
            dash_ctx.unit_size_y *= 0.6;
            let dash_x = (text_start_x + 2.5) / 0.6;
            let dash_y = (text_y + 0.5) / 0.6; // Shift down slightly
            // We can use a custom quad draw for a dash or add a 'dash' to draw_digit logic
            // Let's just draw a raw quad for dash
            // NDC coords
            let dash_ndc_x = ctx.start_x + ((text_start_x + 2.5) * ctx.unit_size_x);
            let dash_ndc_y = ctx.start_y - ((text_y + 0.5) * ctx.unit_size_y);
            let dash_w = 1.0 * ctx.unit_size_x * 0.6;
            let dash_h = 0.2 * ctx.unit_size_y * 0.6;
            draw_quad(&mut vertices, dash_ndc_x, dash_ndc_x + dash_w, dash_ndc_y, dash_ndc_y - dash_h, UI_COLOR);


            for (j, char) in pct_str.chars().enumerate() {
                 if let Some(digit) = char.to_digit(10) {
                     let dx = pct_start_x + (j as f32 * 1.5);
                     let mut text_ctx = ctx;
                     text_ctx.unit_size_x *= 0.6;
                     text_ctx.unit_size_y *= 0.6;
                     let effective_x = dx / 0.6;
                     let effective_y = text_y / 0.6;
                     
                     draw_digit(&mut vertices, text_ctx, effective_x, effective_y, digit as u8);
                }
            }
            
            // Draw % symbol
            // Position after the number
            let pct_val_str_len = pct_str.len() as f32;
            let percent_x = pct_start_x + (pct_val_str_len * 1.5);
            let mut pct_ctx = ctx;
            pct_ctx.unit_size_x *= 0.6;
            pct_ctx.unit_size_y *= 0.6;
            let eff_pct_x = percent_x / 0.6;
            let eff_pct_y = text_y / 0.6;
            draw_percent_symbol(&mut vertices, pct_ctx, eff_pct_x, eff_pct_y);
        }
    }

    vertices
}

fn draw_percent_symbol(vertices: &mut Vec<Vertex>, ctx: DrawContext, x: f32, y: f32) {
    let color = [1.0, 1.0, 1.0];
    let thickness = 0.15;
    
    // Size of the box is roughly 1x2 in logical units (like digits)
    // 1. Slash / (Top-Right to Bottom-Left)
    // Coords: (x+1, y) to (x, y+2) roughly? Digits are 1 wide, 2 high.
    // Actually draw_digit uses 1.0 wide, 2.0 height (approx).
    // Let's use (x+0.8, y) to (x+0.2, y+2.0)
    
    // Slash 
    // We can simulate a diagonal line with a quad? Or just a bunch of small quads?
    // Since we don't have rotation in `draw_quad` (it's axis aligned), 
    // we can draw a "stepped" line or just pass rotated coords if we manually push vertices.
    // Let's manually push a rotated quad (two triangles).
    
    let sx = ctx.start_x + (x * ctx.unit_size_x);
    let sy = ctx.start_y - (y * ctx.unit_size_y);
    let w = 1.0 * ctx.unit_size_x;
    let h = 2.0 * ctx.unit_size_y;
    
    // Slash diagonal
    let slash_thickness_x = thickness * ctx.unit_size_x;
    
    // Top-Right Corner
    let tr_x = sx + w; 
    let tr_y = sy;
    
    // Bottom-Left Corner
    let bl_x = sx;
    let bl_y = sy - h;
    
    // We want a strip from TR to BL.
    // V1: TR
    // V2: BL
    // V3: BL + thickness
    // V4: TR + thickness
    // Note: Simple axis-aligned quad won't work perfectly.
    // Let's just push 2 triangles manually.
    
    // P1 (Top Right Inner)
    let p1_x = tr_x - slash_thickness_x;
    let p1_y = tr_y;
    
    // P2 (Bottom Left Inner)
    let p2_x = bl_x;
    let p2_y = bl_y + (thickness * ctx.unit_size_y); // lift up slightly
    
    // P3 (Bottom Left Outer)
    let p3_x = bl_x + slash_thickness_x;
    let p3_y = bl_y;
    
    // P4 (Top Right Outer)
    let p4_x = tr_x;
    let p4_y = tr_y - (thickness * ctx.unit_size_y);
    
    // Triangle 1 (Top part of slash)
    vertices.push(Vertex { position: [p1_x, p1_y, 0.0], color });
    vertices.push(Vertex { position: [p2_x, p2_y, 0.0], color }); 
    vertices.push(Vertex { position: [p4_x, p4_y, 0.0], color });

    // Triangle 2 (Bottom part of slash)
    vertices.push(Vertex { position: [p2_x, p2_y, 0.0], color });
    vertices.push(Vertex { position: [p3_x, p3_y, 0.0], color });
    vertices.push(Vertex { position: [p4_x, p4_y, 0.0], color });

    // 2. Top-Left Circle (Square for now)
    // Small square at roughly 0.2, 0.2
    let dot_size = 0.25;
    let dot1_x = x + 0.1;
    let dot1_y = y + 0.2;
    // Use draw_quad logic manually or reuse
    let d1_sx = ctx.start_x + (dot1_x * ctx.unit_size_x);
    let d1_sy = ctx.start_y - (dot1_y * ctx.unit_size_y);
    let d1_w = dot_size * ctx.unit_size_x;
    let d1_h = dot_size * ctx.unit_size_y;
    
    // Generic Axis Aligned Quad
    let draw_raw_quad = |verts: &mut Vec<Vertex>, l, r, t, b, c| {
         verts.push(Vertex { position: [l, t, 0.0], color: c });
         verts.push(Vertex { position: [l, b, 0.0], color: c });
         verts.push(Vertex { position: [r, b, 0.0], color: c });
         verts.push(Vertex { position: [l, t, 0.0], color: c });
         verts.push(Vertex { position: [r, b, 0.0], color: c });
         verts.push(Vertex { position: [r, t, 0.0], color: c });
    };

    draw_raw_quad(vertices, d1_sx, d1_sx + d1_w, d1_sy, d1_sy - d1_h, color);

    // 3. Bottom-Right Circle
    let dot2_x = x + 0.65;
    let dot2_y = y + 1.2;
    let d2_sx = ctx.start_x + (dot2_x * ctx.unit_size_x);
    let d2_sy = ctx.start_y - (dot2_y * ctx.unit_size_y);
    
    draw_raw_quad(vertices, d2_sx, d2_sx + d1_w, d2_sy, d2_sy - d1_h, color);
}

#[derive(Clone, Copy)]
struct DrawContext {
    unit_size_x: f32,
    unit_size_y: f32,
    start_x: f32,
    start_y: f32,
}

fn add_block(vertices: &mut Vec<Vertex>, ctx: DrawContext, x: f32, y: f32, color: [f32; 3]) {
    // x, y are logical grid coordinates (0..WIDTH, 0..HEIGHT)
    // Convert to NDC
    let size_w = ctx.unit_size_x * 0.90; // Gap
    let size_h = ctx.unit_size_y * 0.90;

    let pos_x = ctx.start_x + (x * ctx.unit_size_x);
    let pos_y = ctx.start_y - (y * ctx.unit_size_y); // y goes down

    // Quad
    let left = pos_x;
    let right = pos_x + size_w;
    let top = pos_y;
    let bottom = pos_y - size_h;

    draw_quad(vertices, left, right, top, bottom, color);
}

fn draw_rect_outline(vertices: &mut Vec<Vertex>, ctx: DrawContext, x: f32, y: f32, w: f32, h: f32, color: [f32; 3]) {
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

fn draw_quad(vertices: &mut Vec<Vertex>, left: f32, right: f32, top: f32, bottom: f32, color: [f32; 3]) {
    vertices.push(Vertex { position: [left, top, 0.0], color });
    vertices.push(Vertex { position: [left, bottom, 0.0], color });
    vertices.push(Vertex { position: [right, bottom, 0.0], color });

    vertices.push(Vertex { position: [left, top, 0.0], color });
    vertices.push(Vertex { position: [right, bottom, 0.0], color });
    vertices.push(Vertex { position: [right, top, 0.0], color });
}

// Simple 7-segment-ish digit renderer
// Digit size: 1x2 logical units
fn draw_digit(vertices: &mut Vec<Vertex>, ctx: DrawContext, x: f32, y: f32, digit: u8) {
    let color = [1.0, 1.0, 1.0];
    
    // Segments:
    //  --0--
    // |     |
    // 1     2
    // |     |
    //  --3--
    // |     |
    // 4     5
    // |     |
    //  --6--
    
    let thickness = 0.2;
    // Horizontal segments (w=1, h=thickness)
    let h_seg = |vertices: &mut Vec<Vertex>, ctx: DrawContext, dy: f32| {
        // draw raw quad relative to x,y
        let sx = ctx.start_x + (x * ctx.unit_size_x);
        let sy = ctx.start_y - ((y + dy) * ctx.unit_size_y);
        let tw = 1.0 * ctx.unit_size_x;
        let th = thickness * ctx.unit_size_y;
        draw_quad(vertices, sx, sx + tw, sy, sy - th, color);
    };

    // Vertical segments (w=thickness, h=1)
    let v_seg = |vertices: &mut Vec<Vertex>, ctx: DrawContext, dx: f32, dy: f32| {
        let sx = ctx.start_x + ((x + dx) * ctx.unit_size_x);
        let sy = ctx.start_y - ((y + dy) * ctx.unit_size_y);
        let tw = thickness * ctx.unit_size_x;
        let th = 1.0 * ctx.unit_size_y;
        draw_quad(vertices, sx, sx + tw, sy, sy - th, color);
    };

    // Map digits to segments
    // 0: 0,1,2,4,5,6 (Not 3)
    let s = match digit {
        0 => vec![0,1,2,4,5,6],
        1 => vec![2,5],
        2 => vec![0,2,3,4,6],
        3 => vec![0,2,3,5,6],
        4 => vec![1,2,3,5],
        5 => vec![0,1,3,5,6],
        6 => vec![0,1,3,4,5,6],
        7 => vec![0,2,5],
        8 => vec![0,1,2,3,4,5,6],
        9 => vec![0,1,2,3,5,6], 
        _ => vec![],
    };

    if s.contains(&0) { h_seg(vertices, ctx, 0.0); }
    if s.contains(&1) { v_seg(vertices, ctx, 0.0, 0.0); }
    if s.contains(&2) { v_seg(vertices, ctx, 1.0 - thickness, 0.0); }
    if s.contains(&3) { h_seg(vertices, ctx, 1.0); }
    if s.contains(&4) { v_seg(vertices, ctx, 0.0, 1.0); }
    if s.contains(&5) { v_seg(vertices, ctx, 1.0 - thickness, 1.0); }
    if s.contains(&6) { h_seg(vertices, ctx, 2.0 - thickness); }
}