use std::path::PathBuf;

use anyhow::Context;
use freedesktop_file_parser::EntryType;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn process_content(path: &PathBuf, input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;

    let desktop_file = freedesktop_file_parser::parse(&content)?;

    if let Some(bool) = desktop_file.entry.no_display {
        if bool {
            return Ok(());
        }
    }

    // TODO: handle not show in

    if let EntryType::Application(fields) = desktop_file.entry.entry_type {
        let mut keywords: Vec<&str> = vec![&desktop_file.entry.name.default];
        if let Some(ref generic_name) = desktop_file.entry.generic_name {
            keywords.push(&generic_name.default);
        }

        if let Some(ref kwds) = fields.keywords {
            let temp: Vec<&str> = kwds.default.iter().map(AsRef::as_ref).collect();
            keywords.extend(temp);
        }

        println!("{:?}", keywords);

        keywords.iter().for_each(|kwd| {
            if kwd.contains(input) {
                println!("An entry is found: {:?}", desktop_file.entry.name.default);
            }
        });

        for (_, value) in desktop_file.actions {
            if value.name.default.contains(input) {
                println!("An action is found: {:?}", value.name.default);
            }
        }
    }

    Ok(())
}

fn process_path(path: &PathBuf, input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dirs = std::fs::read_dir(path).context("Can't read directory")?;

    let paths = dirs
        .filter_map(|entry| match entry {
            Ok(res) => Some(res.path()),
            Err(_) => None,
        })
        .collect::<Vec<PathBuf>>();

    paths.par_iter().for_each(|p| {
        let _ = process_content(p, input);
    });

    Ok(())
}

pub fn process_apps(input: &str) {
    let paths = if let Ok(v) = std::env::var("XDG_DATA_DIRS") {
        v.split(":")
            .filter_map(|s| {
                let mut res = if let Ok(p) = PathBuf::try_from(s) {
                    p
                } else {
                    #[cfg(debug_assertions)]
                    println!("{:?} is not valid path", s);

                    return None;
                };

                res.push("applications/");
                Some(res)
            })
            .collect::<Vec<PathBuf>>()
    } else {
        println!("Dvoty: cannot read XDG_DATA_DIR");
        return;
    };

    paths.par_iter().for_each(|path| {
        let _ = process_path(path, input);
    });
}
