pub const GRID_SIZE: usize = 128;
use serde::{Serialize, Deserialize};
pub const CELL_SIZE: f32 = 40.0;

pub const DIRS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tool {
    Conveyor, Pipe, Node, Street, Warehouse, Repair, Eraser,
    Miner, Pump, Pumpjack, Lumberjack,
    Solar, Wind, CoalPlant, Nuclear, Battery,
    Smelter, Press, Assembler, ChemPlant, Centrifuge, Quantum,
    Market, House,
}

impl Tool {
    pub fn all() -> &'static [Tool] {
        &[
            Tool::Conveyor, Tool::Pipe, Tool::Node, Tool::Street, Tool::Warehouse, Tool::Repair, Tool::Eraser,
            Tool::Miner, Tool::Pump, Tool::Pumpjack, Tool::Lumberjack,
            Tool::Solar, Tool::Wind, Tool::CoalPlant, Tool::Nuclear, Tool::Battery,
            Tool::Smelter, Tool::Press, Tool::Assembler, Tool::ChemPlant, Tool::Centrifuge, Tool::Quantum,
            Tool::Market, Tool::House,
        ]
    }

    pub fn cost(&self) -> i64 {
        match self {
            Tool::House => 100, Tool::Street => 10, Tool::Node => 50,
            Tool::Miner => 150, Tool::Pump => 150, Tool::Pumpjack => 300, Tool::Lumberjack => 150,
            Tool::Smelter => 300, Tool::Press => 400, Tool::Assembler => 800,
            Tool::ChemPlant => 1500, Tool::Centrifuge => 2000, Tool::Quantum => 5000,
            Tool::Market => 400, Tool::Warehouse => 100, Tool::Conveyor => 15, Tool::Pipe => 20,
            Tool::Solar => 200, Tool::Wind => 300, Tool::CoalPlant => 800, Tool::Nuclear => 5000, Tool::Battery => 400,
            _ => 0,
        }
    }

    pub fn refund(&self) -> i64 {
        match self {
            Tool::House => 50, Tool::Street => 5, Tool::Node => 25,
            Tool::Miner => 75, Tool::Pump => 75, Tool::Pumpjack => 150, Tool::Lumberjack => 75,
            Tool::Smelter => 150, Tool::Press => 200, Tool::Assembler => 400,
            Tool::ChemPlant => 750, Tool::Centrifuge => 1000, Tool::Quantum => 2500,
            Tool::Market => 200, Tool::Warehouse => 50, Tool::Conveyor => 5, Tool::Pipe => 5,
            Tool::Solar => 100, Tool::Wind => 150, Tool::CoalPlant => 400, Tool::Nuclear => 1000, Tool::Battery => 200,
            _ => 0,
        }
    }

    pub fn upkeep(&self) -> i64 {
        match self {
            Tool::House => 15, Tool::Street => 1, Tool::Market => -5,
            Tool::Solar => -2, Tool::Wind => -5, Tool::CoalPlant => -15, Tool::Nuclear => -100, Tool::Battery => -1,
            _ => 0,
        }
    }

    pub fn power_gen(&self) -> f32 {
        match self {
            Tool::Solar => 40.0, Tool::Wind => 60.0, Tool::CoalPlant => 300.0, Tool::Nuclear => 2500.0,
            _ => 0.0,
        }
    }

    pub fn power_cons(&self) -> f32 {
        match self {
            Tool::Miner => 20.0, Tool::Pump => 20.0, Tool::Pumpjack => 40.0, Tool::Lumberjack => 15.0,
            Tool::Smelter => 40.0, Tool::Press => 30.0, Tool::Assembler => 60.0,
            Tool::ChemPlant => 80.0, Tool::Centrifuge => 150.0, Tool::Quantum => 300.0,
            Tool::Market => 10.0, Tool::House => 5.0,
            _ => 0.0,
        }
    }

    pub fn name_pt(&self) -> &'static str {
        match self {
            Tool::Conveyor => "ESTEIRA", Tool::Pipe => "TUBO", Tool::Node => "POSTE",
            Tool::Street => "RUA", Tool::Warehouse => "ARMAZÉM", Tool::Repair => "REPARO", Tool::Eraser => "APAGAR",
            Tool::Miner => "MINERADOR", Tool::Pump => "BOMBA", Tool::Pumpjack => "EXTRATOR", Tool::Lumberjack => "LENHADOR",
            Tool::Solar => "SOLAR", Tool::Wind => "EÓLICA", Tool::CoalPlant => "TERMELÉTRICA", Tool::Nuclear => "NUCLEAR", Tool::Battery => "BATERIA",
            Tool::Smelter => "FUNDIÇÃO", Tool::Press => "PRENSA", Tool::Assembler => "MONTADORA",
            Tool::ChemPlant => "QUÍMICA", Tool::Centrifuge => "CENTRÍFUGA", Tool::Quantum => "QUÂNTICO",
            Tool::Market => "MERCADO", Tool::House => "CASA",
        }
    }

    pub fn hotkey(&self) -> &'static str {
        match self {
            Tool::Conveyor => "1", Tool::Pipe => "2", Tool::Node => "3", Tool::Street => "4",
            Tool::Warehouse => "5", Tool::Repair => "6", Tool::Eraser => "7",
            Tool::Miner => "Q", Tool::Pump => "T", Tool::Pumpjack => "E", Tool::Lumberjack => "R",
            Tool::Solar => "Y", Tool::Wind => "U", Tool::CoalPlant => "I", Tool::Nuclear => "F", Tool::Battery => "G",
            Tool::Smelter => "Z", Tool::Press => "X", Tool::Assembler => "C",
            Tool::ChemPlant => "V", Tool::Centrifuge => "B", Tool::Quantum => "N",
            Tool::Market => "M", Tool::House => "H",
        }
    }

    pub fn from_key(key: macroquad::input::KeyCode) -> Option<Tool> {
        use macroquad::input::KeyCode::*;
        match key {
            Key1 => Some(Tool::Conveyor), Key2 => Some(Tool::Pipe), Key3 => Some(Tool::Node),
            Key4 => Some(Tool::Street), Key5 => Some(Tool::Warehouse), Key6 => Some(Tool::Repair), Key7 => Some(Tool::Eraser),
            Q => Some(Tool::Miner), T => Some(Tool::Pump), E => Some(Tool::Pumpjack), R => Some(Tool::Lumberjack),
            Y => Some(Tool::Solar), U => Some(Tool::Wind), I => Some(Tool::CoalPlant), F => Some(Tool::Nuclear), G => Some(Tool::Battery),
            Z => Some(Tool::Smelter), X => Some(Tool::Press), C => Some(Tool::Assembler),
            V => Some(Tool::ChemPlant), B => Some(Tool::Centrifuge), N => Some(Tool::Quantum),
            M => Some(Tool::Market), H => Some(Tool::House),
            _ => None,
        }
    }

    pub fn category(&self) -> usize {
        match self {
            Tool::Conveyor | Tool::Pipe | Tool::Node | Tool::Warehouse | Tool::Street | Tool::Repair | Tool::Eraser => 0,
            Tool::Miner | Tool::Pump | Tool::Pumpjack | Tool::Lumberjack => 1,
            Tool::Solar | Tool::Wind | Tool::CoalPlant | Tool::Nuclear | Tool::Battery => 2,
            Tool::Smelter | Tool::Press | Tool::Assembler | Tool::ChemPlant | Tool::Centrifuge | Tool::Quantum => 3,
            Tool::Market | Tool::House => 4,
        }
    }
}

