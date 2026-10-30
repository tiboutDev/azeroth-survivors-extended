"""Reviewed ownership and candidate validation, never execution/ABI approval."""
import json
from pathlib import Path
try:
    from .client_registry import ROOT
    from .test_plan import PROFILES
except ImportError:
    from client_registry import ROOT
    from test_plan import PROFILES


def validate_policy(policy, validation, features, suites, root=ROOT):
    if policy.get("schema") != 1 or validation.get("schema") != 1:
        raise ValueError("Unsupported contract attribution schema")
    feature_ids = {f["id"] for f in features}
    links = policy["contracts"]
    evidence = policy.get("evidence", {})
    notes = policy.get("conservative_notes", {})
    if set(evidence) != set(links) or set(notes) & set(links):
        raise ValueError("Explicit ownership needs evidence and cannot be conservative")
    if set(validation["features"]) != feature_ids:
        raise ValueError("Every feature needs an explicit validation disposition")
    for contract, owners in links.items():
        if not isinstance(owners, list) or not owners or len(owners) != len(set(owners)) or not set(owners) <= feature_ids:
            raise ValueError("Invalid explicit feature owners: " + contract)
        item = evidence[contract]
        if not item.get("reason") or item.get("role") not in ("native-operation", "diagnostic", "retained") or item.get("shared") != (len(owners) > 1):
            raise ValueError("Incomplete attribution rationale: " + contract)
        references = item.get("references", [])
        if not references:
            raise ValueError("Explicit attribution needs reviewed source references")
        for ref in references:
            path = Path(ref)
            target = (root / path).resolve()
            if path.is_absolute() or ".." in path.parts or "\\" in ref or not target.is_relative_to(root.resolve()) or not target.is_file():
                raise ValueError("Invalid attribution source reference: " + ref)
    if any(not isinstance(note, str) or not note.strip() for note in notes.values()):
        raise ValueError("Missing conservative attribution reason")
    for feature, item in validation["features"].items():
        profiles = item.get("profiles", [])
        if not profiles or len(profiles) != len(set(profiles)) or not set(profiles) <= set(PROFILES) or not item.get("scope_note"):
            raise ValueError("Unknown or undocumented validation profile: " + feature)
    for feature,item in validation['features'].items():
        for name in item.get('live_suites',[]):
            if name not in suites or feature not in suites[name]['features']:
                raise ValueError('Live scenario does not own feature: '+feature)
        scenario=item.get('targeted_scenario')
        if scenario and (scenario.get('profile')not in PROFILES or any(not scenario.get(key)for key in ('trigger','expected','limit'))):
            raise ValueError('Incomplete targeted scenario: '+feature)
    for suite in suites.values():
        if not set(suite["features"]) <= feature_ids:
            raise ValueError("Scenario refers to an unknown feature")
    return links, evidence, notes


def load_policy(features, suites):
    policy = json.loads((ROOT / "compat/contract-links.json").read_text())
    validation = json.loads((ROOT / "compat/feature-validation.json").read_text())
    links, evidence, notes = validate_policy(policy, validation, features, suites)
    return links, evidence, notes, validation["features"]


def apply_attribution(contracts, features, suites):
    links, evidence, notes, validation = load_policy(features, suites)
    identifiers = [c["id"] for c in contracts]
    if len(identifiers) != len(set(identifiers)):
        raise ValueError("Ambiguous native contract identifiers")
    unknown = (set(links) | set(notes)) - set(identifiers)
    if unknown:
        raise ValueError("Ownership refers to absent native contracts: " + str(sorted(unknown)))
    sources = {c["source"]: set() for c in contracts}
    for contract in contracts:
        sources[contract["source"]].update(links.get(contract["id"], []))
    by_feature = {f["id"]: f for f in features}
    for contract in contracts:
        key = contract["id"]
        if key in links:
            owners = links[key]
            rationale = evidence[key]
            basis = "explicit"
        else:
            owners = sorted(sources[contract["source"]] | {
                f["id"] for f in features if "unclassified" in contract["domains"] or set(f["native_domains"]) & set(contract["domains"])
            })
            rationale = dict(reason=notes.get(key, "New/unreviewed contract: preserve domain and shared-source attribution."), references=[contract["source"]], role="unreviewed", shared=len(owners) > 1)
            basis = "conservative-domain"
        contract.update(features=owners, link_basis=basis, link_evidence=rationale,
                        feature_dispositions={owner: by_feature[owner]["disposition"] for owner in owners},
                        validation_profiles=sorted({profile for owner in owners for profile in validation[owner]["profiles"]}),
                        validation_suites=[name for name, suite in suites.items() if set(owners) & set(suite["features"])],
                        validation_scope="Candidate feature-level checks only; no per-contract execution or full coverage claim.")
    for feature in features:
        declared = [name for name, suite in suites.items() if feature["id"] in suite["features"]]
        feature.update(validation_profiles=validation[feature["id"]]["profiles"],
                       declared_live_suites=declared, live_suite_gap=not bool(declared),
                       validation_note=validation[feature["id"]]["scope_note"],
                       runtime_disposition=feature["disposition"],
                       live_validation_context="retained-disabled" if feature["disposition"] == "disabled" else "upstream-native" if feature["disposition"] == "upstream" else "declared-scenario-context")
    return contracts, features
