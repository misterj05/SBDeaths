// SPDX-FileCopyrightText: Copyright 2025-2026 misterj05 & SBDeaths Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    fs::{File, create_dir_all},
    io::Write,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

use crate::{
    OFFSETS_COMPAT, PROGRAM_NAME, VERSION,
    offsets::{self, OffsetMap},
    update::{check_for_updates, update_offsets, verify_offsets_version},
    util::{
        copy_file_to_dir, exit_with_error, get_cur_exe_or_cur_dir,
        is_dir_empty, write_file,
    },
};

pub struct InitData {
    pub er_warning_passed: bool,
    pub data_path: PathBuf,
    pub offsets: OffsetMap,
}

pub fn init() -> InitData {
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
                            if let Err(e) = create_dir_all(proj_dirs.data_dir())
                            {
                                println!(
                                    "{}:{} ERROR: {}",
                                    file!(),
                                    line!(),
                                    e
                                );
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

    let er_warning_passed: bool =
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
    match verify_offsets_version() {
        Ok(Some(msg)) => {
            println!("{}:{} MSG: {}", file!(), line!(), msg);
            if let Err(e) = update_offsets(&data_path) {
                println!("{}:{} ERROR: {}", file!(), line!(), e);
            }
        }
        Ok(None) => {
            if let Err(e) = update_offsets(&data_path) {
                println!("{}:{} ERROR: {}", file!(), line!(), e);
            }
        }
        Err(e) => {
            println!("{}:{} ERROR: {}", file!(), line!(), e);
        }
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

    // Verify OffsetsFile compat version
    match offsets::get_offsets_compat(&data_path) {
        Ok(Some(compat)) => {
            if compat != OFFSETS_COMPAT {
                exit_with_error(Some(format!(
                    "The local offsets file is not compatible with the currently \
                    running binary.\nExpected version: {}, Found version: {}.\n",
                    OFFSETS_COMPAT, &compat
                )));
            }
        }
        Ok(None) => {
            exit_with_error(Some(String::from(
                "Couldn't get offsets compat info.",
            )));
            unreachable!()
        }
        Err(e) => {
            println!("{}:{} ERROR: {}", file!(), line!(), &e.to_string());
            exit_with_error(Some(String::from(
                "Couldn't get offsets compat info.",
            )));
            unreachable!()
        }
    };

    // Grab Offsets from the OffsetsFile
    let offsets = match offsets::get_offsets(&data_path) {
        Ok(Some(o)) => o,
        Ok(None) => {
            exit_with_error(Some(String::from(
                "Couldn't get offsets from file.",
            )));
            unreachable!()
        }
        Err(e) => {
            println!("{}:{} ERROR: {}", file!(), line!(), &e.to_string());
            exit_with_error(Some(String::from(
                "Couldn't get offsets from file.",
            )));
            unreachable!()
        }
    };

    return InitData {
        er_warning_passed,
        data_path,
        offsets,
    };
}
