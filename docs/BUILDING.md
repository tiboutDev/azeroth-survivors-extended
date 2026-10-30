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
`tools/dev/import_official.py`. It verifies the selected executable, native inventory,
copy hashes and target receipts without importing saves/settings. Preparation
never implies deployment to an installed game or permission to launch it.

Build caches, local reports and workspace notes are described in
[local storage](LOCAL-STORAGE.md). Machine-specific setup and editor handoffs
belong under ignored local state, not at the project root.

## Live validation from WSL

Keep the authoritative Git checkout on the Linux filesystem. Build there, then
explicitly synchronize its current sources/tools and selected DLL to a disposable
Windows receiver. Import the reviewed native assets into that receiver with the
private importer above. A generated receiver is never a second source checkout.
Do not copy an old installed DLL merely because its filename matches.

The live dispatcher uses Windows APIs and must run with a **Windows Python** from
the receiver, rather than Linux Python under WSL. It also requires the isolated
runtime import receipt and an ASX-DEV.json receipt matching the installed DLL hash.
Keep the concrete pairing, tool-cache paths and staging commands in ignored local
workspace notes. Source checks and cross-compilation do not create that pairing.

After explicit staging, these bounded Windows commands exercise different paths:

    python tools/dev/background_desktop.py --extensions-trial 2 --seconds 150
    python tools/dev/background_desktop.py --extensions-trial 2 --extensions-normal --seconds 100

The first requires a diagnostic artifact; the second requires a normal artifact
with no test hooks. They must run sequentially. Review their result.json files,
not only process exit or launch acknowledgements. Allow for native loading and
reward-choice time before reducing the scenario duration.

Disposable profile generation separately checks its reviewed executable allowlist;
a registry update alone does not enable profiles for a new client. The native
save parser remains responsible for accepting the fixture's talents/progression.
Ordinary extension trials take no screenshots. ASX_TEST_VISUAL=1 enables sparse
captures only for an explicitly authorized visual trial; logs alone establish no
claim about the rendered layout.

See the [compatibility workflow](COMPATIBILITY-WORKFLOW.md) for resolved manifests,
artifact receipts, declarative suites and bounded Lua evidence.
