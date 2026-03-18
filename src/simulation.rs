use std::collections::{HashMap, HashSet, VecDeque};
use crate::constants::*;
use crate::types::*;
use rand::Rng;

pub fn recalculate_power(state: &mut GameState) {
    let mut power_blocks = Vec::new();
    for (i, cell) in state.grid.iter().enumerate() {
        if let Some(c) = cell {
            if c.construction_progress < 100.0 { continue; } // Blueprints don't carry power
            if c.tool.power_gen() > 0.0 || c.tool.power_cons() > 0.0 || c.tool == Tool::Node || c.tool == Tool::Battery {
                if c.health > 0.0 { power_blocks.push(i); }
            }
        }
    }

    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &i in &power_blocks { adj.insert(i, Vec::new()); }
    let mut links = Vec::new();
    let mut processed_links = HashSet::new(); // To avoid duplicate links (u,v) and (v,u)

    for &u in &power_blocks {
        let ux = (u % GRID_SIZE) as i32;
        let uy = (u / GRID_SIZE) as i32;
        let cu = state.grid[u].as_ref().unwrap();
        
        // Nodes connect up to 3 tiles away, others only to immediate neighbors (dist 1)
        let radius = if cu.tool == Tool::Node { 3 } else { 1 };
        
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx == 0 && dy == 0 { continue; }
                let vx = ux + dx;
                let vy = uy + dy;
                
                if let Some(v) = GameState::idx(vx, vy) {
                    if let Some(cv) = &state.grid[v] {
                        if cv.construction_progress < 100.0 || cv.health <= 0.0 { continue; }
                        
                        // IMPORTANT: Only connect if target is also power-related
                        if !(cv.tool.power_gen() > 0.0 || cv.tool.power_cons() > 0.0 || cv.tool == Tool::Node || cv.tool == Tool::Battery) {
                            continue;
                        }

                        let dist = dx.abs().max(dy.abs());
                        let mut connected = false;
                        
                        // Rule 1: Direct neighbors (dist 1, straight lines)
                        if dist == 1 && (dx == 0 || dy == 0) { connected = true; }
                        // Rule 2: Nodes reach up to 3 tiles
                        else if dist <= 3 && (cu.tool == Tool::Node || cv.tool == Tool::Node) { connected = true; }
                        
                        if connected {
                            // Add to adjacency (ensure both ways because search radius is asymmetric)
                            adj.entry(u).or_default().push(v);
                            adj.entry(v).or_default().push(u);
                            
                            // Visual links (deduplicated)
                            let (min_uv, max_uv) = if u < v { (u, v) } else { (v, u) };
                            if !processed_links.contains(&(min_uv, max_uv)) {
                                if cu.tool == Tool::Node || cv.tool == Tool::Node {
                                    links.push(PowerLink { u: min_uv, v: max_uv });
                                }
                                processed_links.insert((min_uv, max_uv));
                            }
                        }
                    }
                }
            }
        }
    }

    let mut visited = HashSet::new();
    let mut new_powered = HashSet::new();
    let mut global_gen = 0.0f32;
    let mut global_cons = 0.0f32;

    let mut sunlight = 1.0f32;
    if state.time_of_day < 600.0 || state.time_of_day > 1800.0 { sunlight = 0.0; }
    else if state.time_of_day < 800.0 { sunlight = (state.time_of_day - 600.0) / 200.0; }
    else if state.time_of_day > 1600.0 { sunlight = (1800.0 - state.time_of_day) / 200.0; }

    for &start in &power_blocks {
        if visited.contains(&start) { continue; }
        let mut comp = Vec::new();
        let mut gen = 0.0f32;
        let mut cons = 0.0f32;
        let mut bat_indices = Vec::new();
        let mut q = VecDeque::new();
        q.push_back(start); visited.insert(start);

        while let Some(curr) = q.pop_front() {
            comp.push(curr);
            let cell = state.grid[curr].as_ref().unwrap();
            match cell.tool {
                Tool::Solar => gen += Tool::Solar.power_gen() * sunlight,
                Tool::Wind => gen += Tool::Wind.power_gen() * state.wind_map[curr],
                Tool::CoalPlant => { if cell.fuel > 0.0 { gen += Tool::CoalPlant.power_gen(); } }
                Tool::Nuclear => { if cell.fuel > 0.0 { gen += Tool::Nuclear.power_gen(); } }
                Tool::Battery => { bat_indices.push(curr); }
                _ => {
                    gen += cell.tool.power_gen();
                    cons += cell.tool.power_cons();
                }
            }
            if let Some(neighbors) = adj.get(&curr) {
                for &n in neighbors {
                    if !visited.contains(&n) { visited.insert(n); q.push_back(n); }
                }
            }
        }

        global_gen += gen;
        global_cons += cons;
        let net = gen - cons;

        if net > 0.0 {
            for &c in &comp { new_powered.insert(c); }
            if !bat_indices.is_empty() {
                let charge_per = net / bat_indices.len() as f32;
                for &bi in &bat_indices {
                    if let Some(ref mut cell) = state.grid[bi] {
                        cell.charge = (cell.charge + charge_per).min(BATTERY_CAP);
                    }
                }
            }
        } else {
            let deficit = net.abs();
            let available: f32 = bat_indices.iter().map(|&bi| {
                state.grid[bi].as_ref().map_or(0.0, |c| c.charge.min(BATTERY_MAX_IO))
            }).sum();
            if available >= deficit {
                for &c in &comp { new_powered.insert(c); }
                let drain = deficit / bat_indices.len() as f32;
                for &bi in &bat_indices {
                    if let Some(ref mut cell) = state.grid[bi] {
                        cell.charge = (cell.charge - drain).max(0.0);
                    }
                }
            } else {
                for &bi in &bat_indices {
                    if let Some(ref mut cell) = state.grid[bi] {
                        cell.charge = (cell.charge - BATTERY_MAX_IO).max(0.0);
                    }
                }
            }
        }
    }

    state.powered = new_powered;
    state.power_links = links;
    state.power_gen = global_gen;
    state.power_cons = global_cons;
}

