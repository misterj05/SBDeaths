use std::fs::File;
use std::io::prelude::*;
use std::thread::sleep;
use std::time::Duration;

use libmem::*;

mod offsets;
mod util;

cfg_if::cfg_if! {
    if #[cfg(feature = "updater")] {
        mod updater;
        use updater::*;
    }
}

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

// Prints a failure message and resets loop if bad result,
// shadows/defines and unwraps a variable on a good result.
macro_rules! good_or_continue {
    (@option $ident:ident = $expr:expr, $msg:expr) => {
        let func_result = $expr;
        if func_result.is_none() {
            #[cfg(debug_assertions)]
            println!("Debug: {}", $msg);

            sleep(Duration::from_secs(1));
            continue;
        }
        let $ident = func_result.unwrap();
    };

    (@result $ident:ident = $expr:expr) => {
        let func_result = $expr;
        if let Err(e) = func_result {
            println!("Error: {}", e);
            sleep(Duration::from_secs(1));
            continue;
        }
        let $ident = func_result.unwrap();
    };
}

pub const PROGRAM_NAME: &str = "SBDeaths";
pub const PROCESS_NAMES: [&str; 5] = [
    "DarkSouls",
    "DARKSOULS",
    "sekiro",
    "start_protected_game",
    "eldenring",
];

fn main() {
    let mut prev_deaths: u32 = 0;
    let mut er_warning_passed: bool = false;

    match is_cur_dir_empty() {
        Ok(empty) => {
            handle_empty_dir(empty);

            #[cfg(feature = "updater")]
            #[cfg(not(debug_assertions))]
            if let Err(e) = self_update() {
                println!("Error: {}", e)
            }
        }
        Err(e) => {
            println!("Error: {}", e)
        }
    }

    if File::open("er_warning_passed").is_ok() {
        er_warning_passed = true;
    }

    if File::open("template.txt").is_err() {
        if let Ok(mut f) = File::create("template.txt") {
            if let Err(e) = f.write_all("{}".as_bytes()) {
                println!("Error: {}", e);
            }
        }
    }

    if let Err(e) = write_file(format!("{}.txt", PROGRAM_NAME).as_str(), 0) {
        println!("Error: {}", e);
    }

    println!();
    println!("The program is working in the background, keep the window open.");
    println!();
    println!(
        "Your death count will be written to a txt file called \
        {}.txt in the same folder where you ran this program.",
        PROGRAM_NAME
    );
    println!(
        "You can display the file in OBS by adding a Text Source \
        and selecting the \"From File\" option."
    );
    println!(
        "The template.txt will be used as the template for the final .txt file, \
        all instances of \"{{}}\" will be replaced with the death count."
    );
    println!();

    loop {
        // #[cfg (debug_assertions)]
        // print_all_processes();

        let pid = get_pid();
        if let (None, _) = pid {
            #[cfg(debug_assertions)]
            println!("Debug: Couldn't find a Pid.");

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
        println!("Debug: {}", proc_name);

        if er_warning_passed == false {
            er_warning_passed = handle_er_ac(proc_name);
        }

        good_or_continue!(
            @result offset_map = offsets::deserialize_offsets()
        );

        let mut module_name = "nothing";
        for e in &offset_map {
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
                offset_map.to_owned(),
                module_name
            ),
            "Could not find Offsets by game name."
        );

        let target_module: String = module_name.to_string();
        let mut target_base_offset: usize = 0x0;
        let mut target_offsets: Vec<usize> = Vec::from([0x0]);

        match game_process.arch {
            Arch::X86 => {
                if let Some(o) = offsets.x86_base_offset {
                    target_base_offset = o;
                }
                if let Some(o) = offsets.x86_offsets {
                    target_offsets = o;
                }
            }
            Arch::X64 => {
                if let Some(o) = offsets.base_offset {
                    target_base_offset = o;
                }
                if let Some(o) = offsets.offsets {
                    target_offsets = o;
                }
            }
            _ => {
                println!("Game Arch not supported.");
            }
        }

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

            if let Some(deaths) = val {
                dbg!(deaths);
                if prev_deaths != deaths && deaths != 0 {
                    print!(
                        "\r {}: {} ",
                        main_module.name.trim_end_matches(".exe"),
                        deaths
                    );
                    if let Err(e) = std::io::stdout().flush() {
                        println!("{}", e);
                    }

                    if let Err(e) = write_file(
                        format!("{}.txt", PROGRAM_NAME).as_str(),
                        deaths,
                    ) {
                        println!("{}", e);
                    }
                }
                prev_deaths = deaths;
                sleep(Duration::from_secs(1));
            } else {
                let mut clear_line_string: String = String::from("");
                for _ in 0..=60 {
                    clear_line_string.push(' ');
                }
                print!("\r{}\r", clear_line_string);
                if let Err(e) = std::io::stdout().flush() {
                    println!("{}", e);
                }
                break;
            }
        }
        sleep(Duration::from_secs(1));
    }
}
