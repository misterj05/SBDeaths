// SPDX-FileCopyrightText: Copyright 2025-2026 misterj05 & SBDeaths Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Result, bail};
use libmem::{Arch, Module, Process, read_memory_ex, sig_scan_ex};
use noyalib::compat::serde_yaml;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Offsets {
    pub x86_sig: Option<String>,
    pub x86_sig_offset: Option<u8>,
    pub x86_sig_rip: Option<u8>,
    pub x86_base_offset: Option<usize>,
    pub x86_offsets: Option<Vec<usize>>,
    pub sig: Option<String>,
    pub sig_offset: Option<u8>,
    pub sig_rip: Option<u8>,
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
    let offsets_file: OffsetsFile = serde_yaml::from_reader(reader)?;
    return Ok(offsets_file);
}

pub fn get_offsets<P: AsRef<Path>>(data_path: P) -> Result<Option<OffsetMap>> {
    let data_path = data_path.as_ref();
    let file = File::open(data_path.join("offsets.yaml"))?;
    let reader = BufReader::new(file);
    let offsets_file: OffsetsFile = serde_yaml::from_reader(reader)?;
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
    let offsets_file: OffsetsFile = serde_yaml::from_reader(reader)?;
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

pub fn find_base_offset(
    offsets: &Offsets,
    game_process: &Process,
    main_module: &Module,
    scan_size: usize,
) -> Result<usize> {
    match game_process.arch {
        Arch::X86 => match &offsets.x86_sig {
            Some(sig) => {
                match sig_scan_ex(
                    &game_process,
                    &sig,
                    main_module.base,
                    scan_size,
                ) {
                    Some(sig_addr) => {
                        let offset = match offsets.sig_offset {
                            Some(so) => read_memory_ex::<u32>(
                                &game_process,
                                sig_addr + so as usize,
                            ),
                            None => {
                                read_memory_ex::<u32>(&game_process, sig_addr)
                            }
                        };
                        if let Some(o) = offset {
                            #[cfg(debug_assertions)]
                            println!(
                                "{}:{} DEBUG: {}",
                                file!(),
                                line!(),
                                "USING SIGNATURE"
                            );
                            if let Some(sr) = offsets.sig_rip {
                                return Ok(sig_addr + sr as usize + o as usize
                                    - main_module.base);
                            } else {
                                return Ok(
                                    sig_addr + o as usize - main_module.base
                                );
                            }
                        } else {
                            bail!("fail")
                        }
                    }
                    None => {
                        if let Some(o) = offsets.x86_base_offset {
                            #[cfg(debug_assertions)]
                            println!(
                                "{}:{} DEBUG: {}",
                                file!(),
                                line!(),
                                "USING OFFSET"
                            );
                            return Ok(o);
                        } else {
                            bail!("fail")
                        }
                    }
                }
            }
            None => {
                if let Some(o) = offsets.x86_base_offset {
                    #[cfg(debug_assertions)]
                    println!(
                        "{}:{} DEBUG: {}",
                        file!(),
                        line!(),
                        "USING OFFSET"
                    );
                    return Ok(o);
                } else {
                    bail!("fail")
                }
            }
        },
        Arch::X64 => match &offsets.sig {
            Some(sig) => {
                match sig_scan_ex(
                    &game_process,
                    &sig,
                    main_module.base,
                    scan_size,
                ) {
                    Some(sig_addr) => {
                        let offset = match offsets.sig_offset {
                            Some(so) => read_memory_ex::<u32>(
                                &game_process,
                                sig_addr + so as usize,
                            ),
                            None => {
                                read_memory_ex::<u32>(&game_process, sig_addr)
                            }
                        };
                        if let Some(o) = offset {
                            #[cfg(debug_assertions)]
                            println!(
                                "{}:{} DEBUG: {}",
                                file!(),
                                line!(),
                                "USING SIGNATURE"
                            );
                            if let Some(sr) = offsets.sig_rip {
                                return Ok(sig_addr + sr as usize + o as usize
                                    - main_module.base);
                            } else {
                                return Ok(
                                    sig_addr + o as usize - main_module.base
                                );
                            }
                        } else {
                            bail!("fail")
                        }
                    }
                    None => {
                        if let Some(o) = offsets.base_offset {
                            #[cfg(debug_assertions)]
                            println!(
                                "{}:{} DEBUG: {}",
                                file!(),
                                line!(),
                                "USING OFFSET"
                            );
                            return Ok(o);
                        } else {
                            bail!("fail")
                        }
                    }
                }
            }
            None => {
                if let Some(o) = offsets.base_offset {
                    #[cfg(debug_assertions)]
                    println!(
                        "{}:{} DEBUG: {}",
                        file!(),
                        line!(),
                        "USING OFFSET"
                    );
                    return Ok(o);
                } else {
                    bail!("fail")
                }
            }
        },
        _ => {
            bail!("game arch not supported")
        }
    };
}
