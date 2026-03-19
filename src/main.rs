mod constants;
mod types;
mod terrain;
mod simulation;
mod render;
mod network;
mod sprites;
mod persistence;

use std::collections::HashMap;
use macroquad::prelude::*;
use constants::*;
use types::*;
use simulation::*;
use render::*;

#[derive(PartialEq)]
enum AppState {
    Login,
    ServerBrowser,
    Playing,
}

struct LoginState {
    username: String,
    password: String,
    error_msg: String,
    server_url: String,
    token: Option<String>,
    servers: Vec<network::ServerInfo>,
    selected_server: usize,
    focus: usize, // 0=server_url, 1=username, 2=password
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Factorio 2 - City Builder".to_string(),
        window_width: 1280,
        window_height: 720,
        fullscreen: false,
        window_resizable: true,
        ..Default::default()
    }
}

fn handle_build(state: &mut GameState, col: i32, row: i32, is_drag: bool, force_erase: bool) {
    let terrain = state.get_terrain_at(col, row);
    let tool = if force_erase { Tool::Eraser } else { state.selected_tool };

    // 1. Check if there's already something there and handle special tools
    let existing_cell = state.get_cell_at(col, row).cloned();
    if let Some(c) = existing_cell {
        if tool == Tool::Repair && c.health < 100.0 {
            let cost = 10;
            if state.money >= cost {
                state.money -= cost;
                let mut new_cell = c.clone();
                new_cell.health = 100.0;
                state.set_cell_at(col, row, Some(new_cell));
                state.set_msg("Reparado!");
            }
        } else if tool == Tool::Eraser {
            let refund = c.tool.refund();
            state.money += refund;
            state.set_cell_at(col, row, None);
            // Decrement pre-aggregated counters
            state.total_upkeep -= c.tool.upkeep();
            if c.tool == Tool::House { state.total_pop -= 1; }
            // Remove from machine registry
            if let Some(idx) = state.registry.indices.remove(&(col, row)) {
                state.registry.machines.swap_remove(idx);
                if idx < state.registry.machines.len() {
                    let moved_pos = state.registry.machines[idx].pos;
                    state.registry.indices.insert(moved_pos, idx);
                }
            }
            // Remove from fluid registry if applicable
            if matches!(c.tool, Tool::Pipe | Tool::Pump | Tool::Pumpjack) {
                if let Some(idx) = state.fluid_indices.remove(&(col, row)) {
                    state.fluid_nodes.swap_remove(idx);
                    if idx < state.fluid_nodes.len() {
                        // Fix the displaced node's index
                        let moved = state.fluid_nodes[idx];
                        state.fluid_indices.insert(moved, idx);
                    }
                }
            }
            state.set_msg(&format!("Vendido: ${}", refund));
            // Mark power dirty only if this was a power-relevant tool
            if c.tool.power_gen() > 0.0 || c.tool.power_cons() > 0.0 || matches!(c.tool, Tool::Node | Tool::Battery) {
                state.power_dirty = true;
                state.power_dsu.clear(); // Must rebuild DSU on any erase
            }
        } else if !is_drag && tool == Tool::ConveyorIron && c.tool == Tool::ConveyorIron {
            let mut new_cell = c.clone();
            new_cell.dir = (new_cell.dir + 1) % 4;
            state.set_cell_at(col, row, Some(new_cell));
        }
        return;
    }

    if tool == Tool::Eraser { return; } // Nothing to erase

    // 2. Validate terrain and tool constraints
    if terrain == Terrain::Wasteland { state.set_msg("Terreno Irradiado!"); return; }
    if terrain == Terrain::Water && !matches!(tool, Tool::Pump | Tool::Pipe | Tool::ConveyorIron | Tool::Node | Tool::Street) { return; }
    if terrain == Terrain::Oil && !matches!(tool, Tool::Pumpjack | Tool::Pipe | Tool::ConveyorIron | Tool::Node | Tool::Street) { return; }
    if tool == Tool::Miner && !matches!(terrain, Terrain::Iron | Terrain::Copper | Terrain::Coal | Terrain::Quartz | Terrain::Sand | Terrain::Gold | Terrain::Uranium) { return; }
    if tool == Tool::Lumberjack && terrain != Terrain::Tree { return; }

    // 3. Handle costs
    let cost = tool.cost();
    if state.money < cost { return; }
    state.money -= cost;

    // 4. Determine direction for conveyors
    let mut dir: u8 = 0;
    if tool == Tool::ConveyorIron && is_drag && state.mouse.prev_col != -1 {
        let dx = col - state.mouse.prev_col;
        let dy = row - state.mouse.prev_row;
        if dx.abs() > dy.abs() { dir = if dx > 0 { 1 } else { 3 }; }
        else if dy != 0 { dir = if dy > 0 { 2 } else { 0 }; }
    }

    // 5. Place it
    let cell = Cell::new(tool, dir);
    state.set_cell_at(col, row, Some(cell));
    
    // Increment pre-aggregated counters
    state.total_upkeep += tool.upkeep();
    if tool == Tool::House { state.total_pop += 1; }
    
    // Register machine in shadow table for offline simulation
    let shadow = MachineShadow {
        tool, pos: (col, row),
        power_gen: tool.power_gen(),
        power_cons: tool.power_cons(),
        fuel: 0.0,
    };
    state.registry.machines.push(shadow);
    state.registry.indices.insert((col, row), state.registry.machines.len() - 1);

    // Register in fluid node list if applicable
    if matches!(tool, Tool::Pipe | Tool::Pump | Tool::Pumpjack) {
        state.fluid_indices.insert((col, row), state.fluid_nodes.len());
        state.fluid_nodes.push((col, row));
    }

    // Power: mark dirty OR do incremental DSU union
    if tool.power_gen() > 0.0 || tool.power_cons() > 0.0 || matches!(tool, Tool::Node | Tool::Battery) {
        let generates = tool.power_gen() > 0.0;
        state.power_dsu.make((col, row), generates);
        // Try to union with adjacent power nodes (inline neighbor scan)
        let radius: i32 = if tool == Tool::Node { 3 } else { 1 };
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx == 0 && dy == 0 { continue; }
                let neighbor = (col + dx, row + dy);
                if state.power_dsu.parent.contains_key(&neighbor) {
                    state.power_dsu.union((col, row), neighbor);
                }
            }
        }
        // Still mark dirty so powered HashSet updates at next economy tick
        state.power_dirty = true;
    }
    
    // 6. Update neighbor render types
    update_render_type(state, col, row);
    for &(dx, dy) in DIRS.iter() { update_render_type(state, col + dx, row + dy); }
    refresh_ui_cache(state);
}

