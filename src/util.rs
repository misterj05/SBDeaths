use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::env;
use std::io::{Write, Read, stdin};
use std::process::exit;
use std::ffi::OsString;

use anyhow::{anyhow, Result};

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        use std::path::{Path, PathBuf};
        use std::thread::sleep;
        use std::time::Duration;
    }
}

use crate::{PROGRAM_NAME, PROCESS_NAMES};

#[cfg (debug_assertions)]
pub fn print_all_processes() {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    for (pid, process) in sys.processes() {
        if let Some(pn) = process.name().to_str() {
            println!("PID: {} Process Name: {}", pid, pn);
        }
    }
}

pub fn get_pid() -> (Option<sysinfo::Pid>, Option<OsString>) {
    let target_names = PROCESS_NAMES.map(OsString::from);
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    for t in target_names {
        for p in sys.processes_by_name(&t) {
            if let None = p.thread_kind() {
                return (Some(p.pid()), Some(p.name().to_owned()));
            }
        }
    }
    (None, None)
}

pub fn write_file(name: &str, val: u32) -> Result<()> {
    match File::open(name) {
        Ok(..) => {
            let raw_template = fs::read_to_string("template.txt")?;
            let replaced_template = raw_template.replace("{}", &val.to_string());
            if let Ok(mut f) = File::create(name) {
                if let Err(e) = f.write_all(replaced_template.to_string().as_bytes()) {
                     return Err(anyhow!(
                         "Failed to write file named: {}\n\
                         Error Value: {}\n\
                         File existed before write attempt.",
                         name, e
                     ))
                }
            }
            Ok(())
        }
        Err(..) => {
            if let Ok(mut f) = File::create(name) {
                if let Err(e) = f.write_all(val.to_string().as_bytes()) {
                    return Err(anyhow!(
                        "Failed to write file named: {}\n\
                        Error Value: {}\n\
                        File did not exist before write attempt.",
                        name, e
                    ))
                }
            }
            Ok(())
        }
    }
}

pub fn loop_until_correct_input(correct_inputs: Vec<&str>) -> &str {
    let mut buffer = String::new();
    let multiple_ci: bool = correct_inputs.len() > 1;
    loop {
        let _ = stdin().read_line(&mut buffer);
        for ci in &correct_inputs {
            if buffer.trim() == *ci {
                return *ci
            }
        }
        println!("That was not correct.");
        if multiple_ci {
            print!("Possible inputs")
        } else {
            print!("Expected input");
        }
        print!(": ");
        for ci in &correct_inputs {
            print!("\"{}\"", ci);
            match correct_inputs.last() {
                Some(last) => {
                    if multiple_ci {
                        if ci != last {
                            print!(",");
                        } else {
                            print!(".");
                        }
                    }
                }
                _ => {}
            }
            print!(" ");
        }
        println!();
        buffer.clear();
    }
}

pub fn is_cur_dir_empty() -> Result<bool> {
    let current_dir = env::current_dir()?;
    let files = fs::read_dir(current_dir)?;

    let exe_filename = format!("{}.exe", PROGRAM_NAME);
    let txt_filename = format!("{}.txt", PROGRAM_NAME);
    let allowlist = HashSet::from([
        PROGRAM_NAME,
        &exe_filename.as_str(),
        &txt_filename.as_str(),
        "er_warning_passed",
        "template.txt",
        "offsets.yaml",
    ]);

    let mut found_disallowed = false;
    let mut found_cargo_toml = false;

    for f in files {
        let f = f?;
        let file_name = match f.file_name().into_string() {
            Ok(name) => name,
            Err(_) => return Ok(false),
        };

        if file_name == "Cargo.toml" {
            found_cargo_toml = true;
        } else if !allowlist.contains(file_name.as_str()) {
            found_disallowed = true;
        }
    }

    if found_cargo_toml {
        Ok(true)
    } else if found_disallowed {
        Ok(false)
    } else {
        Ok(true)
    }
}

