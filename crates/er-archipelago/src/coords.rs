use std::collections::HashMap;
use std::sync::OnceLock;

use eldenring::cs::{BlockId, ChrInsExt, PlayerIns};
use eldenring::position::PositionDelta;
use serde::{Deserialize, Serialize};
use shared::utils::LazyFileReader;

const LOG_LOCATION_CHANGES: bool = false;

/// Returns the player's current location and global map position for the tracker.
pub unsafe fn get_position() -> Option<GlobalPosition> {
    let Some(offsets) = load_offsets() else {
        return None;
    };
    let Ok(player) = (unsafe { PlayerIns::local_player() }) else {
        return None;
    };
    let block_id = player.block_id_origin();
    let block_position = player.block_position;
    if block_id == BlockId::none() {
        return None;
    }
    let play_region_id = if (1000000..10000000).contains(&player.play_region_id) {
        player.play_region_id
    } else if block_id == BlockId::from_parts(12, 2, 0, 0) {
        // Fake a play region for Siofra belfry only currently
        (block_id.area() as u32 * 100 + block_id.block() as u32) * 1000 + 999
    } else {
        return None;
    };
    if LOG_LOCATION_CHANGES {
        static LOG: shared::utils::DiffLogger = shared::utils::DiffLogger::new();
        LOG.log(&format!("Region {} Map {}", play_region_id, block_id));
    }
    let (overworld_id, offset) = if block_id.area() == 60 || block_id.area() == 61 {
        match block_id.index() % 10 {
            2 => (BlockId::from_parts(block_id.area(), block_id.block() * 4, block_id.region() * 4, 0), PositionDelta(384.0, 0.0, 384.0)),
            1 => (BlockId::from_parts(block_id.area(), block_id.block() * 2, block_id.region() * 2, 0), PositionDelta(128.0, 0.0, 128.0)),
            _ => (block_id, PositionDelta(0.0, 0.0, 0.0)),
        }
    } else {
        let Some(info) = offsets.get(&block_id.to_string()) else {
            return None;
        };
        (BlockId::from_parts(info.area, info.grid_x, info.grid_z, 0), PositionDelta(info.x, info.y, info.z))
    };
    let tile_offset = PositionDelta(
        (overworld_id.block() as f32 - 28.0) * 256.0 + 128.0,
        0.0,
        (overworld_id.region() as f32 - 24.0) * 256.0 + 128.0);
    let global_pos = block_position + offset + tile_offset;
    let layer = if block_id.area() == 12 {
        1
    } else if overworld_id.area() == 60 {
        0
    } else {
        2
    };
    Some(GlobalPosition { region: play_region_id, layer: layer, x: global_pos.x as i32, z: global_pos.z as i32 })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalPosition {
    pub region: u32,
    pub layer: u8,
    pub x: i32,
    pub z: i32,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
struct DungeonInfo {
    pub area: u8,
    pub grid_x: u8,
    pub grid_z: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

fn load_offsets() -> Option<&'static HashMap<String, DungeonInfo>> {
    static INSTANCE: OnceLock<HashMap<String, DungeonInfo>> = OnceLock::new();
    static READER: LazyFileReader = LazyFileReader::new("offsets.json");
    if let Some(data) = READER.take_data() {
        match json::from_slice::<HashMap<String, DungeonInfo>>(&data) {
            Ok(config) => INSTANCE.set(config).unwrap_or(()),
            Err(err) => {
                log::warn!("Error reading map offsets: {}", err);
            }
        }
    }
    INSTANCE.get()
}