pub fn tick_economy(state: &mut GameState) {
    let mut upkeep: i64 = 0;
    let mut pop = 0;
    let mut unpow = 0;
    state.time_of_day += 50.0;
    if state.time_of_day >= 2400.0 { state.time_of_day -= 2400.0; }

    for i in 0..state.grid.len() {
        if let Some(ref mut cell) = state.grid[i] {
            upkeep += cell.tool.upkeep();
            if matches!(cell.tool, Tool::Solar | Tool::Wind | Tool::CoalPlant | Tool::Nuclear) && cell.health > 0.0 {
                cell.health -= 0.1; // Slower decay
            }
            if cell.tool == Tool::House {
                pop += 1;
                if !state.powered.contains(&i) {
                    upkeep -= Tool::House.upkeep();
                    unpow += 1;
                }
            }
        }
    }

    state.money += upkeep;
    state.income = upkeep + state.pending_sales;
    state.pop = pop;
    state.unpowered = unpow;
    state.pending_sales = 0;
    
    // Always recalculate because sunlight and wind might have changed
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
        });
    }

    let rival_target = state.rival_event.as_ref().map(|e| e.target);
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
        if rival_target == Some(*it) { p = base * 0.2; }
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
}

pub fn tick_fluids(state: &mut GameState) {
    let gs = GRID_SIZE;
    let len = gs * gs;
    let mut next_type: Vec<Option<FluidType>> = vec![None; len];
    let mut next_amt: Vec<f32> = vec![0.0; len];

    for i in 0..len {
        if let Some(ref c) = state.grid[i] {
            if c.tool == Tool::Pipe {
                next_type[i] = c.fluid_type;
                next_amt[i] = c.fluid_amount;
            }
        }
    }

    for i in 0..len {
        if let Some(ref c) = state.grid[i] {
            let cx = (i % gs) as i32;
            let cy = (i / gs) as i32;

            if c.construction_progress < 100.0 { continue; }

            if (c.tool == Tool::Pump || c.tool == Tool::Pumpjack) && state.powered.contains(&i) && c.health > 0.0 {
                let ft = if c.tool == Tool::Pump { FluidType::Water } else { FluidType::CrudeOil };
                for &(dx, dy) in DIRS.iter() {
                    let nx = cx + dx; let ny = cy + dy;
                    if let Some(ni) = GameState::idx(nx, ny) {
                        if let Some(ref nc) = state.grid[ni] {
                            if nc.tool == Tool::Pipe && (nc.fluid_type.is_none() || nc.fluid_type == Some(ft)) {
                                next_type[ni] = Some(ft);
                                next_amt[ni] = (next_amt[ni] + 20.0).min(100.0);
                            }
                        }
                    }
                }
            }

            if c.tool == Tool::Pipe && c.fluid_amount > 0.0 {
                let mut flow_out = 0.0;
                for &(dx, dy) in DIRS.iter() {
                    let nx = cx + dx; let ny = cy + dy;
                    if let Some(ni) = GameState::idx(nx, ny) {
                        if let Some(ref nc) = state.grid[ni] {
                            if nc.tool == Tool::Pipe && (nc.fluid_type.is_none() || nc.fluid_type == c.fluid_type) {
                                let diff = c.fluid_amount - nc.fluid_amount;
                                if diff > 0.0 {
                                    let transfer = diff * 0.25;
                                    next_type[ni] = c.fluid_type;
                                    next_amt[ni] += transfer;
                                    flow_out += transfer;
                                }
                            }
                        }
                    }
                }
                next_amt[i] -= flow_out;
                if next_amt[i] < 0.5 { next_amt[i] = 0.0; next_type[i] = None; }
            }
        }
    }

    for i in 0..len {
        if let Some(ref mut c) = state.grid[i] {
            if c.tool == Tool::Pipe {
                c.fluid_type = next_type[i];
                c.fluid_amount = next_amt[i];
            }
        }
    }
}

