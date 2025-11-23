use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GameOffsets {
    pub x86_base_offset: Option<usize>,
    pub x86_offsets: Option<Vec<usize>>,
    pub base_offset: Option<usize>,
    pub offsets: Option<Vec<usize>>,
}

pub type OffsetMap = HashMap<String, GameOffsets>;

pub fn deserialize_offsets<P: AsRef<Path>>(data_path: P) -> Result<OffsetMap> {
    let data_path = data_path.as_ref();

    let file = File::open(data_path.join("offsets.yaml"))?;
    let reader = BufReader::new(file);

    let offset_map: OffsetMap = serde_yml::de::from_reader(reader)?;

    Ok(offset_map)
}

pub fn get_offsets_from_name(
    offset_map: &OffsetMap,
    name: &str,
) -> Option<GameOffsets> {
    for e in offset_map {
        if e.0 == name {
            return Some(e.1.clone());
        }
    }
    None
}
