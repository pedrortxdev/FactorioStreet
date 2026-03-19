use crate::constants::*;
use crate::network::NpcInfo;
use std::collections::{HashMap, HashSet};
use crate::sprites::SpriteBank;
use macroquad::prelude::{Texture2D, Vec2, vec2, Color, KeyCode};
use serde::{Serialize, Deserialize};
use slotmap::{SlotMap, new_key_type};

new_key_type! { pub struct SectorHandle; }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectorState {
    LOCKED,
    UNLOCKING,
    ACTIVE,
    IDLE,
}

pub mod crystal_forge;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tool {
    ConveyorIron, Pipe, Node, Street, Warehouse, Repair, Eraser,
    Miner, Pump, Pumpjack, Lumberjack,
    Solar, Wind, CoalPlant, Nuclear, Battery,
    Smelter, Press, Assembler, ChemPlant, Centrifuge, Quantum,
    Market, House,
}

impl Tool {
    pub fn all() -> &'static [Tool] {
        &[
            Tool::ConveyorIron, Tool::Pipe, Tool::Node, Tool::Street, Tool::Warehouse, Tool::Repair, Tool::Eraser,
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
            Tool::Market => 400, Tool::Warehouse => 100, Tool::ConveyorIron => 15, Tool::Pipe => 20,
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
            Tool::Market => 200, Tool::Warehouse => 50, Tool::ConveyorIron => 5, Tool::Pipe => 5,
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
            Tool::ConveyorIron => "ESTEIRA FERRO", Tool::Pipe => "TUBO", Tool::Node => "POSTE",
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
            Tool::ConveyorIron => "1", Tool::Pipe => "2", Tool::Node => "3", Tool::Street => "4",
            Tool::Warehouse => "5", Tool::Repair => "6", Tool::Eraser => "7",
            Tool::Miner => "Q", Tool::Pump => "T", Tool::Pumpjack => "E", Tool::Lumberjack => "R",
            Tool::Solar => "Y", Tool::Wind => "U", Tool::CoalPlant => "I", Tool::Nuclear => "F", Tool::Battery => "G",
            Tool::Smelter => "Z", Tool::Press => "X", Tool::Assembler => "C",
            Tool::ChemPlant => "V", Tool::Centrifuge => "B", Tool::Quantum => "N",
            Tool::Market => "M", Tool::House => "H",
        }
    }

    pub fn from_key(key: KeyCode) -> Option<Tool> {
        match key {
            KeyCode::Key1 => Some(Tool::ConveyorIron), KeyCode::Key2 => Some(Tool::Pipe), KeyCode::Key3 => Some(Tool::Node),
            KeyCode::Key4 => Some(Tool::Street), KeyCode::Key5 => Some(Tool::Warehouse), KeyCode::Key6 => Some(Tool::Repair), KeyCode::Key7 => Some(Tool::Eraser),
            KeyCode::Q => Some(Tool::Miner), KeyCode::T => Some(Tool::Pump), KeyCode::E => Some(Tool::Pumpjack), KeyCode::R => Some(Tool::Lumberjack),
            KeyCode::Y => Some(Tool::Solar), KeyCode::U => Some(Tool::Wind), KeyCode::I => Some(Tool::CoalPlant), KeyCode::F => Some(Tool::Nuclear), KeyCode::G => Some(Tool::Battery),
            KeyCode::Z => Some(Tool::Smelter), KeyCode::X => Some(Tool::Press), KeyCode::C => Some(Tool::Assembler),
            KeyCode::V => Some(Tool::ChemPlant), KeyCode::B => Some(Tool::Centrifuge), KeyCode::N => Some(Tool::Quantum),
            KeyCode::M => Some(Tool::Market), KeyCode::H => Some(Tool::House),
            _ => None,
        }
    }

    pub fn category(&self) -> usize {
        match self {
            Tool::ConveyorIron | Tool::Pipe | Tool::Node | Tool::Warehouse | Tool::Street | Tool::Repair | Tool::Eraser => 0,
            Tool::Miner | Tool::Pump | Tool::Pumpjack | Tool::Lumberjack => 1,
            Tool::Solar | Tool::Wind | Tool::CoalPlant | Tool::Nuclear | Tool::Battery => 2,
            Tool::Smelter | Tool::Press | Tool::Assembler | Tool::ChemPlant | Tool::Centrifuge | Tool::Quantum => 3,
            Tool::Market | Tool::House => 4,
        }
    }

    pub fn has_pipes(&self) -> bool {
        matches!(self, Tool::Pipe | Tool::Pump | Tool::Pumpjack | Tool::ChemPlant | Tool::Nuclear | Tool::Market)
    }
}

