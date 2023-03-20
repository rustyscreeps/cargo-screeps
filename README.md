cargo-screeps
=============

[![Linux Build Status][actions-image]][actions-builds]
[![crates.io version badge][cratesio-badge]][crate]

Build tool for deploying Rust WASM repositories to [Screeps][screeps] game servers.

`cargo-screeps` wraps [`wasm-pack`], adding the ability to trim node.js and web javascript code from
the output files, and allows uploading directly to Screeps servers.

The other main project in this organization is [`screeps-game-api`], type-safe bindings to the
in-game Screeps API.

These two tools go together well, but do not depend on eachother. `cargo-screeps` can compile and
upload any screeps WASM project buildable with `wasm-bindgen`'s `wasm-pack`, and `screeps-game-api` is
usable in any project built with `wasm-pack`.

---

# Build Options

### `build`:

Configured in `[build]` config section. No required settings.

1. runs `wasm-pack --target nodejs` to build the rust source for Screeps: World bots, or
   `wasm-pack --target web` to for Screeps: Arena bots.
2. Modifies the generated module's javascript loader file to be compatibile with Screeps;
   adds a polyfill for `TextEncoder`/`TextDecoder`, and replaces the node-compatible module
   loader function with one that works with Screeps: World when the `build_mode` is set to
   `world` (the default)

### `deploy`:

Runs the deployment mode specified by the `--mode` setting, or the `default_deploy_mode`
configuration setting if none is specified.

1. runs build
2. depending on whether the mode uploads (has authentication credentials) or copies (has a
   `destination`), proceeds to deploy the built code

If copying (when `destination` is defined):

1. copies compiled `.js`/`.mjs` and `.wasm`/`.bin` files from the directories specified in
   `include_files` (default `pkg` and `javascript`) to the specified directory and branch
2. if pruning is enabled, deletes all other files in `<destination directory>/<branch name>/`

If uploading (when `auth_token` or `username` and `password` are defined):

1. reads compiled `.js`/`.mjs` and `.wasm`/`.bin` files from the directories specified in
   `include_files` (default `pkg` and `javascript`).
2. reads `screeps.toml` for upload options
3. uploads all read files to server, using filenames as the filenames on the server

### `upload`:

A shortcut for `cargo screeps deploy -m upload`.

### `copy`:

A shortcut for `cargo screeps deploy -m copy`.

# Configuration Options

## No namespace

- `default_deploy_mode`: controls what mode `cargo screeps deploy` uses if the `--mode`/`-m` option
  is not set.

## `[build]`

This configures general build options.

- `build_profile`: The build profile that should be used; `release`, `dev`, or `profiling`.
- `out_name`: The name used for the module created by `wasm-pack` within the `pkg` directory.
  Defaults to the name of your crate as defined in Cargo.toml.
- `extra_options`: Any extra command line flags you'd like to be passed to `wasm-pack`, such as
  enabling features.

Any of these options can be overridden for a given mode with its own build section. For instance,

```
[upload.build]
extra_options = ["--features=alliance_behavior"]
```

would cause a feature on your crate named `alliance_behavior` to be built when running the `upload`
mode.

## Configuration modes

Configuration modes can either copy the built files to a destination directory, or upload to a
destination server using the Screeps API.

A mode should either have a filesystem destination to copy to, or authentication credentials (and
optionally, server information) to upload to.

Optionally, it also have a sub-table `[mode.build]` to override any of the global `[build]` options.

### Copy Options

Options for deploying to a filesystem location.

- `destination`: the directory to copy files into

  If this path is not absolute, it is interpreted as relative to `screeps.toml`.
- `branch`: the "branch" to copy into

  This is the subdirectory of `destination` which the js/wasm files will be copied into. Default is `"default"`.
- `prune`: if true, extra files found in the destination/branch directory will be deleted. Default is `false`.

### Upload Options

Options for deploying to a Screeps server.

- `auth_token`: an auth token for your Screeps account
- `username`: your Screeps username or email
- `password`: your Screeps password

  Either an auth_token or your username/password can be supplied. When both are set the auth token is used. For private servers, set a password using [screepsmod-auth].
- `branch`: the "branch" to copy into

  This is the "branch" on the screeps server to deploy to. Default is `"default"`.
- `prefix`: if set, adds a URL prefix to the upload path.  Use `"ptr"` or `"season"` to upload to
  the public test realm and seasonal servers, respectively.
- `hostname`: the hostname to upload to

  For example, this could be `screeps.com`, `localhost` or `server1.screepsplu.us`. Default is `screeps.com`.
- `ssl`: whether to connect to the server using ssl

  This should generally be true for the main server and false for private servers. Default is `true`.
- `port`: port to connect to server with

  This should generally be set to `21025` for private servers. Default is `443`.

# Updating `cargo screeps`

To update `cargo-screeps`, simply repeat the install process with the `--force` (`-f`) flag.

After updating, you'll want to do a full `cargo clean` to remove any old artifacts which were built
using the older version of `cargo-screeps`.

```sh
cargo install -f cargo-screeps
cargo clean
cargo screeps build
```

[cratesio-badge]: https://img.shields.io/crates/v/cargo-screeps.svg
[crate]: https://crates.io/crates/cargo-screeps/
[actions-image]: https://github.com/rustyscreeps/cargo-screeps/actions/workflows/build.yml/badge.svg
[actions-builds]: https://github.com/rustyscreeps/cargo-screeps/actions/workflows/build.yml
[`screeps-game-api`]: https://github.com/rustyscreeps/screeps-game-api/
[`wasm-pack`]: https://rustwasm.github.io/wasm-pack/
[screepsmod-auth]: https://www.npmjs.com/package/screepsmod-auth
[screeps]: https://screeps.com/
