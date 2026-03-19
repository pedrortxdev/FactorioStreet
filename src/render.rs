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
        Terrain::Mountain => Color::new(0.3, 0.3, 0.35, 1.0),
    }
}

fn draw_building(wx: f32, wy: f32, ws: f32, cell: &Cell, on: bool, cam_scale: f32, time: f64, grid: &[Option<Cell>], c: i32, r: i32) {
    match cell.tool {
        Tool::ConveyorIron => {
            let belt_bg = Color::new(0.18, 0.20, 0.25, 1.0);
            let rail_col = Color::new(0.60, 0.35, 0.15, 1.0);
            let rust_dark = Color::new(0.35, 0.18, 0.08, 1.0);
            let link_col = Color::new(0.25, 0.28, 0.35, 1.0);
            
            let is_curve = cell.render_type != 0;
            let in_dir = match cell.render_type {
                1 => (cell.dir + 3) % 4, // Curve L
                2 => (cell.dir + 1) % 4, // Curve R
                _ => cell.dir,
            };

            draw_rectangle(wx, wy, ws, ws, belt_bg);

            if !is_curve {
                let rail_w = ws * 0.12;
                if cell.dir == 0 || cell.dir == 2 {
                    draw_rectangle(wx, wy, rail_w, ws, rail_col);
                    draw_rectangle(wx + ws - rail_w, wy, rail_w, ws, rail_col);
                    for i in 0..4 {
                        let sy = wy + (i as f32 * 0.25) * ws;
                        draw_rectangle(wx, sy, rail_w, ws * 0.08, rust_dark);
                        draw_rectangle(wx + ws - rail_w, sy + ws * 0.1, rail_w, ws * 0.08, rust_dark);
                    }
                } else {
                    draw_rectangle(wx, wy, ws, rail_w, rail_col);
                    draw_rectangle(wx, wy + ws - rail_w, ws, rail_w, rail_col);
                    for i in 0..4 {
                        let sx = wx + (i as f32 * 0.25) * ws;
                        draw_rectangle(sx, wy, ws * 0.08, rail_w, rust_dark);
                        draw_rectangle(sx + ws * 0.1, wy + ws - rail_w, ws * 0.08, rail_w, rust_dark);
                    }
                }

                let time_offset = (time as f32 * 0.005) % 1.0;
                let link_count = 6;
                for i in -1..link_count {
                    let progress = (i as f32 + time_offset) / link_count as f32;
                    match cell.dir {
                        0 | 2 => {
                            let y = wy + (if cell.dir == 0 { 1.0 - progress } else { progress }) * ws;
                            draw_line(wx + rail_w, y, wx + ws - rail_w, y, 1.5 * cam_scale, link_col);
                        }
                        _ => {
                            let x = wx + (if cell.dir == 1 { progress } else { 1.0 - progress }) * ws;
                            draw_line(x, wy + rail_w, x, wy + ws - rail_w, 1.5 * cam_scale, link_col);
                        }
                    }
                }
            } else {
                let rail_w = ws * 0.12;
                let time_offset = (time as f32 * 0.005) % 1.0;
                let (px, py, start_angle) = match (in_dir, cell.dir) {
                    (1, 0) | (2, 3) => (wx, wy, 0.0),
                    (3, 0) | (2, 1) => (wx + ws, wy, 90.0),
                    (1, 2) | (0, 3) => (wx, wy + ws, 270.0),
                    (3, 2) | (0, 1) => (wx + ws, wy + ws, 180.0),
                    _ => (wx, wy, 0.0),
                };

                let steps = 12;
                for i in 0..steps {
                    let a1 = (start_angle + (i as f32 / steps as f32) * 90.0).to_radians();
                    let a2 = (start_angle + ((i+1) as f32 / steps as f32) * 90.0).to_radians();
                    let r_out = ws; 
                    draw_line(px + a1.cos()*r_out, py + a1.sin()*r_out, px + a2.cos()*r_out, py + a2.sin()*r_out, 3.0*cam_scale, rail_col);
                    let r_in = rail_w;
                    draw_line(px + a1.cos()*r_in, py + a1.sin()*r_in, px + a2.cos()*r_in, py + a2.sin()*r_in, 3.0*cam_scale, rail_col);
                }

                let link_count = 6;
                for i in -1..link_count {
                    let progress = (i as f32 + time_offset) / link_count as f32;
                    let angle = (start_angle + progress * 90.0).to_radians();
                    let r_inner = rail_w; let r_outer = ws - rail_w;
                    draw_line(px + angle.cos()*r_inner, py + angle.sin()*r_inner, px + angle.cos()*r_outer, py + angle.sin()*r_outer, 1.5*cam_scale, link_col);
                }
            }
        }
        Tool::Street => {
            draw_rectangle(wx, wy, ws, ws, Color::new(0.15, 0.18, 0.22, 1.0));
            draw_line(wx, wy + ws * 0.5, wx + ws, wy + ws * 0.5, 2.0, Color::new(0.39, 0.45, 0.52, 0.6));
            draw_line(wx + ws * 0.5, wy, wx + ws * 0.5, wy + ws, 2.0, Color::new(0.39, 0.45, 0.52, 0.6));
        }
        Tool::Pipe => {
            let p_col = Color::new(0.28, 0.33, 0.42, 1.0);
            draw_rectangle(wx + ws * 0.3, wy + ws * 0.3, ws * 0.4, ws * 0.4, p_col);
            let mask = cell.render_type;
            if mask & 1 != 0 { draw_rectangle(wx + ws * 0.35, wy, ws * 0.3, ws * 0.3, p_col); }
            if mask & 2 != 0 { draw_rectangle(wx + ws * 0.7, wy + ws * 0.35, ws * 0.3, ws * 0.3, p_col); }
            if mask & 4 != 0 { draw_rectangle(wx + ws * 0.35, wy + ws * 0.7, ws * 0.3, ws * 0.3, p_col); }
            if mask & 8 != 0 { draw_rectangle(wx, wy + ws * 0.35, ws * 0.3, ws * 0.3, p_col); }
            
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
            let angle = (time / 100.0) as f32;
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
    let size = 22.0 * cam_scale;
    let color = if is_local { Color::new(0.22, 0.74, 0.97, 1.0) } else { Color::new(0.94, 0.27, 0.27, 1.0) };
    let secondary = if is_local { Color::new(0.05, 0.35, 0.60, 1.0) } else { Color::new(0.6, 0.1, 0.1, 1.0) };
    
    // Body (Main Chassis)
    draw_rectangle(x - size * 0.4, y - size * 0.2, size * 0.8, size * 0.7, secondary);
    draw_rectangle_lines(x - size * 0.4, y - size * 0.2, size * 0.8, size * 0.7, 2.0 * cam_scale, color);
    
    // Shoulder joints
    draw_circle(x - size * 0.45, y + size * 0.1, size * 0.15, color);
    draw_circle(x + size * 0.45, y + size * 0.1, size * 0.15, color);
    
    // Head unit
    let head_y = y - size * 0.45;
    draw_circle(x, head_y, size * 0.25, color);
    
    // Glowing visor/eyes
    let eye_w = size * 0.1;
    draw_rectangle(x - eye_w * 1.5, head_y - eye_w * 0.5, eye_w * 3.0, eye_w, WHITE);
    
    // Name tag with cleaner positioning
    let font_size = 13.0 * cam_scale;
    let text_w = name.len() as f32 * font_size * 0.45;
    draw_rectangle(x - text_w * 0.5 - 4.0, y - size * 1.05, text_w + 8.0, font_size + 4.0, Color::new(0.0, 0.0, 0.0, 0.6));
    draw_text(name, x - text_w * 0.5, y - size * 0.75, font_size, if is_local { Color::new(0.5, 1.0, 0.95, 1.0) } else { WHITE });
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
    let sw = screen_width();
    let sh = screen_height();
    let ws = CELL_SIZE * cam.scale;

    // Calculate visible sectors
    // We want to see tiles from (-sw/2)/scale, (-sh/2)/scale to (+sw/2)/scale, (+sh/2)/scale relative to camera center
    let half_w = (sw / 2.0) / cam.scale;
    let half_h = (sh / 2.0) / cam.scale;

    let start_gx = ((cam.gx as f32 * CELL_SIZE + cam.ox - half_w) / CELL_SIZE).floor() as i32 - 1;
    let start_gy = ((cam.gy as f32 * CELL_SIZE + cam.oy - half_h) / CELL_SIZE).floor() as i32 - 1;
    let end_gx = ((cam.gx as f32 * CELL_SIZE + cam.ox + half_w) / CELL_SIZE).ceil() as i32 + 1;
    let end_gy = ((cam.gy as f32 * CELL_SIZE + cam.oy + half_h) / CELL_SIZE).ceil() as i32 + 1;

    let (start_s, _) = GameState::world_to_sector(start_gx, start_gy);
    let (end_s, _) = GameState::world_to_sector(end_gx, end_gy);

    let cam_wx = cam.gx as f32 * CELL_SIZE + cam.ox;
    let cam_wy = cam.gy as f32 * CELL_SIZE + cam.oy;

    for sy in start_s.1..=end_s.1 {
        for sx in start_s.0..=end_s.0 {
            let handle = if let Some(&h) = state.sectors.get(&(sx, sy)) { h } else { continue; };
            let sector = &state.pool[handle];
            
            // Tight loops: only iterate tiles that are actually on screen
            let lx_start = ((start_gx - sx * SECTOR_SIZE as i32).max(0)) as usize;
            let lx_end   = ((end_gx   - sx * SECTOR_SIZE as i32 + 1).min(SECTOR_SIZE as i32)) as usize;
            let ly_start = ((start_gy - sy * SECTOR_SIZE as i32).max(0)) as usize;
            let ly_end   = ((end_gy   - sy * SECTOR_SIZE as i32 + 1).min(SECTOR_SIZE as i32)) as usize;
            let lx_end = lx_end.min(SECTOR_SIZE);
            let ly_end = ly_end.min(SECTOR_SIZE);

            for ly in ly_start..ly_end {
                for lx in lx_start..lx_end {
                    let gx = sx * SECTOR_SIZE as i32 + lx as i32;
                    let gy = sy * SECTOR_SIZE as i32 + ly as i32;

                    let rel_x = (gx - cam.gx) as f32 * CELL_SIZE - cam.ox;
                    let rel_y = (gy - cam.gy) as f32 * CELL_SIZE - cam.oy;
                    let wx = sw / 2.0 + rel_x * cam.scale;
                    let wy = sh / 2.0 + rel_y * cam.scale;

                    let idx = ly * SECTOR_SIZE + lx;

                    // 1. Draw Terrain
                    if let Some(ref texs) = state.textures {
                        let tex = match sector.terrain[idx] {
                            Terrain::Empty    => &texs.grass,
                            Terrain::Water    => &texs.water,
                            Terrain::Tree     => &texs.tree,
                            Terrain::Sand     => &texs.sand,
                            Terrain::Iron     => &texs.iron,
                            Terrain::Copper   => &texs.copper,
                            Terrain::Coal     => &texs.coal,
                            Terrain::Quartz   => &texs.quartz,
                            Terrain::Gold     => &texs.gold,
                            Terrain::Uranium  => &texs.uranium,
                            Terrain::Oil      => &texs.oil,
                            Terrain::Wasteland=> &texs.wasteland,
                            Terrain::Mountain => &texs.mountain,
                        };
                        let overlap = 0.8 * cam.scale;
                        draw_texture_ex(tex, wx - overlap, wy - overlap, WHITE, DrawTextureParams {
                            dest_size: Some(vec2(ws + overlap * 2.0, ws + overlap * 2.0)),
                            ..Default::default()
                        });
                    } else {
                        draw_rectangle(wx, wy, ws, ws, terrain_color(sector.terrain[idx], time));
                    }

                    // 2. Draw Building
                    if let Some(cell) = &sector.grid[idx] {
                        let on = state.powered.contains(&(gx, gy)) ||
                                 (cell.tool == Tool::CoalPlant && cell.fuel > 0.0) ||
                                 (cell.tool == Tool::Nuclear   && cell.fuel > 0.0);
                        draw_building(wx, wy, ws, cell, on, cam.scale, time, &sector.grid, lx as i32, ly as i32);
                    } else if gx == state.mouse.world_col && gy == state.mouse.world_row {
                        draw_rectangle_lines(wx, wy, ws, ws, 2.0, WHITE);
                    }
                }
            }
            // 2. ITEMS - Merged into sector loop for cache locality
            for item in &sector.items {
                let rel_x = (item.x - cam.gx as f32) * CELL_SIZE - cam.ox;
                let rel_y = (item.y - cam.gy as f32) * CELL_SIZE - cam.oy;
                let draw_x = rel_x * cam.scale + sw / 2.0;
                let draw_y = rel_y * cam.scale + sh / 2.0;

                if draw_x >= -20.0 && draw_x <= sw + 20.0 && draw_y >= -20.0 && draw_y <= sh + 20.0 {
                    let sz = CELL_SIZE * cam.scale * 0.35;
                    let drawn = if let Some(ref texs) = state.textures {
                        if let Some(tex) = texs.sprites.items.get(&item.item_type) {
                            draw_texture_ex(tex, draw_x - sz/2.0, draw_y - sz/2.0, WHITE, DrawTextureParams {
                                dest_size: Some(vec2(sz, sz)), ..Default::default()
                            });
                            true
                        } else { false }
                    } else { false };
                    if !drawn {
                        draw_rectangle(draw_x - sz/2.0, draw_y - sz/2.0, sz, sz, item.item_type.color());
                    }
                }
            }
        }
    }

    // --- PLAYERS ---
    for (sender, pos) in &state.other_players {
        let rel_x = (pos.x - cam.gx as f32 * CELL_SIZE) - cam.ox;
        let rel_y = (pos.y - cam.gy as f32 * CELL_SIZE) - cam.oy;
        let wx = sw/2.0 + rel_x * cam.scale;
        let wy = sh/2.0 + rel_y * cam.scale;
        draw_player(wx, wy, cam.scale, false, sender);
    }
    let local_rel_x = (state.local_player.pos.x - cam.gx as f32 * CELL_SIZE) - cam.ox;
    let local_rel_y = (state.local_player.pos.y - cam.gy as f32 * CELL_SIZE) - cam.oy;
    draw_player(sw/2.0 + local_rel_x * cam.scale, sh/2.0 + local_rel_y * cam.scale, cam.scale, true, &state.username);

    // --- NPCS ---
    for npc in &state.npcs {
        let rel_x = (npc.x - cam.gx as f32) * CELL_SIZE - cam.ox;
        let rel_y = (npc.y - cam.gy as f32) * CELL_SIZE - cam.oy;
        draw_npc(sw/2.0 + rel_x * cam.scale, sh/2.0 + rel_y * cam.scale, cam.scale, &npc.name, &npc.state);
    }

    // --- POWER LINKS ---
    for link in &state.global_power_links {
        let rel_ux = (link.u.0 - cam.gx) as f32 * CELL_SIZE + CELL_SIZE/2.0 - cam.ox;
        let rel_uy = (link.u.1 - cam.gy) as f32 * CELL_SIZE + CELL_SIZE/2.0 - cam.oy;
        let rel_vx = (link.v.0 - cam.gx) as f32 * CELL_SIZE + CELL_SIZE/2.0 - cam.ox;
        let rel_vy = (link.v.1 - cam.gy) as f32 * CELL_SIZE + CELL_SIZE/2.0 - cam.oy;
        
        let wx1 = sw/2.0 + rel_ux * cam.scale; let wy1 = sh/2.0 + rel_uy * cam.scale;
        let wx2 = sw/2.0 + rel_vx * cam.scale; let wy2 = sh/2.0 + rel_vy * cam.scale;
        
        let alpha = 0.5 + (time as f32 / 200.0).sin() * 0.3;
        draw_line(wx1, wy1, wx2, wy2, 2.0 * cam.scale, Color::new(0.22, 0.74, 0.97, alpha));
    }

    // --- NIGHT ---
    let mut opacity = 0.0f32;
    if state.time_of_day > 1800.0 { opacity = (state.time_of_day - 1800.0) / 600.0 * 0.7; }
    else if state.time_of_day < 600.0 { opacity = (600.0 - state.time_of_day) / 600.0 * 0.7; }
    if opacity > 0.0 { 
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.01, 0.02, 0.04, opacity));
    }
}

