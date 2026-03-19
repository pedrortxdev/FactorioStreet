use std::collections::{HashMap, HashSet, VecDeque};
use crate::constants::*;
use crate::types::*;
use rand::Rng;

pub fn recalculate_power(state: &mut GameState) {
    // Dirty flag — only rebuild when topology changes
    if !state.power_dirty { return; }
    state.power_dirty = false;

    let mut power_blocks = Vec::new(); // Stores (gx, gy)
    
    // Use registry for national grid power nodes
    for shadow in &state.registry.machines {
        let pos = shadow.pos;
        if shadow.tool.power_gen() > 0.0 || shadow.tool.power_cons() > 0.0 || shadow.tool == Tool::Node || shadow.tool == Tool::Battery {
            power_blocks.push(pos);
        }
    }

    let mut adj: HashMap<(i32, i32), Vec<(i32, i32)>> = HashMap::new();
    for &pos in &power_blocks { adj.insert(pos, Vec::new()); }
    let mut processed_links = HashSet::new();

    for &u_pos in &power_blocks {
        let cell_u = state.get_cell_at(u_pos.0, u_pos.1).unwrap();
        let radius = if cell_u.tool == Tool::Node { 3 } else { 1 };
        
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx == 0 && dy == 0 { continue; }
                let v_pos = (u_pos.0 + dx, u_pos.1 + dy);
                if let Some(cell_v) = state.get_cell_at(v_pos.0, v_pos.1) {
                    if cell_v.construction_progress < 100.0 || cell_v.health <= 0.0 { continue; }
                    if !(cell_v.tool.power_gen() > 0.0 || cell_v.tool.power_cons() > 0.0 || cell_v.tool == Tool::Node || cell_v.tool == Tool::Battery) {
                        continue;
                    }

                    let dist = dx.abs().max(dy.abs());
                    let mut connected = false;
                    if dist == 1 && (dx == 0 || dy == 0) { connected = true; }
                    else if dist <= 3 && (cell_u.tool == Tool::Node || cell_v.tool == Tool::Node) { connected = true; }
                    
                    if connected {
                        adj.entry(u_pos).or_default().push(v_pos);
                        adj.entry(v_pos).or_default().push(u_pos);
                        
                        let (min_uv, max_uv) = if u_pos < v_pos { (u_pos, v_pos) } else { (v_pos, u_pos) };
                        processed_links.insert((min_uv, max_uv));
                    }
                }
            }
        }
    }

    let mut visited = HashSet::new();
    let mut new_powered = HashSet::new(); 
    
    let mut sunlight = 1.0f32;
    if state.time_of_day < 600.0 || state.time_of_day > 1800.0 { sunlight = 0.0; }
    else if state.time_of_day < 800.0 { sunlight = (state.time_of_day - 600.0) / 200.0; }
    else if state.time_of_day > 1600.0 { sunlight = (1800.0 - state.time_of_day) / 200.0; }

    for &start_pos in &power_blocks {
        if visited.contains(&start_pos) { continue; }
        
        let mut comp = Vec::new();
        let mut gen = 0.0f32;
        let mut cons = 0.0f32;
        let mut bat_positions = Vec::new();
        let mut q = VecDeque::new();
        
        q.push_back(start_pos); visited.insert(start_pos);

        while let Some(curr_pos) = q.pop_front() {
            comp.push(curr_pos);
            let cell = state.get_cell_at(curr_pos.0, curr_pos.1).unwrap();
            match cell.tool {
                Tool::Solar => gen += Tool::Solar.power_gen() * sunlight,
                Tool::Wind => {
                    let ((sx, sy), (lx, ly)) = GameState::world_to_sector(curr_pos.0, curr_pos.1);
                    if let Some(s) = state.get_sector(sx, sy) {
                        gen += Tool::Wind.power_gen() * s.wind_map[ly as usize * SECTOR_SIZE + lx as usize];
                    }
                }
                Tool::CoalPlant => {
                    // We need fuel from the real cell if loaded, else from shadow
                    let fuel = state.get_cell_at(curr_pos.0, curr_pos.1).map(|c| c.fuel).unwrap_or(0.0);
                    if fuel > 0.0 { gen += Tool::CoalPlant.power_gen(); }
                }
                Tool::Nuclear => {
                    let fuel = state.get_cell_at(curr_pos.0, curr_pos.1).map(|c| c.fuel).unwrap_or(0.0);
                    if fuel > 0.0 { gen += Tool::Nuclear.power_gen(); }
                }
                Tool::Battery => { bat_positions.push(curr_pos); }
                _ => {
                    gen += cell.tool.power_gen();
                }
            }
            cons += cell.tool.power_cons();
            
            if let Some(neighbors) = adj.get(&curr_pos) {
                for &v_pos in neighbors {
                    if !visited.contains(&v_pos) {
                        visited.insert(v_pos);
                        q.push_back(v_pos);
                    }
                }
            }
        }

        let available = gen;
        let mut total_stored = 0.0f32;
        let mut total_cap = 0.0f32;
        
        for &bat_pos in &bat_positions {
            if let Some(cell) = state.get_cell_at(bat_pos.0, bat_pos.1) {
                total_stored += cell.charge;
                total_cap += BATTERY_CAP;
            }
        }
        
        let mut is_powered = false;
        if available >= cons {
            is_powered = true;
            let surplus = available - cons;
            if total_cap > 0.0 {
                let charge_per_bat = surplus / bat_positions.len() as f32;
                for &bat_pos in &bat_positions {
                    let mut cell = state.get_cell_at(bat_pos.0, bat_pos.1).unwrap().clone();
                    cell.charge = (cell.charge + charge_per_bat * 0.016).min(BATTERY_CAP);
                    state.set_cell_at(bat_pos.0, bat_pos.1, Some(cell));
                }
            }
        } else if available + total_stored >= cons {
            is_powered = true;
            let deficit = cons - available;
            if total_stored > 0.0 {
                let draw_per_bat = deficit / bat_positions.len() as f32;
                for &bat_pos in &bat_positions {
                    let mut cell = state.get_cell_at(bat_pos.0, bat_pos.1).unwrap().clone();
                    cell.charge = (cell.charge - draw_per_bat * 0.016).max(0.0);
                    state.set_cell_at(bat_pos.0, bat_pos.1, Some(cell));
                }
            }
        }
        
        if is_powered {
            for pos in comp {
                new_powered.insert(pos);
            }
        }
    }
    
    state.powered = new_powered;
    state.global_power_links = processed_links.into_iter().map(|(u, v)| PowerLink { u, v }).collect();
}

