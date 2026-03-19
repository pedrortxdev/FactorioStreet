use crate::constants::*;
use crate::types::*;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn generate_sector(state: &mut GameState, sx: i32, sy: i32) -> SectorHandle {
    let mut sector = Sector::new();
    let mut hasher = DefaultHasher::new();
    state.seed.hash(&mut hasher);
    sx.hash(&mut hasher);
    sy.hash(&mut hasher);
    let sector_seed = hasher.finish();
    let mut rng = StdRng::seed_from_u64(sector_seed);

    // 1. Generate Mountain Boundaries
    for y in 0..SECTOR_SIZE as i32 {
        for x in 0..SECTOR_SIZE as i32 {
            let gx = sx * SECTOR_SIZE as i32 + x;
            let gy = sy * SECTOR_SIZE as i32 + y;
            
            // Simple deterministic noise for boundaries
            let noise = ((gx as f32 * 0.1).sin() * (gy as f32 * 0.1).cos()).abs();
            
            // Edge Mask: 1.0 at edges, 0.0 at center
            let dist_x = (x as f32 - (SECTOR_SIZE as f32 / 2.0)).abs() / (SECTOR_SIZE as f32 / 2.0);
            let dist_y = (y as f32 - (SECTOR_SIZE as f32 / 2.0)).abs() / (SECTOR_SIZE as f32 / 2.0);
            let edge_mask = dist_x.max(dist_y);
            
            // Threshold for mountains (higher near edges)
            if noise * edge_mask > 0.45 {
                sector.terrain[y as usize * SECTOR_SIZE + x as usize] = Terrain::Mountain;
            }
        }
    }

    // 2. Generate Resource Veins (only if not a mountain)
    let add_veins = |sector: &mut Sector, rng: &mut StdRng, t: Terrain, count: usize, scale: usize| {
        for _ in 0..count {
            let x = rng.gen_range(0..SECTOR_SIZE);
            let y = rng.gen_range(0..SECTOR_SIZE);
            if sector.terrain[y * SECTOR_SIZE + x] == Terrain::Mountain { continue; }
            sector.terrain[y * SECTOR_SIZE + x] = t;
            for _ in 0..(4 * scale) {
                if rng.gen::<f32>() > 0.5 {
                    let nx = x as i32 + rng.gen_range(-2..=2);
                    let ny = y as i32 + rng.gen_range(-2..=2);
                    if nx >= 0 && nx < SECTOR_SIZE as i32 && ny >= 0 && ny < SECTOR_SIZE as i32 {
                        let idx = ny as usize * SECTOR_SIZE + nx as usize;
                        if sector.terrain[idx] != Terrain::Mountain {
                            sector.terrain[idx] = t;
                        }
                    }
                }
            }
        }
    };

    add_veins(&mut sector, &mut rng, Terrain::Water, 10, 3);
    add_veins(&mut sector, &mut rng, Terrain::Tree, 15, 2);
    add_veins(&mut sector, &mut rng, Terrain::Iron, 20, 2);
    add_veins(&mut sector, &mut rng, Terrain::Copper, 15, 2);
    add_veins(&mut sector, &mut rng, Terrain::Coal, 20, 2);
    add_veins(&mut sector, &mut rng, Terrain::Quartz, 10, 1);
    add_veins(&mut sector, &mut rng, Terrain::Gold, 5, 1);
    add_veins(&mut sector, &mut rng, Terrain::Oil, 8, 2);
    add_veins(&mut sector, &mut rng, Terrain::Uranium, 4, 1);

    // 3. Sand beaches and Wind Map
    for y in 0..SECTOR_SIZE {
        for x in 0..SECTOR_SIZE {
            let idx = y * SECTOR_SIZE + x;
            if sector.terrain[idx] == Terrain::Empty {
                for &(dx, dy) in DIRS.iter() {
                    let ny = y as i32 + dy;
                    let nx = x as i32 + dx;
                    if nx >= 0 && nx < SECTOR_SIZE as i32 && ny >= 0 && ny < SECTOR_SIZE as i32 {
                        if sector.terrain[ny as usize * SECTOR_SIZE + nx as usize] == Terrain::Water && rng.gen::<f32>() > 0.3 {
                            sector.terrain[idx] = Terrain::Sand;
                            break;
                        }
                    }
                }
            }
            let gx = sx * SECTOR_SIZE as i32 + x as i32;
            let gy = sy * SECTOR_SIZE as i32 + y as i32;
            let wind = ((gx as f32 / 5.0).sin() + (gy as f32 / 5.0).cos()) / 2.0 + 0.5 + (rng.gen::<f32>() * 0.2 - 0.1);
            sector.wind_map[idx] = wind.clamp(0.0, 1.0);
        }
    }

    sector.state = if sx == 0 && sy == 0 { SectorState::ACTIVE } else { SectorState::LOCKED };
    
    let handle = state.pool.insert(sector);
    state.sectors.insert((sx, sy), handle);
    if state.pool[handle].state == SectorState::ACTIVE {
        state.active_pool.push(handle);
    }
    handle
}

pub fn generate_terrain(state: &mut GameState) {
    // Generate the initial home sector
    generate_sector(state, 0, 0);
}
