use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{anyhow, Result};
use reqwest::blocking::get;

use crate::util::{sha256_bytes, sha256_file, loop_until_correct_input};
use crate::PROGRAM_NAME;

const URL: &str = "https://codeberg.org/misterj05/SBDeaths/raw/branch/dev/offsets.yaml";

fn fetch_file(url: &str) -> Result<Vec<u8>> {
    let mut resp = get(url);
    match resp {
        Ok(r) => {
            if !r.status().is_success() {
                return Err(anyhow!(
                    "Failed to update offsets.\n\
                    HTTP Status Code was not a success code.\n\
                    Status Code: {}",
                    r.status()
                ))
            }
            resp = Ok(r);
        }
        Err(e) => {
            return Err(anyhow!(
                "Failed to update offsets.\n\
                {}\n\
                Are you connected to the internet?\n\
                If so it's likely the source website for {} is down right now.",
                e,
                PROGRAM_NAME
            ))
        }
    }
    let mut buf = Vec::new();
    resp.unwrap().read_to_end(&mut buf)?;
    return Ok(buf)
}

fn replace_if_different(url: &str, local_path: &Path) -> Result<()> {
    let downloaded = fetch_file(url)?;

    let downloaded_hash = sha256_bytes(&downloaded);

    let different = if local_path.exists() {
        match sha256_file(local_path) {
            Ok(existing_hash) => existing_hash != downloaded_hash,
            Err(_) => true,
        }
    } else {
        true
    };

    if different {
        println!();
        println!("An updated version of the offsets.yaml is available.\n\
            (The file that lets the program know how to find the death value.)\n\
            You usually want to update this file as it will contain \
            updates for the games in it.");
        println!();
        println!("If you modified the offsets.yaml yourself this \
            is likely a false positive.");
        println!();
        println!("Would you like to update it? y or n?");
        println!();

        let correct_inputs = Vec::from(["y", "n"]);
        let input_given = loop_until_correct_input(correct_inputs);
        if input_given == "n" {
            println!("Did NOT update {:?}", local_path);
            return Ok(());
        }

        let tmp_path = local_path.with_extension("tmp");
        {
            let mut tmp = File::create(&tmp_path)?;
            tmp.write_all(&downloaded)?;
            tmp.flush()?;
        }
        fs::rename(&tmp_path, local_path)?;
        println!("Updated {:?}", local_path);
    }

    return Ok(())
}

pub fn self_update() -> Result<()> {
    replace_if_different(URL, Path::new("offsets.yaml"))?;
    return Ok(())
}
