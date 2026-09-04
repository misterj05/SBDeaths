// SPDX-FileCopyrightText: Copyright 2025-2026 misterj05 & SBDeaths Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::prelude::*;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;

use libmem::*;

use crate::offsets::find_base_offset;
use crate::startup::InitData;
use crate::{PROGRAM_NAME, dbg, good_or_continue, startup};
use crate::{offsets, util::*};

pub fn run(
    init_data: InitData,
    deaths: Arc<RwLock<u32>>,
    dtx: std::sync::mpsc::Sender<u32>,
) {
    let mut prev_deaths: u32 = 0;

    let (mut er_warning_passed, data_path, offsets) = (
        init_data.er_warning_passed,
        init_data.data_path,
        init_data.offsets,
    );

    loop {
        // #[cfg (debug_assertions)]
        // print_all_processes();

        let pid = get_pid();
        if let (None, _) = pid {
            #[cfg(debug_assertions)]
            println!("{}:{} DEBUG: Couldn't find a Pid.", file!(), line!());

            sleep(Duration::from_secs(1));
            continue;
        }

        good_or_continue!(
            @option proc_name = pid.1,
            "proc_name is a None."
        );
        good_or_continue!(
            @option proc_name = proc_name.to_str(),
            "Was unable to convert proc_name to a str."
        );

        let pid = pid.0.unwrap();

        good_or_continue!(
            @option game_process = get_process_ex(pid.as_u32()),
            "Couldn't get Process from Pid"
        );

        #[cfg(debug_assertions)]
        println!("{}:{} DEBUG: {}", file!(), line!(), proc_name);

        if er_warning_passed == false {
            er_warning_passed = handle_er_ac(&data_path, proc_name);
        }

        let offset_map = &offsets;

        let mut module_name = "nothing";
        for e in offset_map {
            // On Linux DSR gets trimmed to "DarkSoulsRemast"
            // and DS3 gets trimmed to "DarkSoulsIII.ex"
            // which causes get_offsets_from_name to fail at
            // finding a correct entry in the OffsetMap.
            #[cfg(target_os = "linux")]
            {
                if proc_name.contains("DarkSoulsRemast") {
                    module_name = "DarkSoulsRemastered.exe"
                } else if proc_name.contains("DarkSoulsII") {
                    if proc_name == "DarkSoulsII.exe" {
                        module_name = "DarkSoulsII.exe"
                    } else {
                        module_name = "DarkSoulsIII.exe"
                    }
                } else if proc_name == e.0 {
                    module_name = e.0
                }
            }

            #[cfg(target_os = "windows")]
            {
                if proc_name == e.0 {
                    module_name = e.0
                }
            }
        }

        good_or_continue!(
            @option offsets = offsets::get_offsets_from_name(
                &offset_map,
                module_name
            ),
            "Could not find Offsets by game name."
        );

        let target_module: String = module_name.to_string();
        let mut target_base_offset: usize = 0x0;
        let mut target_offsets: Vec<usize> = Vec::from([0x0]);

        #[cfg(debug_assertions)]
        dbg!(&target_module);

        good_or_continue!(
            @option main_module = find_module_ex(
                &game_process,
                &target_module
            ),
            "Could not find Main Module for Game Process."
        );

        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                dbg!(&main_module.name);
                dbg!(main_module.base);
            }
        }

        // Work around libmem's Wine/Proton module size detection
        #[cfg(target_os = "linux")]
        let scan_size = match module_name.trim_end_matches(".exe") {
            "DARKSOULS" => 31850496,
            "DarkSoulsRemastered" => 31850496,
            "DarkSoulsII" => 135168,
            "DarkSoulsIII" => 74780672,
            "sekiro" => 69107712,
            "eldenring" => 78966784,
            _ => 0,
        };

        #[cfg(target_os = "windows")]
        let scan_size = main_module.size;

        #[cfg(debug_assertions)]
        dbg!(&scan_size);

        target_base_offset = match find_base_offset(
            &offsets,
            &game_process,
            &main_module,
            scan_size,
        ) {
            Ok(base_offset) => base_offset,
            Err(e) => {
                #[cfg(debug_assertions)]
                println!("{}:{} ERROR: {}", file!(), line!(), e);
                target_base_offset
            }
        };

        target_offsets = match game_process.arch {
            Arch::X86 => match offsets.x86_offsets {
                Some(o) => o,
                None => target_offsets,
            },
            Arch::X64 => match offsets.offsets {
                Some(o) => o,
                None => target_offsets,
            },
            _ => target_offsets,
        };

        let final_base_offset = main_module.base + target_base_offset;

        good_or_continue!(
            @option pointer = deep_pointer_ex(
                &game_process,
                final_base_offset,
                &target_offsets
            ),
            "No Pointer Found."
        );

        #[cfg(debug_assertions)]
        dbg!(pointer);

        loop {
            let val = read_memory_ex::<u32>(&game_process, pointer);

            if let Some(death_value) = val {
                dbg!(death_value);
                if prev_deaths != death_value && death_value != 0 {
                    let mut w_lock = deaths.try_write();
                    match w_lock {
                        Ok(ref mut d) => {
                            **d = death_value;
                        }
                        Err(ref e) => {
                            println!(
                                "ERROR trying to aquire write lock for death value: {e}"
                            );
                        }
                    }
                    drop(w_lock);

                    if let Err(e) = dtx.send(death_value) {
                        println!("{}:{} ERROR: {}", file!(), line!(), e);
                    };

                    print!(
                        "\r {}: {} ",
                        main_module.name.trim_end_matches(".exe"),
                        death_value
                    );
                    if let Err(e) = std::io::stdout().flush() {
                        println!("{}:{} ERROR: {}", file!(), line!(), e);
                    }

                    if let Err(e) = write_file(
                        &data_path,
                        format!("{}.txt", PROGRAM_NAME).as_str(),
                        death_value,
                    ) {
                        println!("{}:{} ERROR: {}", file!(), line!(), e);
                    }
                }
                prev_deaths = death_value;
                sleep(Duration::from_secs(1));
            } else {
                let mut clear_line_string: String = String::from("");
                for _ in 0..=60 {
                    clear_line_string.push(' ');
                }
                print!("\r{}\r", clear_line_string);
                if let Err(e) = std::io::stdout().flush() {
                    println!("{}:{} ERROR: {}", file!(), line!(), e);
                }
                break;
            }
        }
        sleep(Duration::from_secs(1));
    }
}
