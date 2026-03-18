use std::collections::{HashMap, HashSet};
use crate::constants::*;
use crate::network::NpcInfo;
use macroquad::prelude::{Texture2D, Vec2, vec2};
use serde::{Serialize, Deserialize};

pub mod crystal_forge;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub tool: Tool,
    pub dir: u8,
    pub health: f32,
    pub construction_progress: f32, // 0.0 to 1.0 (1.0 = finished)
    pub processing: f32,
    pub buffer: HashMap<ItemType, i32>,
    pub fluid_type: Option<FluidType>,
    pub fluid_amount: f32,
    pub fuel: f32,
    pub heat: f32,
    pub charge: f32,
}

impl Cell {
    pub fn new(tool: Tool, dir: u8) -> Self {
        Cell {
            tool, dir, health: 100.0, construction_progress: 1.0, processing: 0.0,
            buffer: HashMap::new(),
            fluid_type: None,
            fluid_amount: 0.0,
            fuel: 0.0, heat: 0.0, charge: 0.0,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FluidType { Water, CrudeOil }

impl FluidType {
    // method name_pt removed as it was unused
}

#[derive(Clone, Debug)]
pub struct ConveyorItem {
    pub item_type: ItemType,
    pub x: f32,
    pub y: f32,
    pub progress: f32,
}

#[derive(Clone, Debug)]
pub struct Demand {
    pub item: ItemType,
    pub multiplier: i64,
    pub ticks: i32,
}

#[derive(Clone, Debug)]
pub struct RivalEvent {
    pub msg: String,
    pub target: ItemType,
    pub ticks: i32,
}

#[derive(Clone, Debug)]
pub struct PowerLink {
    pub u: usize,
    pub v: usize,
}

pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera { x: 0.0, y: 0.0, scale: 1.0 }
    }

    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        ((sx - self.x) / self.scale, (sy - self.y) / self.scale)
    }
}

pub struct MouseState {
    pub is_panning: bool,
    pub is_building: bool,
    pub is_erasing: bool,
    pub start_pan_x: f32,
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

pub struct GameState {
    pub terrain: Vec<Terrain>,
    pub wind_map: Vec<f32>,
    pub grid: Vec<Option<Cell>>,
    pub items: Vec<ConveyorItem>,
    pub powered: HashSet<usize>,
    pub power_links: Vec<PowerLink>,
    pub money: i64,
    pub inventory: HashMap<ItemType, i32>,
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
    pub stats_msg: String,
    pub stats_msg_timer: f32,
    pub income: i64,
    pub pop: i32,
    pub unpowered: i32,
    pub power_gen: f32,
    pub power_cons: f32,
    // Timers
    pub last_econ_tick: f64,
    pub last_market_tick: f64,
    pub last_industry_tick: f64,
    pub last_fluid_tick: f64,
    // Multiplayer
    pub player_id: String,
    pub other_cursors: Vec<(f32, f32)>,
    pub last_cursor_sync: f64,
    pub last_cursor_fetch: f64,
    pub textures: Option<Resources>,
    // Physical Player & Multi
    pub username: String,
    pub local_player: Player,
    pub other_players: HashMap<String, Vec2>,
    pub npcs: Vec<NpcInfo>,
    pub help_open: bool,
    pub help_tab: usize, // 0=Buildings, 1=Recipes, 2=Chains, 3=Power
    pub seed: u64,
    pub grid_dirty: bool,
    pub last_sync_time: f64,
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
    pub transitions: HashMap<(Terrain, Terrain, u8), Texture2D>,
}

impl GameState {
    pub fn new() -> Self {
        let mut prices = HashMap::new();
        for it in ItemType::all() {
            prices.insert(*it, it.base_price());
        }
        GameState {
            terrain: vec![Terrain::Empty; GRID_SIZE * GRID_SIZE],
            wind_map: vec![0.0; GRID_SIZE * GRID_SIZE],
            grid: vec![None; GRID_SIZE * GRID_SIZE],
            items: Vec::new(),
            powered: HashSet::new(),
            power_links: Vec::new(),
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
            selected_tool: Tool::Conveyor,
            selected_category: 0,
            stats_msg: String::new(),
            stats_msg_timer: 0.0,
            income: 0,
            pop: 0,
            unpowered: 0,
            power_gen: 0.0,
            power_cons: 0.0,
            last_econ_tick: 0.0,
            last_market_tick: 0.0,
            last_industry_tick: 0.0,
            last_fluid_tick: 0.0,
            player_id: uuid::Uuid::new_v4().to_string(),
            other_cursors: Vec::new(),
            last_cursor_sync: 0.0,
            last_cursor_fetch: 0.0,
            textures: None,
            local_player: Player::new(GRID_SIZE as f32 * CELL_SIZE / 2.0, GRID_SIZE as f32 * CELL_SIZE / 2.0),
            other_players: HashMap::new(),
            npcs: Vec::new(),
            help_open: false,
            help_tab: 0,
            username: String::new(),
            seed: 0,
            grid_dirty: false,
            last_sync_time: 0.0,
            forge: crystal_forge::Generator::new(0.0),
        }
    }

    pub fn set_msg(&mut self, msg: &str) {
        self.stats_msg = msg.to_string();
        self.stats_msg_timer = 3.0;
    }

    pub fn idx(col: i32, row: i32) -> Option<usize> {
        if col >= 0 && col < GRID_SIZE as i32 && row >= 0 && row < GRID_SIZE as i32 {
            Some(row as usize * GRID_SIZE + col as usize)
        } else { None }
    }
}