fn update_render_type(state: &mut GameState, gx: i32, gy: i32) {
    let mut cell = match state.get_cell_at(gx, gy).cloned() { Some(c) => c, None => return };
    
    if cell.tool == Tool::Pipe {
        let mut mask = 0u8;
        for d in 0..4 {
            let (dx, dy) = DIRS[d];
            if let Some(other) = state.get_cell_at(gx + dx, gy + dy) {
                if other.tool.has_pipes() {
                    mask |= 1 << d;
                }
            }
        }
        cell.render_type = mask;
        state.set_cell_at(gx, gy, Some(cell));
        return;
    }

    if cell.tool != Tool::ConveyorIron { return; }
    
    let mut neighbors = [false; 4];
    for d in 0..4 {
        let (dx, dy) = DIRS[d];
        if let Some(other) = state.get_cell_at(gx + dx, gy + dy) {
            if other.tool == Tool::ConveyorIron {
                let d_back = (d + 2) % 4;
                if other.dir == d_back as u8 { neighbors[d] = true; }
            }
        }
    }
    
    let d = cell.dir as usize;
    let l = (d + 3) % 4;
    let r = (d + 1) % 4;
    
    if neighbors[l] && !neighbors[r] { cell.render_type = 1; } // Curve L
    else if neighbors[r] && !neighbors[l] { cell.render_type = 2; } // Curve R
    else { cell.render_type = 0; } // Straight
    
    state.set_cell_at(gx, gy, Some(cell));
}

