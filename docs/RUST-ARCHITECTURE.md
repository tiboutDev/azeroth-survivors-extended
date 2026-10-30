# Rust architecture and cleanup decisions

The supported target is official 0.130.0. This document describes source ownership,
not a promise that every feature is independent of native layouts or that the
repository is ready to publish.

## Workspace shape

Keep one workspace and four packages. The root package is the Windows DLL and
composition root; `asx-gameplay` contains portable policy; `asx-game` owns
selected native translation and verified build data; `avrt-probe` is an independent
Windows export diagnostic. A root package inside a workspace is a standard Cargo
layout; a virtual workspace or a crate per feature is not required. See the
[Cargo workspace reference](https://doc.rust-lang.org/cargo/reference/workspaces.html).

The workspace explicitly uses resolver 2 and shares version, edition, license,
publication policy and internal path dependencies through Cargo inheritance.
These preserve the existing 2021 edition and dependency
versions. Release profiles and the lockfile remain workspace-wide. The existing
`src/`, integration `tests/` and `crates/` locations follow
[Cargo package conventions](https://doc.rust-lang.org/cargo/guide/project-layout.html).

The two internal library packages use the `asx-` project prefix, with matching
directory names. Rust imports use `asx_game` and `asx_gameplay`. This naming change
does not rename the Windows loader artifact `avrt.dll` or external environment,
addon and protocol identifiers. No edition or root-package move is required.

Windows build resources live under `resources/windows/`. Lua presentation stays
under `native-ui/`: it is packaged as an addon and interpreted by the game, not
compiled as a Rust module. The separate directory keeps packaging and Lua 5.1
checks explicit; it does not imply an independent UI application.

## Dependency direction

| Owner | May depend on | Must not own |
| --- | --- | --- |
| Portable gameplay | Standard-library values and policy contracts | Windows APIs, native offsets, hooks, transport or process memory |
| Selected game access | Portable policy types and bounded memory interfaces | Global feature/session authority |
| DLL orchestration | Selected native access and portable policy | Version-dependent offsets or a second simulation |
| Presentation | Owned cosmetic views and selected native formatting | Gameplay authority or guessed owner identity |
| Test fixtures | Explicit diagnostic services and stand-ins | Automatic deployment or normal-build test activation |

Use modules for cohesive features and crates for independent dependency/platform
boundaries. Keep implementation details private; widen visibility to the smallest
ancestor that needs an interface. Rust's
[module and privacy rules](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)
provide the enforcement mechanism. Avoid wildcard production imports in touched
coordinators and avoid exposing Runtime, memory accessors, mutable containers,
Deref/Any escapes or arbitrary callbacks through service interfaces.

Session uses callback-specific ports. Jewelry, XP, pause, menu, lifecycle, elite
rewards and blessings use bounded services whose production bindings own native
access. Other feature modules still have direct Runtime dependencies. These
internal interfaces do not constitute a public plugin ABI or a completed global
native-free contract.

## Current native data

`compat/clients/0.130.0.json` and `compat/binding-template.rs` now own the
resolved current descriptor. Historical modules are test-only references, not
production dependencies or selectable clients. Canonical keys remain explicit;
changing a generated literal still requires native review.

Historical readers, profiles and their exclusive tools/tests are retired. They
have no production or current-tool consumer. Their history remains in Git and
private evidence backups; there is no inspection feature to maintain solely for
its own tests. Current support still uses `memory.rs` and `enemies.rs`, covered
through selected current-client memory fixtures.

## Verification and publication

Use portable policy/memory tests, explicit Lua behavior and Windows stand-ins for
the changed boundaries. Compiler checks and synthetic/native stand-ins do not
establish live gameplay or rendered UI evidence. See [testing](TESTING.md).

Current-tree local-information scans and reachable-history scans are distinct.
History findings are not removed by sanitizing a working file. Reproducibility
references identify a builder, toolchain, input fingerprint and exact artifact;
a candidate remains pending review until explicitly approved. Do not substitute
Windows and WSL builder hashes for each other or weaken publication gates to
make an unfinished audit pass.

## Source naming

Rust modules use `snake_case`, following the
[Rust naming conventions](https://rust-lang.github.io/api-guidelines/naming.html).
The DLL's active coordination module is `runtime`, covering solo and cooperative
extensions. Both `runtime.rs` with its `runtime/` children and the separate
`presentation/` module are conventional layouts. Exact-build descriptors keep
version identity where that identity is part of their native-data contract;
inspection tests are grouped separately instead of pretending those formats are
current clients.

## Resolved selected descriptor

Production native consumers use `asx_game::current`. The current manifest and
binding template generate a self-contained descriptor; historical descriptors
are test-only references. Native field gaps remain explicit failures. Manifest
changes require exact-image and ABI review, not merely successful generation.
