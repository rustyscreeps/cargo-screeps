cargo-screeps
=============

[![crates.io version badge][cratesio-badge]][crate]

Build tool for deploying Rust WASM repositories to [Screeps][screeps] game servers.

`cargo-screeps` wraps [`cargo-web`], adding the ability to trim node.js and web javascript code from
the output files, and allows uploading directly to Screeps servers.

The other main project in this organization is [`screeps-game-api`], type-safe bindings to the
in-game Screeps API.

These two tools go together well, but do not depend on eachother. `cargo-screeps` can compile and
upload any screeps WASM project buildable with `stdweb`'s `cargo-web`, and `screeps-game-api` is
usable in any project built with `cargo-web`.

---

# Build Options

### `build`:

Configured in `[build]` config section. No required settings.

1. runs `cargo-web build --release` to build the rust source
2. strips off header `cargo-web` generates for loading WASM file from a URL or the local filesystem
3. appends initialization call using bytes from `require('<compiled module name>')`
4. puts processed JS into `target/main.js` copy compiled WASM into `target/compiled.wasm`

### `check`:

Does not require configuration.

1. performs type checking and lifetime checking without compiling code
  - runs `cargo web check` (see `cargo check` for non-WASM codebases)

### `deploy`:

Runs the deployment mode specified by the `--mode` setting, or the `default_deploy_mode`
configuration setting if none is specified.

1. runs build
2. depending on whether the mode uploads (has authentication credentials) or copies (has a
   `destination`), proceeds to deploy the built code

If copying:

1. copies compiled main file and WASM file (default `main.js` and `compiled.wasm`) from `target/` to
   `<destination directory>/<branch name>/`
2. if pruning is enabled, deletes all other files in `<destination directory>/<branch name>/`

If uploading:

1. reads `target/*.js` and `target/*.wasm`, keeping track of filenames
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

- `output_js_file`: the javascript file to export bindings and bootstrapping as
  (default `"main.js"`)
- `output_wasm_file`: the WASM file to rename compile WASM to (default `"compiled.wasm"`)
- `initialize_header_file`: a file containing the JavaScript for starting the WASM instance. See
  [overriding the default initialization header](#overriding-the-default-initialization-header)
- `features`: a list of crate features to be enabled during the build

Any of these options can be overridden for a given mode with its own build section. For instance,

```
[upload.build]
features = ["alliance_behavior"]
```

would cause a feature on your crate named `alliance_behavior` to be built when running the `upload`
mode.

### Overriding the default initialization header

`cargo-screeps` tries to make a reasonable `main.js` file to load the WASM. However, it's pretty
basic, and you might find you want to do some things in JavaScript before loading the WASM module.

Luckily, you can override this initialization! Set `build.initialize_header_file` to a file
containing the JavaScript initialization code.

Two utility functions `wasm_fetch_module_bytes` and `wasm_create_stdweb_vars` will always be
created, but the initialization header controls what actually runs.

See [docs/initialization-header.md] for more information on this.

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

  Either an auth_token or your username/password can be supplied. When both are set the auth token is used. For private servers set a password using [screepsmod-auth].
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

[cratesio-badge]: http://meritbadge.herokuapp.com/cargo-screeps
[crate]: https://crates.io/crates/cargo-screeps/
[`screeps-game-api`]: https://github.com/rustyscreeps/screeps-game-api/
[`cargo-web`]: https://github.com/koute/cargo-web
[screepsmod-auth]: https://www.npmjs.com/package/screepsmod-auth
[screeps]: https://screeps.com/