fn refresh_ui_cache(state: &mut GameState) {
    let mut sorted: Vec<(ItemType, i32)> = state.inventory.iter().map(|(&k, &v)| (k, v)).collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    state.inventory_cache = sorted;
    state.ui_money_str = format!("Saldo: ${}", state.money);
    state.ui_income_str = format!("Renda: ${}/tick", state.income);
    state.ui_power_str = format!("{}W / {}W", state.power_gen as i32, state.power_cons as i32);
    state.ui_pop_str = format!("Pop: {} (Crise: {})", state.pop, state.unpowered);
}

fn generate_resources(forge: &crate::types::crystal_forge::Generator, time: f32) -> Resources {
    Resources {
        grass: forge.generate_texture(Terrain::Empty, time),
        water: forge.generate_texture(Terrain::Water, time),
        tree: forge.generate_texture(Terrain::Tree, time),
        sand: forge.generate_texture(Terrain::Sand, time),
        iron: forge.generate_texture(Terrain::Iron, time),
        copper: forge.generate_texture(Terrain::Copper, time),
        coal: forge.generate_texture(Terrain::Coal, time),
        quartz: forge.generate_texture(Terrain::Quartz, time),
        gold: forge.generate_texture(Terrain::Gold, time),
        oil: forge.generate_texture(Terrain::Oil, time),
        uranium: forge.generate_texture(Terrain::Uranium, time),
        wasteland: forge.generate_texture(Terrain::Wasteland, time),
        mountain: forge.generate_texture(Terrain::Mountain, time),
        transitions: HashMap::new(),
        sprites: sprites::SpriteBank::load(),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app_state = AppState::Login;
    let mut login = LoginState {
        username: String::new(),
        password: String::new(),
        error_msg: String::new(),
        server_url: "http://151.243.24.240:8081".to_string(),
        token: None,
        servers: Vec::new(),
        selected_server: 0,
        focus: 1,
    };

    let mut state = GameState::new();
    state.textures = Some(generate_resources(&state.forge, 0.0));
    terrain::generate_terrain(&mut state);
    
    let mut last_time = get_time();
    let mut net_tx: Option<std::sync::mpsc::Sender<network::NetCmd>> = None;
    let mut net_rx: Option<std::sync::mpsc::Receiver<network::NetEvent>> = None;
    let mut online_players: Vec<String> = Vec::new();

    loop {
        let (mx, my) = mouse_position();
        let sw = screen_width();
        let sh = screen_height();

        // Animate procedural textures
        if let Some(texs) = &mut state.textures {
            let t = get_time() as f32 * 1000.0; // Pass ms to match JS logic
            texs.water = state.forge.generate_texture(Terrain::Water, t);
            texs.uranium = state.forge.generate_texture(Terrain::Uranium, t);
        }

        match app_state {
            AppState::Login => {
                clear_background(Color::new(0.03, 0.05, 0.08, 1.0));
                let bw = 440.0; let bh = 380.0;
                let bx = (sw - bw) / 2.0; let by = (sh - bh) / 2.0;

                draw_rectangle(bx, by, bw, bh, Color::new(0.06, 0.09, 0.14, 1.0));
                draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::new(0.22, 0.74, 0.97, 1.0));

                draw_text("FACTORIO 2", bx + 110.0, by + 45.0, 36.0, Color::new(0.22, 0.74, 0.97, 1.0));
                draw_text("LOGIN / REGISTRO", bx + 125.0, by + 70.0, 18.0, Color::new(0.58, 0.64, 0.70, 1.0));

                let fw = bw - 40.0; let fh = 32.0;
                let focus_col = Color::new(0.22, 0.74, 0.97, 1.0);
                let normal_col = Color::new(0.28, 0.33, 0.42, 1.0);
                let blink = (get_time() * 3.0) as i32 % 2 == 0;

                // Fields
                let fields = [("SERVIDOR:", &login.server_url), ("USUARIO:", &login.username), ("SENHA:", &login.password)];
                for (i, (label, val)) in fields.iter().enumerate() {
                    let fy = by + 108.0 + i as f32 * 60.0;
                    draw_text(label, bx + 20.0, fy - 4.0, 16.0, Color::new(0.58, 0.64, 0.70, 1.0));
                    draw_rectangle(bx + 20.0, fy, fw, fh, Color::new(0.02, 0.03, 0.06, 1.0));
                    draw_rectangle_lines(bx + 20.0, fy, fw, fh, if login.focus == i { 2.0 } else { 1.0 }, if login.focus == i { focus_col } else { normal_col });
                    let display = if i == 2 { "*".repeat(val.len()) } else { (*val).clone() };
                    draw_text(&display, bx + 28.0, fy + 22.0, 16.0, WHITE);
                    if login.focus == i && blink { draw_text("|", bx + 28.0 + display.len() as f32 * 8.2, fy + 22.0, 16.0, focus_col); }
                }

                // Buttons
                let btn_w = (bw - 50.0) / 3.0;
                let btn_y = by + 280.0;
                draw_rectangle(bx + 20.0, btn_y, btn_w, 36.0, Color::new(0.08, 0.52, 0.20, 1.0));
                draw_text("ENTRAR", bx + 32.0, btn_y + 24.0, 18.0, WHITE);
                draw_rectangle(bx + 25.0 + btn_w, btn_y, btn_w, 36.0, Color::new(0.22, 0.50, 0.74, 1.0));
                draw_text("CRIAR", bx + 37.0 + btn_w, btn_y + 24.0, 18.0, WHITE);
                draw_rectangle(bx + 30.0 + btn_w * 2.0, btn_y, btn_w, 36.0, Color::new(0.39, 0.45, 0.52, 1.0));
                draw_text("OFFLINE", bx + 40.0 + btn_w * 2.0, btn_y + 24.0, 16.0, WHITE);

                if !login.error_msg.is_empty() { draw_text(&login.error_msg, bx + 20.0, btn_y + 76.0, 15.0, RED); }

                if is_mouse_button_pressed(MouseButton::Left) {
                    if mx >= bx + 20.0 && mx <= bx + 20.0 + fw {
                        for i in 0..3 {
                            let fy = by + 108.0 + i as f32 * 60.0;
                            if my >= fy && my <= fy + fh { login.focus = i; }
                        }
                    }
                    if my >= btn_y && my <= btn_y + 36.0 {
                        if mx >= bx + 20.0 && mx <= bx + 20.0 + btn_w {
                            match network::login(&login.server_url, &login.username, &login.password) {
                                Ok((token, _)) => { login.token = Some(token); login.servers = network::fetch_servers(&login.server_url); app_state = AppState::ServerBrowser; }
                                Err(e) => login.error_msg = e,
                            }
                        }
                        if mx >= bx + 25.0 + btn_w && mx <= bx + 25.0 + btn_w * 2.0 {
                            match network::register(&login.server_url, &login.username, &login.password) {
                                Ok(()) => login.error_msg = "Conta criada! Clique em Entrar.".to_string(),
                                Err(e) => login.error_msg = e,
                            }
                        }
                        if mx >= bx + 30.0 + btn_w * 2.0 && mx <= bx + 30.0 + btn_w * 3.0 { app_state = AppState::Playing; }
                    }
                }

                if let Some(ch) = get_char_pressed() {
                    if ch == '\t' { login.focus = (login.focus + 1) % 3; }
                    else if ch == '\u{8}' { match login.focus { 0 => { login.server_url.pop(); } 1 => { login.username.pop(); } 2 => { login.password.pop(); } _ => {} } }
                    else if ch.is_ascii() && !ch.is_control() { match login.focus { 0 => login.server_url.push(ch), 1 => login.username.push(ch), 2 => login.password.push(ch), _ => {} } }
                }
                next_frame().await; continue;
            }

            AppState::ServerBrowser => {
                clear_background(Color::new(0.03, 0.05, 0.08, 1.0));
                let bw = 500.0; let bh = 400.0;
                let bx = (sw - bw) / 2.0; let by = (sh - bh) / 2.0;

                draw_rectangle(bx, by, bw, bh, Color::new(0.06, 0.09, 0.14, 1.0));
                draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::new(0.22, 0.74, 0.97, 1.0));
                draw_text("SERVIDORES", bx + 170.0, by + 35.0, 28.0, Color::new(0.22, 0.74, 0.97, 1.0));

                for (i, srv) in login.servers.iter().enumerate() {
                    let sy = by + 60.0 + i as f32 * 54.0;
                    let selected = i == login.selected_server;
                    draw_rectangle(bx + 15.0, sy, bw - 30.0, 48.0, if selected { Color::new(0.14, 0.18, 0.26, 1.0) } else { Color::new(0.08, 0.10, 0.16, 1.0) });
                    draw_text(&srv.name, bx + 25.0, sy + 22.0, 20.0, WHITE);
                    if is_mouse_button_pressed(MouseButton::Left) && mx >= bx + 15.0 && mx <= bx + bw - 15.0 && my >= sy && my <= sy + 48.0 { login.selected_server = i; }
                }

                let cby = by + bh - 50.0;
                draw_rectangle(bx + 15.0, cby, bw - 30.0, 36.0, Color::new(0.08, 0.52, 0.30, 1.0));
                draw_text("CONECTAR", bx + 200.0, cby + 24.0, 20.0, WHITE);

                if is_mouse_button_pressed(MouseButton::Left) && my >= cby && my <= cby + 36.0 && mx >= bx + 15.0 && mx <= bx + bw - 15.0 {
                    if let Some(ref token) = login.token {
                        let region = login.servers[login.selected_server].region.clone();
                        match network::connect_ws(&login.server_url, token, &region) {
                            Ok((tx, rx)) => { net_tx = Some(tx); net_rx = Some(rx); app_state = AppState::Playing; state.player_id = login.username.clone(); state.username = login.username.clone(); }
                            Err(e) => login.error_msg = e,
                        }
                    }
                }
                if is_key_pressed(KeyCode::Escape) { app_state = AppState::Login; }
                next_frame().await; continue;
            }

            AppState::Playing => {
                let time = get_time();
                let dt = (time - last_time) as f32 * 1000.0;
                last_time = time;

                // --- PLAYER MOVEMENT ---
                let mut move_vec = vec2(0.0, 0.0);
                if is_key_down(KeyCode::W) { move_vec.y -= 1.0; }
                if is_key_down(KeyCode::S) { move_vec.y += 1.0; }
                if is_key_down(KeyCode::A) { move_vec.x -= 1.0; }
                if is_key_down(KeyCode::D) { move_vec.x += 1.0; }
                if move_vec.length() > 0.0 { move_vec = move_vec.normalize(); }
                
                state.local_player.vel = state.local_player.vel * 0.82 + move_vec * 1.2;
                let next_pos = state.local_player.pos + state.local_player.vel * (dt / 16.0);
                
                let mut can_move = true;
                let col = (next_pos.x / CELL_SIZE) as i32;
                let row = (next_pos.y / CELL_SIZE) as i32;
                
                let terrain = state.get_terrain_at(col, row);
                if terrain == Terrain::Water && state.get_cell_at(col, row).map_or(true, |c| c.tool != Tool::Street) { 
                    can_move = false; 
                }
                if let Some(c) = state.get_cell_at(col, row) { 
                    if !matches!(c.tool, Tool::Street | Tool::ConveyorIron | Tool::Pipe | Tool::Node) { 
                        can_move = false; 
                    } 
                }

                if can_move { state.local_player.pos = next_pos; } else { state.local_player.vel = vec2(0.0, 0.0); }

                // --- CAMERA FOLLOW ---
                // Continuous target in world space
                let tx = state.local_player.pos.x; 
                let ty = state.local_player.pos.y;
                
                // Current camera world position
                let mut current_world_x = state.camera.gx as f32 * CELL_SIZE + state.camera.ox;
                let mut current_world_y = state.camera.gy as f32 * CELL_SIZE + state.camera.oy;
                
                current_world_x += (tx - current_world_x) * 0.1;
                current_world_y += (ty - current_world_y) * 0.1;
                
                state.camera.gx = (current_world_x / CELL_SIZE).floor() as i32;
                state.camera.gy = (current_world_y / CELL_SIZE).floor() as i32;
                state.camera.ox = current_world_x.rem_euclid(CELL_SIZE);
                state.camera.oy = current_world_y.rem_euclid(CELL_SIZE);

                // --- ZOOM ---
                let (_, scroll_y) = mouse_wheel();
                if scroll_y != 0.0 {
                    let zoom_speed = 1.1f32;
                    let factor = if scroll_y > 0.0 { zoom_speed } else { 1.0 / zoom_speed };
                    state.camera.scale = (state.camera.scale * factor).clamp(0.15, 5.0);
                }

                let (mx, my) = mouse_position();
                let (wc, _, wr, _) = state.camera.screen_to_world(mx, my, sw, sh);
                state.mouse.world_col = wc;
                state.mouse.world_row = wr;

                // --- TOOLS ---
                if let Some(key) = get_last_key_pressed() {
                    if let Some(tool) = Tool::from_key(key) {
                        state.selected_tool = tool;
                        state.selected_category = tool.category();
                    }
                    if key == KeyCode::F5 {
                        match persistence::save(&state) {
                            Ok(_) => state.set_msg("Jogo Salvo! (save.bin)"),
                            Err(e) => state.set_msg(&format!("Erro ao Salvar: {}", e)),
                        }
                    }
                    if key == KeyCode::F9 {
                        match persistence::load() {
                            Ok(mut new_state) => {
                                // Restore transient fields
                                new_state.textures = Some(generate_resources(&new_state.forge, 0.0));
                                new_state.forge = crate::types::crystal_forge::Generator::new(new_state.seed as f32);
                                state = new_state;
                                refresh_ui_cache(&mut state);
                                state.set_msg("Jogo Carregado!");
                            }
                            Err(e) => state.set_msg(&format!("Erro ao Carregar: {}", e)),
                        }
                    }
                }

                let (wc, _, wr, _) = state.camera.screen_to_world(mx, my, sw, sh);
                state.mouse.world_col = wc;
                state.mouse.world_row = wr;

                if is_mouse_button_pressed(MouseButton::Left) {
                    let mut ui_click = false;

                    // --- ENCYCLOPEDIA PRIORITY ---
                    if state.help_open {
                        let hw = 720.0; let hh = 560.0;
                        let hx = (sw - hw) / 2.0; let hy = (sh - hh) / 2.0;
                        
                        // Check tabs
                        if my >= hy + 40.0 && my <= hy + 68.0 {
                            for i in 0..4 {
                                let tx = hx + 10.0 + i as f32 * 180.0;
                                if mx >= tx && mx <= tx + 175.0 {
                                    state.help_tab = i;
                                    ui_click = true;
                                    break;
                                }
                            }
                        }
                        
                        // Check close area (top right label)
                        if !ui_click && mx >= hx + hw - 170.0 && mx <= hx + hw && my >= hy && my <= hy + 40.0 {
                            state.help_open = false;
                            ui_click = true;
                        }

                        // Consume all clicks inside the encyclopedia box
                        if !ui_click && mx >= hx && mx <= hx + hw && my >= hy && my <= hy + hh {
                            ui_click = true;
                        }
                    }

                    // --- OTHER UI ELEMENTS ---
                    if !ui_click {
                        let bx = 10.0; let by = sh - 280.0; // Updated to sh - 280.0
                        for ci in 0..CATEGORY_NAMES.len() { if mx >= bx + ci as f32 * 104.0 && mx <= bx + ci as f32 * 104.0 + 102.0 && my >= by && my <= by + 30.0 { state.selected_category = ci; ui_click = true; } }
                        
                        if !ui_click {
                            let tools: Vec<Tool> = Tool::all().iter().filter(|t| t.category() == state.selected_category).copied().collect();
                            for (ti, tool) in tools.iter().enumerate() {
                                // Updated to 96x96 and 100 vertical spacing
                                let tx = bx + 4.0 + (ti % 5) as f32 * 100.0; 
                                let ty = by + 34.0 + (ti / 5) as f32 * 100.0;
                                if mx >= tx && mx <= tx + 96.0 && my >= ty && my <= ty + 96.0 { state.selected_tool = *tool; ui_click = true; }
                            }
                        }

                        if !ui_click {
                            let rw = 340.0; let rx = sw - rw - 10.0; let ry = 10.0;
                            let cy = ry + 205.0; // Matches cy in render.rs
                            if my >= cy && my <= cy + 30.0 {
                                if mx >= rx && mx <= rx + rw / 2.0 - 3.0 {
                                    if let Some(ref tx) = net_tx {
                                        // TODO: Implement sector-based sync. For now, send empty grid.
                                        let _ = tx.send(network::NetCmd::SendState { 
                                            money: state.money, 
                                            inventory: state.inventory.iter().map(|(k, v): (&ItemType, &i32)| (k.key().to_string(), *v)).collect(), 
                                            production_rate: state.income as f64, 
                                            x: state.local_player.pos.x, 
                                            y: state.local_player.pos.y, 
                                            grid: Some("[]".to_string()),
                                            seed: state.seed,
                                        });
                                        state.set_msg("Salvo no Servidor!");
                                    }
                                }
                                if mx >= rx + rw / 2.0 + 3.0 && mx <= rx + rw { state.help_open = !state.help_open; }
                                ui_click = true;
                            }
                        }
                    }

                    if !ui_click {
                        let is_eraser = state.selected_tool == Tool::Eraser;
                        if is_eraser { state.mouse.is_erasing = true; } else { state.mouse.is_building = true; }
                        let (wc, wr) = (state.mouse.world_col, state.mouse.world_row);
                        handle_build(&mut state, wc, wr, false, is_eraser);
                    }
                }
                
                let (wc, wr) = (state.mouse.world_col, state.mouse.world_row);
                if state.mouse.is_building && is_mouse_button_down(MouseButton::Left) { handle_build(&mut state, wc, wr, true, false); }
                if state.mouse.is_erasing && is_mouse_button_down(MouseButton::Left) { handle_build(&mut state, wc, wr, true, true); }
                if is_mouse_button_released(MouseButton::Left) { state.mouse.is_building = false; state.mouse.is_erasing = false; }
                state.mouse.prev_col = state.mouse.world_col; state.mouse.prev_row = state.mouse.world_row;

                if time - state.last_econ_tick > 3.0 { state.last_econ_tick = time; tick_economy(&mut state); refresh_ui_cache(&mut state); }
                if time - state.last_market_tick > 6.0 { state.last_market_tick = time; tick_market(&mut state); tick_npcs(&mut state); refresh_ui_cache(&mut state); }
                if time - state.last_industry_tick > 1.5 { state.last_industry_tick = time; tick_industry(&mut state); }
                if time - state.last_fluid_tick > 0.1 { state.last_fluid_tick = time; tick_fluids(&mut state); }
                tick_conveyors(&mut state, dt);

                if let Some(rx) = &mut net_rx {
                    while let Ok(evt) = rx.try_recv() {
                        match evt {
                            network::NetEvent::Connected { money, inventory, grid, x, y, seed, .. } => {
                                state.money = money;
                                state.seed = seed;
                                state.forge = crate::types::crystal_forge::Generator::new(seed as f32);
                                state.textures = Some(generate_resources(&state.forge, 0.0));
                                crate::terrain::generate_terrain(&mut state);
                                for (k, v) in inventory { if let Some(it) = ItemType::from_key(&k) { state.inventory.insert(it, v); } }
                                    if let Some(g) = grid { 
                                        // TODO: Implement sector-based grid sync
                                        /*if let Ok(ng) = serde_json::from_str::<Vec<Option<Cell>>>(&g) { 
                                            // state.grid = ng; 
                                        } */
                                    }
                                    if x > 0.0 || y > 0.0 { state.local_player.pos = vec2(x, y); }
                                }
                                network::NetEvent::CursorFrom { sender, x, y } => { state.other_players.insert(sender, vec2(x, y)); }
                                network::NetEvent::PlayerList(names) => { online_players = names; }
                                network::NetEvent::NpcUpdate(npcs) => { state.npcs = npcs; }
                                network::NetEvent::StateSync { sender: _, money, inventory, grid, x: _, y: _, .. } => {
                                    state.money = money;
                                    for (k, v) in inventory { if let Some(it) = ItemType::from_key(&k) { state.inventory.insert(it, v); } }
                                    if let Some(g) = grid { 
                                        // TODO: Sector sync
                                        /*if let Ok(ng) = serde_json::from_str::<Vec<Option<Cell>>>(&g) { 
                                            // state.grid = ng; 
                                        }*/
                                    }
                                }
                            _ => {}
                        }
                    }
                }

                if let Some(tx) = &net_tx {
                    if time - state.last_cursor_sync > 0.05 {
                        let _ = tx.send(network::NetCmd::SendCursor { x: state.local_player.pos.x, y: state.local_player.pos.y });
                        state.last_cursor_sync = time;
                        if time - state.last_cursor_fetch > 10.0 || (state.grid_dirty && time - state.last_sync_time > 0.2) {
                            let grid_to_send = if state.grid_dirty { Some("[]".to_string()) } else { None };
                            let _ = tx.send(network::NetCmd::SendState { 
                                money: state.money, 
                                inventory: state.inventory.iter().map(|(k, v): (&ItemType, &i32)| (k.key().to_string(), *v)).collect(), 
                                production_rate: state.income as f64, 
                                x: state.local_player.pos.x, 
                                y: state.local_player.pos.y, 
                                grid: grid_to_send,
                                seed: state.seed,
                            });
                            state.last_cursor_fetch = time;
                            state.last_sync_time = time;
                            state.grid_dirty = false;
                        }
                    }
                }

                render_game(&mut state, get_time() * 1000.0);
                let hover = get_hover_info(&state);
                render_ui(&state, &hover, mx, my);
                
                if !online_players.is_empty() {
                    let oy = sh - 250.0 - online_players.len() as f32 * 18.0;
                    draw_rectangle(10.0, oy, 160.0, 20.0 + online_players.len() as f32 * 18.0, Color::new(0.04, 0.06, 0.10, 1.0));
                    draw_text("ONLINE", 16.0, oy + 16.0, 14.0, GREEN);
                    for (i, name) in online_players.iter().enumerate() { draw_text(name, 16.0, oy + 34.0 + i as f32 * 18.0, 14.0, WHITE); }
                }
                next_frame().await;
            }
        }
    }
}