// ============ UI — 100% OPAQUE, BIG TEXT ============
pub fn render_ui(state: &GameState, hover_info: &Option<HoverInfo>, mouse_x: f32, mouse_y: f32) {
    let sw = screen_width(); let sh = screen_height();
    // Premium Color Palette - Updated to Gray for HUD
    let hud_bg = Color::new(0.25, 0.25, 0.25, 0.95); // Gray background
    let hud_bg2 = Color::new(0.35, 0.35, 0.35, 1.0); // Lighter gray for buttons
    let hud_border = Color::new(0.6, 0.6, 0.6, 0.8); // Gray border
    let text_dim = Color::new(0.9, 0.9, 0.9, 1.0);
    let accent = Color::new(0.4, 0.8, 1.0, 1.0);

    // === TOP LEFT: Inventory ===
    let px = 10.0; let py = 10.0; let pw = 260.0;
    draw_rectangle(px, py, pw, 30.0, hud_bg);
    draw_rectangle_lines(px, py, pw, 30.0, 1.0, hud_border);
    draw_text("INVENTARIO", px+8.0, py+22.0, 24.0, Color::new(0.58,0.64,0.70,1.0));
    let inventory = &state.inventory_cache;
    let inv_h = inventory.len().min(14) as f32 * 24.0 + 4.0;
    draw_rectangle(px, py+30.0, pw, inv_h, hud_bg);
    draw_rectangle_lines(px, py+30.0, pw, inv_h, 1.0, hud_border);
    for (i,(k,v)) in inventory.iter().take(14).enumerate() {
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
    draw_text(&state.ui_money_str, rx+18.0, ry+58.0, 24.0, mc);
    
    let ic = if state.income>=0 { Color::new(0.3, 0.95, 0.55, 1.0) } else { RED };
    draw_text(&state.ui_income_str, rx+12.0, ry+85.0, 18.0, ic);
    
    let hour = (state.time_of_day/100.0) as i32;
    draw_text(&format!("Horário: {:02}:00",hour), rx+12.0, ry+110.0, 18.0, text_dim);
    
    let pc = if state.power_gen>=state.power_cons { accent } else { RED };
    draw_text(&format!("Energia: {}", state.ui_power_str), rx+12.0, ry+135.0, 18.0, pc);
    
    draw_text(&state.ui_pop_str, rx+12.0, ry+160.0, 18.0, text_dim);

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
    let bx = 10.0; let by = sh - 280.0; // Moved up to accommodate larger buttons
    for (ci,name) in CATEGORY_NAMES.iter().enumerate() {
        let tx = bx+ci as f32*104.0;
        let sel = ci == state.selected_category;
        let bg = if sel { Color::new(0.4, 0.4, 0.4, 1.0) } else { hud_bg };
        draw_rectangle(tx, by, 102.0, 30.0, bg);
        draw_rectangle_lines(tx, by, 102.0, 30.0, 1.0, if sel { accent } else { hud_border });
        draw_text(name, tx+6.0, by+22.0, 16.0, if sel { accent } else { Color::new(0.8, 0.8, 0.8, 1.0) });
    }
    let tools: Vec<&Tool> = Tool::all().iter().filter(|t| t.category()==state.selected_category).collect();
    let rows_needed = (tools.len()+4)/5;
    let tw = 96.0; let th = 96.0; // 96x96 as requested
    let palette_w = 5.0 * (tw + 4.0) + 4.0;
    let palette_h = rows_needed as f32 * (th + 4.0) + 4.0;
    
    draw_rectangle(bx, by+30.0, palette_w, palette_h.min(sh - by - 40.0), hud_bg);
    draw_rectangle_lines(bx, by+30.0, palette_w, palette_h.min(sh - by - 40.0), 1.0, hud_border);
    for (ti,tool) in tools.iter().enumerate() {
        let col = ti%5; let row = ti/5;
        let tx = bx+4.0+col as f32*(tw+4.0);
        let ty = by+34.0+row as f32*(th+4.0);
        let sel = state.selected_tool == **tool;
        draw_rectangle(tx, ty, tw, th, if sel { Color::new(0.45, 0.45, 0.45, 1.0) } else { hud_bg2 });
        draw_rectangle_lines(tx, ty, tw, th, if sel { 2.0 } else { 1.0 }, if sel { accent } else { hud_border });
        
        // Redesigned HUD: Full-sized sprite (96x96 area)
        if let Some(ref texs) = state.textures {
            if let Some(sprite) = texs.sprites.hud.get(tool) {
                draw_texture_ex(sprite, tx, ty, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(tw, th)),
                    ..Default::default()
                });
            } else {
                draw_text(tool.name_pt(), tx + 4.0, ty + 30.0, 16.0, if sel { accent } else { WHITE });
            }
        }

        // Bottom overlay for Name and Price
        let overlay_h = 16.0;
        draw_rectangle(tx + 1.0, ty + th - overlay_h - 1.0, tw - 2.0, overlay_h, Color::new(0.0, 0.0, 0.0, 0.6));
        draw_text(tool.name_pt(), tx + 4.0, ty + th - 4.0, 12.0, if sel { accent } else { WHITE });
        if !matches!(tool, Tool::Eraser | Tool::Repair) {
            let cost_str = format!("${}", tool.cost());
            let cost_w = cost_str.len() as f32 * 6.5;
            draw_text(&cost_str, tx + tw - cost_w - 4.0, ty + th - 4.0, 11.0, Color::new(0.94, 0.27, 0.27, 1.0));
        }

        // Hotkey in top-left corner
        draw_rectangle(tx + 1.0, ty + 1.0, 14.0, 14.0, Color::new(0.0, 0.0, 0.0, 0.7));
        draw_text(tool.hotkey(), tx + 4.0, ty + 12.0, 13.0, Color::new(0.8, 0.8, 0.9, 1.0));

        if sel {
            draw_rectangle_lines(tx, ty, tw, th, 2.0, accent);
        }
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
                let entries: &[(&str, &str, &str)] = &[
                    ("1", "ESTEIRA",        "Transporta itens entre máquinas. Clique p/ girar."),
                    ("2", "TUBO",           "Transporta fluidos. Conecte Bomba -> Química."),
                    ("3", "POSTE",          "Distribui energia elétrica num raio de 3 células."),
                    ("4", "RUA",            "Permite caminhar sobre água. Protege de NPCs."),
                    ("5", "ARMAZÉM",        "Coleta itens das esteiras para o inventário."),
                    ("6", "REPARO",         "Clique p/ reparar máquinas (custa 50% do valor)."),
                    ("Q", "MINERADOR",      "Extrai minério. Coloque sobre: Ferro, Cobre, etc."),
                    ("T", "BOMBA",          "Extrai ÁGUA de células de água."),
                    ("E", "EXTRATOR",       "Extrai PETRÓLEO de células de óleo."),
                    ("R", "LENHADOR",       "Corta árvores e ejeta MADEIRA adjacente."),
                    ("Y", "SOLAR",          "Gera 40W de dia. Zero energia à noite."),
                    ("U", "EOLICA",         "Gera 60W * vento local (variável)."),
                    ("I", "TERMELÉTRICA",   "Gera 300W. Consome CARVÃO via esteira."),
                    ("F", "NUCLEAR",        "Gera 2500W. Precisa de ÁGUA + URÂNIO."),
                    ("G", "BATERIA",        "Armazena energia excedente para emergências."),
                    ("Z", "FUNDIÇÃO",       "Converte minério em chapas (ex: Fe-Ore -> Fe)."),
                    ("X", "PRENSA",         "Converte chapa de cobre em fio de cobre."),
                    ("C", "MONTADORA",      "Combina itens. Ex: Vidro+Fio -> Processador."),
                    ("V", "QUÍMICA",        "Água + Petróleo -> Plástico. Precisa de tubos."),
                    ("B", "CENTRÍFUGA",     "Processa URÂNIO para combustível nuclear."),
                    ("N", "QUÂNTICO",       "Produz NÚCLEO IA. Equipamento avançado."),
                    ("M", "MERCADO",        "Vende itens e gera renda. Preços variam."),
                    ("H", "CASA",           "Adiciona população. Gera $15/tick e consome 5W."),
                ];
                for (i, (key, name, desc)) in entries.iter().enumerate() {
                    let y = cy + i as f32 * line_h;
                    if y > hy + hh - 30.0 { break; }
                    let bg_alpha = if i % 2 == 0 { 0.05 } else { 0.0 };
                    draw_rectangle(hx + 10.0, y - 16.0, hw - 20.0, 20.0, Color::new(1.0, 1.0, 1.0, bg_alpha));
                    
                    draw_text(&format!("[{}]", key), cx, y, 14.0, warn_col);
                    draw_text(name, cx + 35.0, y, 15.0, title_col);
                    draw_text(desc, cx + 180.0, y, 14.0, text_col);
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
                draw_rectangle(cx, cy - 5.0, 696.0, 25.0, Color::new(0.22, 0.74, 0.97, 0.2));
                draw_text("PRODUTO", cx + 8.0, cy + 14.0, 16.0, title_col);
                draw_text("INGREDIENTES", cx + 210.0, cy + 14.0, 16.0, title_col);
                draw_text("MÁQUINA", cx + 520.0, cy + 14.0, 16.0, title_col);
                for (i, (out, ing, machine)) in recipes.iter().enumerate() {
                    let y = cy + 28.0 + i as f32 * 26.0;
                    if y > hy + hh - 30.0 { break; }
                    let bg = if i % 2 == 0 { Color::new(1.0, 1.0, 1.0, 0.05) } else { Color::new(0.0, 0.0, 0.0, 0.0) };
                    draw_rectangle(cx, y - 18.0, 696.0, 24.0, bg);
                    draw_text(out, cx + 8.0, y, 14.0, good_col);
                    draw_text(ing, cx + 210.0, y, 14.0, text_col);
                    draw_text(machine, cx + 520.0, y, 14.0, warn_col);
                }
            }
            2 => { // CADEIAS PRODUTIVAS
                let chains: &[(&str, &[&str])] = &[
                    ("CADEIA BÁSICA - FERRO",     &["• Mapa: Terreno FERRO (cinza)", "• Minerador (Q) -> Esteira (1) -> Fundição (Z)", "• Fundição produz: Chapa Ferro", "• Chapa Ferro -> Montadora (C) para fazer Aço (+Carvão)", "• Aço -> Mercado (M) para vender"]),
                    ("CADEIA ELÉTRICA",           &["• Mapa: Terreno COBRE (laranja)", "• Minerador -> Fundição -> Chapa Cobre", "• Prensa (X) transforma Chapa em Fio de Cobre", "• Fio + Vidro (areia/fundição) -> Montadora = PROCESSADOR"]),
                    ("CADEIA QUÍMICA",            &["• Instale Extrator (E) em PETRÓLEO e Bomba (T) em ÁGUA", "• Conecte ambos com TUBOS (2) à QUÍMICA (V)", "• A Química produz PLÁSTICO automaticamente", "• Plástico + Processador + Ouro -> NÚCLEO IA (Montadora)"]),
                    ("NUCLEAR (AVANÇADO)",        &["• Instale NUCLEAR (F) e conecte ÁGUA via tubos", "• CENTRÍFUGA (B) sobre URÂNIO produz Cél. Urânio", "• Leve as células via esteira até o Reator Nuclear", "• CUIDADO: Sem água o reator causará um Meltdown!"]),
                ];
                for (ci, (chain_name, steps)) in chains.iter().enumerate() {
                    let section_y = cy + ci as f32 * 115.0;
                    if section_y > hy + hh - 30.0 { break; }
                    draw_rectangle(cx, section_y - 5.0, 696.0, 20.0, Color::new(0.22, 0.74, 0.97, 0.15));
                    draw_text(chain_name, cx + 8.0, section_y + 11.0, 16.0, title_col);
                    for (si, step) in steps.iter().enumerate() {
                        let sy = section_y + 18.0 + si as f32 * 18.0;
                        draw_text(step, cx + 15.0, sy, 14.0, text_col);
                    }
                }
            }
            3 => { // ENERGIA & REATOR
                let lines: &[(&str, bool)] = &[
                    ("SISTEMA DE DISTRIBUIÇÃO", true),
                    ("Máquinas precisam de postes (3) para receber energia (raio 3).", false),
                    ("Sem energia, a máquina mostrará um símbolo de '!' vermelho.", false),
                    ("", false),
                    ("FONTES DE GERAÇÃO", true),
                    ("• SOLAR (Y): 40W. Grátis, mas só funciona das 06h às 18h.", false),
                    ("• EÓLICA (U): 60W * fator vento. Variável, mas 24h.", false),
                    ("• TERMELÉTRICA (I): 300W. Precisa de Carvão na esteira.", false),
                    ("• NUCLEAR (F): 2500W. Água + Cél. Urânio. Alta complexidade.", false),
                    ("• BATERIA (G): Armazena excessos para uso posterior.", false),
                    ("", false),
                    ("PROTOCOLO NUCLEAR", true),
                    ("!!! PERIGO: Sem água, o reator derreterá em um MELTDOWN !!!", false),
                    ("1. Instale o Reator (F) em local seco.", false),
                    ("2. Conecte Bomba de ÁGUA (T) via Tubos diretamente ao Reator.", false),
                    ("3. Abasteça com Células de Urânio vindas da Centrífuga (B).", false),
                    ("4. Use Postes para injetar os 2500W na sua rede industrial.", false),
                ];
                for (i, (line, is_title)) in lines.iter().enumerate() {
                    let y = cy + i as f32 * 22.0;
                    if y > hy + hh - 25.0 { break; }
                    if *is_title {
                        draw_rectangle(cx, y - 15.0, 696.0, 20.0, Color::new(0.22, 0.74, 0.97, 0.15));
                        draw_text(line, cx + 8.0, y, 15.0, title_col);
                    } else if line.starts_with("!!!") {
                        draw_text(line, cx, y, 14.0, Color::new(0.97, 0.27, 0.27, 1.0));
                    } else {
                        draw_text(line, cx + 15.0, y, 14.0, text_col);
                    }
                }
            }
            _ => {}
        }
    }
}


