use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use log::*;

pub fn copy<P: AsRef<Path>>(
    root: P,
    build_path: &Option<PathBuf>,
    destination: &PathBuf,
    branch: &String,
    include_files: &Vec<PathBuf>,
    prune: bool,
) -> Result<(), failure::Error> {
    let root = root.as_ref();

    // join root here so relative directories are correct even if 'cargo screeps' is
    // run in sub-directory.
    let output_dir = root.join(&destination).join(&branch);

    fs::create_dir_all(&output_dir)?;

    let mut deployed: HashSet<PathBuf> = HashSet::new();

    for target in include_files {
        let target_dir = build_path
            .as_ref()
            .map(|p| root.join(p))
            .unwrap_or_else(|| root.into())
            .join(target);

        for entry in fs::read_dir(target_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let (Some(name), Some(extension)) = (path.file_name(), path.extension()) {
                if extension == "wasm" || extension == "js" || extension == "mjs" {
                    let output_path = output_dir.join(name);
                    fs::copy(&path, &output_path)?;
                    deployed.insert(output_path);
                }
            }
        }
    }

    if prune {
        for entry in fs::read_dir(output_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !deployed.contains(&path) {
                info!("pruning: removing {}", path.display());
                fs::remove_file(path)?;
            }
        }
    }

    Ok(())
}
