use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use reqwest::blocking::get;
use serde::Deserialize;

use crate::util::{loop_until_correct_input, sha256_bytes, sha256_file};
use crate::{PROGRAM_NAME, VERSION};

const PROTOCOL: &str = "https://";
const FORGE: &str = "codeberg.org/";
const REPOS_API: &str = "api/v1/repos/";
const REPO: &str = "misterj05/SBDeaths/";
const OFFSETS: &str = "raw/branch/dev/offsets.yaml";
const RELEASES_API: &str = "releases?page=1&limit=1";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
}

fn fetch_file(url: &String) -> Result<Vec<u8>> {
    let mut resp = get(url);
    match resp {
        Ok(r) => {
            if !r.status().is_success() {
                bail!(
                    "Failed to fetch file.\n\
                    HTTP Status Code was not a success code.\n\
                    Status Code: {}",
                    r.status()
                );
            }
            resp = Ok(r);
        }
        Err(e) => {
            bail!(
                "Failed to update offsets.\n\
                {}\n\
                Are you connected to the internet?\n\
                If so it's likely the source website for {} is down right now.",
                e,
                PROGRAM_NAME
            );
        }
    }
    let mut buf = Vec::new();
    resp.unwrap().read_to_end(&mut buf)?;
    return Ok(buf);
}

fn replace_if_different(url: &String, local_path: &Path) -> Result<()> {
    let downloaded = fetch_file(url)?;

    let downloaded_hash = sha256_bytes(&downloaded);

    let different = if local_path.exists() {
        match sha256_file(local_path) {
            Ok(existing_hash) => existing_hash != downloaded_hash,
            Err(_) => true,
        }
    } else {
        false
    };

    if different {
        println!();
        println!(
            "An updated version of the offsets.yaml is available.\n\
            (The file that lets the program know how to find the death value.)\n\
            You usually want to update this file as it will contain \
            updates for the games in it."
        );
        println!();
        println!(
            "If you modified the offsets.yaml yourself this \
            is likely a false positive."
        );
        println!();
        println!("Would you like to update it? y or n?");
        println!();

        let correct_inputs = Vec::from(["y", "n"]);
        let input_given = loop_until_correct_input(correct_inputs);
        if input_given == "n" {
            println!("Did NOT update.");
            return Ok(());
        }

        let tmp_path = local_path.with_extension("tmp");
        {
            let mut tmp = File::create(&tmp_path)?;
            tmp.write_all(&downloaded)?;
            tmp.flush()?;
        }
        fs::rename(&tmp_path, local_path)?;
        println!("Update was successful.");
    }

    return Ok(());
}

pub fn update_offsets(data_path: &PathBuf) -> Result<()> {
    let offsets_path = data_path.join("offsets.yaml");
    let offsets_url =
        String::from(PROTOCOL.to_string() + FORGE + REPO + OFFSETS);
    if !offsets_path.exists() {
        println!();
        println!("You do not have an offsets.yaml in your data path.");
        println!("Would you like to download the latest version? y or n?");
        println!();
        let correct_inputs = Vec::from(["y", "n"]);
        let input_given = loop_until_correct_input(correct_inputs);
        if input_given == "y" {
            let downloaded_offsets = fetch_file(&offsets_url)?;
            let mut f = File::create(&offsets_path)?;
            f.write_all(&downloaded_offsets)?;
        }
    } else {
        replace_if_different(&offsets_url, offsets_path.as_path())?;
    }
    return Ok(());
}

#[cfg(feature = "update-notifier")]
pub fn check_for_updates() -> Result<Option<String>> {
    let version = if let Some(v) = VERSION {
        String::from("v".to_string() + v)
    } else {
        bail!("could not get version to compare against")
    };

    let api_url = String::from(
        PROTOCOL.to_string() + FORGE + REPOS_API + REPO + RELEASES_API,
    );

    let resp = get(api_url)?;
    if resp.status().is_success() {
        let json = resp.json::<Vec<Release>>()?;
        if let Some(r) = json.last() {
            if r.tag_name != version {
                return Ok(Some(r.tag_name.clone()));
            }
        }
    }
    return Ok(None);
}
