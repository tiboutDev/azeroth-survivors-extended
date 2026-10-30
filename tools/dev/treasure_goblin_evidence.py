"""Bounded evidence for optional native Treasure Goblin trials."""
import json
import math
import re


def assess(logs, disabled=False):
    checks = {
        "core_asx_active": bool(logs) and all("ASX active:" in text for text in logs),
        "no_event_failure": all(not any(error in text for error in (
            "Treasure Goblin event stopped:", "Treasure Goblins unavailable:",
            "Goblin appearance fallback rejected:", "Goblin cleanup failed once:", "Goblin notice rejected:", "Goblin equipment unavailable:", "Goblin appearance ownership restore failed:", "Goblin recycle exclusion failed:", "Goblin final motion rejected:",
        )) for text in logs),
    }
    if disabled:
        checks.update(
            optional_disabled=bool(logs) and all("Treasure Goblins disabled;" in text for text in logs),
            no_goblin=all("treasure goblin spawned:" not in text for text in logs),
        )
    else:
        host = logs[0] if logs else ""
        spawned = re.findall(r"treasure goblin spawned:.*guid=(\d+)", host)
        escaped = re.findall(r"treasure goblin escaped: guid=(\d+)", host)
        rewards = re.findall(r"treasure goblin rewarded: guid=(\d+) recipients=(\d+)", host)
        checks.update(
            two_creatures=len(spawned) == len(set(spawned)) == 2,
            outcomes_unique=len(escaped)+len(rewards)==len(spawned) and len(set(escaped+[guid for guid,_ in rewards]))==len(spawned) and set(escaped+[guid for guid,_ in rewards])==set(spawned),
            escape_without_reward=all(guid not in escaped for guid,_ in rewards),
            unique_kill_reward=bool(rewards) and len(rewards)==len(set(guid for guid,_ in rewards)) and all(guid in spawned and int(gold)>0 for guid,gold in rewards),
            taunts_excluded_with_normal_control=bool(re.search(r"treasure goblin taunt control: normal=(?:true|false) goblin=false", host)) if len(logs) == 1 else "treasure goblin taunt control: normal=true goblin=false" in host,
        )
        awards=re.findall(r"treasure goblin amulets: event=(\d+) slot=(\d+) guid=(\d+) count=(\d+) marks=([\d.]+) keys=(\[[^\r\n]+\])",host)
        checks["personal_amulets_and_marks"] = bool(rewards) and all(
            {(int(slot)) for event,slot,guid,count,marks,keys in awards if event==reward and int(count)==3 and float(marks)>=0 and float(marks).is_integer()}==set(range(len(logs)))
            for reward,_ in rewards)
        checks["personal_rewards_unique"] = len(awards)==len({(event,slot,guid)for event,slot,guid,_,_,_ in awards})
        samples = {}
        for guid, pose in re.findall(r"treasure goblin native: guid=(\d+).*position=(\[[^\r\n]+\])", host):
            try:
                value = json.loads(pose)
                if len(value) == 3 and all(math.isfinite(v) for v in value):
                    samples.setdefault(guid, []).append(value)
            except (TypeError, ValueError):
                pass
        # Sparse two-second chords cannot establish direction along curved paths.
        # Require bounded, per-update actual movement aggregates instead.
        fine = re.findall(r"treasure goblin fine motion: guid=(\d+) samples=(\d+) min_projection=([-\d.eE+]+) max_gap=([\d.eE+]+)", host)
        checks["sampled_flee_never_approaches"] = (
            len(fine) == len(spawned) and {row[0] for row in fine} == set(spawned)
            and all(int(count) > 100 and math.isfinite(float(projection))
                    and float(projection) >= -0.1 and 0 < float(gap) < 0.25
                    for _, count, projection, gap in fine))
        checks["native_armor_compositor"] = bool(logs) and all("treasure goblin equipment: native armor compositor selected" in text for text in logs)
        arena = re.search(r"treasure goblin arena: center=(\[[^\r\n]+\]) radius=([\d.]+) clearance=([\d.]+)", host)
        inside = False
        if arena:
            try:
                center, radius, clearance = json.loads(arena[1]), float(arena[2]), float(arena[3])
                inside = bool(samples) and all(math.dist(center, point[:2]) <= radius-clearance+0.01 for points in samples.values() for point in points)
            except (TypeError, ValueError):
                pass
        checks["sampled_inside_native_arena"] = inside
        checks["native_elite_priority"] = "elite=true" in host
        checks["native_flee_moves"] = any(
            guid in spawned and len(values) > 1 and any(
                sum((a - b) ** 2 for a, b in zip(values[0], point)) > 4
                for point in values[1:]
            ) for guid, values in samples.items()
        )
        for index, guest in enumerate(logs[1:], 1):
            checks[f"guest{index}_native_name"] = "treasure goblin received: opcode=61" in guest
            for opcode, label in (("a9", "creation"), ("dd", "movement"), ("aa", "despawn")):
                received = set(re.findall(rf"treasure goblin received: opcode={opcode} guid=(\d+)", guest))
                checks[f"guest{index}_native_{label}"] = bool(spawned) and set(spawned) <= received
    return dict(checks=checks, passed=all(checks.values()),
                scope="Native diagnostic behavior and receipt evidence; screenshots require separate visual review.")
