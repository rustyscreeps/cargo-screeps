use std::path::Path;

use crate::config::{BuildConfiguration, BuildMode};

mod arena;
mod world;

pub fn build(root: &Path, build_config: &BuildConfiguration) -> Result<(), failure::Error> {
    let mode = build_config.build_mode.clone().unwrap_or(BuildMode::World);

    match mode {
        BuildMode::Arena => arena::build(root, build_config),
        BuildMode::World => world::build(root, build_config),
    }
}