pub fn tick_economy(state: &mut GameState) {
    state.time_of_day += 50.0;
    if state.time_of_day >= 2400.0 { state.time_of_day -= 2400.0; }

    // O(1): use pre-aggregated counters maintained at build/erase time
    // upkeep_penalty is a temporary drain from Sabotage attacks
    let earnings = state.total_upkeep - state.upkeep_penalty + state.pending_sales;
    state.money += earnings;
    state.income = earnings;
    state.pop = state.total_pop;
    state.pending_sales = 0;

    // O(N_machines): count unpowered houses from registry only
    let mut unpow = 0;
    for shadow in &state.registry.machines {
        if shadow.tool == Tool::House && !state.powered.contains(&shadow.pos) {
            unpow += 1;
        }
    }
    state.unpowered = unpow;

    // O(N_machines): health degradation — only touches machines that exist
    let positions: Vec<(i32, i32)> = state.registry.machines.iter()
        .filter(|s| s.tool.upkeep() != 0 &&
            !matches!(s.tool, Tool::Node | Tool::Street | Tool::Battery | Tool::ConveyorIron | Tool::Pipe | Tool::Warehouse))
        .map(|s| s.pos)
        .collect();

    for (gx, gy) in positions {
        if let Some(mut cell) = state.get_cell_at(gx, gy).cloned() {
            if cell.health > 0.0 {
                cell.health -= 0.1;
                state.set_cell_at(gx, gy, Some(cell));
            }
        }
    }

    // Recalculate power grid if topology changed
    recalculate_power(state);
}

