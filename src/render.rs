use macroquad::prelude::*;
use crate::constants::*;
use crate::types::*;

fn terrain_color(t: Terrain, time: f64) -> Color {
    match t {
        Terrain::Empty => Color::new(0.06, 0.09, 0.14, 1.0),
        Terrain::Water => { let b = 0.25 + (time / 500.0).sin() as f32 * 0.05; Color::new(0.0, 0.3, b + 0.5, 1.0) }
        Terrain::Tree => Color::new(0.08, 0.39, 0.20, 1.0),
        Terrain::Sand => Color::new(0.79, 0.54, 0.03, 1.0),
        Terrain::Iron => Color::new(0.28, 0.33, 0.42, 1.0),
        Terrain::Copper => Color::new(0.49, 0.18, 0.07, 1.0),
        Terrain::Coal => Color::new(0.04, 0.06, 0.10, 1.0),
        Terrain::Quartz => Color::new(0.03, 0.57, 0.78, 1.0),
        Terrain::Gold => Color::new(0.52, 0.30, 0.05, 1.0),
        Terrain::Oil => Color::new(0.03, 0.03, 0.03, 1.0),
        Terrain::Uranium => Color::new(0.08, 0.33, 0.18, 1.0),
        Terrain::Wasteland => Color::new(0.25, 0.38, 0.07, 1.0),
    }
}

