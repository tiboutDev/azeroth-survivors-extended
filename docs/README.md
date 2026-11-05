# AZSX documentation

Choose a task below. Maintainer procedures describe current source; published
archives and their recorded validation have their own scope.

## Start here

- [Make a focused change](development/DEVELOPMENT.md): source ownership, implementation, proportionate checks and stopping.
- [Port AZSX to a native client update](compatibility/COMPATIBILITY-CHECKLIST.md): ordered review, tests, solo/coop validation and evidence.
- [Prepare a release](release/RELEASE-CHECKLIST.md): reproducible builds, signing, exact-archive trials, approval and final handoff.
- [Check expected AZSX behavior](compatibility/AZSX-BEHAVIOR.md): stable acceptance criteria and recurring regressions.
- [Install and play](player/INSTALLATION.md): player instructions and maintenance.

## Documentation map

| Folder | Purpose | Main references |
| --- | --- | --- |
| [player/](player/INSTALLATION.md) | Player installation, features and experimental plans | [Features](player/FEATURES.md), [workshop](player/WORKSHOP.md) |
| [development/](development/DEVELOPMENT.md) | Development environment, builds, test execution, architecture and local storage | [Building](development/BUILDING.md), [testing](development/TESTING.md), [architecture](development/RUST-ARCHITECTURE.md), [storage](development/LOCAL-STORAGE.md) |
| [compatibility/](compatibility/COMPATIBILITY-CHECKLIST.md) | Native client integration, expected behavior and patch qualification | [Workflow](compatibility/COMPATIBILITY-WORKFLOW.md), [adapter](compatibility/CLIENT-ADAPTER.md), [current client](compatibility/CLIENT-COMPATIBILITY.md), [generated feature inventory](compatibility/AZSX-FEATURES.md) |
| [release/](release/RELEASE-CHECKLIST.md) | Distribution, installer ownership, reproducibility and preparation | [Installation contract](release/AZSX-INSTALLATION.md), [reproducibility](release/REPRODUCIBILITY.md) |
| [features/](features/FEATURE-RULES.md) | Detailed AZSX gameplay, balancing, addon API and presentation contracts | [Balance review](features/BALANCE-REVIEW.md), [addon API](features/ADDON-API.md), [pause](features/COOP-PAUSE.md), [party UI](features/OFFICIAL-PARTY-UI.md), [world UI](features/NATIVE-WORLD-UI.md) |
| [native-balance/](native-balance/NATIVE-BALANCE.md) | Native-only balancing data, independent of AZSX bonuses | [Enemies](native-balance/ENEMIES.md), [economy](native-balance/ECONOMY.md), [jewelry](native-balance/JEWELRY-RANKS.md), [amulets](native-balance/AMULET-RANKS.md) |

## Maintenance

Update the authoritative contract with the corresponding implementation. Keep
operational checklists linked to detailed rules instead of copying their values.
The development guide owns the daily sequence, TESTING.md owns check selection,
and each checklist owns only its compatibility or release steps. Keep past debug
notes and run status in ignored evidence.
The feature inventory is generated from `compat/features.json` and its template;
regenerate it with `python3 tools/dev/feature_catalog.py --write-doc`.

Native balance versions record provenance; refresh their values only on explicit
request. Store run-specific results, machine paths, failed attempts and release
proofs in ignored project evidence, not in these maintained references.

- [ASX to AZSX migration](player/AZSX-MIGRATION.md)