pub fn tick_market(state: &mut GameState) {
    let mut rng = rand::thread_rng();

    if let Some(ref mut ev) = state.rival_event {
        ev.ticks -= 1;
        if ev.ticks <= 0 { state.rival_event = None; }
    } else if rng.gen::<f32>() < 0.05 {
        let target = RIVAL_TARGETS[rng.gen_range(0..RIVAL_TARGETS.len())];
        let corp = RIVAL_CORPS[rng.gen_range(0..RIVAL_CORPS.len())];
        state.rival_event = Some(RivalEvent {
            msg: format!("{} desovou {}!", corp, target.name_pt()),
            target,
            ticks: 10,
            attack: None,
        });
    }

    let rival_target = state.rival_event.as_ref().map(|e| e.target);

    // Apply active Dumping: force target item to floor price
    let dumping_target = if let Some(ref ev) = state.rival_event {
        if let Some(NpcAttack::Dumping { target }) = &ev.attack { Some(*target) } else { None }
    } else { None };

    for it in ItemType::all() {
        let sold = *state.sales_counter.get(it).unwrap_or(&0);
        let base = it.base_price() as f32;
        let cur = *state.prices.get(it).unwrap_or(&it.base_price()) as f32;
        let mut mult = 1.0 + (rng.gen::<f32>() * 0.1 - 0.05);
        if sold > 15 { mult -= 0.35; }
        else if sold > 5 { mult -= 0.15; }
        else if sold > 0 { mult -= 0.02; }
        else { mult += 0.10; }

        let mut p = (cur * mult).clamp(base * 0.2, base * 4.0);
        // Apply Dumping attack: force price floor
        if dumping_target == Some(*it) { p = base * 0.2; }
        // Legacy rival_event non-NPC dumping
        if rival_target == Some(*it) && dumping_target.is_none() { p = base * 0.2; }
        let rp = p.round() as i64;
        let old = *state.prices.get(it).unwrap_or(&it.base_price());
        if rp > old { state.price_trends.insert(*it, 1); }
        else if rp < old { state.price_trends.insert(*it, -1); }
        else { state.price_trends.insert(*it, 0); }
        state.prices.insert(*it, rp);
    }
    state.sales_counter.clear();

    state.demand.ticks -= 1;
    if state.demand.ticks <= 0 {
        state.demand = Demand {
            item: DEMAND_ITEMS[rng.gen_range(0..DEMAND_ITEMS.len())],
            multiplier: rng.gen_range(2..=4),
            ticks: 15 + rng.gen_range(0..15),
        };
    }

    // Apply active Embargo: zero out sales of blocked item
    if state.embargo_ticks > 0 {
        state.embargo_ticks -= 1;
        if state.embargo_ticks == 0 {
            state.embargo_item = None;
            state.set_msg("Embargo encerrado — mercado normalizado.");
        }
    }
}

const CORPS: &[&str] = &["Grupo Barigui", "Coprel Máfia", "Sindicato do Sul", "Cartel Gaúcho"];

/// NPC Mafia AI tick — called alongside tick_market
pub fn tick_npcs(state: &mut GameState) {
    // Tick down cooldown
    if state.npc_cooldown > 0 { state.npc_cooldown -= 1; }

    // Tick down sabotage penalty
    if state.upkeep_penalty > 0 {
        state.upkeep_penalty = (state.upkeep_penalty - 50).max(0);
        if state.upkeep_penalty == 0 {
            state.set_msg("Tensão social aliviada — upkeep normalizado.");
        }
    }

    // Can only attack when cooldown is zero and no active rival event
    if state.npc_cooldown > 0 || state.rival_event.is_some() { return; }

    let pop = state.total_pop;
    if pop == 0 { return; } // nothing to exploit yet

    // ── Aggression Formula ──────────────────────────────────────────
    // A = (income / K) + (unpowered / pop) × multiplier
    // Higher income → AI gets greedy (dumping)
    // Higher power crisis → AI smells blood (sabotage / embargo)
    let income_pressure = (state.income.abs() as f32 / 500.0).min(1.0);
    let crisis_ratio = if pop > 0 { state.unpowered as f32 / pop as f32 } else { 0.0 };
    let aggression = income_pressure + crisis_ratio * 2.0;

    // Only attack if sufficiently aggressive
    if aggression < 0.4 { return; }

    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen();

    // Attack selection weights: heavier aggression → more severe attacks
    let attack = if aggression > 1.5 && crisis_ratio > 0.3 {
        // Severe crisis: sabotage (upkeep shock)
        let penalty = (state.total_upkeep.abs() as i64 / 2).max(100);
        NpcAttack::Sabotage { extra_upkeep: penalty }
    } else if roll < 0.45 || aggression > 1.0 {
        // Dumping: pick the player's most-stocked item
        let target = state.inventory_cache
            .iter()
            .filter(|(it, qty)| *qty > 5 && it.base_price() > 20)
            .max_by_key(|(_, qty)| qty)
            .map(|(it, _)| *it)
            .unwrap_or(RIVAL_TARGETS[rng.gen_range(0..RIVAL_TARGETS.len())]);
        NpcAttack::Dumping { target }
    } else {
        // Embargo: block an advanced item
        let blocked = [ItemType::Processor, ItemType::AiCore, ItemType::CircuitBoard, ItemType::Steel]
            [rng.gen_range(0..4)];
        NpcAttack::Embargo { blocked }
    };

    let corp = CORPS[rng.gen_range(0..CORPS.len())];
    let (msg, ticks) = match &attack {
        NpcAttack::Dumping { target } =>
            (format!("⚠ {} despeja {} no mercado! Preço colapsou.", corp, target.name_pt()), 8),
        NpcAttack::Embargo { blocked } => {
            state.embargo_item = Some(*blocked);
            state.embargo_ticks = 12;
            (format!("🚫 {} bloqueou exportações de {}!", corp, blocked.name_pt()), 12)
        }
        NpcAttack::Sabotage { extra_upkeep } => {
            state.upkeep_penalty += extra_upkeep;
            (format!("💥 {} fomentou greve industrial! Upkeep +{}/tick.", corp, extra_upkeep), 6)
        }
    };

    state.rival_event = Some(RivalEvent {
        msg: msg.clone(),
        target: match &attack { NpcAttack::Dumping { target } => *target, _ => ItemType::IronOre },
        ticks,
        attack: Some(attack),
    });
    state.set_msg(&msg);
    // Cooldown: 20-40 ticks before AI can strike again
    state.npc_cooldown = 20 + rng.gen_range(0..20);
}