fn sip_fluid(state: &mut GameState, cx: i32, cy: i32, target: FluidType, needed: f32) -> bool {
    let mut gathered = 0.0;
    for &(dx, dy) in DIRS.iter() {
        let nx = cx + dx; let ny = cy + dy;
        if let Some(ni) = GameState::idx(nx, ny) {
            if let Some(ref mut nc) = state.grid[ni] {
                if nc.tool == Tool::Pipe && nc.fluid_type == Some(target) && nc.fluid_amount > 0.0 {
                    let sip = (needed - gathered).min(nc.fluid_amount);
                    nc.fluid_amount -= sip;
                    if nc.fluid_amount < 0.5 { nc.fluid_amount = 0.0; nc.fluid_type = None; }
                    gathered += sip;
                    if gathered >= needed { return true; }
                }
            }
        }
    }
    gathered >= needed
}

fn eject_to_conveyor(state: &mut GameState, cx: i32, cy: i32, out_type: ItemType, hp_cost: f32) -> bool {
    for &(dx, dy) in DIRS.iter() {
        let nx = cx + dx; let ny = cy + dy;
        if let Some(ni) = GameState::idx(nx, ny) {
            if let Some(ref nc) = state.grid[ni] {
                if nc.tool == Tool::Conveyor {
                    // check direction doesn't point back
                    let d_idx = DIRS.iter().position(|&d| d == (dx, dy)).unwrap();
                    if (nc.dir as usize + 2) % 4 == d_idx { continue; }
                    let occupied = state.items.iter().any(|item| item.x as i32 == nx && item.y as i32 == ny);
                    if !occupied {
                        state.items.push(ConveyorItem {
                            item_type: out_type, x: nx as f32, y: ny as f32, progress: 0.1,
                        });
                        if let Some(ref mut src) = state.grid[cy as usize * GRID_SIZE + cx as usize] {
                            src.health = (src.health - hp_cost).max(0.0);
                        }
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn tick_industry(state: &mut GameState) {
    let gs = GRID_SIZE;
    let indices: Vec<usize> = (0..(gs * gs)).collect();

    for &i in &indices {
        let (cx, cy) = ((i % gs) as i32, (i / gs) as i32);
        let (tool, _health, _fuel, has_power);
        // Advance construction on cells that exist but aren't done yet
        if let Some(ref mut cell) = state.grid[i] {
            if cell.construction_progress < 100.0 {
                cell.construction_progress = (cell.construction_progress + 12.5).min(100.0);
                continue;
            }
            if cell.health <= 0.0 { continue; }
            tool = cell.tool;
            _health = cell.health;
            _fuel = cell.fuel;
            has_power = state.powered.contains(&i) || matches!(tool, Tool::Solar | Tool::Wind | Tool::CoalPlant | Tool::Nuclear);
        } else {
            continue; // Empty cell
        }

        if !has_power { continue; }

        match tool {
            Tool::Miner => {
                let terrain = state.terrain[i];
                if let Some(ore) = terrain.ore_type() {
                    eject_to_conveyor(state, cx, cy, ore, 2.0);
                }
            }
            Tool::Lumberjack => {
                if state.terrain[i] == Terrain::Tree {
                    eject_to_conveyor(state, cx, cy, ItemType::Wood, 2.0);
                }
            }
            Tool::CoalPlant => {
                let cell = state.grid[i].as_mut().unwrap();
                if cell.fuel > 0.0 { cell.fuel -= 1.0; }
                else if *cell.buffer.get(&ItemType::CoalOre).unwrap_or(&0) >= 1 {
                    *cell.buffer.get_mut(&ItemType::CoalOre).unwrap() -= 1;
                    cell.fuel = 10.0;
                }
            }
            Tool::Nuclear => {
                let needs_coolant = {
                    let cell = state.grid[i].as_mut().unwrap();
                    if cell.fuel > 0.0 {
                        cell.fuel -= 1.0;
                        true
                    } else if *cell.buffer.get(&ItemType::UraniumCell).unwrap_or(&0) >= 1 {
                        *cell.buffer.get_mut(&ItemType::UraniumCell).unwrap() -= 1;
                        cell.fuel = 40.0;
                        false
                    } else {
                        let cell = state.grid[i].as_mut().unwrap();
                        cell.heat = (cell.heat - 5.0).max(0.0);
                        false
                    }
                };
                if needs_coolant {
                    if sip_fluid(state, cx, cy, FluidType::Water, 10.0) {
                        state.grid[i].as_mut().unwrap().heat = (state.grid[i].as_ref().unwrap().heat - 10.0).max(0.0);
                    } else {
                        state.grid[i].as_mut().unwrap().heat += 15.0;
                        if state.grid[i].as_ref().unwrap().heat >= 100.0 {
                            trigger_meltdown(state, cx, cy);
                        }
                    }
                }
            }
            Tool::Smelter => {
                let cell = state.grid[i].as_ref().unwrap();
                if cell.processing >= 100.0 {
                    let buf_keys: Vec<ItemType> = cell.buffer.keys().cloned().collect();
                    if let Some(&ore) = buf_keys.first() {
                        let out = match ore {
                            ItemType::IronOre => ItemType::IronPlate,
                            ItemType::CopperOre => ItemType::CopperPlate,
                            ItemType::CoalOre => ItemType::CoalPlate,
                            ItemType::QuartzOre => ItemType::Silicon,
                            ItemType::SandOre => ItemType::Glass,
                            ItemType::GoldOre => ItemType::GoldIngot,
                            _ => ore,
                        };
                        if eject_to_conveyor(state, cx, cy, out, 5.0) {
                            let cell = state.grid[i].as_mut().unwrap();
                            cell.buffer.clear();
                            cell.processing = 0.0;
                        }
                    }
                } else if !cell.buffer.is_empty() {
                    let cell = state.grid[i].as_mut().unwrap();
                    cell.processing += 50.0;
                }
            }
            Tool::Press => {
                let cell = state.grid[i].as_ref().unwrap();
                if cell.processing >= 100.0 {
                    let has_copper = cell.buffer.contains_key(&ItemType::CopperPlate);
                    let out = if has_copper { ItemType::CopperWire } else { continue };
                    if eject_to_conveyor(state, cx, cy, out, 3.0) {
                        let cell = state.grid[i].as_mut().unwrap();
                        cell.buffer.clear();
                        cell.processing = 0.0;
                    }
                } else if !cell.buffer.is_empty() {
                    let cell = state.grid[i].as_mut().unwrap();
                    cell.processing += 50.0;
                }
            }
            Tool::Centrifuge => {
                let cell = state.grid[i].as_ref().unwrap();
                if cell.processing >= 100.0 {
                    if eject_to_conveyor(state, cx, cy, ItemType::UraniumCell, 10.0) {
                        let cell = state.grid[i].as_mut().unwrap();
                        *cell.buffer.entry(ItemType::UraniumOre).or_insert(0) -= 3;
                        cell.processing = 0.0;
                    }
                } else if *cell.buffer.get(&ItemType::UraniumOre).unwrap_or(&0) >= 3 {
                    let cell = state.grid[i].as_mut().unwrap();
                    cell.processing += 20.0;
                }
            }
            Tool::Assembler => {
                let cell = state.grid[i].as_ref().unwrap();
                if cell.processing >= 100.0 {
                    let has_iron_coal = *cell.buffer.get(&ItemType::IronPlate).unwrap_or(&0) >= 1 && *cell.buffer.get(&ItemType::CoalPlate).unwrap_or(&0) >= 1;
                    let has_glass_wire = *cell.buffer.get(&ItemType::Glass).unwrap_or(&0) >= 1 && *cell.buffer.get(&ItemType::CopperWire).unwrap_or(&0) >= 1;
                    let has_si_wire = *cell.buffer.get(&ItemType::Silicon).unwrap_or(&0) >= 1 && *cell.buffer.get(&ItemType::CopperWire).unwrap_or(&0) >= 1;
                    let (out, r1, r2) = if has_iron_coal { (ItemType::Steel, ItemType::IronPlate, ItemType::CoalPlate) }
                        else if has_glass_wire { (ItemType::Processor, ItemType::Glass, ItemType::CopperWire) }
                        else if has_si_wire { (ItemType::CircuitBoard, ItemType::Silicon, ItemType::CopperWire) }
                        else { continue };
                    if eject_to_conveyor(state, cx, cy, out, 10.0) {
                        let cell = state.grid[i].as_mut().unwrap();
                        *cell.buffer.get_mut(&r1).unwrap() -= 1;
                        *cell.buffer.get_mut(&r2).unwrap() -= 1;
                        cell.processing = 0.0;
                    }
                } else {
                    let cell = state.grid[i].as_ref().unwrap();
                    let can = (*cell.buffer.get(&ItemType::Silicon).unwrap_or(&0) >= 1 && *cell.buffer.get(&ItemType::CopperWire).unwrap_or(&0) >= 1)
                        || (*cell.buffer.get(&ItemType::IronPlate).unwrap_or(&0) >= 1 && *cell.buffer.get(&ItemType::CoalPlate).unwrap_or(&0) >= 1)
                        || (*cell.buffer.get(&ItemType::Glass).unwrap_or(&0) >= 1 && *cell.buffer.get(&ItemType::CopperWire).unwrap_or(&0) >= 1);
                    if can { let cell = state.grid[i].as_mut().unwrap(); cell.processing += 25.0; }
                }
            }
            Tool::ChemPlant => {
                let cell = state.grid[i].as_ref().unwrap();
                if cell.processing >= 100.0 {
                    if eject_to_conveyor(state, cx, cy, ItemType::Plastic, 10.0) {
                        let cell = state.grid[i].as_mut().unwrap();
                        cell.processing = 0.0;
                    }
                } else {
                    if sip_fluid(state, cx, cy, FluidType::Water, 10.0) && sip_fluid(state, cx, cy, FluidType::CrudeOil, 10.0) {
                        let cell = state.grid[i].as_mut().unwrap();
                        cell.processing += 20.0;
                    }
                }
            }
            Tool::Quantum => {
                let cell = state.grid[i].as_ref().unwrap();
                if cell.processing >= 100.0 {
                    if eject_to_conveyor(state, cx, cy, ItemType::AiCore, 20.0) {
                        let cell = state.grid[i].as_mut().unwrap();
                        *cell.buffer.get_mut(&ItemType::Processor).unwrap() -= 1;
                        *cell.buffer.get_mut(&ItemType::Plastic).unwrap() -= 1;
                        *cell.buffer.get_mut(&ItemType::GoldIngot).unwrap() -= 1;
                        cell.processing = 0.0;
                    }
                } else {
                    let can = *cell.buffer.get(&ItemType::Processor).unwrap_or(&0) >= 1
                        && *cell.buffer.get(&ItemType::Plastic).unwrap_or(&0) >= 1
                        && *cell.buffer.get(&ItemType::GoldIngot).unwrap_or(&0) >= 1;
                    if can { let cell = state.grid[i].as_mut().unwrap(); cell.processing += 10.0; }
                }
            }
            _ => {}
        }
    }
}

fn trigger_meltdown(state: &mut GameState, cx: i32, cy: i32) {
    let radius = 8;
    for r in (cy - radius)..=(cy + radius) {
        for c in (cx - radius)..=(cx + radius) {
            if let Some(idx) = GameState::idx(c, r) {
                if (((c - cx) * (c - cx) + (r - cy) * (r - cy)) as f32).sqrt() <= radius as f32 {
                    state.grid[idx] = None;
                    state.terrain[idx] = Terrain::Wasteland;
                    state.items.retain(|item| !(item.x as i32 == c && item.y as i32 == r));
                }
            }
        }
    }
    state.set_msg("MELTDOWN NUCLEAR DETETADO!");
}

pub fn tick_conveyors(state: &mut GameState, dt: f32) {
    let gs = GRID_SIZE;
    let mut cell_block = vec![1.0f32; gs * gs];
    for item in &state.items {
        let cx = item.x as usize; let cy = item.y as usize;
        if cx < gs && cy < gs {
            let idx = cy * gs + cx;
            cell_block[idx] = cell_block[idx].min(item.progress);
        }
    }

    let speed = 0.0025;
    let mut to_remove = Vec::new();

    for i in 0..state.items.len() {
        let item = &mut state.items[i];
        let cx = item.x as i32; let cy = item.y as i32;
        let idx = match GameState::idx(cx, cy) { Some(i) => i, None => { to_remove.push(i); continue; } };
        let is_conv = state.grid[idx].as_ref().map_or(false, |c| c.tool == Tool::Conveyor);
        if !is_conv { continue; }

        item.progress += dt * speed;
        if item.progress >= 1.0 {
            let dir = state.grid[idx].as_ref().unwrap().dir as usize;
            let (dx, dy) = DIRS[dir];
            let nx = cx + dx; let ny = cy + dy;
            if let Some(ni) = GameState::idx(nx, ny) {
                let n_tool = state.grid[ni].as_ref().map(|c| c.tool);
                match n_tool {
                    Some(Tool::Conveyor) => {
                        if cell_block[ni] < 0.5 { item.progress = 1.0; }
                        else { item.x = nx as f32; item.y = ny as f32; item.progress -= 1.0; cell_block[ni] = item.progress; }
                    }
                    Some(Tool::Smelter) => {
                        let item_t = item.item_type;
                        if item_t.is_ore() && item_t != ItemType::Wood {
                            let c = state.grid[ni].as_ref().unwrap();
                            if c.health > 0.0 && c.buffer.is_empty() {
                                let cell = state.grid[ni].as_mut().unwrap();
                                cell.buffer.insert(item_t, 1);
                                cell.processing = 0.0;
                                to_remove.push(i); continue;
                            }
                        }
                        item.progress = 1.0;
                    }
                    Some(Tool::Press) => {
                        let item_t = item.item_type;
                        if item_t == ItemType::CopperPlate {
                            let c = state.grid[ni].as_ref().unwrap();
                            if c.health > 0.0 && c.buffer.is_empty() {
                                let cell = state.grid[ni].as_mut().unwrap();
                                cell.buffer.insert(item_t, 1);
                                cell.processing = 0.0;
                                to_remove.push(i); continue;
                            }
                        }
                        item.progress = 1.0;
                    }
                    Some(Tool::Assembler) => {
                        let item_t = item.item_type;
                        if matches!(item_t, ItemType::Silicon | ItemType::CopperWire | ItemType::IronPlate | ItemType::CoalPlate | ItemType::Glass) {
                            let c = state.grid[ni].as_ref().unwrap();
                            if c.health > 0.0 && *c.buffer.get(&item_t).unwrap_or(&0) < 5 {
                                let cell = state.grid[ni].as_mut().unwrap();
                                *cell.buffer.entry(item_t).or_insert(0) += 1;
                                to_remove.push(i); continue;
                            }
                        }
                        item.progress = 1.0;
                    }
                    Some(Tool::Quantum) => {
                        let item_t = item.item_type;
                        if matches!(item_t, ItemType::Processor | ItemType::Plastic | ItemType::GoldIngot) {
                            let c = state.grid[ni].as_ref().unwrap();
                            if c.health > 0.0 && *c.buffer.get(&item_t).unwrap_or(&0) < 5 {
                                let cell = state.grid[ni].as_mut().unwrap();
                                *cell.buffer.entry(item_t).or_insert(0) += 1;
                                to_remove.push(i); continue;
                            }
                        }
                        item.progress = 1.0;
                    }
                    Some(Tool::Centrifuge) => {
                        if item.item_type == ItemType::UraniumOre {
                            let c = state.grid[ni].as_ref().unwrap();
                            if c.health > 0.0 && *c.buffer.get(&ItemType::UraniumOre).unwrap_or(&0) < 10 {
                                let cell = state.grid[ni].as_mut().unwrap();
                                *cell.buffer.entry(ItemType::UraniumOre).or_insert(0) += 1;
                                to_remove.push(i); continue;
                            }
                        }
                        item.progress = 1.0;
                    }
                    Some(Tool::CoalPlant) => {
                        if item.item_type == ItemType::CoalOre {
                            let c = state.grid[ni].as_ref().unwrap();
                            if c.health > 0.0 && *c.buffer.get(&ItemType::CoalOre).unwrap_or(&0) < 50 {
                                let cell = state.grid[ni].as_mut().unwrap();
                                *cell.buffer.entry(ItemType::CoalOre).or_insert(0) += 1;
                                to_remove.push(i); continue;
                            }
                        }
                        item.progress = 1.0;
                    }
                    Some(Tool::Nuclear) => {
                        if item.item_type == ItemType::UraniumCell {
                            let c = state.grid[ni].as_ref().unwrap();
                            if c.health > 0.0 && *c.buffer.get(&ItemType::UraniumCell).unwrap_or(&0) < 20 {
                                let cell = state.grid[ni].as_mut().unwrap();
                                *cell.buffer.entry(ItemType::UraniumCell).or_insert(0) += 1;
                                to_remove.push(i); continue;
                            }
                        }
                        item.progress = 1.0;
                    }
                    Some(Tool::Market) => {
                        let item_t = item.item_type;
                        let mut profit = *state.prices.get(&item_t).unwrap_or(&0);
                        if state.demand.item == item_t { profit *= state.demand.multiplier; }
                        state.money += profit;
                        state.pending_sales += profit;
                        *state.sales_counter.entry(item_t).or_insert(0) += 1;
                        to_remove.push(i); continue;
                    }
                    Some(Tool::Warehouse) => {
                        *state.inventory.entry(item.item_type).or_insert(0) += 1;
                        to_remove.push(i); continue;
                    }
                    _ => { item.progress = 1.0; }
                }
            } else { to_remove.push(i); }
        }
    }

    to_remove.sort_unstable();
    to_remove.dedup();
    for &idx in to_remove.iter().rev() {
        if idx < state.items.len() { state.items.swap_remove(idx); }
    }
}