pub const CATEGORY_NAMES: [&str; 5] = ["LOGÍSTICA", "EXTRAÇÃO", "ENERGIA", "FÁBRICAS", "AVANÇADO"];

pub const BATTERY_CAP: f32 = 5000.0;
pub const BATTERY_MAX_IO: f32 = 200.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    IronOre, CopperOre, CoalOre, QuartzOre, SandOre, GoldOre, UraniumOre, Wood,
    IronPlate, CopperPlate, CoalPlate, Silicon, Glass, GoldIngot, UraniumCell,
    CopperWire, Steel, Plastic, CircuitBoard, Processor, AiCore,
}

impl ItemType {
    pub fn name_pt(&self) -> &'static str {
        match self {
            ItemType::IronOre => "MINÉRIO FERRO", ItemType::CopperOre => "MINÉRIO COBRE",
            ItemType::CoalOre => "CARVÃO", ItemType::QuartzOre => "QUARTZO",
            ItemType::SandOre => "AREIA", ItemType::GoldOre => "MINÉRIO OURO",
            ItemType::UraniumOre => "URÂNIO", ItemType::Wood => "MADEIRA",
            ItemType::IronPlate => "CHAPA FERRO", ItemType::CopperPlate => "CHAPA COBRE",
            ItemType::CoalPlate => "CHAPA CARVÃO", ItemType::Silicon => "SILÍCIO",
            ItemType::Glass => "VIDRO", ItemType::GoldIngot => "LINGOTE OURO",
            ItemType::UraniumCell => "CÉL. URÂNIO", ItemType::CopperWire => "FIO COBRE",
            ItemType::Steel => "AÇO", ItemType::Plastic => "PLÁSTICO",
            ItemType::CircuitBoard => "PLACA CIRC.", ItemType::Processor => "PROCESSADOR",
            ItemType::AiCore => "NÚCLEO IA",
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            ItemType::IronOre => "iron_ore", ItemType::CopperOre => "copper_ore",
            ItemType::CoalOre => "coal_ore", ItemType::QuartzOre => "quartz_ore",
            ItemType::SandOre => "sand_ore", ItemType::GoldOre => "gold_ore",
            ItemType::UraniumOre => "uranium_ore", ItemType::Wood => "wood",
            ItemType::IronPlate => "iron_plate", ItemType::CopperPlate => "copper_plate",
            ItemType::CoalPlate => "coal_plate", ItemType::Silicon => "silicon",
            ItemType::Glass => "glass", ItemType::GoldIngot => "gold_ingot",
            ItemType::UraniumCell => "uranium_cell", ItemType::CopperWire => "copper_wire",
            ItemType::Steel => "steel", ItemType::Plastic => "plastic",
            ItemType::CircuitBoard => "circuit_board", ItemType::Processor => "processor",
            ItemType::AiCore => "ai_core",
        }
    }

    pub fn from_key(k: &str) -> Option<ItemType> {
        match k {
            "iron_ore" => Some(ItemType::IronOre), "copper_ore" => Some(ItemType::CopperOre),
            "coal_ore" => Some(ItemType::CoalOre), "quartz_ore" => Some(ItemType::QuartzOre),
            "sand_ore" => Some(ItemType::SandOre), "gold_ore" => Some(ItemType::GoldOre),
            "uranium_ore" => Some(ItemType::UraniumOre), "wood" => Some(ItemType::Wood),
            "iron_plate" => Some(ItemType::IronPlate), "copper_plate" => Some(ItemType::CopperPlate),
            "coal_plate" => Some(ItemType::CoalPlate), "silicon" => Some(ItemType::Silicon),
            "glass" => Some(ItemType::Glass), "gold_ingot" => Some(ItemType::GoldIngot),
            "uranium_cell" => Some(ItemType::UraniumCell), "copper_wire" => Some(ItemType::CopperWire),
            "steel" => Some(ItemType::Steel), "plastic" => Some(ItemType::Plastic),
            "circuit_board" => Some(ItemType::CircuitBoard), "processor" => Some(ItemType::Processor),
            "ai_core" => Some(ItemType::AiCore),
            _ => None,
        }
    }

    pub fn base_price(&self) -> i64 {
        match self {
            ItemType::IronOre => 2, ItemType::CopperOre => 3, ItemType::CoalOre => 2,
            ItemType::QuartzOre => 4, ItemType::SandOre => 1, ItemType::GoldOre => 15,
            ItemType::UraniumOre => 25, ItemType::Wood => 2,
            ItemType::IronPlate => 20, ItemType::CopperPlate => 30, ItemType::CoalPlate => 10,
            ItemType::Silicon => 45, ItemType::Glass => 15, ItemType::GoldIngot => 120, ItemType::UraniumCell => 300,
            ItemType::CopperWire => 50, ItemType::Steel => 60, ItemType::Plastic => 80,
            ItemType::CircuitBoard => 150, ItemType::Processor => 600, ItemType::AiCore => 3500,
        }
    }

    pub fn is_ore(&self) -> bool {
        matches!(self, ItemType::IronOre | ItemType::CopperOre | ItemType::CoalOre | ItemType::QuartzOre | ItemType::SandOre | ItemType::GoldOre | ItemType::UraniumOre | ItemType::Wood)
    }

    pub fn tradeable_items() -> &'static [ItemType] {
        &[
            ItemType::IronPlate, ItemType::CopperPlate, ItemType::CoalPlate,
            ItemType::Silicon, ItemType::Glass, ItemType::GoldIngot, ItemType::UraniumCell,
            ItemType::CopperWire, ItemType::Steel, ItemType::Plastic,
            ItemType::CircuitBoard, ItemType::Processor, ItemType::AiCore,
        ]
    }

    pub fn all() -> &'static [ItemType] {
        &[
            ItemType::IronOre, ItemType::CopperOre, ItemType::CoalOre, ItemType::QuartzOre,
            ItemType::SandOre, ItemType::GoldOre, ItemType::UraniumOre, ItemType::Wood,
            ItemType::IronPlate, ItemType::CopperPlate, ItemType::CoalPlate,
            ItemType::Silicon, ItemType::Glass, ItemType::GoldIngot, ItemType::UraniumCell,
            ItemType::CopperWire, ItemType::Steel, ItemType::Plastic,
            ItemType::CircuitBoard, ItemType::Processor, ItemType::AiCore,
        ]
    }

    pub fn color(&self) -> macroquad::color::Color {
        use macroquad::color::Color;
        match self {
            ItemType::IronOre => Color::new(0.58, 0.64, 0.68, 1.0),
            ItemType::CopperOre => Color::new(0.76, 0.25, 0.07, 1.0),
            ItemType::CoalOre => Color::new(0.06, 0.09, 0.14, 1.0),
            ItemType::QuartzOre => Color::new(0.49, 0.83, 0.99, 1.0),
            ItemType::SandOre => Color::new(0.79, 0.52, 0.03, 1.0),
            ItemType::GoldOre => Color::new(0.92, 0.70, 0.03, 1.0),
            ItemType::UraniumOre => Color::new(0.29, 0.78, 0.50, 1.0),
            ItemType::Wood => Color::new(0.47, 0.21, 0.04, 1.0),
            ItemType::IronPlate => Color::new(0.58, 0.64, 0.70, 1.0),
            ItemType::CopperPlate => Color::new(0.92, 0.35, 0.05, 1.0),
            ItemType::CoalPlate => Color::new(0.12, 0.16, 0.23, 1.0),
            ItemType::Silicon => Color::new(0.20, 0.25, 0.33, 1.0),
            ItemType::Glass => Color::new(0.90, 0.95, 1.0, 0.6),
            ItemType::GoldIngot => Color::new(0.92, 0.70, 0.03, 1.0),
            ItemType::UraniumCell => Color::new(0.29, 0.78, 0.50, 1.0),
            ItemType::CopperWire => Color::new(0.97, 0.58, 0.10, 1.0),
            ItemType::Steel => Color::new(0.20, 0.26, 0.33, 1.0),
            ItemType::Plastic => Color::new(0.98, 0.80, 0.08, 1.0),
            ItemType::CircuitBoard => Color::new(0.08, 0.39, 0.20, 1.0),
            ItemType::Processor => Color::new(0.45, 0.35, 0.65, 1.0),
            ItemType::AiCore => Color::new(0.66, 0.33, 0.97, 1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Empty, Water, Tree, Sand, Iron, Copper, Coal, Quartz, Gold, Oil, Uranium, Wasteland,
}

impl Terrain {
    pub fn name_pt(&self) -> &'static str {
        match self {
            Terrain::Empty => "TERRENO", Terrain::Water => "ÁGUA", Terrain::Tree => "FLORESTA",
            Terrain::Sand => "AREIA", Terrain::Iron => "FILÃO FERRO", Terrain::Copper => "FILÃO COBRE",
            Terrain::Coal => "VEIA CARVÃO", Terrain::Quartz => "VEIA QUARTZO",
            Terrain::Gold => "VEIO OURO", Terrain::Oil => "POÇO PETRÓLEO",
            Terrain::Uranium => "VEIA URÂNIO", Terrain::Wasteland => "IRRADIADO",
        }
    }

    pub fn ore_type(&self) -> Option<ItemType> {
        match self {
            Terrain::Iron => Some(ItemType::IronOre), Terrain::Copper => Some(ItemType::CopperOre),
            Terrain::Coal => Some(ItemType::CoalOre), Terrain::Quartz => Some(ItemType::QuartzOre),
            Terrain::Sand => Some(ItemType::SandOre), Terrain::Gold => Some(ItemType::GoldOre),
            Terrain::Uranium => Some(ItemType::UraniumOre),
            _ => None,
        }
    }
}

pub const RIVAL_CORPS: [&str; 6] = ["TechCorp", "AquaGlobal", "SteelDynamics", "NanoLogic", "QuantumSyn", "Atomix"];
pub const DEMAND_ITEMS: [ItemType; 10] = [
    ItemType::Silicon, ItemType::CopperWire, ItemType::CircuitBoard, ItemType::Steel, ItemType::Glass,
    ItemType::Processor, ItemType::Plastic, ItemType::GoldIngot, ItemType::UraniumCell, ItemType::AiCore,
];
pub const RIVAL_TARGETS: [ItemType; 5] = [ItemType::Steel, ItemType::CircuitBoard, ItemType::Processor, ItemType::Plastic, ItemType::AiCore];
