use std::{borrow::Cow, env, ffi::OsStr, fs, io::Write, path::Path};

use cargo_web::{BuildOpts, CargoWebOpts, CheckOpts};
use failure::{bail, ensure, format_err};
use log::*;
use structopt::StructOpt;

use crate::config::{BuildConfiguration, Configuration};

pub fn check(root: &Path) -> Result<(), failure::Error> {
    debug!("running check");

    debug!("changing directory to {}", root.display());

    env::set_current_dir(&root)?;

    debug!("running cargo-web check --target=wasm32-unknown-unknown");

    let res = cargo_web::run(CargoWebOpts::Check(
        CheckOpts::from_iter_safe(&["--target=wasm32-unknown-unknown"])
            .expect("expected hardcoded cargo-web args to be valid"),
    ));
    if let Err(e) = res {
        bail!("cargo-web check failed: {}", e);
    }

    debug!("finished executing cargo-web check");
    Ok(())
}

pub fn build(root: &Path, config: &Configuration) -> Result<(), failure::Error> {
    debug!("building");

    debug!("changing directory to {}", root.display());

    env::set_current_dir(&root)?;

    debug!("running cargo-web build --target=wasm32-unknown-unknown --release");

    let res = cargo_web::run(CargoWebOpts::Build(
        BuildOpts::from_iter_safe(&["--target=wasm32-unknown-unknown", "--release"])
            .expect("expected hardcoded cargo-web args to be valid"),
    ));
    if let Err(e) = res {
        bail!("cargo-web build failed: {}", e);
    }

    debug!("finished executing cargo-web build");

    let target_dir = root
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release");
    // TODO: actually use 'cargo metadata' to get exact filename that will be
    // built, rather than using this hack.
    let mut wasm_file = None;
    let mut generated_js = None;
    for r in fs::read_dir(&target_dir)? {
        let entry = r?;
        let file_name = entry.file_name();
        let file_name = Path::new(&file_name);
        match file_name.extension().and_then(OsStr::to_str) {
            Some("wasm") => {
                ensure!(
                    wasm_file.is_none(),
                    "error: multiple wasm files found in {}",
                    target_dir.display()
                );
                wasm_file = Some(entry.path());
            }
            Some("js") => {
                ensure!(
                    generated_js.is_none(),
                    "error: multiple js files found in {}",
                    target_dir.display()
                );
                generated_js = Some(entry.path());
            }
            _ => {}
        }
    }
    let wasm_file = wasm_file
        .ok_or_else(|| format_err!("error: no wasm files found in {}", target_dir.display()))?;
    let generated_js = generated_js
        .ok_or_else(|| format_err!("error: no js files found in {}", target_dir.display()))?;

    let out_dir = root.join("target");

    fs::create_dir_all(&out_dir)?;

    let copy_wasm = if config.build.optimize {
        info!("optimizing wasm file...");

        let wasm_opt_res = std::process::Command::new("wasm-opt")
            .arg(wasm_file.clone())
            .arg("-g")
            .arg("-Oz")
            .arg("-o")
            .arg(out_dir.join(config.build.output_wasm_file.clone()))
            .output();

        match wasm_opt_res {
            Ok(output) => {
                ensure!(
                    output.status.success(),
                    "error: wasm-opt finished with exit-code {}",
                    output
                        .status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or("(none)".to_string())
                );

                info!("optimized.");

                false
            },
            Err(err) => {
                error!("failed to run wasm-opt: {}", err.to_string());

                true
            }
        }
    } else {
        true
    };

    if copy_wasm {
        debug!("copying wasm file");

        fs::copy(wasm_file.clone(), out_dir.join(&config.build.output_wasm_file))?;
    }

    debug!("processing js file");

    let generated_js_contents = fs::read_to_string(&generated_js)?;

    let processed_js = process_js(&generated_js, &generated_js_contents, &root, &config.build)?;

    let out_file = out_dir.join(&config.build.output_js_file);

    debug!("writing to {}", out_file.display());

    let mut output_handle = fs::File::create(out_file)?;
    output_handle.write_all(processed_js.as_bytes())?;
    output_handle.flush()?;

    Ok(())
}

