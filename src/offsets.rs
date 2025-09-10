use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct GameOffsets {
    pub x86_base_offset: Option<usize>,
    pub x86_offsets: Option<Vec<usize>>,
    pub base_offset: Option<usize>,
    pub offsets: Option<Vec<usize>>,
}

pub type OffsetMap = HashMap<String, GameOffsets>;

pub fn deserialize_offsets() -> Result<OffsetMap> {
    let file = File::open("offsets.yaml")?;
    let reader = BufReader::new(file);

    let offset_map: OffsetMap = serde_yml::de::from_reader(reader)?;

    Ok(offset_map)
}

pub fn get_offsets_from_name(offset_map: OffsetMap, name: &str) -> Option<GameOffsets> {
    for e in offset_map {
        if e.0 == name {
            return Some(e.1);
        }
    }
    None
}