pub fn tick_fluids(state: &mut GameState) {
    // O(N_fluid_nodes): iterate only registered pipes, pumps, pumpjacks
    let fluid_positions: Vec<(i32, i32)> = state.fluid_nodes.clone();

    let mut next_type: HashMap<(i32, i32), Option<FluidType>> = HashMap::new();
    let mut next_amt: HashMap<(i32, i32), f32> = HashMap::new();

    // Snapshot current fluid state for all fluid nodes
    for &(gx, gy) in &fluid_positions {
        if let Some(c) = state.get_cell_at(gx, gy) {
            if c.tool == Tool::Pipe {
                next_type.insert((gx, gy), c.fluid_type);
                next_amt.insert((gx, gy), c.fluid_amount);
            }
        }
    }

    // Simulate sources (Pump / Pumpjack) and pipe flow
    for &(gx, gy) in &fluid_positions {
        let c = match state.get_cell_at(gx, gy) {
            Some(c) => c.clone(),
            None => continue,
        };
        if c.construction_progress < 100.0 || c.health <= 0.0 { continue; }

        if (c.tool == Tool::Pump || c.tool == Tool::Pumpjack) && state.powered.contains(&(gx, gy)) {
            let ft = if c.tool == Tool::Pump { FluidType::Water } else { FluidType::CrudeOil };
            for &(dx, dy) in DIRS.iter() {
                let n_gx = gx + dx; let n_gy = gy + dy;
                if let Some(nc) = state.get_cell_at(n_gx, n_gy) {
                    if nc.tool == Tool::Pipe && (nc.fluid_type.is_none() || nc.fluid_type == Some(ft)) {
                        next_type.insert((n_gx, n_gy), Some(ft));
                        let amt = next_amt.entry((n_gx, n_gy)).or_insert(0.0);
                        *amt = (*amt + 20.0).min(100.0);
                    }
                }
            }
        }

        if c.tool == Tool::Pipe && c.fluid_amount > 0.0 {
            let mut flow_out = 0.0;
            for &(dx, dy) in DIRS.iter() {
                let n_gx = gx + dx; let n_gy = gy + dy;
                if let Some(nc) = state.get_cell_at(n_gx, n_gy) {
                    if nc.tool == Tool::Pipe && (nc.fluid_type.is_none() || nc.fluid_type == c.fluid_type) {
                        let diff = c.fluid_amount - nc.fluid_amount;
                        if diff > 1.0 {
                            let transfer = diff * 0.25;
                            next_type.insert((n_gx, n_gy), c.fluid_type);
                            *next_amt.entry((n_gx, n_gy)).or_insert(0.0) += transfer;
                            flow_out += transfer;
                        }
                    }
                }
            }
            if let Some(amt) = next_amt.get_mut(&(gx, gy)) {
                *amt -= flow_out;
                if *amt < 0.5 { *amt = 0.0; next_type.insert((gx, gy), None); }
            }
        }
    }

    // Apply results
    for ((gx, gy), amt) in next_amt {
        let ft = next_type.get(&(gx, gy)).cloned().flatten();
        if let Some(mut cell) = state.get_cell_at(gx, gy).cloned() {
            if cell.tool == Tool::Pipe {
                cell.fluid_type = ft;
                cell.fluid_amount = amt;
                state.set_cell_at(gx, gy, Some(cell));
            }
        }
    }
}

fn sip_fluid(state: &mut GameState, cx: i32, cy: i32, target: FluidType, needed: f32) -> bool {
    let mut gathered = 0.0;
    for &(dx, dy) in DIRS.iter() {
        let nx = cx + dx; let ny = cy + dy;
        if let Some(cell) = state.get_cell_at(nx, ny) {
            if cell.tool == Tool::Pipe && cell.fluid_type == Some(target) && cell.fluid_amount > 0.0 {
                let mut new_cell = cell.clone();
                let sip = (needed - gathered).min(new_cell.fluid_amount);
                new_cell.fluid_amount -= sip;
                if new_cell.fluid_amount < 0.5 { new_cell.fluid_amount = 0.0; new_cell.fluid_type = None; }
                gathered += sip;
                state.set_cell_at(nx, ny, Some(new_cell));
                if gathered >= needed { return true; }
            }
        }
    }
    gathered >= needed
}