fn process_js(
    file_name: &Path,
    input: &str,
    root: &Path,
    config: &BuildConfiguration,
) -> Result<String, failure::Error> {
    // first, strip out bootstrap code which relates to the browser. We don't want
    // to run this, we just want to call `__initialize` ourself.
    //
    // TODO: this is currently quite brittle and tied to the
    // version of "cargo web"...
    let whitespace_regex = regex::Regex::new("\\s+").expect("expected pre-set regex to succeed");
    let make_into_slightly_less_brittle_regex = |input: &str| {
        whitespace_regex
            .replace_all(&regex::escape(input), "\\s*")
            .replace("XXX", "[A-Za-z0-9_-]*")
    };
    let expected_prefix = r#""use strict";

if( typeof Rust === "undefined" ) {
    var Rust = {};
}

(function( root, factory ) {
    if( typeof define === "function" && define.amd ) {
        define( [], factory );
    } else if( typeof module === "object" && module.exports ) {
        module.exports = factory();
    } else {
        Rust.XXX = factory();
    }
}( this, function() {
    return (function( module_factory ) {
        var instance = module_factory();

        if( typeof process === "object" && typeof process.versions === "object" && typeof process.versions.node === "string" ) {
            var fs = require( "fs" );
            var path = require( "path" );
            var wasm_path = path.join( __dirname, "XXX.wasm" );
            var buffer = fs.readFileSync( wasm_path );
            var mod = new WebAssembly.Module( buffer );
            var wasm_instance = new WebAssembly.Instance( mod, instance.imports );
            return instance.initialize( wasm_instance );
        } else {
            var file = fetch( "XXX.wasm", {credentials: "same-origin"} );

            var wasm_instance = ( typeof WebAssembly.instantiateStreaming === "function"
                ? WebAssembly.instantiateStreaming( file, instance.imports )
                    .then( function( result ) { return result.instance; } )

                : file
                    .then( function( response ) { return response.arrayBuffer(); } )
                    .then( function( bytes ) { return WebAssembly.compile( bytes ); } )
                    .then( function( mod ) { return WebAssembly.instantiate( mod, instance.imports ) } ) );

            return wasm_instance
                .then( function( wasm_instance ) {
                    var exports = instance.initialize( wasm_instance );
                    console.log( "Finished loading Rust wasm module 'XXX'" );
                    return exports;
                })
                .catch( function( error ) {
                    console.log( "Error loading Rust wasm module 'XXX':", error );
                    throw error;
                });
        }
    }( function() {"#;

    let expected_suffix = r#"
    }
     ));
    }));
    "#;

    let expected_prefix = regex::Regex::new(&format!(
        "^{}",
        make_into_slightly_less_brittle_regex(expected_prefix)
    ))?;

    let expected_suffix = regex::Regex::new(&format!(
        "{}$",
        make_into_slightly_less_brittle_regex(expected_suffix)
    ))?;

    debug!("expected prefix:\n```{}```", expected_prefix);
    debug!("expected suffix:\n```{}```", expected_suffix);

    let prefix_match = expected_prefix.find(input).ok_or_else(|| {
        format_err!(
            "'cargo web' generated unexpected JS prefix! This means it's updated without \
             'cargo screeps' also having updated. Please report this issue to \
             https://github.com/rustyscreeps/cargo-screeps/issues and include \
             the first ~30 lines of {}",
            file_name.display(),
        )
    })?;

    let suffix_match = expected_suffix.find(input).ok_or_else(|| {
        format_err!(
            "'cargo web' generated unexpected JS suffix! This means it's updated without \
             'cargo screeps' also having updated. Please report this issue to \
             https://github.com/rustyscreeps/cargo-screeps/issues and include \
             the last ~30 lines of {}",
            file_name.display(),
        )
    })?;

    let initialize_function = &input[prefix_match.end()..suffix_match.start()];

    // screeps doesn't have `console.error`, so we define our own `console_error`
    // function, and call it.
    let initialize_function = initialize_function.replace("console.error", "console_error");

    let wasm_module_name = config
        .output_wasm_file
        .file_stem()
        .ok_or_else(|| {
            format_err!(
                "expected output_wasm_file ending in a filename, but found {}",
                config.output_wasm_file.display()
            )
        })?
        .to_str()
        .ok_or_else(|| {
            format_err!(
                "expected output_wasm_file with UTF8 filename, but found {}",
                config.output_wasm_file.display()
            )
        })?;

    let initialization_header: Cow<'static, str> = match config.initialization_header_file.as_ref()
    {
        Some(header_file) => fs::read_to_string(root.join(header_file))?.into(),
        None => include_str!("../resources/default_initialization_header.js").into(),
    };

    Ok(format!(
        r#"{}

function wasm_fetch_module_bytes() {{
    "use strict";
    return require('{}');
}}

function wasm_create_stdweb_vars() {{
    "use strict";
    {}
}}
"#,
        initialization_header, wasm_module_name, initialize_function,
    ))
}
