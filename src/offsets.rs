// SPDX-FileCopyrightText: Copyright 2025-2026 misterj05 & SBDeaths Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Result, bail};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Offsets {
    pub x86_base_offset: Option<usize>,
    pub x86_offsets: Option<Vec<usize>>,
    pub base_offset: Option<usize>,
    pub offsets: Option<Vec<usize>>,
}

pub type OffsetMap = HashMap<String, Offsets>;

#[derive(Debug, Clone, Deserialize)]
pub struct OffsetsFile {
    pub compat: Option<u16>,
    pub offsets: Option<OffsetMap>,
}

pub fn get_offsets_file<P: AsRef<Path>>(data_path: P) -> Result<OffsetsFile> {
    let data_path = data_path.as_ref();
    let file = File::open(data_path.join("offsets.yaml"))?;
    let reader = BufReader::new(file);
    let offsets_file: OffsetsFile = serde_yml::de::from_reader(reader)?;
    return Ok(offsets_file);
}

pub fn get_offsets<P: AsRef<Path>>(data_path: P) -> Result<Option<OffsetMap>> {
    let data_path = data_path.as_ref();
    let file = File::open(data_path.join("offsets.yaml"))?;
    let reader = BufReader::new(file);
    let offsets_file: OffsetsFile = serde_yml::de::from_reader(reader)?;
    if let Some(o) = offsets_file.offsets {
        return Ok(Some(o));
    } else {
        return Ok(None);
    }
}

pub fn get_offsets_compat<P: AsRef<Path>>(data_path: P) -> Result<Option<u16>> {
    let data_path = data_path.as_ref();
    let file = File::open(data_path.join("offsets.yaml"))?;
    let reader = BufReader::new(file);
    let offsets_file: OffsetsFile = serde_yml::de::from_reader(reader)?;
    if let Some(c) = offsets_file.compat {
        return Ok(Some(c));
    } else {
        return Ok(None);
    }
}

pub fn get_offsets_from_name(
    offset_map: &OffsetMap,
    name: &str,
) -> Option<Offsets> {
    for e in offset_map {
        if e.0 == name {
            return Some(e.1.clone());
        }
    }
    None
}
