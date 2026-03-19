use crate::types::*;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};

const SAVE_PATH: &str = "save.bin";
const VERSION: u8 = 1;

pub fn save(state: &GameState) -> Result<(), String> {
    let file = File::create(SAVE_PATH).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(file);
    
    // Write version byte
    writer.write_all(&[VERSION]).map_err(|e| e.to_string())?;
    
    // Serialize GameState
    bincode::serialize_into(writer, state).map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn load() -> Result<GameState, String> {
    let file = File::open(SAVE_PATH).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);
    
    // Read version byte
    let mut version = [0u8; 1];
    reader.read_exact(&mut version).map_err(|e| e.to_string())?;
    
    if version[0] != VERSION {
        return Err(format!("Versão incompatível: esperado {}, encontrado {}", VERSION, version[0]));
    }
    
    // Deserialize GameState
    let state: GameState = bincode::deserialize_from(reader).map_err(|e| e.to_string())?;
    
    Ok(state)
}
