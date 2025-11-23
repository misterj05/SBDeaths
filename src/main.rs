use std::fs::{File, create_dir};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use directories::ProjectDirs;
use libmem::*;

mod offsets;
mod util;

cfg_if::cfg_if! {
    if #[cfg(any(feature = "update-notifier", feature = "update-offsets"))] {
        mod update;
        use update::*;
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
            println!("{}:{} DEBUG: {}", file!(), line!(), $msg);

            sleep(Duration::from_secs(1));
            continue;
        }
        let $ident = func_result.unwrap();
    };

    (@result $ident:ident = $expr:expr) => {
        let func_result = $expr;
        if let Err(e) = func_result {
            println!("{}:{} ERROR: {}", file!(), line!(), e);
            sleep(Duration::from_secs(1));
            continue;
        }
        let $ident = func_result.unwrap();
    };
}

pub const PROGRAM_NAME: &str = "SBDeaths";
pub const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
pub const PROCESS_NAMES: [&str; 5] = [
    "DarkSouls",
    "DARKSOULS",
    "sekiro",
    "start_protected_game",
    "eldenring",
];

fn main() {
    let mut prev_deaths: u32 = 0;

    let portable: bool = match get_cur_exe_or_cur_dir() {
        Ok(p) => match is_dir_empty(p) {
            Ok(b) => b,
            Err(e) => {
                println!("{}:{} ERROR: {}", file!(), line!(), e);
                true
            }
        },
        Err(e) => {
            println!("{}:{} ERROR: {}", file!(), line!(), e);
            true
        }
    };

    let data_path: PathBuf = if portable {
        match get_cur_exe_or_cur_dir() {
            Ok(p) => p,
            Err(e) => {
                exit_with_error(Some(format!("{} {}:{}", e, file!(), line!())));
                unreachable!();
            }
        }
    } else {
        match ProjectDirs::from("", "", PROGRAM_NAME) {
            Some(proj_dirs) => {
                match Path::try_exists(proj_dirs.data_dir()) {
                    Ok(exists) => {
                        if !exists {
                            if let Err(e) = create_dir(proj_dirs.data_dir()) {
                                println!("{}", e);
                            }
                        }

                        #[cfg(debug_assertions)]
                        if let Ok(exists) =
                            Path::try_exists(Path::new("offsets.yaml"))
                        {
                            if exists {
                                if let Err(e) = copy_file_to_dir(
                                    Path::new("offsets.yaml"),
                                    proj_dirs.data_dir(),
                                ) {
                                    println!(
                                        "{}:{} ERROR: {}",
                                        file!(),
                                        line!(),
                                        e
                                    );
                                };
                            } else {
                                println!(
                                    "{}:{} DEBUG: offsets.yaml does not exist for copy to data_dir",
                                    file!(),
                                    line!()
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("{}:{} ERROR: {}", file!(), line!(), e);
                    }
                }
                proj_dirs.data_dir().to_path_buf()
            }
            None => {
                exit_with_error(Some(format!(
                    "could not find project dir {}:{}",
                    file!(),
                    line!()
                )));
                unreachable!();
            }
        }
    };

    let mut er_warning_passed: bool =
        match data_path.join("er_warning_passed").try_exists() {
            Ok(exists) => {
                if exists {
                    true
                } else {
                    false
                }
            }
            Err(e) => {
                println!("{}:{} ERROR: {}", file!(), line!(), e);
                false
            }
        };

    #[cfg(feature = "update-offsets")]
    #[cfg(not(debug_assertions))]
    if let Err(e) = update_offsets(&data_path) {
        println!("{}:{} ERROR: {}", file!(), line!(), e);
    }

    let printable_data_path =
        match std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
            Ok(user) => {
                let path = if let Some((_, p)) =
                    data_path.to_string_lossy().split_once(&user.to_string())
                {
                    p.to_string()
                } else {
                    String::default()
                };
                path.chars().skip(1).collect()
            }
            Err(e) => {
                println!("{}:{} ERROR: {}", file!(), line!(), e);
                String::default()
            }
        };

    match data_path.join("template.txt").try_exists() {
        Ok(exists) => {
            if !exists {
                if let Ok(mut f) = File::create(data_path.join("template.txt"))
                {
                    if let Err(e) = f.write_all("{}".as_bytes()) {
                        println!("{}:{} ERROR: {}", file!(), line!(), e);
                    }
                }
            }
        }
        Err(e) => {
            println!("{}:{} ERROR: {}", file!(), line!(), e);
        }
    }

    if let Err(e) =
        write_file(&data_path, format!("{}.txt", PROGRAM_NAME).as_str(), 0)
    {
        println!("{}:{} ERROR: {}", file!(), line!(), e);
    }

    println!();
    match cfg!(feature = "update-notifier") {
        true => match check_for_updates() {
            Ok(Some(tag)) => {
                println!(
                    "SBDeaths v{}. Update available: {}",
                    VERSION.unwrap_or("unknown version"),
                    tag
                );
            }
            Ok(None) => {
                println!(
                    "SBDeaths v{}. No updates available.",
                    VERSION.unwrap_or("unknown version")
                );
            }
            Err(e) => {
                println!("{}:{} ERROR: {}", file!(), line!(), e);
            }
        },
        false => {
            println!("SBDeaths v{}", VERSION.unwrap_or("unknown version"));
        }
    }
    println!();
    println!("The program is working in the background, keep the window open.");
    println!();
    match portable {
        true => {
            println!(
                "Your death count will be written to a txt file called \
                {}.txt in the same folder where you ran this program.",
                PROGRAM_NAME
            );
        }
        false => {
            println!(
                "Your death count will be written to a txt file called \
                {}.txt in {}",
                PROGRAM_NAME, printable_data_path
            );
        }
    }
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

        good_or_continue!(
            @result offset_map = offsets::deserialize_offsets(&data_path)
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
                &offset_map,
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
                        &data_path,
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