pub const ITEM_COUNT: usize = 21;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub tool: Tool,
    pub dir: u8,
    pub health: f32,
    pub construction_progress: f32,
    pub processing: f32,
    pub buffer: [i32; ITEM_COUNT],
    pub fluid_type: Option<FluidType>,
    pub fluid_amount: f32,
    pub fuel: f32,
    pub heat: f32,
    pub charge: f32,
    pub render_type: u8, // 0=Default, for Conveyors: 0=Straight, 1=CurvL, 2=CurvR ...
}

impl Cell {
    pub fn new(tool: Tool, dir: u8) -> Self {
        Cell {
            tool, dir, health: 100.0, construction_progress: 1.0, processing: 0.0,
            buffer: [0; ITEM_COUNT],
            fluid_type: None,
            fluid_amount: 0.0,
            fuel: 0.0, heat: 0.0, charge: 0.0, render_type: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub inventory: HashMap<ItemType, i32>,
    pub reach: f32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Player {
            pos: vec2(x, y),
            vel: vec2(0.0, 0.0),
            inventory: HashMap::new(),
            reach: 200.0,
        }
    }
}

impl Default for Player {
    fn default() -> Self { Player::new(0.0, 0.0) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FluidType { Water, CrudeOil }

impl FluidType {
    // method name_pt removed as it was unused
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    IronOre, CopperOre, CoalOre, QuartzOre, SandOre, GoldOre, UraniumOre, Wood,
    IronPlate, CopperPlate, CoalPlate, Silicon, Glass, GoldIngot, UraniumCell,
    CopperWire, Steel, Plastic, CircuitBoard, Processor, AiCore,
}

impl ItemType {
    pub fn to_index(&self) -> usize {
        match self {
            ItemType::IronOre => 0, ItemType::CopperOre => 1, ItemType::CoalOre => 2, ItemType::QuartzOre => 3,
            ItemType::SandOre => 4, ItemType::GoldOre => 5, ItemType::UraniumOre => 6, ItemType::Wood => 7,
            ItemType::IronPlate => 8, ItemType::CopperPlate => 9, ItemType::CoalPlate => 10, ItemType::Silicon => 11,
            ItemType::Glass => 12, ItemType::GoldIngot => 13, ItemType::UraniumCell => 14, ItemType::CopperWire => 15,
            ItemType::Steel => 16, ItemType::Plastic => 17, ItemType::CircuitBoard => 18, ItemType::Processor => 19,
            ItemType::AiCore => 20,
        }
    }

    pub fn from_index(idx: usize) -> ItemType {
        match idx {
            0 => ItemType::IronOre, 1 => ItemType::CopperOre, 2 => ItemType::CoalOre, 3 => ItemType::QuartzOre,
            4 => ItemType::SandOre, 5 => ItemType::GoldOre, 6 => ItemType::UraniumOre, 7 => ItemType::Wood,
            8 => ItemType::IronPlate, 9 => ItemType::CopperPlate, 10 => ItemType::CoalPlate, 11 => ItemType::Silicon,
            12 => ItemType::Glass, 13 => ItemType::GoldIngot, 14 => ItemType::UraniumCell, 15 => ItemType::CopperWire,
            16 => ItemType::Steel, 17 => ItemType::Plastic, 18 => ItemType::CircuitBoard, 19 => ItemType::Processor,
            _ => ItemType::AiCore,
        }
    }

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

    pub fn color(&self) -> Color {
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
    Empty, Water, Tree, Sand, Iron, Copper, Coal, Quartz, Gold, Oil, Uranium, Wasteland, Mountain,
}

impl Terrain {
    pub fn name_pt(&self) -> &'static str {
        match self {
            Terrain::Empty => "VAZIO",
            Terrain::Water => "ÁGUA",
            Terrain::Tree => "ÁRVORE",
            Terrain::Sand => "AREIA",
            Terrain::Iron => "VEIA FERRO",
            Terrain::Copper => "VEIA COBRE",
            Terrain::Coal => "VEIA CARVÃO",
            Terrain::Quartz => "VEIA QUARTZO",
            Terrain::Gold => "VEIA OURO",
            Terrain::Oil => "POÇO PETRÓLEO",
            Terrain::Uranium => "VEIA URÂNIO",
            Terrain::Wasteland => "IRRADIADO",
            Terrain::Mountain => "MONTANHA",
        }
    }

    pub fn priority(&self) -> i32 {
        match self {
            Terrain::Empty => 4, // Grass/Empty
            Terrain::Water => 1,
            Terrain::Tree => 2,
            Terrain::Sand => 3,
            Terrain::Iron => 5,
            Terrain::Copper => 6,
            Terrain::Coal => 7,
            Terrain::Quartz => 8,
            Terrain::Gold => 9,
            Terrain::Oil => 10,
            Terrain::Uranium => 11,
            Terrain::Wasteland => 12,
            Terrain::Mountain => 100, // Imunidade total
            _ => 0,
        }
    }

    pub fn ore_type(&self) -> Option<ItemType> {
        match self {
            Terrain::Iron => Some(ItemType::IronOre),
            Terrain::Copper => Some(ItemType::CopperOre),
            Terrain::Coal => Some(ItemType::CoalOre),
            Terrain::Quartz => Some(ItemType::QuartzOre),
            Terrain::Sand => Some(ItemType::SandOre),
            Terrain::Gold => Some(ItemType::GoldOre),
            Terrain::Uranium => Some(ItemType::UraniumOre),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConveyorItem {
    pub item_type: ItemType,
    pub x: f32,
    pub y: f32,
    pub progress: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Demand {
    pub item: ItemType,
    pub multiplier: i64,
    pub ticks: i32,
}

/// NPC Mafia attack type — carried by rival_event
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NpcAttack {
    /// Force target item price to base*0.2 for N ticks
    Dumping { target: ItemType },
    /// Block sales of advanced items for N ticks
    Embargo { blocked: ItemType },
    /// Add temporary upkeep penalty (population unrest)
    Sabotage { extra_upkeep: i64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RivalEvent {
    pub msg: String,
    pub target: ItemType,
    pub ticks: i32,
    pub attack: Option<NpcAttack>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PowerLink {
    pub u: (i32, i32),
    pub v: (i32, i32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Camera {
    pub gx: i32,
    pub gy: i32,
    pub ox: f32,
    pub oy: f32,
    pub scale: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera { gx: 0, gy: 0, ox: 0.0, oy: 0.0, scale: 1.0 }
    }

    pub fn screen_to_world(&self, sx: f32, sy: f32, sw: f32, sh: f32) -> (i32, f32, i32, f32) {
        let view_x = sx - sw / 2.0;
        let view_y = sy - sh / 2.0;
        
        let tx = self.ox + view_x / self.scale;
        let ty = self.oy + view_y / self.scale;
        
        let dx = (tx / CELL_SIZE).floor() as i32;
        let dy = (ty / CELL_SIZE).floor() as i32;
        
        (self.gx + dx, tx.rem_euclid(CELL_SIZE), self.gy + dy, ty.rem_euclid(CELL_SIZE))
    }
}

#[derive(Serialize, Deserialize)]
pub struct MouseState {
    #[serde(skip)]
    pub is_panning: bool,
    #[serde(skip)]
    pub is_building: bool,
    #[serde(skip)]
    pub is_erasing: bool,
    #[serde(skip)]
    pub start_pan_x: f32,
    #[serde(skip)]
    pub start_pan_y: f32,
    pub init_trans_x: f32,
    pub init_trans_y: f32,
    pub world_col: i32,
    pub world_row: i32,
    pub prev_col: i32,
    pub prev_row: i32,
}

impl MouseState {
    pub fn new() -> Self {
        MouseState {
            is_panning: false, is_building: false, is_erasing: false,
            start_pan_x: 0.0, start_pan_y: 0.0, init_trans_x: 0.0, init_trans_y: 0.0,
            world_col: -1, world_row: -1, prev_col: -1, prev_row: -1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MachineShadow {
    pub tool: Tool,
    pub pos: (i32, i32),
    pub power_gen: f32,
    pub power_cons: f32,
    pub fuel: f32,
}

/// Disjoint Set Union for the power grid.
/// Supports O(α(N)) union on build; full rebuild only on erase.
pub struct PowerDSU {
    pub parent: HashMap<(i32, i32), (i32, i32)>,
    pub rank: HashMap<(i32, i32), u8>,
    /// True if a component in this root has a generator
    pub has_gen: HashMap<(i32, i32), bool>,
}

impl Default for PowerDSU {
    fn default() -> Self {
        PowerDSU { parent: HashMap::new(), rank: HashMap::new(), has_gen: HashMap::new() }
    }
}

impl PowerDSU {
    pub fn make(&mut self, pos: (i32, i32), generates: bool) {
        if !self.parent.contains_key(&pos) {
            self.parent.insert(pos, pos);
            self.rank.insert(pos, 0);
            self.has_gen.insert(pos, generates);
        }
    }

    pub fn find(&mut self, pos: (i32, i32)) -> (i32, i32) {
        if self.parent.get(&pos) == Some(&pos) { return pos; }
        let parent = *self.parent.get(&pos).unwrap_or(&pos);
        let root = self.find(parent);
        self.parent.insert(pos, root); // path compression
        root
    }

    pub fn union(&mut self, a: (i32, i32), b: (i32, i32)) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb { return; }
        let rank_a = *self.rank.get(&ra).unwrap_or(&0);
        let rank_b = *self.rank.get(&rb).unwrap_or(&0);
        let gen_a = *self.has_gen.get(&ra).unwrap_or(&false);
        let gen_b = *self.has_gen.get(&rb).unwrap_or(&false);
        // Union by rank: attach smaller tree under larger
        let (new_root, old_root) = if rank_a >= rank_b { (ra, rb) } else { (rb, ra) };
        self.parent.insert(old_root, new_root);
        if rank_a == rank_b { *self.rank.entry(new_root).or_insert(0) += 1; }
        // Propagate generator flag to combined root
        self.has_gen.insert(new_root, gen_a || gen_b);
    }

    pub fn component_has_gen(&mut self, pos: (i32, i32)) -> bool {
        if !self.parent.contains_key(&pos) { return false; }
        let root = self.find(pos);
        *self.has_gen.get(&root).unwrap_or(&false)
    }

    pub fn clear(&mut self) {
        self.parent.clear();
        self.rank.clear();
        self.has_gen.clear();
    }
}

#[derive(Serialize, Deserialize)]
pub struct GlobalRegistry {
    pub machines: Vec<MachineShadow>,
    pub indices: HashMap<(i32, i32), usize>,
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    // Transient sector pool — regenerated from seed on load
    #[serde(skip)]
    pub sectors: HashMap<(i32, i32), SectorHandle>,
    #[serde(skip)]
    pub pool: SlotMap<SectorHandle, Sector>,
    #[serde(skip)]
    pub active_pool: Vec<SectorHandle>,
    #[serde(skip)]
    pub global_power_links: Vec<PowerLink>,
    pub registry: GlobalRegistry,
    pub money: i64,
    pub inventory: HashMap<ItemType, i32>,
    #[serde(skip)]
    pub inventory_cache: Vec<(ItemType, i32)>,
    #[serde(skip)]
    pub ui_money_str: String,
    #[serde(skip)]
    pub ui_income_str: String,
    #[serde(skip)]
    pub ui_power_str: String,
    #[serde(skip)]
    pub ui_pop_str: String,
    pub prices: HashMap<ItemType, i64>,
    pub price_trends: HashMap<ItemType, i8>,
    pub sales_counter: HashMap<ItemType, i32>,
    pub pending_sales: i64,
    pub demand: Demand,
    pub rival_event: Option<RivalEvent>,
    pub time_of_day: f32,
    pub camera: Camera,
    pub mouse: MouseState,
    pub selected_tool: Tool,
    pub selected_category: usize,
    #[serde(skip)]
    pub stats_msg: String,
    #[serde(skip)]
    pub stats_msg_timer: f32,
    pub powered: HashSet<(i32, i32)>,
    pub income: i64,
    pub pop: i32,
    pub unpowered: i32,
    pub power_gen: f32,
    pub power_cons: f32,
    // Pre-aggregated counters — maintained incrementally at build/erase time
    pub total_upkeep: i64,
    pub total_pop: i32,
    // Dirty flag: set to true when power network topology changes
    pub power_dirty: bool,
    // Dense fluid node registry: O(N_pipes) fluid simulation
    pub fluid_nodes: Vec<(i32, i32)>,
    pub fluid_indices: HashMap<(i32, i32), usize>,
    // Power DSU: O(α(N)) connectivity — transient, rebuilt when power_dirty
    #[serde(skip)]
    pub power_dsu: PowerDSU,
    // NPC Mafia AI state
    pub npc_cooldown: i32,        // ticks until AI can attack again
    pub upkeep_penalty: i64,      // temporary upkeep drain from Sabotage
    pub embargo_item: Option<ItemType>, // blocked item type (from Embargo)
    pub embargo_ticks: i32,       // remaining embargo duration
    pub last_econ_tick: f64,
    pub last_market_tick: f64,
    pub last_industry_tick: f64,
    pub last_fluid_tick: f64,
    pub player_id: String,
    #[serde(skip)]
    pub other_cursors: Vec<(f32, f32)>,
    #[serde(skip)]
    pub last_cursor_sync: f64,
    #[serde(skip)]
    pub last_cursor_fetch: f64,
    #[serde(skip)]
    pub textures: Option<Resources>,
    pub username: String,
    // Player position saved separately (Vec2 is not Serialize)
    pub player_x: f32,
    pub player_y: f32,
    #[serde(skip)]
    pub local_player: Player,
    #[serde(skip)]
    pub other_players: HashMap<String, Vec2>,
    #[serde(skip)]
    pub npcs: Vec<NpcInfo>,
    #[serde(skip)]
    pub help_open: bool,
    #[serde(skip)]
    pub help_tab: usize, // 0=Buildings, 1=Recipes, 2=Chains, 3=Power
    pub seed: u64,
    #[serde(skip)]
    pub grid_dirty: bool,
    #[serde(skip)]
    pub last_sync_time: f64,
    #[serde(skip)]
    pub forge: crystal_forge::Generator,
}

#[derive(Clone, Debug)]
pub struct Resources {
    pub grass: Texture2D,
    pub water: Texture2D,
    pub tree: Texture2D,
    pub sand: Texture2D,
    pub iron: Texture2D,
    pub copper: Texture2D,
    pub coal: Texture2D,
    pub quartz: Texture2D,
    pub gold: Texture2D,
    pub oil: Texture2D,
    pub uranium: Texture2D,
    pub wasteland: Texture2D,
    pub mountain: Texture2D,
    pub transitions: HashMap<(Terrain, Terrain, u8), Texture2D>,
    pub sprites: SpriteBank,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sector {
    pub terrain: Vec<Terrain>,
    pub grid: Vec<Option<Cell>>,
    pub wind_map: Vec<f32>,
    pub items: Vec<ConveyorItem>,
    #[serde(skip)]
    pub next_items: Vec<ConveyorItem>,
    #[serde(skip)]
    pub occupancy: Vec<f32>,
    pub power_links: Vec<PowerLink>,
    pub state: SectorState,
}

impl Sector {
    pub fn new() -> Self {
        Sector {
            terrain: vec![Terrain::Empty; SECTOR_SIZE * SECTOR_SIZE],
            grid: vec![None; SECTOR_SIZE * SECTOR_SIZE],
            wind_map: vec![0.0; SECTOR_SIZE * SECTOR_SIZE],
            items: Vec::new(),
            next_items: Vec::new(),
            occupancy: vec![1.0; SECTOR_SIZE * SECTOR_SIZE],
            power_links: Vec::new(),
            state: SectorState::LOCKED,
        }
    }
}

impl GameState {
    pub fn new() -> Self {
        let mut prices = HashMap::new();
        for it in ItemType::all() {
            prices.insert(*it, it.base_price());
        }
        GameState {
            sectors: HashMap::new(),
            pool: SlotMap::with_key(),
            active_pool: Vec::new(),
            global_power_links: Vec::new(),
            powered: HashSet::new(),
            money: 50000,
            inventory: HashMap::new(),
            prices,
            price_trends: HashMap::new(),
            sales_counter: HashMap::new(),
            pending_sales: 0,
            demand: Demand { item: ItemType::CopperWire, multiplier: 3, ticks: 15 },
            rival_event: None,
            time_of_day: 600.0,
            camera: Camera::new(),
            mouse: MouseState::new(),
            selected_tool: Tool::ConveyorIron,
            selected_category: 0,
            stats_msg: String::new(),
            stats_msg_timer: 0.0,
            income: 0,
            pop: 0,
            unpowered: 0,
            power_gen: 0.0,
            power_cons: 0.0,
            total_upkeep: 0,
            total_pop: 0,
            power_dirty: true,
            fluid_nodes: Vec::new(),
            fluid_indices: HashMap::new(),
            power_dsu: PowerDSU::default(),
            npc_cooldown: 0,
            upkeep_penalty: 0,
            embargo_item: None,
            embargo_ticks: 0,
            last_econ_tick: 0.0,
            last_market_tick: 0.0,
            last_industry_tick: 0.0,
            last_fluid_tick: 0.0,
            player_id: uuid::Uuid::new_v4().to_string(),
            other_cursors: Vec::new(),
            last_cursor_sync: 0.0,
            last_cursor_fetch: 0.0,
            textures: None,
            local_player: Player::new(SECTOR_SIZE as f32 * CELL_SIZE / 2.0, SECTOR_SIZE as f32 * CELL_SIZE / 2.0),
            player_x: SECTOR_SIZE as f32 * CELL_SIZE / 2.0,
            player_y: SECTOR_SIZE as f32 * CELL_SIZE / 2.0,
            other_players: HashMap::new(),
            npcs: Vec::new(),
            help_open: false,
            help_tab: 0,
            username: String::new(),
            seed: 0,
            grid_dirty: false,
            last_sync_time: 0.0,
            forge: crystal_forge::Generator::new(0.0),
            registry: GlobalRegistry { machines: Vec::new(), indices: HashMap::new() },
            inventory_cache: Vec::new(),
            ui_money_str: "$0".to_string(),
            ui_income_str: "$0/tick".to_string(),
            ui_power_str: "0W / 0W".to_string(),
            ui_pop_str: "Pop: 0".to_string(),
        }
    }

    pub fn set_msg(&mut self, msg: &str) {
        self.stats_msg = msg.to_string();
        self.stats_msg_timer = 3.0;
    }

    pub fn idx(col: i32, row: i32) -> Option<usize> {
        if col >= 0 && col < SECTOR_SIZE as i32 && row >= 0 && row < SECTOR_SIZE as i32 {
            Some(row as usize * SECTOR_SIZE + col as usize)
        } else { None }
    }

    pub fn world_to_sector(gx: i32, gy: i32) -> ((i32, i32), (i32, i32)) {
        let sx = if gx >= 0 { gx / SECTOR_SIZE as i32 } else { (gx - (SECTOR_SIZE as i32 - 1)) / SECTOR_SIZE as i32 };
        let sy = if gy >= 0 { gy / SECTOR_SIZE as i32 } else { (gy - (SECTOR_SIZE as i32 - 1)) / SECTOR_SIZE as i32 };
        let lx = gx.rem_euclid(SECTOR_SIZE as i32);
        let ly = gy.rem_euclid(SECTOR_SIZE as i32);
        ((sx, sy), (lx, ly))
    }

    pub fn get_sector(&self, sx: i32, sy: i32) -> Option<&Sector> {
        self.sectors.get(&(sx, sy)).map(|&h| &self.pool[h])
    }

    pub fn get_terrain_at(&self, gx: i32, gy: i32) -> Terrain {
        let ((sx, sy), (lx, ly)) = Self::world_to_sector(gx, gy);
        self.get_sector(sx, sy).map(|s| s.terrain[ly as usize * SECTOR_SIZE + lx as usize]).unwrap_or(Terrain::Empty)
    }

    pub fn get_cell_at(&self, gx: i32, gy: i32) -> Option<&Cell> {
        let ((sx, sy), (lx, ly)) = Self::world_to_sector(gx, gy);
        self.get_sector(sx, sy).and_then(|s| s.grid[ly as usize * SECTOR_SIZE + lx as usize].as_ref())
    }

    pub fn get_sector_mut(&mut self, sx: i32, sy: i32) -> Option<&mut Sector> {
        self.sectors.get(&(sx, sy)).map(|&h| &mut self.pool[h])
    }

    pub fn set_cell_at(&mut self, gx: i32, gy: i32, cell: Option<Cell>) {
        let ((sx, sy), (lx, ly)) = Self::world_to_sector(gx, gy);
        if let Some(&h) = self.sectors.get(&(sx, sy)) {
            let sector = &mut self.pool[h];
            if sector.state == SectorState::ACTIVE || sector.state == SectorState::UNLOCKING {
                sector.grid[ly as usize * SECTOR_SIZE + lx as usize] = cell;
                self.grid_dirty = true;
            }
        }
    }
}
