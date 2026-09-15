// SPDX-FileCopyrightText: Copyright 2025-2026 misterj05 & SBDeaths Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::env::{current_dir, current_exe};
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::{Read, Write, stdin};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Result, bail, ensure};
use sha2::{Digest, Sha256};

use crate::{PROCESS_NAMES, PROGRAM_NAME};

#[cfg(debug_assertions)]
pub fn print_all_processes() {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    for (pid, process) in sys.processes() {
        if let Some(pn) = process.name().to_str() {
            println!("PID: {} Process Name: {}", pid, pn);
        }
    }
}

pub fn exit_with_error(msg: Option<String>) {
    match msg {
        Some(m) => println!("An unrecoverable error occured: {}", m),
        None => println!("An unrecoverable error occured"),
    }
    println!("Exiting in 10 seconds");
    sleep(Duration::from_secs(10));
    exit(1);
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

pub fn write_file<P: AsRef<Path>>(path: P, name: &str, val: u32) -> Result<()> {
    let path = path.as_ref();
    ensure!(path.is_dir(), "path is not a dir");
    let mut f = File::create(path.join(name))?;
    let template_exists = path.join("template.txt").try_exists()?;
    let value_to_write = if template_exists {
        fs::read_to_string(path.join("template.txt"))?
            .replace("{}", &val.to_string())
    } else {
        val.to_string()
    };
    f.write_all(value_to_write.to_string().as_bytes())?;
    return Ok(());
}

pub fn copy_file_to_dir<P: AsRef<Path>>(
    file_path: P,
    dir_path: P,
) -> Result<()> {
    let file_path = file_path.as_ref();
    let dir_path = dir_path.as_ref();

    #[cfg(debug_assertions)]
    ensure!(file_path != dir_path, "both paths provided were the same");

    let file_exists = file_path.try_exists()?;
    ensure!(file_exists, "file_path does not exist");
    ensure!(file_path.is_file(), "file_path does not point to a file");

    let dir_exists = dir_path.try_exists()?;
    ensure!(dir_exists, "dir_path does not exist");
    ensure!(dir_path.is_dir(), "dir_path does not point to a dir");

    let filename = if let Some(filename) = file_path.iter().last() {
        filename
    } else {
        bail!("could not preserve filename");
    };

    let mut f = File::open(file_path)?;
    let mut df = File::create(dir_path.join(filename))?;
    let buf = &mut vec![];
    f.read_to_end(buf)?;
    df.write_all(buf)?;
    df.flush()?;

    return Ok(());
}

pub fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    return hasher.finalize().to_vec();
}

pub fn sha256_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut f = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    return Ok(hasher.finalize().to_vec());
}