fn eject_to_conveyor(state: &mut GameState, cx: i32, cy: i32, out_type: ItemType, hp_cost: f32) -> bool {
    for &(dx, dy) in DIRS.iter() {
        let nx = cx + dx; let ny = cy + dy;
        if let Some(cell) = state.get_cell_at(nx, ny) {
            if cell.tool == Tool::ConveyorIron {
                let d_idx = DIRS.iter().position(|&d| d == (dx, dy)).unwrap();
                if (cell.dir as usize + 2) % 4 == d_idx { continue; }
                let ((sx, sy), _) = GameState::world_to_sector(nx, ny);
                let occupied = state.get_sector(sx, sy).map(|s| s.items.iter().any(|item| item.x as i32 == nx && item.y as i32 == ny)).unwrap_or(false);
                if !occupied {
                    if let Some(sector) = state.get_sector_mut(sx, sy) {
                        sector.items.push(ConveyorItem {
                            item_type: out_type,
                            x: nx as f32,
                            y: ny as f32,
                            progress: 0.0,
                        });
                    }
                    if let Some(mut src) = state.get_cell_at(cx, cy).cloned() {
                        src.health = (src.health - hp_cost).max(0.0);
                        state.set_cell_at(cx, cy, Some(src));
                    }
                    return true;
                }
            }
        }
    }
    false
}

