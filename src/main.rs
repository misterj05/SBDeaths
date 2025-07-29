use std::ffi::OsString;
use std::thread::sleep;
use std::time::Duration;
use std::io::prelude::*;
use std::fs::File;

use libmem::*;

mod offsets;
mod util;

use crate::util::*;

macro_rules! dbg {
    ($($x:tt)*) => {
        {
            #[cfg(debug_assertions)]
            {
                std::dbg!($($x)*)
            }
            #[cfg(not(debug_assertions))]
            {
                ($($x)*)
            }
        }
    }
}

pub const PROGRAM_NAME: &str = "SBDeaths";
pub const PROCESS_NAMES: [&str; 5] = ["DarkSouls", "DARKSOULS", "sekiro", "start_protected_game", "eldenring"];

fn main() {
    let mut target_pid: (Option<sysinfo::Pid>, Option<OsString>);
    let mut game_process: Option<Process>;
    let mut prev_deaths: u32 = 0;
    let mut er_warning_passed: bool = false;

    match is_cur_dir_empty() {
        Ok(empty) => {
            handle_empty_dir(empty);
        }
        Err(e) => {
            println!("{}", e)
        }
    }

    if let Ok(..) = File::open("er_warning_passed") {
        er_warning_passed = true;
    }

    if let Err(..) = File::open("template.txt") {
        if let Ok(mut f) = File::create("template.txt") {
            if let Err(e) = f.write_all("{}".as_bytes()) {
                println!("{}", e);
            }
        }
    }

    if let Err(e) = write_file(format!("{}.txt", PROGRAM_NAME).as_str(), 0) {
        println!("{}", e);
    }

    println!();
    println!("The program is working in the background, keep the window open.");
    println!();
    println!("Your death count will be written to a txt file called \
            {}.txt in the same folder where you ran this program.", PROGRAM_NAME);
    println!("You can display the file in OBS by adding a Text Source \
            and selecting the \"From File\" option.");
    println!("The template.txt will be used as the template for the final .txt file, \
            all instances of \"{{}}\" will be replaced with the death count.");
    println!();

    loop {
        // #[cfg (debug_assertions)]
        // print_all_processes();

        target_pid = get_pid();
        match target_pid.0 {
            None => {
                #[cfg(debug_assertions)]
                println!("No Games Found.");

                sleep(Duration::from_secs(1));
                continue;
            }
            Some(pid) => {
                game_process = get_process_ex(pid.as_u32());
                match game_process {
                    None => {
                        #[cfg(debug_assertions)]
                        println!("Game Process is None.");

                        sleep(Duration::from_secs(1));
                        continue;
                    }
                    Some(ref proc) => {
                        let mut target_module: String = String::from("nothing");
                        let mut target_base_offset: usize = 0x0;
                        let mut target_offsets: Vec<usize> = Vec::from([0x0]);
                        match target_pid.1 {
                            None => {
                                #[cfg(debug_assertions)]
                                println!("Couldn't get process name.");

                                continue;
                            }
                            Some(proc_name) => {
                                let proc_name: &str = proc_name.to_str().unwrap();
                                if er_warning_passed == false {
                                    er_warning_passed = handle_er_ac(proc_name);
                                }
                                match offsets::deserialize_offsets() {
                                    Ok(offset_map) => {
                                        let mut offsets = None;
                                        let mut module_name = "nothing";
                                        for e in &offset_map {
                                            // On Linux DSR gets trimmed to "DarkSoulsRemast"
                                            // and DS3 gets trimmed to "DarkSoulsIII.ex"
                                            // which causes get_offsets_from_name to fail at
                                            // finding a correct entry in the OffsetMap.
                                            #[cfg(target_os = "linux")] {
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

                                            #[cfg(target_os = "windows")] {
                                                if proc_name == e.0 {
                                                    module_name = e.0
                                                }
                                            }

                                            offsets = offsets::get_offsets_from_name(offset_map.clone(), module_name)
                                        }
                                        match offsets {
                                            Some(o) => {
                                                target_module = module_name.to_string();
                                                match proc.arch {
                                                    Arch::X86 => {
                                                        if let Some(o) = o.x86_base_offset {
                                                            target_base_offset = o;
                                                        }
                                                        if let Some(o) = o.x86_offsets {
                                                            target_offsets = o;
                                                        }
                                                    }
                                                    Arch::X64 => {
                                                        if let Some(o) = o.base_offset {
                                                            target_base_offset = o;
                                                        }
                                                        if let Some(o) = o.offsets {
                                                            target_offsets = o;
                                                        }
                                                    }
                                                    _ => {
                                                        println!("Game Arch not supported.");
                                                    }
                                                }
                                            }
                                            None => {
                                                println!("Could not find Offsets by game name.")
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("Deserialization Error: {}", e)
                                    }
                                }
                            }
                        }
                        #[cfg(debug_assertions)]
                        dbg!(&target_module);

                        let main_module = find_module_ex(&proc, &target_module);
                        match main_module {
                            None => {
                                #[cfg(debug_assertions)]
                                println!("Couldn't Find Main Module for Game Process.");

                                sleep(Duration::from_secs(1));
                                continue;
                            }
                            Some(main_mod) => {
                                #[cfg(debug_assertions)]
                                dbg!(&main_mod.name);
                                dbg!(main_mod.base);

                                let final_base_offset = main_mod.base + target_base_offset;
                                let pointer = deep_pointer_ex(&game_process.clone().unwrap(), final_base_offset, &target_offsets);
                                match pointer {
                                    None => {
                                        #[cfg(debug_assertions)]
                                        println!("No Pointer found.");

                                        sleep(Duration::from_secs(1));
                                        continue;
                                    }
                                    Some(p) => {
                                        dbg!(p);
                                        loop {
                                            let val = read_memory_ex::<u32>(&game_process.clone().unwrap(), p);
                                            match val {
                                                None => {
                                                    break;
                                                }
                                                Some(deaths) => {
                                                    dbg!(deaths);
                                                    if prev_deaths != deaths && deaths != 0 {
                                                        println!("{}: {}", main_mod.name.trim_end_matches(".exe"), deaths);
                                                        if let Err(e) = write_file(format!("{}.txt", PROGRAM_NAME).as_str(), deaths) {
                                                            println!("{}", e);
                                                        }
                                                    }
                                                    prev_deaths = deaths;
                                                    sleep(Duration::from_secs(1));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        sleep(Duration::from_secs(1));
    }
}