fn draw_building(wx: f32, wy: f32, ws: f32, cell: &Cell, on: bool, cam_scale: f32, _time: f64) {
    match cell.tool {
        Tool::Conveyor => {
            draw_rectangle(wx, wy, ws, ws, Color::new(0.20, 0.25, 0.33, 1.0));
            // Big arrow
            let cx = wx + ws * 0.5; let cy = wy + ws * 0.5;
            let (dx, dy, txt) = match cell.dir { 0 => (0.0,-0.3,"^"), 1 => (0.3,0.0,">"), 2 => (0.0,0.3,"v"), _ => (-0.3,0.0,"<") };
            draw_text(txt, cx + ws * dx - ws * 0.12, cy + ws * dy + ws * 0.1, 22.0 * cam_scale, Color::new(0.58, 0.64, 0.70, 1.0));
        }
        Tool::Street => {
            draw_rectangle(wx, wy, ws, ws, Color::new(0.15, 0.18, 0.22, 1.0));
            draw_line(wx, wy + ws * 0.5, wx + ws, wy + ws * 0.5, 2.0, Color::new(0.39, 0.45, 0.52, 0.6));
            draw_line(wx + ws * 0.5, wy, wx + ws * 0.5, wy + ws, 2.0, Color::new(0.39, 0.45, 0.52, 0.6));
        }
        Tool::Pipe => {
            draw_rectangle(wx + ws * 0.3, wy + ws * 0.3, ws * 0.4, ws * 0.4, Color::new(0.28, 0.33, 0.42, 1.0));
            for d in 0..4 {
                let (ddx, ddy) = DIRS[d];
                let _nc = ((wx - 1.0) / ws) as i32 + ddx; // approximate
                let _nr = ((wy - 1.0) / ws) as i32 + ddy;
                // Just draw all 4 pipe stubs for simplicity
                match d {
                    0 => draw_rectangle(wx + ws * 0.4, wy, ws * 0.2, ws * 0.4, Color::new(0.28, 0.33, 0.42, 0.5)),
                    1 => draw_rectangle(wx + ws * 0.6, wy + ws * 0.4, ws * 0.4, ws * 0.2, Color::new(0.28, 0.33, 0.42, 0.5)),
                    2 => draw_rectangle(wx + ws * 0.4, wy + ws * 0.6, ws * 0.2, ws * 0.4, Color::new(0.28, 0.33, 0.42, 0.5)),
                    _ => draw_rectangle(wx, wy + ws * 0.4, ws * 0.4, ws * 0.2, Color::new(0.28, 0.33, 0.42, 0.5)),
                }
            }
            if cell.fluid_amount > 0.0 {
                let fc = if cell.fluid_type == Some(FluidType::Water) { Color::new(0.22, 0.74, 0.97, 0.9) } else { Color::new(0.1, 0.1, 0.1, 0.9) };
                draw_circle(wx + ws * 0.5, wy + ws * 0.5, ws * 0.12, fc);
            }
        }
        Tool::Node => {
            let glow = if on { Color::new(0.22, 0.74, 0.97, 1.0) } else { Color::new(0.39, 0.45, 0.52, 1.0) };
            draw_line(wx + ws * 0.35, wy + ws * 0.85, wx + ws * 0.5, wy + ws * 0.15, 3.0 * cam_scale, Color::new(0.50, 0.55, 0.60, 1.0));
            draw_line(wx + ws * 0.65, wy + ws * 0.85, wx + ws * 0.5, wy + ws * 0.15, 3.0 * cam_scale, Color::new(0.50, 0.55, 0.60, 1.0));
            draw_circle(wx + ws * 0.5, wy + ws * 0.15, ws * 0.14, glow);
        }
        Tool::Battery => {
            draw_rectangle(wx + ws * 0.15, wy + ws * 0.1, ws * 0.7, ws * 0.8, Color::new(0.12, 0.16, 0.23, 1.0));
            draw_rectangle(wx + ws * 0.35, wy + ws * 0.02, ws * 0.3, ws * 0.1, Color::new(0.45, 0.50, 0.55, 1.0));
            let fill = cell.charge / BATTERY_CAP;
            let fh = ws * 0.65 * fill;
            let fc = if fill > 0.5 { Color::new(0.06, 0.72, 0.38, 1.0) } else if fill > 0.2 { Color::new(0.96, 0.62, 0.04, 1.0) } else { RED };
            draw_rectangle(wx + ws * 0.2, wy + ws * 0.85 - fh, ws * 0.6, fh, fc);
            draw_text("BAT", wx + ws * 0.2, wy + ws * 0.55, 12.0 * cam_scale, Color::new(0.7, 0.7, 0.7, 0.8));
        }
        Tool::Nuclear => {
            draw_circle(wx + ws * 0.5, wy + ws * 0.5, ws * 0.48, Color::new(0.10, 0.14, 0.20, 1.0));
            draw_circle(wx + ws * 0.5, wy + ws * 0.5, ws * 0.38, Color::new(0.18, 0.23, 0.30, 1.0));
            let ic = if cell.fuel > 0.0 { Color::new(0.29, 0.87, 0.50, 1.0) } else { Color::new(0.02, 0.31, 0.23, 1.0) };
            draw_circle(wx + ws * 0.5, wy + ws * 0.5, ws * 0.22, ic);
            draw_text("N", wx + ws * 0.38, wy + ws * 0.58, 16.0 * cam_scale, WHITE);
            if cell.heat > 0.0 { draw_circle(wx + ws * 0.5, wy + ws * 0.5, ws * 0.48, Color::new(0.94, 0.27, 0.27, cell.heat / 100.0)); }
        }
        Tool::CoalPlant => {
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, Color::new(0.20, 0.25, 0.33, 1.0));
            // Chimney
            draw_rectangle(wx + ws * 0.65, wy + ws * 0.05, ws * 0.2, ws * 0.45, Color::new(0.30, 0.35, 0.40, 1.0));
            if cell.fuel > 0.0 { draw_rectangle(wx + ws * 0.2, wy + ws * 0.6, ws * 0.3, ws * 0.15, Color::new(0.98, 0.45, 0.09, 1.0)); }
            draw_text("C", wx + ws * 0.3, wy + ws * 0.5, 14.0 * cam_scale, Color::new(0.7, 0.7, 0.7, 0.8));
        }
        Tool::Solar => {
            let bg = if on { Color::new(0.01, 0.52, 0.78, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            for i in 1..3 { let f = i as f32 / 3.0;
                draw_line(wx + ws * 0.05, wy + ws * f, wx + ws * 0.95, wy + ws * f, 1.0, Color::new(0.9, 0.95, 1.0, 0.4));
                draw_line(wx + ws * f, wy + ws * 0.05, wx + ws * f, wy + ws * 0.95, 1.0, Color::new(0.9, 0.95, 1.0, 0.4));
            }
            draw_text("S", wx + ws * 0.35, wy + ws * 0.65, 16.0 * cam_scale, Color::new(1.0, 1.0, 1.0, 0.7));
        }
        Tool::Wind => {
            draw_circle(wx + ws * 0.5, wy + ws * 0.5, ws * 0.18, Color::new(0.58, 0.64, 0.70, 1.0));
            let angle = (_time / 100.0) as f32;
            for b in 0..3 {
                let a = angle + b as f32 * std::f32::consts::TAU / 3.0;
                draw_line(wx + ws * 0.5, wy + ws * 0.5, wx + ws * 0.5 + a.cos() * ws * 0.4, wy + ws * 0.5 + a.sin() * ws * 0.4, 3.0 * cam_scale, Color::new(0.90, 0.92, 0.95, 0.9));
            }
        }
        Tool::Miner => {
            let bg = if on { Color::new(0.96, 0.62, 0.04, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            draw_text("M", wx + ws * 0.28, wy + ws * 0.68, 20.0 * cam_scale, WHITE);
        }
        Tool::Pump => {
            let bg = if on { Color::new(0.05, 0.65, 0.91, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            draw_text("B", wx + ws * 0.28, wy + ws * 0.68, 20.0 * cam_scale, WHITE);
        }
        Tool::Pumpjack => {
            let bg = if on { Color::new(0.25, 0.30, 0.38, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            draw_circle(wx + ws * 0.5, wy + ws * 0.5, ws * 0.25, Color::new(0.05, 0.05, 0.05, 1.0));
            draw_text("P", wx + ws * 0.35, wy + ws * 0.6, 16.0 * cam_scale, WHITE);
        }
        Tool::Lumberjack => {
            let bg = if on { Color::new(0.13, 0.77, 0.37, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            draw_text("L", wx + ws * 0.28, wy + ws * 0.68, 20.0 * cam_scale, WHITE);
        }
        Tool::Smelter => {
            let bg = if on { Color::new(0.47, 0.21, 0.06, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            if on { draw_triangle(Vec2::new(wx+ws*0.5,wy+ws*0.25), Vec2::new(wx+ws*0.2,wy+ws*0.75), Vec2::new(wx+ws*0.8,wy+ws*0.75), Color::new(0.98,0.45,0.09,1.0)); }
            draw_text("F", wx + ws * 0.35, wy + ws * 0.6, 14.0 * cam_scale, WHITE);
        }
        Tool::Press => {
            let bg = if on { Color::new(0.30, 0.35, 0.43, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            draw_rectangle(wx + ws * 0.15, wy + ws * 0.2, ws * 0.7, ws * 0.15, Color::new(0.50, 0.55, 0.60, 1.0));
            draw_rectangle(wx + ws * 0.15, wy + ws * 0.65, ws * 0.7, ws * 0.15, Color::new(0.50, 0.55, 0.60, 1.0));
            draw_text("PR", wx + ws * 0.22, wy + ws * 0.58, 12.0 * cam_scale, WHITE);
        }
        Tool::Assembler => {
            let bg = if on { Color::new(0.08, 0.72, 0.65, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            if on { draw_circle(wx + ws * 0.5, wy + ws * 0.5, ws * 0.28, Color::new(0.04, 0.06, 0.10, 1.0)); }
            draw_text("A", wx + ws * 0.33, wy + ws * 0.62, 18.0 * cam_scale, WHITE);
        }
        Tool::ChemPlant => {
            let bg = if on { Color::new(0.06, 0.46, 0.43, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            if on { draw_circle(wx+ws*0.35,wy+ws*0.5,ws*0.18,Color::new(0.66,0.33,0.97,1.0)); draw_circle(wx+ws*0.65,wy+ws*0.5,ws*0.18,Color::new(0.93,0.28,0.60,1.0)); }
            draw_text("Q", wx + ws * 0.35, wy + ws * 0.58, 14.0 * cam_scale, WHITE);
        }
        Tool::Centrifuge => {
            let bg = if on { Color::new(0.08, 0.39, 0.20, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            if on {
                draw_circle(wx+ws*0.5,wy+ws*0.5,ws*0.32,Color::new(0.06,0.28,0.15,1.0));
                for b in 0..3 { let a = b as f32 * std::f32::consts::TAU / 3.0;
                    draw_circle(wx+ws*0.5+a.cos()*ws*0.2, wy+ws*0.5+a.sin()*ws*0.2, ws*0.08, Color::new(0.29,0.87,0.50,1.0)); }
            }
        }
        Tool::Quantum => {
            let bg = if on { Color::new(0.19, 0.18, 0.51, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            if on { draw_rectangle_lines(wx+ws*0.2,wy+ws*0.2,ws*0.6,ws*0.6,2.0,Color::new(0.66,0.33,0.97,1.0));
                draw_circle(wx+ws*0.5,wy+ws*0.5,ws*0.15,Color::new(0.85,0.71,0.99,1.0)); }
            draw_text("QC", wx + ws * 0.22, wy + ws * 0.58, 12.0 * cam_scale, WHITE);
        }
        Tool::Market => {
            let bg = if on { Color::new(0.70, 0.33, 0.04, 1.0) } else { Color::new(0.15, 0.18, 0.22, 1.0) };
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, bg);
            draw_text("$", wx + ws * 0.28, wy + ws * 0.72, 24.0 * cam_scale, Color::new(0.06, 0.72, 0.38, 1.0));
        }
        Tool::House => {
            draw_rectangle(wx + ws * 0.1, wy + ws * 0.4, ws * 0.8, ws * 0.55, Color::new(0.80, 0.84, 0.88, 1.0));
            draw_triangle(Vec2::new(wx+ws*0.5,wy+ws*0.05), Vec2::new(wx+ws*0.05,wy+ws*0.42), Vec2::new(wx+ws*0.95,wy+ws*0.42), Color::new(0.94,0.27,0.27,1.0));
            draw_rectangle(wx + ws * 0.35, wy + ws * 0.6, ws * 0.3, ws * 0.35, Color::new(0.47, 0.21, 0.04, 1.0));
        }
        Tool::Warehouse => {
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, Color::new(0.28, 0.33, 0.42, 1.0));
            draw_rectangle(wx + ws * 0.15, wy + ws * 0.2, ws * 0.7, ws * 0.25, Color::new(0.39, 0.45, 0.52, 1.0));
            draw_rectangle(wx + ws * 0.15, wy + ws * 0.55, ws * 0.7, ws * 0.25, Color::new(0.39, 0.45, 0.52, 1.0));
            draw_text("W", wx + ws * 0.3, wy + ws * 0.55, 14.0 * cam_scale, WHITE);
        }
        _ => {
            draw_rectangle(wx + ws * 0.05, wy + ws * 0.05, ws * 0.9, ws * 0.9, Color::new(0.15, 0.18, 0.22, 1.0));
        }
    }
}

fn draw_player(x: f32, y: f32, cam_scale: f32, is_local: bool, name: &str) {
    let size = 20.0 * cam_scale;
    let color = if is_local { Color::new(0.22, 0.74, 0.97, 1.0) } else { Color::new(0.94, 0.27, 0.27, 1.0) };
    let body_r = size * 0.55;
    // Shadow
    draw_circle(x, y + body_r * 0.5, body_r * 0.8, Color::new(0.0, 0.0, 0.0, 0.25));
    // Body
    draw_circle(x, y, body_r, color);
    // Inner ring
    draw_circle(x, y, body_r * 0.5, Color::new(1.0, 1.0, 1.0, 0.25));
    // Head nub
    draw_circle(x, y - body_r * 0.7, body_r * 0.35, WHITE);
    // Nameplate background
    let label_w = name.len() as f32 * 7.0 + 8.0;
    let label_x = x - label_w / 2.0;
    let label_y = y - body_r * 1.8;
    draw_rectangle(label_x - 2.0, label_y - 13.0, label_w, 16.0, Color::new(0.0, 0.0, 0.0, 0.55));
    draw_text(name, label_x, label_y, 13.0, if is_local { Color::new(0.5, 1.0, 0.95, 1.0) } else { WHITE });
}

fn draw_npc(x: f32, y: f32, cam_scale: f32, name: &str, state_str: &str) {
    let size = 18.0 * cam_scale;
    let is_inspecting = state_str == "inspect";
    // Body: golden diamond shape (NPC badge)
    let color = if is_inspecting {
        Color::new(0.94, 0.27, 0.27, 1.0)
    } else {
        Color::new(0.82, 0.68, 0.16, 1.0)
    };
    // Draw a rotated square (diamond) using 4 triangles trick → just circle with star-ish look
    draw_circle(x, y, size * 0.5, color);
    draw_circle(x, y, size * 0.22, Color::new(0.06, 0.04, 0.01, 0.8));
    // Name tag
    let label = if is_inspecting { format!("⚠ {}", name) } else { name.to_string() };
    draw_text(&label, x - 28.0, y - size * 0.8, 12.0 * cam_scale.max(0.8), color);
}

pub fn render_game(state: &mut GameState, time: f64) {
    clear_background(Color::new(0.01, 0.02, 0.04, 1.0));
    let cam = &state.camera;
    let inv_scale = 1.0 / cam.scale;
    let vl = -cam.x * inv_scale; let vt = -cam.y * inv_scale;
    let vr = vl + screen_width() * inv_scale; let vb = vt + screen_height() * inv_scale;
    let sc = ((vl / CELL_SIZE).floor() as i32 - 1).max(0) as usize;
    let sr = ((vt / CELL_SIZE).floor() as i32 - 1).max(0) as usize;
    let ec = ((vr / CELL_SIZE).ceil() as i32 + 1).min(GRID_SIZE as i32) as usize;
    let er = ((vb / CELL_SIZE).ceil() as i32 + 1).min(GRID_SIZE as i32) as usize;

    // Terrain
    for r in sr..er { for c in sc..ec {
        let wx = c as f32 * CELL_SIZE * cam.scale + cam.x;
        let wy = r as f32 * CELL_SIZE * cam.scale + cam.y;
        let ws = CELL_SIZE * cam.scale;
        let t = state.terrain[r * GRID_SIZE + c];
        
        if let Some(ref texs) = state.textures {
            let tex = match t {
                Terrain::Empty => &texs.grass,
                Terrain::Water => &texs.water,
                Terrain::Tree => &texs.tree,
                Terrain::Sand => &texs.sand,
                Terrain::Iron => &texs.iron,
                Terrain::Copper => &texs.copper,
                Terrain::Coal => &texs.coal,
                Terrain::Quartz => &texs.quartz,
                Terrain::Gold => &texs.gold,
                Terrain::Oil => &texs.oil,
                Terrain::Uranium => &texs.uranium,
                Terrain::Wasteland => &texs.wasteland,
            };

            let mut tint = WHITE;
            if t == Terrain::Empty { tint = Color::new(0.7, 0.8, 0.7, 1.0); } // Grass filter (darker/greener)

            // Draw base texture
            let overlap = 0.8;
            draw_texture_ex(tex, wx - overlap, wy - overlap, tint, DrawTextureParams {
                dest_size: Some(vec2(ws + overlap * 2.0, ws + overlap * 2.0)),
                ..Default::default()
            });

            // --- Blending / Degradê Logic ---
            let p = t.priority();
            // Check neighbors (North, East, South, West)
            let neighbors = [
                (0, -1, 0), // North
                (1, 0, 1),  // East
                (0, 1, 2),  // South
                (-1, 0, 3), // West
            ];

            for (di, dj, side) in neighbors {
                let nc = (c as i32 + di).clamp(0, GRID_SIZE as i32 - 1) as usize;
                let nr = (r as i32 + dj).clamp(0, GRID_SIZE as i32 - 1) as usize;
                let nt = state.terrain[nr * GRID_SIZE + nc];

                if nt.priority() > p {
                    // This neighbor should bleed into current tile
                    let key = (t, nt, side);
                    
                    // We need to access textures mutably to check the cache
                    if let Some(texs) = &mut state.textures {
                        let blend_tex = if let Some(cached) = texs.transitions.get(&key) {
                            cached
                        } else {
                            // Generate and cache
                            let new_tex = state.forge.generate_blend_overlay(t, nt, side);
                            texs.transitions.insert(key, new_tex);
                            texs.transitions.get(&key).unwrap()
                        };

                        draw_texture_ex(blend_tex, wx - overlap, wy - overlap, WHITE, DrawTextureParams {
                            dest_size: Some(vec2(ws + overlap * 2.0, ws + overlap * 2.0)),
                            ..Default::default()
                        });
                    }
                }
            }

            if t == Terrain::Wasteland {
                draw_rectangle(wx, wy, ws, ws, Color::new(0.52, 0.80, 0.10, 0.4));
            }
        } else {
            draw_rectangle(wx, wy, ws, ws, terrain_color(t, time));
            match t {
                Terrain::Iron | Terrain::Copper | Terrain::Coal | Terrain::Quartz | Terrain::Gold | Terrain::Uranium => {
                    let h = match t { Terrain::Iron=>Color::new(0.58,0.64,0.70,0.8), Terrain::Copper=>Color::new(0.92,0.35,0.05,0.8),
                        Terrain::Quartz=>Color::new(0.73,0.90,0.99,0.8), Terrain::Gold=>Color::new(0.99,0.94,0.27,0.8),
                        Terrain::Uranium=>Color::new(0.29,0.87,0.50,0.8), _=>Color::new(0.20,0.25,0.33,0.8) };
                    draw_circle(wx+ws*0.3,wy+ws*0.3,ws*0.08,h); draw_circle(wx+ws*0.7,wy+ws*0.65,ws*0.08,h); draw_circle(wx+ws*0.5,wy+ws*0.75,ws*0.06,h);
                }
                Terrain::Tree => { draw_rectangle(wx+ws*0.4,wy+ws*0.5,ws*0.2,ws*0.4,Color::new(0.47,0.21,0.04,1.0)); draw_circle(wx+ws*0.5,wy+ws*0.35,ws*0.3,Color::new(0.08,0.39,0.20,1.0)); }
                Terrain::Wasteland => { draw_circle(wx+ws*0.5,wy+ws*0.5,ws*0.25,Color::new(0.52,0.80,0.10,0.5)); }
                _ => {}
            }
        }
    }}

    // Buildings
    for r in sr..er { for c in sc..ec {
        let wx = c as f32 * CELL_SIZE * cam.scale + cam.x;
        let wy = r as f32 * CELL_SIZE * cam.scale + cam.y;
        let ws = CELL_SIZE * cam.scale;
        let idx = r * GRID_SIZE + c;
        // Removed grid lines as requested for better immersion
        // draw_rectangle_lines(wx, wy, ws, ws, 1.0, Color::new(1.0,1.0,1.0,0.03));
        let cell = match &state.grid[idx] { Some(c) => c, None => {
            if c as i32 == state.mouse.world_col && r as i32 == state.mouse.world_row { draw_rectangle_lines(wx,wy,ws,ws,2.0,WHITE); }
            continue;
        }};
        let has_power = state.powered.contains(&idx);
        let is_broken = cell.health <= 0.0;
        let on = has_power && !is_broken;

        // Blueprint effect (transparency if under construction)
        if cell.construction_progress < 100.0 {
            // Draw a ghost version (simulated by low alpha)
            // Note: Macroquad doesn't have a global alpha for all draw calls, 
            // but we can pass a modified color to draw_building if we want.
            // For now, let's just draw the rectangle/outline differently.
            draw_rectangle_lines(wx, wy, ws, ws, 2.0, Color::new(1.0, 1.0, 1.0, 0.3));
            draw_text(&format!("{:.0}%", cell.construction_progress), wx + 2.0, wy + 12.0 * cam.scale, 12.0 * cam.scale, WHITE);
        } else {
            draw_building(wx, wy, ws, cell, on, cam.scale, time);
        }
        // Processing bar
        if matches!(cell.tool, Tool::Smelter|Tool::Press|Tool::Assembler|Tool::ChemPlant|Tool::Quantum|Tool::Centrifuge) && !is_broken && cell.processing > 0.0 {
            let bc = match cell.tool { Tool::Quantum=>Color::new(0.85,0.71,0.99,1.0), Tool::Assembler|Tool::Centrifuge=>Color::new(0.20,0.83,0.60,1.0), _=>Color::new(0.94,0.27,0.27,1.0) };
            draw_rectangle(wx+ws*0.1,wy+ws*0.88,ws*0.8*(cell.processing/100.0),ws*0.08,bc);
        }
        if is_broken { draw_rectangle(wx,wy,ws,ws,Color::new(0.0,0.0,0.0,0.6)); draw_text("X",wx+ws*0.3,wy+ws*0.65,22.0*cam.scale,RED); }
        else if !has_power && !matches!(cell.tool, Tool::Warehouse|Tool::Node|Tool::Street|Tool::Solar|Tool::Wind|Tool::CoalPlant|Tool::Nuclear|Tool::Battery|Tool::Conveyor|Tool::Pipe) {
            draw_circle(wx+ws*0.8,wy+ws*0.2,ws*0.12,RED); draw_text("!",wx+ws*0.74,wy+ws*0.27,12.0*cam.scale,WHITE);
        }
        if !is_broken && cell.health < 100.0 && cell.health > 0.0 {
            let hc = if cell.health > 50.0 { Color::new(0.06,0.72,0.38,1.0) } else { RED };
            draw_rectangle(wx+ws*0.1,wy+ws*0.92,ws*0.8*(cell.health/100.0),ws*0.05,hc);
        }
        if c as i32 == state.mouse.world_col && r as i32 == state.mouse.world_row { draw_rectangle_lines(wx,wy,ws,ws,2.0,WHITE); }
    }}

    // --- PLAYERS ---
    // Other players
    for (sender, pos) in &state.other_players {
        draw_player(pos.x * cam.scale + cam.x, pos.y * cam.scale + cam.y, cam.scale, false, sender);
    }
    // Local player
    draw_player(state.local_player.pos.x * cam.scale + cam.x, state.local_player.pos.y * cam.scale + cam.y, cam.scale, true, &state.username);

    // --- NPCS ---
    for npc in &state.npcs {
        let sx = npc.x * CELL_SIZE * cam.scale + cam.x;
        let sy = npc.y * CELL_SIZE * cam.scale + cam.y;
        draw_npc(sx, sy, cam.scale, &npc.name, &npc.state);
    }

    // Items
    for item in &state.items {
        let cx = item.x as i32; let cy = item.y as i32;
        if cx < sc as i32 || cx >= ec as i32 || cy < sr as i32 || cy >= er as i32 { continue; }
        if let Some(idx) = GameState::idx(cx, cy) {
            if let Some(ref cell) = state.grid[idx] { if cell.tool == Tool::Conveyor {
                let (dx,dy) = DIRS[cell.dir as usize];
                let fx = cx as f32+0.5+dx as f32*(item.progress-0.5);
                let fy = cy as f32+0.5+dy as f32*(item.progress-0.5);
                let sx = fx*CELL_SIZE*cam.scale+cam.x; let sy = fy*CELL_SIZE*cam.scale+cam.y;
                let sz = CELL_SIZE*0.35*cam.scale;
                // Draw item sprite if available, else fall back to colored rectangle
                let drawn = if let Some(ref texs) = state.textures {
                    if let Some(tex) = texs.sprites.items.get(&item.item_type) {
                        draw_texture_ex(tex, sx-sz/2.0, sy-sz/2.0, WHITE, DrawTextureParams {
                            dest_size: Some(vec2(sz, sz)),
                            ..Default::default()
                        });
                        true
                    } else { false }
                } else { false };
                if !drawn {
                    draw_rectangle(sx-sz/2.0, sy-sz/2.0, sz, sz, item.item_type.color());
                }
            }}
        }
    }

    // Power links (LAYER 4 — aerial, on top)
    for link in &state.power_links {
        let ux=(link.u%GRID_SIZE)as f32*CELL_SIZE+CELL_SIZE/2.0; let uy=(link.u/GRID_SIZE)as f32*CELL_SIZE+CELL_SIZE/2.0;
        let vx=(link.v%GRID_SIZE)as f32*CELL_SIZE+CELL_SIZE/2.0; let vy=(link.v/GRID_SIZE)as f32*CELL_SIZE+CELL_SIZE/2.0;
        let alpha = 0.5+(time/200.0).sin()as f32*0.3;
        draw_line(ux*cam.scale+cam.x,uy*cam.scale+cam.y,vx*cam.scale+cam.x,vy*cam.scale+cam.y, 2.0, Color::new(0.22,0.74,0.97,alpha));
    }

    for &(cx,cy) in &state.other_cursors { draw_circle(cx,cy,5.0,Color::new(0.94,0.27,0.27,0.8)); }

    // Night
    let mut opacity = 0.0f32;
    if state.time_of_day > 1800.0 { opacity = (state.time_of_day-1800.0)/600.0*0.7; }
    else if state.time_of_day < 600.0 { opacity = (600.0-state.time_of_day)/600.0*0.7; }
    if opacity > 0.0 { draw_rectangle(0.0,0.0,screen_width(),screen_height(),Color::new(0.01,0.02,0.04,opacity)); }
}

// ============ UI — 100% OPAQUE, BIG TEXT ============
pub fn render_ui(state: &GameState, hover_info: &Option<HoverInfo>, mouse_x: f32, mouse_y: f32) {
    let sw = screen_width(); let sh = screen_height();
    // Premium Color Palette
    let hud_bg = Color::new(0.02, 0.04, 0.08, 0.95);
    let hud_bg2 = Color::new(0.05, 0.08, 0.15, 1.0);
    let hud_border = Color::new(0.15, 0.65, 0.85, 0.6);
    let text_dim = Color::new(0.6, 0.7, 0.8, 1.0);
    let accent = Color::new(0.22, 0.74, 0.97, 1.0);

    // === TOP LEFT: Inventory ===
    let px = 10.0; let py = 10.0; let pw = 260.0;
    draw_rectangle(px, py, pw, 30.0, hud_bg);
    draw_rectangle_lines(px, py, pw, 30.0, 1.0, hud_border);
    draw_text("INVENTARIO", px+8.0, py+22.0, 24.0, Color::new(0.58,0.64,0.70,1.0));
    let mut sorted: Vec<(&ItemType,&i32)> = state.inventory.iter().filter(|(_,v)|**v>0).collect();
    sorted.sort_by(|a,b| b.1.cmp(a.1));
    let inv_h = sorted.len().min(14) as f32 * 24.0 + 4.0;
    draw_rectangle(px, py+30.0, pw, inv_h, hud_bg);
    draw_rectangle_lines(px, py+30.0, pw, inv_h, 1.0, hud_border);
    for (i,(k,v)) in sorted.iter().take(14).enumerate() {
        let iy = py+34.0+i as f32*24.0;
        draw_rectangle(px+6.0,iy+2.0,16.0,16.0,k.color());
        draw_text(k.name_pt(), px+26.0, iy+17.0, 18.0, Color::new(0.85,0.88,0.92,1.0));
        draw_text(&format!("{}",v), px+pw-55.0, iy+17.0, 18.0, WHITE);
    }

    // === TOP RIGHT: Economy ===
    let rw = 340.0; let rx = sw-rw-10.0; let ry = 10.0;
    draw_rectangle(rx, ry, rw, 175.0, hud_bg);
    draw_rectangle_lines(rx, ry, rw, 175.0, 2.0, hud_border);
    
    draw_text("CENTRAL DE COMANDO", rx+12.0, ry+26.0, 20.0, accent);
    
    // Money display with subtle background
    draw_rectangle(rx+10.0, ry+35.0, rw-20.0, 32.0, Color::new(1.0,1.0,1.0,0.03));
    let mc = if state.money<0 { RED } else { Color::new(0.3, 0.95, 0.55, 1.0) };
    draw_text(&format!("Saldo: ${}",state.money), rx+18.0, ry+58.0, 24.0, mc);
    
    let ic = if state.income>=0 { Color::new(0.3, 0.95, 0.55, 1.0) } else { RED };
    draw_text(&format!("Fluxo: {}/tick",state.income), rx+12.0, ry+85.0, 18.0, ic);
    
    let hour = (state.time_of_day/100.0) as i32;
    draw_text(&format!("Horário: {:02}:00",hour), rx+12.0, ry+110.0, 18.0, text_dim);
    
    let pc = if state.power_gen>=state.power_cons { accent } else { RED };
    draw_text(&format!("Energia: {}W / {}W",state.power_gen as i32,state.power_cons as i32), rx+12.0, ry+135.0, 18.0, pc);
    
    draw_text(&format!("População: {} (Crise: {})",state.pop,state.unpowered), rx+12.0, ry+160.0, 18.0, text_dim);

    if state.rival_event.is_none() {
        draw_text(&format!("Meta: {} ({}X)",state.demand.item.name_pt(),state.demand.multiplier), rx+12.0, ry+185.0, 18.0, Color::new(0.96,0.62,0.04,1.0));
    }
    if let Some(ref ev) = state.rival_event { draw_text(&format!("RIVAL: {}",ev.msg), rx+12.0, ry+185.0, 18.0, RED); }

    // Cloud buttons
    let cy = ry+205.0;
    draw_rectangle(rx, cy, rw/2.0-3.0, 30.0, hud_bg2);
    draw_rectangle_lines(rx, cy, rw/2.0-3.0, 30.0, 1.0, hud_border);
    draw_text("NUVEM: SALVAR", rx+10.0, cy+21.0, 16.0, Color::new(0.22,0.74,0.97,1.0));
    
    draw_rectangle(rx+rw/2.0+3.0, cy, rw/2.0-3.0, 30.0, hud_bg2);
    draw_rectangle_lines(rx+rw/2.0+3.0, cy, rw/2.0-3.0, 30.0, 1.0, hud_border);
    draw_text("GUIA (AJUDA)", rx+rw/2.0+12.0, cy+21.0, 16.0, Color::new(0.96,0.62,0.04,1.0));

    // Market
    let my = cy+38.0;
    draw_rectangle(rx, my, rw, 28.0, hud_bg2);
    draw_rectangle_lines(rx, my, rw, 28.0, 1.0, hud_border);
    draw_text("BOLSA DE VALORES", rx+12.0, my+21.0, 18.0, accent);
    let items = ItemType::tradeable_items();
    let mkt_h = items.len() as f32 * 24.0 + 4.0;
    draw_rectangle(rx, my+26.0, rw, mkt_h.min(sh-my-266.0), hud_bg);
    draw_rectangle_lines(rx, my+26.0, rw, mkt_h.min(sh-my-266.0), 1.0, hud_border);
    for (i,it) in items.iter().enumerate() {
        let py2 = my+30.0+i as f32*24.0;
        if py2 > sh-240.0 { break; }
        let bp = it.base_price();
        let price = state.prices.get(it).unwrap_or(&bp);
        let trend = state.price_trends.get(it).unwrap_or(&0);
        let tc = match trend { 1=>Color::new(0.29,0.87,0.50,1.0), -1=>RED, _=>Color::new(0.58,0.64,0.70,1.0) };
        let ts = match trend { 1=>"^", -1=>"v", _=>"-" };
        draw_rectangle(rx+6.0,py2+2.0,16.0,16.0,it.color());
        draw_text(it.name_pt(), rx+26.0, py2+17.0, 16.0, Color::new(0.75,0.78,0.82,1.0));
        draw_text(&format!("${} {}",price,ts), rx+rw-95.0, py2+17.0, 16.0, tc);
    }

    // === BOTTOM: Tool palette ===
    let bx = 10.0; let by = sh-230.0;
    for (ci,name) in CATEGORY_NAMES.iter().enumerate() {
        let tx = bx+ci as f32*104.0;
        let sel = ci == state.selected_category;
        let bg = if sel { Color::new(0.14,0.18,0.26,1.0) } else { hud_bg };
        draw_rectangle(tx, by, 102.0, 30.0, bg);
        draw_rectangle_lines(tx, by, 102.0, 30.0, 1.0, if sel { Color::new(0.22,0.74,0.97,1.0) } else { hud_border });
        draw_text(name, tx+6.0, by+22.0, 16.0, if sel { Color::new(0.22,0.74,0.97,1.0) } else { Color::new(0.50,0.55,0.60,1.0) });
    }
    let tools: Vec<&Tool> = Tool::all().iter().filter(|t| t.category()==state.selected_category).collect();
    let rows_needed = (tools.len()+4)/5;
    let palette_h = rows_needed as f32 * 62.0 + 4.0;
    draw_rectangle(bx, by+30.0, 520.0, palette_h, hud_bg);
    draw_rectangle_lines(bx, by+30.0, 520.0, palette_h, 1.0, hud_border);
    for (ti,tool) in tools.iter().enumerate() {
        let col = ti%5; let row = ti/5;
        let tw = 100.0; let th = 58.0;
        let tx = bx+4.0+col as f32*(tw+4.0);
        let ty = by+34.0+row as f32*(th+4.0);
        let sel = state.selected_tool == **tool;
        draw_rectangle(tx, ty, tw, th, if sel { Color::new(0.18,0.22,0.32,1.0) } else { hud_bg2 });
        draw_rectangle_lines(tx, ty, tw, th, if sel { 2.0 } else { 1.0 }, if sel { Color::new(0.22,0.74,0.97,1.0) } else { hud_border });
        draw_text(tool.hotkey(), tx+4.0, ty+16.0, 14.0, Color::new(0.58,0.64,0.70,1.0));
        // Draw HUD sprite if available, else show text name
        let sprite_drawn = if let Some(ref texs) = state.textures {
            if let Some(sprite) = texs.sprites.hud.get(tool) {
                // Draw sprite: centered horizontally, below the hotkey label
                let spad = 4.0;
                let avail_w = tw - spad * 2.0;
                let avail_h = th - 20.0 - spad; // below hotkey text
                let sprite_size = avail_w.min(avail_h);
                let sx = tx + (tw - sprite_size) / 2.0;
                let sy = ty + 18.0;
                draw_texture_ex(sprite, sx, sy, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(sprite_size, sprite_size)),
                    ..Default::default()
                });
                true
            } else { false }
        } else { false };
        if !sprite_drawn {
            draw_text(tool.name_pt(), tx+4.0, ty+34.0, 16.0, if sel { Color::new(0.22,0.74,0.97,1.0) } else { WHITE });
        }
        if sel {
            // Highlight border glow when selected and sprite shown
            draw_rectangle_lines(tx+1.0, ty+1.0, tw-2.0, th-2.0, 1.0, Color::new(0.22,0.74,0.97,0.4));
        }
        if !matches!(tool, Tool::Eraser|Tool::Repair) { draw_text(&format!("-${}",tool.cost()), tx+4.0, ty+52.0, 14.0, Color::new(0.94,0.27,0.27,0.9)); }
    }

    // === TOOLTIP ===
    if let Some(ref info) = hover_info {
        let tw = 280.0;
        let tx = (mouse_x+20.0).min(sw-tw-10.0);
        let ty = (mouse_y+20.0).min(sh-140.0);
        draw_rectangle(tx, ty, tw, 130.0, hud_bg);
        draw_rectangle_lines(tx, ty, tw, 130.0, 1.0, hud_border);
        draw_text(&info.title, tx+10.0, ty+26.0, 22.0, WHITE);
        if let Some(hp) = info.health {
            let (s,c) = if hp>0.0 { (format!("OK ({:.0}%)",hp), Color::new(0.29,0.87,0.50,1.0)) } else { ("QUEBRADO".into(), RED) };
            draw_text(&s, tx+10.0, ty+50.0, 18.0, c);
        }
        draw_text(&format!("Vento: {}%",info.wind), tx+10.0, ty+72.0, 18.0, Color::new(0.22,0.74,0.97,1.0));
        if let Some(f) = info.fuel { draw_text(&format!("Fuel: {:.0}",f), tx+10.0, ty+94.0, 18.0, Color::new(0.96,0.62,0.04,1.0)); }
        if let Some(h) = info.heat { draw_text(&format!("Calor: {:.0}%",h), tx+150.0, ty+94.0, 18.0, if h>75.0{RED}else{Color::new(0.96,0.62,0.04,1.0)}); }
        if let Some(ch) = info.charge { draw_text(&format!("Carga: {:.0}W",ch), tx+10.0, ty+116.0, 18.0, Color::new(0.29,0.87,0.50,1.0)); }
    }

    // Alert
    if !state.stats_msg.is_empty() {
        let mw = 520.0; let mx = (sw-mw)/2.0;
        draw_rectangle(mx, 55.0, mw, 55.0, Color::new(0.80,0.10,0.10,1.0));
        draw_rectangle_lines(mx, 55.0, mw, 55.0, 2.0, Color::new(0.94,0.27,0.27,1.0));
        draw_text(&state.stats_msg, mx+16.0, 90.0, 26.0, WHITE);
    }

    // === ENCYCLOPEDIA (JEI-style) ===
    if state.help_open {
        let hw = 720.0; let hh = 560.0;
        let hx = (sw - hw) / 2.0; let hy = (sh - hh) / 2.0;

        // Background
        draw_rectangle(hx, hy, hw, hh, Color::new(0.05, 0.07, 0.12, 0.98));
        draw_rectangle_lines(hx, hy, hw, hh, 2.0, Color::new(0.22, 0.74, 0.97, 1.0));
        draw_text("ENCICLOPEDIA", hx + 260.0, hy + 30.0, 28.0, Color::new(0.22, 0.74, 0.97, 1.0));
        draw_text("[ESC para fechar]", hx + hw - 160.0, hy + 30.0, 14.0, Color::new(0.4, 0.4, 0.5, 1.0));

        // Tabs
        let tab_names = ["CONSTRUCOES", "RECEITAS", "CADEIAS", "ENERGIA"];
        for (i, tab) in tab_names.iter().enumerate() {
            let tx = hx + 10.0 + i as f32 * 180.0;
            let ty = hy + 40.0;
            let sel = i == state.help_tab;
            draw_rectangle(tx, ty, 175.0, 28.0, if sel { Color::new(0.22, 0.74, 0.97, 0.3) } else { Color::new(0.08, 0.11, 0.18, 1.0) });
            draw_rectangle_lines(tx, ty, 175.0, 28.0, 1.0, if sel { Color::new(0.22, 0.74, 0.97, 1.0) } else { Color::new(0.2, 0.25, 0.35, 1.0) });
            draw_text(tab, tx + 10.0, ty + 20.0, 16.0, if sel { Color::new(0.22, 0.74, 0.97, 1.0) } else { Color::new(0.6, 0.6, 0.7, 1.0) });
        }

        let cx = hx + 12.0; let cy = hy + 80.0;
        let line_h = 22.0;
        let title_col = Color::new(0.22, 0.74, 0.97, 1.0);
        let text_col = Color::new(0.88, 0.90, 0.95, 1.0);
        let warn_col = Color::new(0.97, 0.82, 0.17, 1.0);
        let good_col = Color::new(0.29, 0.87, 0.50, 1.0);

        match state.help_tab {
            0 => { // CONSTRUCOES
                let entries: &[(&str, &str)] = &[
                    ("ESTEIRA [1]",        "Transporta itens entre maquinas. Clique p/ girar a direcao."),
                    ("TUBO [2]",           "Transporta fluidos (Agua, Petroleo). Conecte Bomba -> Quimica."),
                    ("POSTE [3]",          "Distribui energia eletrica num raio de 3 celulas."),
                    ("RUA [4]",            "Permite que o jogador caminhe sobre agua. Protege de NPCs."),
                    ("ARMAZEM [5]",        "Coleta itens das esteiras. Adiciona ao seu inventario."),
                    ("REPARO [6]",         "Clique numa maquina quebrada p/ reparar (custa 50% do valor)."),
                    ("MINERADOR [Q]",      "Extrai minerio do terreno. Coloque sobre: Fe, Cu, Carvao, etc."),
                    ("BOMBA [T]",          "Extrai AGUA de celulas de agua. Conecte a tubos."),
                    ("EXTRATOR [E]",       "Extrai PETROLEO de celulas de oleo. Conecte a tubos."),
                    ("LENHADOR [R]",       "Corta arvores e ejecta MADEIRA em esteiras adjacentes."),
                    ("SOLAR [Y]",          "Gera 40W de dia. Zero energia a noite."),
                    ("EOLICA [U]",         "Gera 60W * vento local. Variavel."),
                    ("TERMOELETRICA [I]",  "Gera 300W. Alimenta com Carvao via esteira."),
                    ("NUCLEAR [F]",        ">LEIA A GUIA ENERGIA< Gera 2500W. Precisa de agua+uranio."),
                    ("BATERIA [G]",        "Armazena energia excedente. Carrega automaticamente."),
                    ("FUNDICAO [Z]",       "Converte minerio em chapas. Ex: Fe-Ore -> Chapa Ferro."),
                    ("PRENSA [X]",         "Converte Chapa Cobre -> Fio Cobre."),
                    ("MONTADORA [C]",      "Combina itens. Fe+Carvao->Aco, Vidro+Fio->Proc, Si+Fio->Placa."),
                    ("QUIMICA [V]",        "Agua + Petroleo -> Plastico. Precisa de tubos conectados."),
                    ("CENTRIFUGA [B]",     "3x Uranio -> Celula Uranio. Precisa de 3+ minerio acumulado."),
                    ("QUANTICO [N]",       "Proc + Plastico + Ouro -> Nucleo IA. Raro e caro."),
                    ("MERCADO [M]",        "Vende itens que chegam por esteira. Preco varia pelo mercado."),
                    ("CASA [H]",           "Adiciona populacao. Cada casa consome 5W. Gera renda +15/tick."),
                ];
                for (i, (name, desc)) in entries.iter().enumerate() {
                    let y = cy + i as f32 * line_h;
                    if y > hy + hh - 30.0 { break; }
                    draw_text(name, cx, y, 15.0, title_col);
                    draw_text(desc, cx + 170.0, y, 14.0, text_col);
                }
            }
            1 => { // RECEITAS
                let recipes: &[(&str, &str, &str)] = &[
                    // (Output, Ingredients, Machine)
                    ("Chapa Ferro",    "Minerio Ferro",                "Fundicao"),
                    ("Chapa Cobre",    "Minerio Cobre",                "Fundicao"),
                    ("Placa Carvao",   "Minerio Carvao",               "Fundicao"),
                    ("Silicio",        "Minerio Quartzo",              "Fundicao"),
                    ("Vidro",          "Minerio Areia",                "Fundicao"),
                    ("Lingote Ouro",   "Minerio Ouro",                 "Fundicao"),
                    ("Cel. Uranio",    "3x Minerio Uranio",            "Centrifuga"),
                    ("Fio Cobre",      "Chapa Cobre",                  "Prensa"),
                    ("Aco",            "Chapa Ferro + Placa Carvao",   "Montadora"),
                    ("Processador",    "Vidro + Fio Cobre",            "Montadora"),
                    ("Placa Circ.",    "Silicio + Fio Cobre",          "Montadora"),
                    ("Plastico",       "Agua + Petroleo (tubos)",      "Quimica"),
                    ("Nucleo IA",      "Proc + Plastico + L. Ouro",    "Quantico"),
                    ("Energia 300W",   "Carvao (na esteira)",          "Termoeletrica"),
                    ("Energia 2500W",  "Cel. Uranio + Agua (tubos)",   "Nuclear"),
                ];
                draw_rectangle(cx, cy - 5.0, 696.0, 22.0, Color::new(0.10, 0.15, 0.25, 1.0));
                draw_text("PRODUTO", cx + 4.0, cy + 13.0, 15.0, title_col);
                draw_text("INGREDIENTES", cx + 210.0, cy + 13.0, 15.0, title_col);
                draw_text("MAQUINA", cx + 520.0, cy + 13.0, 15.0, title_col);
                for (i, (out, ing, machine)) in recipes.iter().enumerate() {
                    let y = cy + 26.0 + i as f32 * 25.0;
                    if y > hy + hh - 30.0 { break; }
                    let bg = if i % 2 == 0 { Color::new(0.07, 0.10, 0.17, 1.0) } else { Color::new(0.05, 0.07, 0.13, 1.0) };
                    draw_rectangle(cx, y - 16.0, 696.0, 22.0, bg);
                    draw_text(out, cx + 4.0, y, 14.0, good_col);
                    draw_text(ing, cx + 210.0, y, 14.0, text_col);
                    draw_text(machine, cx + 520.0, y, 14.0, warn_col);
                }
            }
            2 => { // CADEIAS PRODUTIVAS
                let chains: &[(&str, &[&str])] = &[
                    ("CADEIA BASICA - FERRO",     &["Mapa: Terreno FERRO (marrom)", "Minerador (Q) -> Esteira (1) -> Fundicao (Z)", "Fundicao faz: Chapa Ferro", "Chapa Ferro -> Montadora (C) para fazer Aco (precisa Carvao tb)", "Aco -> Mercado (M) para vender"]),
                    ("CADEIA ELETRICA",           &["Mapa: Terreno COBRE (laranja)", "Minerador -> Fundicao -> Chapa Cobre", "Prensa (X) transforma Chapa Cobre em Fio Cobre", "Fio + Vidro (areia/fundicao) -> Montadora = PROCESSADOR (caro!)"]),
                    ("CADEIA ENERGIA BASICA",     &["Instale POSTES para conectar maquinas", "SOLAR (Y): funciona de dia gratis", "EOLICA (U): funciona sempre, varia com vento", "TERMOELETRICA (I): 300W mas precisa Carvao na esteira"]),
                    ("CADEIA NUCLEAR (AVANCADA)", &["1. Instale NUCLEAR (F) numa posicao livre", "2. Instale BOMBA (T) em celula de AGUA proxima", "3. Conecte TUBOS (2) da Bomba ao Nuclear", "4. Instale CENTRIFUGA (B) em Uranio do mapa", "5. Esteira: Centrifuga -> Nuclear (cel. uranio)", "6. Poste (N) p/ conectar na rede. Pronto: 2500W!"]),
                    ("CADEIA QUIMICA",            &["1. Instale EXTRATOR (E) em celula de PETROLEO", "2. Instale BOMBA (T) em celula de AGUA", "3. Conecte ambos com TUBOS (2) a QUIMICA (V)", "4. Quimica produz PLASTICO automaticamente", "5. Plastico + Processador + Ouro -> Nucleo IA (montedera)"]),
                ];
                for (ci, (chain_name, steps)) in chains.iter().enumerate() {
                    let section_y = cy + ci as f32 * 105.0;
                    if section_y > hy + hh - 30.0 { break; }
                    draw_rectangle(cx, section_y - 5.0, 696.0, 18.0, Color::new(0.15, 0.20, 0.30, 1.0));
                    draw_text(chain_name, cx + 4.0, section_y + 10.0, 15.0, title_col);
                    for (si, step) in steps.iter().enumerate() {
                        let sy = section_y + 15.0 + si as f32 * 18.0;
                        if sy > hy + hh - 30.0 { break; }
                        draw_text(&format!("  {}", step), cx, sy, 14.0, text_col);
                    }
                }
            }
            3 => { // ENERGIA & REATOR
                let lines: &[(&str, bool)] = &[
                    ("SISTEMA DE ENERGIA", true),
                    ("Maquinas precisam de eletricidade. Sem energia -> simbolo ! vermelho.", false),
                    ("Use POSTES (3) para conectar geradores a maquinas (raio 3 celulas).", false),
                    ("Tipos de geradores:", true),
                    ("  SOLAR (Y): 40W, gratis, so de dia (06h-18h). Ideal para inicio.", false),
                    ("  EOLICA (U): 60W * fator vento. Variavel mas constante.", false),
                    ("  TERMOELETRICA (I): 300W, precisa Carvao na esteira adjacente.", false),
                    ("  NUCLEAR (F): 2500W. A mais poderosa. Veja guia abaixo.", false),
                    ("  BATERIA (G): Armazena excesso de energia. Descarrega quando falta.", false),
                    ("", false),
                    ("COMO OPERAR O REATOR NUCLEAR", true),
                    ("!!! PERIGO: Sem agua, o reator esquenta e causa MELTDOWN (destroi area) !!!", false),
                    ("Passo 1: Construa o NUCLEAR (F) em qualquer terreno livre.", false),
                    ("Passo 2: Encontre celulas de AGUA no mapa. Instale BOMBA (T) nelas.", false),
                    ("Passo 3: Conecte TUBOS (2) da Bomba ate o Nuclear. Devem ser adjacentes.", false),
                    ("Passo 4: Instale CENTRIFUGA (B) sobre terreno de URANIO.", false),
                    ("Passo 5: Esteira leva Cel. Uranio da Centrifuga para o Nuclear.", false),
                    ("Passo 6: Instale POSTES (3) para distribuir os 2500W na rede.", false),
                    ("         Monitore o CALOR no tooltip (hover). Nunca deve passar 75%!", false),
                    ("", false),
                    ("DICAS DE EFICIENCIA", true),
                    ("  - Baterias garantem energia noturna para paineis solares.", false),
                    ("  - Uma nuclear alimenta ~40 maquinas pesadas simultaneamente.", false),
                    ("  - Nuclear + Bateria = combinacao ideal para producao 24h.", false),
                ];
                for (i, (line, is_title)) in lines.iter().enumerate() {
                    let y = cy + i as f32 * 21.0;
                    if y > hy + hh - 25.0 { break; }
                    if *is_title {
                        draw_rectangle(cx, y - 14.0, 696.0, 18.0, Color::new(0.12, 0.18, 0.28, 1.0));
                        draw_text(line, cx + 4.0, y, 15.0, title_col);
                    } else if line.starts_with("!!!") {
                        draw_text(line, cx, y, 14.0, Color::new(0.97, 0.27, 0.27, 1.0));
                    } else {
                        draw_text(line, cx, y, 14.0, text_col);
                    }
                }
            }
            _ => {}
        }
    }
}


pub struct HoverInfo { pub title: String, pub health: Option<f32>, pub wind: i32, pub fuel: Option<f32>, pub heat: Option<f32>, pub charge: Option<f32> }

pub fn get_hover_info(state: &GameState) -> Option<HoverInfo> {
    let col = state.mouse.world_col; let row = state.mouse.world_row;
    if col<0||row<0||col>=GRID_SIZE as i32||row>=GRID_SIZE as i32 { return None; }
    let idx = row as usize*GRID_SIZE+col as usize;
    let wind = (state.wind_map[idx]*100.0) as i32;
    if let Some(ref cell) = state.grid[idx] {
        Some(HoverInfo { title: cell.tool.name_pt().to_string(), health: Some(cell.health), wind,
            fuel: if matches!(cell.tool, Tool::CoalPlant|Tool::Nuclear){Some(cell.fuel)}else{None},
            heat: if cell.tool==Tool::Nuclear{Some(cell.heat)}else{None},
            charge: if cell.tool==Tool::Battery{Some(cell.charge)}else{None} })
    } else {
        Some(HoverInfo { title: state.terrain[idx].name_pt().to_string(), health:None, wind, fuel:None, heat:None, charge:None })
    }
}
