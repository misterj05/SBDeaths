// SPDX-FileCopyrightText: Copyright 2025-2026 misterj05 & SBDeaths Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    sync::{Arc, RwLock, mpsc::channel},
    time::Duration,
};

mod cli;
mod offsets;
mod startup;
mod util;

cfg_if::cfg_if! {
    if #[cfg(any(feature = "update-notifier", feature = "update-offsets"))] {
        mod update;
    }
}

#[macro_export]
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
#[macro_export]
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
            eprintln!("{}:{} ERROR: {}", file!(), line!(), e);
            sleep(Duration::from_secs(1));
            continue;
        }
        let $ident = func_result.unwrap();
    };
}

pub const PROGRAM_NAME: &str = "SBDeaths";
pub const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
pub const OFFSETS_COMPAT: u16 = 3;
pub const PROCESS_NAMES: [&str; 5] = [
    "DarkSouls",
    "DARKSOULS",
    "sekiro",
    "start_protected_game",
    "eldenring",
];

fn main() {
    let deaths = Arc::new(RwLock::new(0));

    let (dtx, drx) = channel();

    {
        let init_data = startup::init();
        let deaths2 = Arc::clone(&deaths);
        let dtx2 = dtx.clone();
        std::thread::spawn(move || {
            cli::run(init_data, deaths2, dtx2);
        });
    }

    loop {
        std::thread::sleep(Duration::from_secs(1));
        let r_lock = deaths.try_read();
        match r_lock {
            Ok(ref d) => {
                #[cfg(debug_assertions)]
                println!("DEBUG: Shared Death Val is {}", *d);
            }
            Err(ref e) => {
                eprintln!("ERROR aquiring deaths read lock: {e}");
            }
        }
        drop(r_lock);
        if let Ok(dv) = drx.try_recv() {
            #[cfg(debug_assertions)]
            println!("DEBUG: Deaths recv'd: {}", dv);
        }
    }
}