pub struct HoverInfo { pub title: String, pub health: Option<f32>, pub wind: i32, pub fuel: Option<f32>, pub heat: Option<f32>, pub charge: Option<f32> }

pub fn get_hover_info(state: &GameState) -> Option<HoverInfo> {
    let gx = state.mouse.world_col;
    let gy = state.mouse.world_row;
    
    let cell = state.get_cell_at(gx, gy);
    let terrain = state.get_terrain_at(gx, gy);
    
    let ((sx, sy), (lx, ly)) = GameState::world_to_sector(gx, gy);
    let wind = if let Some(s) = state.get_sector(sx, sy) {
        (s.wind_map[ly as usize * SECTOR_SIZE + lx as usize] * 100.0) as i32
    } else { 0 };

    if let Some(c) = cell {
        Some(HoverInfo {
            title: c.tool.name_pt().to_string(),
            health: Some(c.health),
            wind,
            fuel: if matches!(c.tool, Tool::CoalPlant | Tool::Nuclear) { Some(c.fuel) } else { None },
            heat: if c.tool == Tool::Nuclear { Some(c.heat) } else { None },
            charge: if c.tool == Tool::Battery { Some(c.charge) } else { None },
        })
    } else {
        Some(HoverInfo {
            title: terrain.name_pt().to_string(),
            health: None,
            wind,
            fuel: None,
            heat: None,
            charge: None,
        })
    }
}
