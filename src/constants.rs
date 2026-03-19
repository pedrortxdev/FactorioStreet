pub const SECTOR_SIZE: usize = 64;
use serde::{Serialize, Deserialize};
pub const CELL_SIZE: f32 = 40.0;

pub const DIRS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

pub const BATTERY_CAP: f32 = 5000.0;
pub const BATTERY_MAX_IO: f32 = 200.0;

pub const CATEGORY_NAMES: [&str; 5] = ["LOGÍSTICA", "EXTRAÇÃO", "ENERGIA", "FÁBRICAS", "AVANÇADO"];

pub const RIVAL_CORPS: [&str; 6] = ["TechCorp", "AquaGlobal", "SteelDynamics", "NanoLogic", "QuantumSyn", "Atomix"];

pub const DEMAND_ITEMS: [crate::types::ItemType; 10] = {
    use crate::types::ItemType::*;
    [
        Silicon, CopperWire, CircuitBoard, Steel, Glass,
        Processor, Plastic, GoldIngot, UraniumCell, AiCore,
    ]
};

pub const RIVAL_TARGETS: [crate::types::ItemType; 5] = {
    use crate::types::ItemType::*;
    [Steel, CircuitBoard, Processor, Plastic, AiCore]
};