pub fn tick_industry(state: &mut GameState) {
    let active_sectors: Vec<((i32, i32), SectorHandle)> = state.active_pool.iter().map(|&h| {
        let pos = *state.sectors.iter().find(|(_, &sh)| sh == h).unwrap().0;
        (pos, h)
    }).collect();

    for ((sx, sy), _) in active_sectors {
        for ly in 0..SECTOR_SIZE as i32 {
            for lx in 0..SECTOR_SIZE as i32 {
                let gx = sx * SECTOR_SIZE as i32 + lx;
                let gy = sy * SECTOR_SIZE as i32 + ly;
                
                let cell = match state.get_cell_at(gx, gy).cloned() {
                    Some(mut c) => {
                        if c.construction_progress < 100.0 {
                            c.construction_progress = (c.construction_progress + 12.5).min(100.0);
                            state.set_cell_at(gx, gy, Some(c));
                            continue;
                        }
                        if c.health <= 0.0 { continue; }
                        c
                    }
                    None => continue,
                };

                let has_power = state.powered.contains(&(gx, gy)) || 
                    (cell.tool == Tool::CoalPlant && cell.fuel > 0.0) || 
                    (cell.tool == Tool::Nuclear && cell.fuel > 0.0);
                if !has_power { continue; }

                let mut new_cell = cell.clone();

                match cell.tool {
                    Tool::Miner => {
                        let terrain = state.get_terrain_at(gx, gy);
                        if let Some(ore) = terrain.ore_type() {
                            eject_to_conveyor(state, gx, gy, ore, 2.0);
                        }
                    }
                    Tool::Lumberjack => {
                        if state.get_terrain_at(gx, gy) == Terrain::Tree {
                            eject_to_conveyor(state, gx, gy, ItemType::Wood, 2.0);
                        }
                    }
                    Tool::CoalPlant => {
                        if new_cell.fuel > 0.0 { new_cell.fuel -= 1.0; }
                        else if new_cell.buffer[ItemType::CoalOre.to_index()] >= 1 {
                            new_cell.buffer[ItemType::CoalOre.to_index()] -= 1;
                            new_cell.fuel = 10.0;
                        }
                        state.set_cell_at(gx, gy, Some(new_cell));
                    }
                    Tool::Nuclear => {
                        let needs_coolant = if new_cell.fuel > 0.0 {
                            new_cell.fuel -= 1.0;
                            true
                        } else if new_cell.buffer[ItemType::UraniumCell.to_index()] >= 1 {
                            new_cell.buffer[ItemType::UraniumCell.to_index()] -= 1;
                            new_cell.fuel = 40.0;
                            false
                        } else {
                            new_cell.heat = (new_cell.heat - 5.0).max(0.0);
                            false
                        };
                        state.set_cell_at(gx, gy, Some(new_cell));

                        if needs_coolant {
                            if sip_fluid(state, gx, gy, FluidType::Water, 10.0) {
                                if let Some(mut nc) = state.get_cell_at(gx, gy).cloned() {
                                    nc.heat = (nc.heat - 10.0).max(0.0);
                                    state.set_cell_at(gx, gy, Some(nc));
                                }
                            } else {
                                if let Some(mut nc) = state.get_cell_at(gx, gy).cloned() {
                                    nc.heat += 15.0;
                                    if nc.heat >= 100.0 {
                                        trigger_meltdown(state, gx, gy);
                                    } else {
                                        state.set_cell_at(gx, gy, Some(nc));
                                    }
                                }
                            }
                        }
                    }
                    Tool::Smelter => {
                        if cell.processing >= 100.0 {
                            let mut ore_opt = None;
                            for i in 0..ITEM_COUNT {
                                if cell.buffer[i] > 0 {
                                    ore_opt = Some(ItemType::from_index(i));
                                    break;
                                }
                            }
                            if let Some(ore) = ore_opt {
                                let out = match ore {
                                    ItemType::IronOre => ItemType::IronPlate,
                                    ItemType::CopperOre => ItemType::CopperPlate,
                                    ItemType::CoalOre => ItemType::CoalPlate,
                                    ItemType::QuartzOre => ItemType::Silicon,
                                    ItemType::SandOre => ItemType::Glass,
                                    ItemType::GoldOre => ItemType::GoldIngot,
                                    _ => ore,
                                };
                                if eject_to_conveyor(state, gx, gy, out, 5.0) {
                                    if let Some(mut nc) = state.get_cell_at(gx, gy).cloned() {
                                        nc.buffer = [0; ITEM_COUNT];
                                        nc.processing = 0.0;
                                        state.set_cell_at(gx, gy, Some(nc));
                                    }
                                }
                            }
                        } else {
                            let mut has_any = false;
                            for i in 0..ITEM_COUNT { if cell.buffer[i] > 0 { has_any = true; break; } }
                            if has_any {
                                new_cell.processing += 50.0;
                                state.set_cell_at(gx, gy, Some(new_cell));
                            }
                        }
                    }
                    Tool::Press => {
                        if cell.processing >= 100.0 {
                            if cell.buffer[ItemType::CopperPlate.to_index()] > 0 {
                                if eject_to_conveyor(state, gx, gy, ItemType::CopperWire, 3.0) {
                                    if let Some(mut nc) = state.get_cell_at(gx, gy).cloned() {
                                        nc.buffer = [0; ITEM_COUNT];
                                        nc.processing = 0.0;
                                        state.set_cell_at(gx, gy, Some(nc));
                                    }
                                }
                            }
                        } else {
                            let mut has_any = false;
                            for i in 0..ITEM_COUNT { if cell.buffer[i] > 0 { has_any = true; break; } }
                            if has_any {
                                new_cell.processing += 50.0;
                                state.set_cell_at(gx, gy, Some(new_cell));
                            }
                        }
                    }
                    Tool::Centrifuge => {
                        if cell.processing >= 100.0 {
                            if eject_to_conveyor(state, gx, gy, ItemType::UraniumCell, 10.0) {
                                if let Some(mut nc) = state.get_cell_at(gx, gy).cloned() {
                                    nc.buffer[ItemType::UraniumOre.to_index()] -= 3;
                                    nc.processing = 0.0;
                                    state.set_cell_at(gx, gy, Some(nc));
                                }
                            }
                        } else if cell.buffer[ItemType::UraniumOre.to_index()] >= 3 {
                            new_cell.processing += 20.0;
                            state.set_cell_at(gx, gy, Some(new_cell));
                        }
                    }
                    Tool::Assembler => {
                        if cell.processing >= 100.0 {
                            let has_iron_coal = cell.buffer[ItemType::IronPlate.to_index()] >= 1 && cell.buffer[ItemType::CoalPlate.to_index()] >= 1;
                            let has_glass_wire = cell.buffer[ItemType::Glass.to_index()] >= 1 && cell.buffer[ItemType::CopperWire.to_index()] >= 1;
                            let has_si_wire = cell.buffer[ItemType::Silicon.to_index()] >= 1 && cell.buffer[ItemType::CopperWire.to_index()] >= 1;
                            let (out, r1, r2) = if has_iron_coal { (ItemType::Steel, ItemType::IronPlate, ItemType::CoalPlate) }
                                else if has_glass_wire { (ItemType::Processor, ItemType::Glass, ItemType::CopperWire) }
                                else if has_si_wire { (ItemType::CircuitBoard, ItemType::Silicon, ItemType::CopperWire) }
                                else { (ItemType::Wood, ItemType::Wood, ItemType::Wood) };
                            if out != ItemType::Wood && eject_to_conveyor(state, gx, gy, out, 10.0) {
                                if let Some(mut nc) = state.get_cell_at(gx, gy).cloned() {
                                    nc.buffer[r1.to_index()] -= 1;
                                    nc.buffer[r2.to_index()] -= 1;
                                    nc.processing = 0.0;
                                    state.set_cell_at(gx, gy, Some(nc));
                                }
                            }
                        } else {
                            let can = (cell.buffer[ItemType::Silicon.to_index()] >= 1 && cell.buffer[ItemType::CopperWire.to_index()] >= 1)
                                || (cell.buffer[ItemType::IronPlate.to_index()] >= 1 && cell.buffer[ItemType::CoalPlate.to_index()] >= 1)
                                || (cell.buffer[ItemType::Glass.to_index()] >= 1 && cell.buffer[ItemType::CopperWire.to_index()] >= 1);
                            if can { new_cell.processing += 25.0; state.set_cell_at(gx, gy, Some(new_cell)); }
                        }
                    }
                    Tool::ChemPlant => {
                        if cell.processing >= 100.0 {
                            if eject_to_conveyor(state, gx, gy, ItemType::Plastic, 10.0) {
                                if let Some(mut nc) = state.get_cell_at(gx, gy).cloned() {
                                    nc.processing = 0.0;
                                    state.set_cell_at(gx, gy, Some(nc));
                                }
                            }
                        } else {
                            if sip_fluid(state, gx, gy, FluidType::Water, 10.0) && sip_fluid(state, gx, gy, FluidType::CrudeOil, 10.0) {
                                if let Some(mut nc) = state.get_cell_at(gx, gy).cloned() {
                                    nc.processing += 20.0;
                                    state.set_cell_at(gx, gy, Some(nc));
                                }
                            }
                        }
                    }
                    Tool::Quantum => {
                        if cell.processing >= 100.0 {
                            if eject_to_conveyor(state, gx, gy, ItemType::AiCore, 20.0) {
                                if let Some(mut nc) = state.get_cell_at(gx, gy).cloned() {
                                    nc.buffer[ItemType::Processor.to_index()] -= 1;
                                    nc.buffer[ItemType::Plastic.to_index()] -= 1;
                                    nc.buffer[ItemType::GoldIngot.to_index()] -= 1;
                                    nc.processing = 0.0;
                                    state.set_cell_at(gx, gy, Some(nc));
                                }
                            }
                        } else {
                            let can = cell.buffer[ItemType::Processor.to_index()] >= 1
                                && cell.buffer[ItemType::Plastic.to_index()] >= 1
                                && cell.buffer[ItemType::GoldIngot.to_index()] >= 1;
                            if can { new_cell.processing += 10.0; state.set_cell_at(gx, gy, Some(new_cell)); }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn trigger_meltdown(state: &mut GameState, cx: i32, cy: i32) {
    let radius = 8;
    for r in (cy - radius)..=(cy + radius) {
        for c in (cx - radius)..=(cx + radius) {
            if (((c - cx) * (c - cx) + (r - cy) * (r - cy)) as f32).sqrt() <= radius as f32 {
                state.set_cell_at(c, r, None);
                let ((sx, sy), (lx, ly)) = GameState::world_to_sector(c, r);
                if let Some(s) = state.get_sector_mut(sx, sy) {
                s.grid[ly as usize * SECTOR_SIZE + lx as usize] = None;
                s.items.retain(|item| !(item.x as i32 == c && item.y as i32 == r));
                s.terrain[ly as usize * SECTOR_SIZE + lx as usize] = Terrain::Wasteland;
            }
            }
        }
    }
    state.set_msg("MELTDOWN NUCLEAR DETETADO!");
}

pub fn tick_conveyors(state: &mut GameState, dt: f32) {
    let speed = 0.005;
    let mut global_transfers: Vec<(SectorHandle, ConveyorItem)> = Vec::new();
    let active_handles = state.active_pool.clone();

    // 1. Clear occupancy maps — O(N_items) not O(sector_size)
    //    We zero only occupied cells, not the whole 4096-element array.
    for &h in &active_handles {
        let sector = &mut state.pool[h];
        for item in &sector.items {
            let lx = (item.x.floor() as i32).rem_euclid(SECTOR_SIZE as i32) as usize;
            let ly = (item.y.floor() as i32).rem_euclid(SECTOR_SIZE as i32) as usize;
            let idx = ly * SECTOR_SIZE + lx;
            if idx < sector.occupancy.len() {
                sector.occupancy[idx] = 1.0; // clear
            }
        }
    }

    // 2. Process movement
    for &h in &active_handles {
        let (sx, sy) = *state.sectors.iter().find(|(_, &sh)| sh == h).unwrap().0;
        
        // Elite Move: Use take() to avoid allocation/cloning
        let items = std::mem::take(&mut state.pool[h].items);
        
        for mut item in items {
            let gx = item.x as i32;
            let gy = item.y as i32;
            
            let cell = match state.get_cell_at(gx, gy) {
                Some(c) => c,
                None => continue, // Item deleted (e.g. terrain changed)
            };
            
            if cell.tool != Tool::ConveyorIron {
                state.pool[h].next_items.push(item);
                continue;
            }

            item.progress += dt * speed;
            if item.progress >= 1.0 {
                let (dx, dy) = DIRS[cell.dir as usize];
                let nx = gx + dx;
                let ny = gy + dy;

                if let Some(n_cell) = state.get_cell_at(nx, ny) {
                    match n_cell.tool {
                        Tool::ConveyorIron => {
                            let ((nsx, nsy), (nlx, nly)) = GameState::world_to_sector(nx, ny);
                            let n_block = if let Some(nh) = state.sectors.get(&(nsx, nsy)) {
                                state.pool[*nh].occupancy[nly as usize * SECTOR_SIZE + nlx as usize]
                            } else { 1.0 };

                            if n_block < 0.5 {
                                item.progress = 1.0;
                                // Mark current cell as occupied in incremental map
                                let lx_cur = (gx.rem_euclid(SECTOR_SIZE as i32)) as usize;
                                let ly_cur = (gy.rem_euclid(SECTOR_SIZE as i32)) as usize;
                                state.pool[h].occupancy[ly_cur * SECTOR_SIZE + lx_cur] = item.progress;
                                state.pool[h].next_items.push(item);
                            } else {
                                item.x = nx as f32;
                                item.y = ny as f32;
                                item.progress -= 1.0;
                                
                                // Inter-sector transfer check with collision guard
                                if nsx != sx || nsy != sy {
                                    if let Some(&nh) = state.sectors.get(&(nsx, nsy)) {
                                        // Collision guard: check if target cell is free
                                        let t_occ = state.pool[nh].occupancy[nly as usize * SECTOR_SIZE + nlx as usize];
                                        if t_occ >= 0.5 {
                                            let item_progress = item.progress;
                                            global_transfers.push((nh, item));
                                            // Mark target cell occupied immediately to prevent second transfer
                                            state.pool[nh].occupancy[nly as usize * SECTOR_SIZE + nlx as usize] = item_progress;
                                        } else {
                                            // Target blocked — keep item here
                                            item.x = gx as f32; item.y = gy as f32; item.progress = 1.0;
                                            let lx_cur = (gx.rem_euclid(SECTOR_SIZE as i32)) as usize;
                                            let ly_cur = (gy.rem_euclid(SECTOR_SIZE as i32)) as usize;
                                            state.pool[h].occupancy[ly_cur * SECTOR_SIZE + lx_cur] = 1.0;
                                            state.pool[h].next_items.push(item);
                                        }
                                    } else {
                                        // No sector there? Block it.
                                        item.x = gx as f32; item.y = gy as f32; item.progress = 1.0;
                                        state.pool[h].next_items.push(item);
                                    }
                                } else {
                                    // Same-sector move: update occupancy incrementally
                                    let nlx_s = (nx.rem_euclid(SECTOR_SIZE as i32)) as usize;
                                    let nly_s = (ny.rem_euclid(SECTOR_SIZE as i32)) as usize;
                                    state.pool[h].occupancy[nly_s * SECTOR_SIZE + nlx_s] = item.progress;
                                    state.pool[h].next_items.push(item);
                                }
                            }
                        }
                        Tool::Market => {
                            let profit = *state.prices.get(&item.item_type).unwrap_or(&0);
                            state.money += profit;
                            state.pending_sales += profit;
                        }
                        Tool::Warehouse => {
                            *state.inventory.entry(item.item_type).or_insert(0) += 1;
                        }
                        _ => {
                            let mut nc = n_cell.clone();
                            let it = item.item_type;
                            let idx_it = it.to_index();
                            let mut accepted = false;
                            match nc.tool {
                                Tool::Smelter => if it.is_ore() { accepted = true; }
                                Tool::Press => if it == ItemType::CopperPlate { accepted = true; }
                                Tool::Assembler => if it == ItemType::IronPlate || it == ItemType::CopperPlate { accepted = true; }
                                _ => {}
                            }
                            if accepted {
                                nc.buffer[idx_it] += 1;
                                state.set_cell_at(nx, ny, Some(nc));
                            } else {
                                item.progress = 1.0;
                                state.pool[h].next_items.push(item);
                            }
                        }
                    }
                } else {
                    item.progress = 1.0;
                    state.pool[h].next_items.push(item);
                }
            } else {
                state.pool[h].next_items.push(item);
            }
        }
    }

    // 3. Finalize Transfers
    for (target_h, item) in global_transfers {
        state.pool[target_h].next_items.push(item);
    }

    // 4. Atomic Swap
    for &h in &active_handles {
        let sector = &mut state.pool[h];
        std::mem::swap(&mut sector.items, &mut sector.next_items);
    }
}
