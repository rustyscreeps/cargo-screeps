use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use log::*;

use crate::config::{Configuration, CopyConfiguration};

pub fn copy<P: AsRef<Path>>(
    root: P,
    config: &Configuration,
    copy_config: &CopyConfiguration,
) -> Result<(), failure::Error> {
    let root = root.as_ref();

    // join root here so relative directories are correct even if 'cargo screeps' is
    // run in sub-directory.
    let output_dir = root
        .join(&copy_config.destination)
        .join(&copy_config.branch);

    fs::create_dir_all(&output_dir)?;

    let target_dir = root.join("target");

    let mut deployed: HashSet<PathBuf> = HashSet::new();

    for filename in &[&config.build.output_js_file, &config.build.output_wasm_file] {
        let path = target_dir.join(filename);
        let output_path = output_dir.join(filename);
        fs::copy(&path, &output_path)?;
        deployed.insert(output_path);
    }

    if copy_config.prune {
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
