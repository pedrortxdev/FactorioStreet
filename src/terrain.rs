use crate::constants::*;
use crate::types::*;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub fn generate_terrain(state: &mut GameState) {
    let mut rng = StdRng::seed_from_u64(state.seed);
    
    let add_veins = |terrain: &mut Vec<Terrain>, rng: &mut StdRng, t: Terrain, count: usize, scale: usize| {
        for _ in 0..count {
            let x = rng.gen_range(0..GRID_SIZE);
            let y = rng.gen_range(0..GRID_SIZE);
            terrain[y * GRID_SIZE + x] = t;
            for _ in 0..(8 * scale) {
                if rng.gen::<f32>() > 0.4 {
                    let nx = x as i32 + rng.gen_range(-2..=2);
                    let ny = y as i32 + rng.gen_range(-2..=2);
                    if nx >= 0 && nx < GRID_SIZE as i32 && ny >= 0 && ny < GRID_SIZE as i32 {
                        terrain[ny as usize * GRID_SIZE + nx as usize] = t;
                    }
                }
            }
        }
    };

    add_veins(&mut state.terrain, &mut rng, Terrain::Water, 40, 4);
    add_veins(&mut state.terrain, &mut rng, Terrain::Tree, 50, 3);
    add_veins(&mut state.terrain, &mut rng, Terrain::Iron, 70, 2);
    add_veins(&mut state.terrain, &mut rng, Terrain::Copper, 60, 2);
    add_veins(&mut state.terrain, &mut rng, Terrain::Coal, 80, 2);
    add_veins(&mut state.terrain, &mut rng, Terrain::Quartz, 40, 1);
    add_veins(&mut state.terrain, &mut rng, Terrain::Gold, 20, 1);
    add_veins(&mut state.terrain, &mut rng, Terrain::Oil, 30, 2);
    add_veins(&mut state.terrain, &mut rng, Terrain::Uranium, 15, 1);

    for r in 0..GRID_SIZE {
        for c in 0..GRID_SIZE {
            if state.terrain[r * GRID_SIZE + c] == Terrain::Empty {
                for &(dx, dy) in DIRS.iter() {
                    let nr = r as i32 + dy;
                    let nc = c as i32 + dx;
                    if nr >= 0 && nr < GRID_SIZE as i32 && nc >= 0 && nc < GRID_SIZE as i32 {
                        if state.terrain[nr as usize * GRID_SIZE + nc as usize] == Terrain::Water && rng.gen::<f32>() > 0.3 {
                            state.terrain[r * GRID_SIZE + c] = Terrain::Sand;
                            break;
                        }
                    }
                }
            }
            let wind = ((c as f32 / 5.0).sin() + (r as f32 / 5.0).cos()) / 2.0 + 0.5 + (rng.gen::<f32>() * 0.2 - 0.1);
            state.wind_map[r * GRID_SIZE + c] = wind.clamp(0.0, 1.0);
        }
    }
}
