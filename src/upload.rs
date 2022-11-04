use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

use failure::{bail, ensure};
use log::*;
use serde::Serialize;

use crate::config::Authentication;

const CODE_SIZE_LIMIT: u32 = 5 * 1024 * 1024;

pub fn upload(
    root: &Path,
    build_path: &Option<PathBuf>,
    authentication: &Authentication,
    branch: &String,
    include_files: &Vec<PathBuf>,
    url: &String,
    http_timeout: Option<u32>,
) -> Result<(), failure::Error> {
    let mut files = HashMap::new();
    let mut files_total_bytes = 0u32;

    for target in include_files {
        let target_dir = build_path
            .as_ref()
            .map(|p| root.join(p))
            .unwrap_or_else(|| root.into())
            .join(target);

        for entry in fs::read_dir(target_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let (Some(name), Some(extension)) = (path.file_stem(), path.extension()) {
                let contents = if extension == "js" {
                    let data = {
                        let mut buf = String::new();
                        fs::File::open(&path)?.read_to_string(&mut buf)?;
                        buf
                    };
                    files_total_bytes += data.chars().count() as u32;
                    serde_json::Value::String(data)
                } else if extension == "wasm" {
                    let data = {
                        let mut buf = Vec::new();
                        fs::File::open(&path)?.read_to_end(&mut buf)?;
                        buf
                    };
                    let data = base64::encode(data);
                    files_total_bytes += data.chars().count() as u32;
                    serde_json::json!({ "binary": data })
                } else {
                    continue;
                };

                files.insert(name.to_string_lossy().into_owned(), contents);
            }
        }
    }

    let pct_consumed = files_total_bytes as f64 / CODE_SIZE_LIMIT as f64;
    let mb_consumed = files_total_bytes as f64 / 1024. / 1024.;
    if files_total_bytes > CODE_SIZE_LIMIT {
        warn!(
            "Files to upload over limit, failure expected! {:.2}MiB of 5MiB limit ({:.2}%)",
            mb_consumed,
            pct_consumed * 100.
        );
    } else if pct_consumed > 0.9 {
        warn!(
            "Files to upload near limit! {:.2}MiB of 5MiB limit ({:.2}%)",
            mb_consumed,
            pct_consumed * 100.
        );
    } else {
        debug!(
            "Files to upload consuming {:.2}MiB of 5MiB limit ({:.2}%)",
            mb_consumed,
            pct_consumed * 100.
        );
    }

    let client_builder = reqwest::blocking::Client::builder();
    let client = match http_timeout {
        None => client_builder.build()?,
        Some(value) => client_builder
            .timeout(Duration::from_secs(value as u64))
            .build()?,
    };

    #[derive(Serialize)]
    struct RequestData {
        modules: HashMap<String, serde_json::Value>,
        branch: String,
    }

    let response = authenticate(client.post(url), authentication)
        .json(&RequestData {
            modules: files,
            branch: branch.clone(),
        })
        .send()?;

    let response_status = response.status();
    let response_url = response.url().clone();
    let response_text = response.text()?;

    ensure!(
        response_status.is_success(),
        "uploading to '{}' failed: {}",
        response_url,
        response_text,
    );

    debug!("upload finished: {}", response_text);

    let response_json: serde_json::Value = serde_json::from_str(&response_text)?;

    if let Some(s) = response_json.get("error") {
        bail!(
            "error sending to branch '{}' of '{}': {}",
            branch,
            response_url,
            s
        );
    }

    Ok(())
}

fn authenticate(
    request: reqwest::blocking::RequestBuilder,
    authentication: &Authentication,
) -> reqwest::blocking::RequestBuilder {
    match authentication {
        Authentication::Token { ref auth_token } => request.header("X-Token", auth_token.as_str()),
        Authentication::Basic {
            ref username,
            ref password,
        } => request.basic_auth(username, Some(password)),
    }
}
