# Building and checking ASX

The production DLL and export probe target Windows x86-64. Portable policy and
selected-memory tests can run on Linux or Windows. Use the Rust toolchain pinned
in `rust-toolchain.toml` and the committed `Cargo.lock`.

## Portable checks

The standard Rust installation and Python test dependencies must be available.
Run the project check runner from the repository root:

```sh
sh tools/dev/check.sh --profile full
sh tools/dev/check.sh --profile official-party-ui
```

The first includes current Rust/Python tests and Clippy.
The second runs required Lua 5.1 behavior separately. These commands do not
compile a DLL, deploy or launch a game. See [testing](TESTING.md) for smaller scopes.

`tools/dev/env.sh` respects normal Cargo/Rustup environment variables. An optional
`AZCOOP_TOOLS_DIR` can select a private tool cache. Keep its resolved value and any
SDK overlay outside shared documentation; no particular editor or WSL
installation is required by this guide.

## Windows-target checks

Use a native Windows Rust toolchain, or cargo-xwin with compatible Clang and
Windows SDK configuration on Linux. Check platform-specific source explicitly:

```sh
cargo xwin check --locked --target x86_64-pc-windows-msvc -p azeroth-coop-native --tests --features test-hooks
```

Native Windows can use `cargo check` with the same target/package/features.
Windows unit tests use stand-ins and should run serially; cross-compiling alone
is not execution evidence. Current Rust integration targets are `client_binding`
`jewelry` and `enemies`.

## Intentional artifacts

`tools/dev/build-windows.sh` compiles a normal DLL; `build-chest-fixture.sh`
compiles diagnostic fixtures. Both compile only. A normal artifact for sharing
or reproducibility uses the path-remapped production builder in
`tools/build/asx_release_binary.py`. Follow [reproducibility](REPRODUCIBILITY.md)
and the artifact policy before building; normal/diagnostic artifacts are distinct.

Private game import requires explicit source/receiver paths through
`tools/dev/import122.py`. It verifies the selected executable, native inventory,
copy hashes and target receipts without importing saves/settings. Preparation
never implies deployment to an installed game or permission to launch it.

Build caches, local reports and workspace notes are described in
[local storage](LOCAL-STORAGE.md). Machine-specific setup and editor handoffs
belong under ignored local state, not at the project root.