pub fn loop_until_correct_input(correct_inputs: Vec<&str>) -> &str {
    let mut buffer = String::new();
    let multiple_ci: bool = correct_inputs.len() > 1;
    loop {
        let _ = stdin().read_line(&mut buffer);
        for ci in &correct_inputs {
            if buffer.trim() == *ci {
                return *ci;
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

pub fn is_dir_empty<P: AsRef<Path>>(path: P) -> Result<bool> {
    let path = path.as_ref();
    ensure!(path.is_dir(), "path is not a dir");

    let files = fs::read_dir(path)?;

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

    for f in files {
        let f = f?;
        let file_name = match f.file_name().into_string() {
            Ok(name) => name,
            Err(_) => return Ok(false),
        };

        if !allowlist.contains(file_name.as_str()) {
            return Ok(false);
        }
    }

    return Ok(true);
}

pub fn get_cur_exe_or_cur_dir() -> Result<PathBuf> {
    match current_exe() {
        Ok(cur_exe) => match cur_exe.parent() {
            Some(p) => Ok(p.to_path_buf()),
            None => {
                eprintln!(
                    "{}:{} ERROR: could not find current_exe's parent",
                    file!(),
                    line!()
                );
                println!("attempting fallback to current_dir");
                match current_dir() {
                    Ok(p) => return Ok(p),
                    Err(e) => {
                        bail!(e);
                    }
                }
            }
        },
        Err(e) => {
            eprintln!("{}:{} ERROR: {}", file!(), line!(), e);
            println!("attempting fallback to current_dir");
            match current_dir() {
                Ok(p) => return Ok(p),
                Err(e) => {
                    bail!(e);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn locate_er() -> Result<PathBuf> {
    const ER_APP_ID: u32 = 1_245_620;
    let steam_dir = steamlocate::SteamDir::locate()?;

    let Some((elden_ring_app, er_lib)) = steam_dir.find_app(ER_APP_ID)? else {
        bail!("Could not find Elden Ring directory.");
    };
    let er_dir = er_lib.resolve_app_dir(&elden_ring_app);
    let expected_rel_path =
        Path::new("steamapps").join("common").join("ELDEN RING");
    assert!(er_dir.ends_with(&expected_rel_path));
    Ok(er_dir)
}

pub fn handle_er_ac<P: AsRef<Path>>(data_path: P, proc_name: &str) -> bool {
    if !proc_name.contains("start_protected")
        && !proc_name.contains("eldenring")
    {
        return false;
    }
    println!();
    println!();
    println!("PLEASE MAKE SURE YOU ARE DISABLING ELDEN RING'S ANTI-CHEAT!!!!");
    println!();

    #[cfg(target_os = "linux")]
    {
        println!("On Linux you can do so by using this launch option:");
        println!();
        println!(
            "echo \"%command%\" | sed \'s/start_protected_game.exe/eldenring.exe/\' | sh"
        );
        println!();
        println!(
            "Or by replacing start_protected_game.exe \
            with a copy of eldenring.exe renamed to \
            start_protected_game.exe in the game folder."
        );
        println!(
            "Or any Wine/Proton trickery that gets eldenring.exe \
            to launch instead of start_protected_game.exe."
        );
    }

    #[cfg(target_os = "windows")]
    {
        println!("On Windows you can do so by using a launch option.");
        let dir = locate_er();
        match dir {
            Ok(d) => {
                dbg!(&d);
                let full_path = d.join("Game").join("eldenring.exe");
                println!(
                    "Copy and paste this into Elden Ring's launch options on Steam."
                );
                println!(
                    "{}{}{}{}",
                    '\"',
                    full_path.display(),
                    '\"',
                    " %command%"
                );
            }
            Err(e) => {
                dbg!(e);
                println!(
                    "It must be in this format INCLUDING THE QUOTES: \
                    \"THE_FULL_PATH_TO_ELDENRING.EXE\" %command%"
                );
                println!(
                    "Example: \
                    \"C:\\Program Files (x86)\\Steam\\steamapps\\common\\\
                    ELDEN RING\\Game\\eldenring.exe\" %command%"
                );
            }
        }

        // On Windows the EXE stays as "start_protected_game.exe" whereas
        // on Linux it changes to "eldenring.exe" after the EAC startup screen
        // so on Windows the behaviour is to quit until EAC is correctly
        // disabled so it can be read as "eldenring.exe" properly.
        if proc_name == "start_protected_game.exe" {
            println!(
                "Please restart {} after you have disabled Elden Ring's Anti-Cheat.",
                PROGRAM_NAME
            );
            sleep(Duration::from_secs(600));
            exit(0);
        }
    }

    println!();
    println!(
        "Please type \"Yes, I understand.\" to confirm you understand \
        the above text and \n if you haven't disabled Anti-Cheat beyond \
        this point you are highly likely to be banned by the game."
    );
    println!();

    let correct_inputs = Vec::from(["Yes, I understand."]);
    let input_given = loop_until_correct_input(correct_inputs);
    if input_given == "Yes, I understand." {
        println!();
        println!(
            "The program is now working and reading deaths for Elden Ring."
        );
        println!("This warning will not show again.");
        println!();
        if let Err(e) = write_file(data_path, "er_warning_passed", 0) {
            eprintln!("{}:{} ERROR: {}", file!(), line!(), e);
        }
        return true;
    }
    unreachable!()
}
