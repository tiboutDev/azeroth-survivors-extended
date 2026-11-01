# ASX documentation

Choose a task below. Maintainer procedures describe current source; published
archives and their recorded validation have their own scope.

## Start here

- [Port ASX to a native client update](compatibility/COMPATIBILITY-CHECKLIST.md): ordered review, tests, solo/coop validation and evidence.
- [Prepare a release](release/RELEASE-CHECKLIST.md): reproducible builds, signing, exact-archive trials, approval and final handoff.
- [Check expected ASX behavior](compatibility/ASX-BEHAVIOR.md): stable acceptance criteria and recurring regressions.
- [Install and play](player/INSTALLATION.md): player instructions and maintenance.

## Documentation map

| Folder | Purpose | Main references |
| --- | --- | --- |
| [player/](player/INSTALLATION.md) | Player installation, features and experimental plans | [Features](player/FEATURES.md), [workshop](player/WORKSHOP.md) |
| [development/](development/DEVELOPMENT.md) | Development environment, builds, test execution, architecture and local storage | [Building](development/BUILDING.md), [testing](development/TESTING.md), [architecture](development/RUST-ARCHITECTURE.md), [storage](development/LOCAL-STORAGE.md) |
| [compatibility/](compatibility/COMPATIBILITY-CHECKLIST.md) | Native client integration, expected behavior and patch qualification | [Workflow](compatibility/COMPATIBILITY-WORKFLOW.md), [adapter](compatibility/CLIENT-ADAPTER.md), [current client](compatibility/CLIENT-COMPATIBILITY.md), [generated feature inventory](compatibility/ASX-FEATURES.md) |
| [release/](release/RELEASE-CHECKLIST.md) | Distribution, installer ownership, reproducibility and preparation | [Installation contract](release/ASX-INSTALLATION.md), [reproducibility](release/REPRODUCIBILITY.md) |
| [features/](features/FEATURE-RULES.md) | Detailed ASX gameplay, balancing, addon API and presentation contracts | [Balance review](features/BALANCE-REVIEW.md), [addon API](features/ADDON-API.md), [pause](features/COOP-PAUSE.md), [party UI](features/OFFICIAL-PARTY-UI.md), [world UI](features/NATIVE-WORLD-UI.md) |
| [native-balance/](native-balance/NATIVE-BALANCE.md) | Native-only balancing data, independent of ASX bonuses | [Enemies](native-balance/ENEMIES.md), [economy](native-balance/ECONOMY.md), [jewelry](native-balance/JEWELRY-RANKS.md), [amulets](native-balance/AMULET-RANKS.md) |

## Maintenance

Update the authoritative contract with the corresponding implementation. Keep
operational checklists linked to detailed rules instead of copying their values.
The feature inventory is generated from `compat/features.json` and its template;
regenerate it with `python3 tools/dev/feature_catalog.py --write-doc`.

Native balance versions record provenance; refresh their values only on explicit
request. Store run-specific results, machine paths, failed attempts and release
proofs in ignored project evidence, not in these maintained references.