pub fn handle_empty_dir(empty: bool) {
    if empty == true {
        return;
    }
    println!("{} is intended to be ran in an empty directory because it \
            makes a few files wherever the program is placed.", PROGRAM_NAME);
    println!("Would you like to continue the program? y or n? \
            This is so you have a chance to move it to an empty directory before any files are made.");

    let correct_inputs = Vec::from(["y", "n"]);
    let input_given = loop_until_correct_input(correct_inputs);
    if input_given == "y" {
        return
    } else if input_given == "n" {
        exit(0)
    }
    unreachable!()
}

#[cfg(target_os = "windows")]
pub fn locate_er() -> Result<PathBuf> {
    const ER_APP_ID: u32 = 1_245_620;
    let steam_dir = steamlocate::SteamDir::locate()?;

    let Some((elden_ring_app, er_lib)) = steam_dir.find_app(ER_APP_ID)? else {
        return Err(anyhow!("Could not find Elden Ring directory."))
    };
    let er_dir = er_lib.resolve_app_dir(&elden_ring_app);
    let expected_rel_path = Path::new("steamapps").join("common").join("ELDEN RING");
    assert!(er_dir.ends_with(&expected_rel_path));
    Ok(er_dir)
}

pub fn handle_er_ac(proc_name: &str) -> bool {
    if !proc_name.contains("start_protected") && !proc_name.contains("eldenring"){
        return false;
    }
    println!();
    println!();
    println!("PLEASE MAKE SURE YOU ARE DISABLING ELDEN RING'S ANTI-CHEAT!!!!");
    println!();

    #[cfg(target_os = "linux")] {
        println!("On Linux you can do so by using this launch option:");
        println!();
        println!("echo \"%command%\" | sed \'s/start_protected_game.exe/eldenring.exe/\' | sh");
        println!();
        println!("Or by replacing start_protected_game.exe with a copy of \
                eldenring.exe renamed to start_protected_game.exe in the game folder.");
        println!("Or any Wine/Proton trickery that gets eldenring.exe \
                to launch instead of start_protected_game.exe.");
    }

    #[cfg(target_os = "windows")] {
        println!("On Windows you can do so by using a launch option.");
        let dir = locate_er();
        match dir {
            Ok(d) => {
                dbg!(&d);
                let full_path = d.join("Game").join("eldenring.exe");
                println!("Copy and paste this into Elden Ring's launch options on Steam.");
                println!("{}{}{}{}", '\"', full_path.display(), '\"', " %command%");
            }
            Err(e) => {
                dbg!(e);
                println!("It must be in this format INCLUDING THE QUOTES: \
                \"THE_FULL_PATH_TO_ELDENRING.EXE\" %command%");
                println!("Example: \
                \"C:\\Program Files (x86)\\Steam\\steamapps\\common\\ELDEN RING\\Game\\eldenring.exe\" %command%");
            }
        }

        // On Windows the EXE stays as "start_protected_game.exe" whereas
        // on Linux it changes to "eldenring.exe" after the EAC startup screen
        // so on Windows the behaviour is to quit until EAC is correctly
        // disabled so it can be read as "eldenring.exe" properly.
        if proc_name == "start_protected_game.exe" {
            println!("Please restart {} after you have disabled Elden Ring's Anti-Cheat.", PROGRAM_NAME);
            sleep(Duration::from_secs(600));
            exit(0);
        }
    }

    println!();
    println!("Please type \"Yes, I understand.\" to confirm you understand the above text and \n\
            if you haven't disabled Anti-Cheat beyond this point you are highly likely to be banned by the game.");
    println!();

    let correct_inputs = Vec::from(["Yes, I understand."]);
    let input_given = loop_until_correct_input(correct_inputs);
    if input_given == "Yes, I understand." {
        println!();
        println!("The program is now working and reading deaths for Elden Ring.");
        println!("This warning will not show again.");
        println!();
        if let Err(e) = write_file("er_warning_passed", 0) {
            println!("{}", e);
        }
        return true;
    }
    unreachable!()
}
