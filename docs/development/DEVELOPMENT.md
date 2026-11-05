# AZSX development

AZSX extends the selected official native client. See the
[current client boundary](../compatibility/CLIENT-COMPATIBILITY.md) for its identity
and [README](../../README.md) for the player-facing overview.

## Daily workflow

1. Read `AGENTS.md`, inspect Git status and identify the affected behavior.
2. Implement one coherent block in the authoritative WSL checkout.
3. Run its focused checks from [TESTING.md](TESTING.md); diagnose failures and
   rerun the affected checks.
4. At a compatibility/release milestone, run `full` once after the code is ready.
   Carry valid results forward when their inputs have not changed.
5. Build/stage only if needed, then use the bounded live scenario that covers the
   remaining observation. Stop when that scope passes; retain any unresolved issue.

Documentation-only work needs a content/link review. Tooling changes need their
own tests. Neither requires a DLL or game campaign. No command in the check runner
builds, deploys or launches a game.

For a native update, follow the [compatibility checklist](../compatibility/COMPATIBILITY-CHECKLIST.md):
finish **all code wiring and code validation before candidate builds or live work**.
Release preparation has its own [checklist](../release/RELEASE-CHECKLIST.md).

## Source organization

| Area | Location | Responsibility |
| --- | --- | --- |
| Rules | `crates/azsx-gameplay/src/` | Native-independent decisions and state transitions |
| Orchestration | `src/runtime.rs`, `src/runtime/` | Activation, feature coordination and lifecycle |
| Native access | `crates/azsx-game/src/official.rs`, `official_native/`, `bindings/` | Selected schemas, memory IO, calls and hooks |
| Presentation | `src/presentation/`, `native-ui/`, `addons/AZSXUI/` | Retained native UI and optional addon |
| Distribution | `azsx/`, `tools/build/` | Launcher, packaging and transactional updates |
| Development tools | `tools/dev/`, `tests/` | Check selection, native review, fixtures and evidence assessors |
| Windows resources | `resources/windows/`, `build.rs` | Version resource and avrt export forwarding |
| Build references | `validation/` | Approved artifact identities and reproducibility inputs |

Keep one Cargo workspace with the root DLL, `azsx-gameplay`, `azsx-game` and the
independent `avrt-probe`. See [Rust architecture](RUST-ARCHITECTURE.md) for dependency
boundaries and [building](BUILDING.md) for commands/toolchain setup.

`native-ui/official-party.lua` is loaded by the game, separately from the DLL.
`version.rc`, its compiled `version.res` and the export list are intentional build
inputs. Historical descriptors remain test references; production resolves native
data through `azsx_game::current` rather than chaining old translations.

## Coordination owners

| Module under `src/runtime/` | Ownership |
| --- | --- |
| `bootstrap.rs` | Activation and ordered hook installation |
| `xp_hooks.rs`, `experience.rs` | Shared XP and native dead-player receipt admission |
| `pause.rs`, `menu_pause.rs` | Group/personal menu ownership and native UI callbacks |
| `session.rs` | Native command, world, update and HUD callbacks |
| `lifecycle.rs` | Run/session teardown and transitions |
| `event_services.rs`, `reward_services.rs` | Bounded native bindings for event/reward coordinators |

Services expose operations and owned observations. They do not expose native
containers or mutable Runtime guards. Some feature modules still use Runtime;
this organization is not a public plugin ABI or a complete domain migration.
Native reentry and identity rules are in the [adapter contract](../compatibility/CLIENT-ADAPTER.md).

## Documentation and generated metadata

Use the [documentation index](../README.md) to find each authoritative reference.
Change affected acceptance criteria and detailed rules with the implementation.
Native balance references keep their recorded provenance; refresh values only
when requested.

`azsx/supported-games.json` owns executable identity/archive inventory.
`compat/clients/` and its binding template own native literals. The feature catalog
owns stable IDs and dispositions. Generate, then check:

```sh
python3 tools/dev/binding_registry.py --write
python3 tools/dev/feature_catalog.py --write-doc
python3 tools/dev/azsx_dev.py check
```

Generation does not review a native ABI. Tool/configuration details are in the
[compatibility tooling reference](../compatibility/COMPATIBILITY-WORKFLOW.md).

## Cleanup and local state

Check consumers before moving/removing a module, test or dependency. Versioned
filenames can still be useful oracles, and old branding can be required by
migration. Retire exclusive tests with obsolete code; retain boundary regressions.
Avoid splitting modules or introducing a new abstraction solely to make the tree
look smaller.

Machine paths, pairing, handoffs, outputs, private images and unique work backups
belong in ignored local storage; see [LOCAL-STORAGE.md](LOCAL-STORAGE.md). Generated
Windows receivers are execution copies. Do not develop there or copy their old
binaries back into source.

## Public source and release work

Preserve public Git history. A requested public source import uses a reviewed
snapshot in a checkout based on the public branch, without private ancestry,
ignored evidence, build outputs or Git metadata. Compare its intended base and
player documentation before applying it. The [publication checks](TESTING.md#publication-checks)
and release procedure apply when publication/release work is actually requested.
Commit and push remain explicit actions.
