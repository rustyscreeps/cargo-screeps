Unreleased
==================

- (Breaking) Replaced cargo-web functionality with wasm-pack; use 0.4.x release for stdweb-based
  crates!
- (Breaking) Remove `output_js_file` and `output_wasm_file` build options which configured the
  related cargo-web features; add `build_profile` and `out_name` build options which configure
  related wasm-pack features.
- (Breaking) Remove `features` build option, replacing with the `extra_options` build option
  which can pass flags like `--features=feature-name` or any other needed command line flags to
  wasm-pack.
- (Breaking) Remove `initialization_header_file` deployment configuration option, replacing with
  `include_files` option, which specifies which directories inside your working directory to
  include files from for deployment.
- (Breaking) Remove `check` command, as it's not available under wasm-pack - use `cargo check`.

0.4.0 (2022-09-08)
==================

- (Breaking) Remove `ptr` option from configuration, replaced with `prefix` which can be set to
  a desired API URL prefix - `prefix = "ptr"` replaces the functionality of `ptr = true`
- (Breaking) The default `port` is now 443 and `ssl` now defaults to true (previous default was
  HTTPS for screeps.com and HTTP for other hosts)
- Added configuration for upload HTTP request timeout `http_timeout` (#26)
- `branch` option is no longer required, `"default"` is the default if none is configured
- Options from the `[build]` section can be overridden for any individual mode in `[modename.build]`
- `[build]` options now support specifying `features`, a list of crate features to be passed to
  cargo-web

0.3.3 (2019-07-20)
==================

`cargo-screeps` no longer requires installing `cargo-web`, so if you've
previously installed both, you can now freely remove the latter. It won't hurt
to have it installed, but `cargo-screeps` will not use it.

- Fix `cargo screeps check` (#144)
- Removed encompassing cargo workspace (#180)
- Update `cargo-screeps` to more rust 2018 idioms (#182)
- Depend directly on `cargo-web` (#183)

0.3.2 (2019-05-20)
==================

- Add support for using Screeps auth tokens instead of username/password ((#137), thanks [@npfund]!)
- Support cargo-web version 0.6.25 (#138, thanks [@babariviere]!)

0.3.1 (2019-02-19)
==================

- Fix Cargo.toml configuration for uploading.

0.3.0 (2019-02-19)
==================

- Update to `reqwest` 0.9
- Change default WASM module initialization and allow projects to override
  module initialization via configuration (see
  [docs/initialiation-header.md](https://github.com/rustyscreeps/cargo-screeps/blob/master/docs/initialization-header.md))

0.2.1 (2018-10-26)
==================

- Update expected format to match `cargo-web` version `0.6.19`
- Fix subcommand documentation in README

0.2.0 (2018-09-15)
==================

- (Breaking) Move upload configuration into an '[upload]' config section
- Add copy deployment option with '[copy]' config section.
- Rework commands so 'cargo screeps --upload' and '--build' are now
  'cargo screeps upload' and 'cargo screeps build'.
- Add 'cargo screeps deploy' command which uploads or copies  depending on
  'default_deploy_mode' config option.
- Add warnings for invalid configuration options. This is primarily  to help
  with migration, but it can also help catch typos.
- Add '-c' cli option to load screeps.toml configuration besides 'screeps.toml'.

_Thanks to @jechase for contributing to this cargo-screeps release._

0.1.6 (2018-09-05)
==================

- Add creation of '__initialize' function for easy re-initalization of WASM
  module to cargo-screeps.

0.1.5 (2018-04-05)
==================

- Fix mistake in cargo-web prefix test.

0.1.4 (2018-08-16)
==================

- Update to cargo-web version 0.6.15 output format.

0.1.3 (2018-06-17)
==================

- Update to match cargo-web version 0.6.12.

0.1.2 (2018-04-10)
==================

- Fix bug with path joins not being cross-platform compatible.

0.1.1 (2018-04-05)
==================

- Fix bug where new projects could not build due to an OS error not finding the
  output file.

0.1.0 (2018-04-03)
==================

- Remove dependency on nightly rust.

0.0.5 (2018-03-13)
==================

- Switch to using `console_error` rather than trying to polyfill `console.error`
  since `console` prototype is changed every tick in screeps.


0.0.4 (2018-03-06)
==================

- Put author/version on subcommand for correct information.
- Update to match latest 'cargo web' generated format.

0.0.3 (2018-03-06)
==================

- Add support for custom output JS/WASM files to `cargo-screeps`

0.0.2 (2018-03-01)
==================

- Support for crate names with dashes

0.0.1 (2018-03-01)
==================

- Initial release

[@babariviere]: https://github.com/babariviere
[@npfund]: https://github.com/npfund
