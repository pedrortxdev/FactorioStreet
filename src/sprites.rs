use std::collections::HashMap;
use macroquad::prelude::*;
use crate::constants::{Tool, ItemType};

// ── HUD sprites (embedded at compile time) ───────────────────────────────────
macro_rules! hud_bytes {
    ($name:expr) => { include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/sprites/hud/", $name)) };
}
macro_rules! mat_bytes {
    ($name:expr) => { include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/sprites/materiais/", $name)) };
}
macro_rules! ore_bytes {
    ($name:expr) => { include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/sprites/ores/", $name)) };
}
macro_rules! bar_bytes {
    ($name:expr) => { include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/sprites/barras/", $name)) };
}

/// Decode a PNG byte slice → `image::RgbaImage`
fn decode_png(bytes: &[u8]) -> Option<image::RgbaImage> {
    image::load_from_memory(bytes).ok().map(|img| img.into_rgba8())
}

/// Resize an `RgbaImage` to exact dimensions using nearest-neighbour.
fn resize_nearest(src: &image::RgbaImage, w: u32, h: u32) -> image::RgbaImage {
    image::imageops::resize(src, w, h, image::imageops::FilterType::Nearest)
}

/// Remove red-dominant background pixels → set alpha to 0.
/// A pixel is considered "red background" when R is clearly dominant over G and B
/// and the overall brightness is low-to-medium (typical sprite-sheet background).
fn remove_red_bg(img: &mut image::RgbaImage) {
    for px in img.pixels_mut() {
        let [r, g, b, _a] = px.0;
        let r = r as f32;
        let g = g as f32;
        let b = b as f32;
        // Classic chroma-key: red is at least 1.5× both green and blue
        // and green+blue are relatively low (not a warm lit object like gold/copper)
        if r > g * 1.5 && r > b * 1.5 && g < 100.0 && b < 100.0 {
            px.0[3] = 0; // transparent
        }
    }
}

/// Convert an `image::RgbaImage` into a macroquad `Texture2D`.
fn to_texture(img: &image::RgbaImage) -> Texture2D {
    let (w, h) = img.dimensions();
    let bytes: &[u8] = img.as_raw();
    let mq_img = Image {
        bytes: bytes.to_vec(),
        width: w as u16,
        height: h as u16,
    };
    let tex = Texture2D::from_image(&mq_img);
    tex.set_filter(FilterMode::Nearest);
    tex
}

/// Load & resize a PNG to `size × size`, returning a `Texture2D`.
fn load_resized(bytes: &[u8], size: u32) -> Option<Texture2D> {
    let img = decode_png(bytes)?;
    let resized = resize_nearest(&img, size, size);
    Some(to_texture(&resized))
}

/// Load & resize a PNG, then remove red background, returning a `Texture2D`.
fn load_resized_no_red(bytes: &[u8], size: u32) -> Option<Texture2D> {
    let img = decode_png(bytes)?;
    let mut resized = resize_nearest(&img, size, size);
    remove_red_bg(&mut resized);
    Some(to_texture(&resized))
}

#[derive(Clone, Debug)]
pub struct SpriteBank {
    pub hud: HashMap<Tool, Texture2D>,
    pub items: HashMap<ItemType, Texture2D>,
}

impl SpriteBank {
    pub fn load() -> Self {
        let mut hud: HashMap<Tool, Texture2D> = HashMap::new();
        let mut items: HashMap<ItemType, Texture2D> = HashMap::new();

        // ── HUD sprites (64×64) ──────────────────────────────────────────────
        // Infrastructure
        if let Some(t) = load_resized(hud_bytes!("esteirahud.png"), 64)  { hud.insert(Tool::Conveyor, t); }
        if let Some(t) = load_resized(hud_bytes!("pipehud.png"), 64)      { hud.insert(Tool::Pipe, t); }
        if let Some(t) = load_resized(hud_bytes!("postehud.png"), 64)     { hud.insert(Tool::Node, t); }
        if let Some(t) = load_resized(hud_bytes!("ruahud.png"), 64)       { hud.insert(Tool::Street, t); }
        if let Some(t) = load_resized(hud_bytes!("galpaohud.png"), 64)    { hud.insert(Tool::Warehouse, t); }
        if let Some(t) = load_resized(hud_bytes!("repararhud.png"), 64)   { hud.insert(Tool::Repair, t); }
        if let Some(t) = load_resized(hud_bytes!("apagarhud.png"), 64)    { hud.insert(Tool::Eraser, t); }
        // Extraction
        if let Some(t) = load_resized(hud_bytes!("mineradorahud.png"), 64){ hud.insert(Tool::Miner, t); }
        if let Some(t) = load_resized(hud_bytes!("pumphud.png"), 64)      { hud.insert(Tool::Pump, t); }
        if let Some(t) = load_resized(hud_bytes!("extratorahud.png"), 64) { hud.insert(Tool::Pumpjack, t); }
        if let Some(t) = load_resized(hud_bytes!("madeireirahud.png"), 64){ hud.insert(Tool::Lumberjack, t); }
        // Energy
        if let Some(t) = load_resized(hud_bytes!("painelsolarhud.png"), 64){ hud.insert(Tool::Solar, t); }
        if let Some(t) = load_resized(hud_bytes!("eolicahud.png"), 64)    { hud.insert(Tool::Wind, t); }
        if let Some(t) = load_resized(hud_bytes!("coalplanthud.png"), 64) { hud.insert(Tool::CoalPlant, t); }
        if let Some(t) = load_resized(hud_bytes!("usinanuclearhud.png"), 64){ hud.insert(Tool::Nuclear, t); }
        if let Some(t) = load_resized(hud_bytes!("bateriahud.png"), 64)   { hud.insert(Tool::Battery, t); }
        // Factories
        if let Some(t) = load_resized(hud_bytes!("smelterhud.png"), 64)   { hud.insert(Tool::Smelter, t); }
        if let Some(t) = load_resized(hud_bytes!("prensahud.png"), 64)    { hud.insert(Tool::Press, t); }
        if let Some(t) = load_resized(hud_bytes!("montadorahud.png"), 64) { hud.insert(Tool::Assembler, t); }
        if let Some(t) = load_resized(hud_bytes!("quimicahud.png"), 64)   { hud.insert(Tool::ChemPlant, t); }
        if let Some(t) = load_resized(hud_bytes!("centrifogahud.png"), 64){ hud.insert(Tool::Centrifuge, t); }
        if let Some(t) = load_resized(hud_bytes!("processadorquanticohud.png"), 64){ hud.insert(Tool::Quantum, t); }
        // Advanced
        if let Some(t) = load_resized(hud_bytes!("mercadohud.png"), 64)   { hud.insert(Tool::Market, t); }
        if let Some(t) = load_resized(hud_bytes!("casahud.png"), 64)      { hud.insert(Tool::House, t); }

        // ── Ore sprites (32×32, no red bg) ───────────────────────────────────
        if let Some(t) = load_resized_no_red(ore_bytes!("ferroore.png"), 32)     { items.insert(ItemType::IronOre, t); }
        if let Some(t) = load_resized_no_red(ore_bytes!("cobreore.png"), 32)     { items.insert(ItemType::CopperOre, t); }
        if let Some(t) = load_resized_no_red(ore_bytes!("carvao.png"), 32)       { items.insert(ItemType::CoalOre, t); }
        if let Some(t) = load_resized_no_red(ore_bytes!("aluminioore.png"), 32)  { items.insert(ItemType::QuartzOre, t); } // Quartz mapped to aluminium ore visually
        if let Some(t) = load_resized_no_red(ore_bytes!("goldore.png"), 32)      { items.insert(ItemType::GoldOre, t); }
        if let Some(t) = load_resized_no_red(ore_bytes!("uranioore.png"), 32)    { items.insert(ItemType::UraniumOre, t); }
        if let Some(t) = load_resized_no_red(ore_bytes!("chumboore.png"), 32)    { items.insert(ItemType::SandOre, t); } // Sand mapped to chumbo visually
        if let Some(t) = load_resized_no_red(ore_bytes!("prataore.png"), 32)     { items.insert(ItemType::Wood, t); } // Wood not in ores - skip or use placeholder

        // ── Barra sprites (32×32, no red bg) ─────────────────────────────────
        if let Some(t) = load_resized_no_red(bar_bytes!("barraferro.png"), 32)   { items.insert(ItemType::IronPlate, t); }
        if let Some(t) = load_resized_no_red(bar_bytes!("barracobre.png"), 32)   { items.insert(ItemType::CopperPlate, t); }
        if let Some(t) = load_resized_no_red(bar_bytes!("barraaco.png"), 32)     { items.insert(ItemType::Steel, t); }
        if let Some(t) = load_resized_no_red(bar_bytes!("barraouro.png"), 32)    { items.insert(ItemType::GoldIngot, t); }
        if let Some(t) = load_resized_no_red(bar_bytes!("barraplastico.png"), 32){ items.insert(ItemType::Plastic, t); }
        if let Some(t) = load_resized_no_red(bar_bytes!("barratitanio.png"), 32) { items.insert(ItemType::Silicon, t); }
        if let Some(t) = load_resized_no_red(bar_bytes!("barraaluminio.png"), 32){ items.insert(ItemType::UraniumCell, t); }
        if let Some(t) = load_resized_no_red(bar_bytes!("barrabronze.png"), 32)  { items.insert(ItemType::CopperWire, t); }
        if let Some(t) = load_resized_no_red(bar_bytes!("barrachumbo.png"), 32)  { items.insert(ItemType::CoalPlate, t); }
        if let Some(t) = load_resized_no_red(bar_bytes!("barraprata.png"), 32)   { items.insert(ItemType::CircuitBoard, t); }

        // ── Material / placa sprites (32×32) ──────────────────────────────────
        if let Some(t) = load_resized(mat_bytes!("placaferro.png"), 32)    { items.entry(ItemType::IronPlate).or_insert(t); }
        if let Some(t) = load_resized(mat_bytes!("placacobre.png"), 32)    { items.entry(ItemType::CopperPlate).or_insert(t); }
        if let Some(t) = load_resized(mat_bytes!("placaaco.png"), 32)      { items.entry(ItemType::Steel).or_insert(t); }
        if let Some(t) = load_resized(mat_bytes!("placaouro.png"), 32)     { items.entry(ItemType::GoldIngot).or_insert(t); }
        if let Some(t) = load_resized(mat_bytes!("placaplastico.png"), 32) { items.entry(ItemType::Plastic).or_insert(t); }
        if let Some(t) = load_resized(mat_bytes!("placatitanio.png"), 32)  { items.entry(ItemType::Silicon).or_insert(t); }
        if let Some(t) = load_resized(mat_bytes!("placaaluminio.png"), 32) { items.entry(ItemType::UraniumCell).or_insert(t); }
        if let Some(t) = load_resized(mat_bytes!("placachumbo.png"), 32)   { items.entry(ItemType::CoalPlate).or_insert(t); }
        if let Some(t) = load_resized(mat_bytes!("placabronze.png"), 32)   { items.entry(ItemType::CopperWire).or_insert(t); }
        if let Some(t) = load_resized(mat_bytes!("placaprata.png"), 32)    { items.entry(ItemType::CircuitBoard).or_insert(t); }
        if let Some(t) = load_resized(mat_bytes!("placatungstenio.png"), 32){ items.entry(ItemType::Processor).or_insert(t); }
        if let Some(t) = load_resized(mat_bytes!("placamadeira.png"), 32)  { items.entry(ItemType::Wood).or_insert(t); }

        SpriteBank { hud, items }
    }
}
