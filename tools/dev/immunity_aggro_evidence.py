"""Assess immunity exclusion, native immunity and explicit taunt override."""
import re
def assess(text):
 cases=range(4)
 rows=re.findall(r"immunity fixture retarget: case=(\d+) target=(None|Some\(\d+\)) expected=(\d+) restored=(true|false) effective=Some\(([^)]+)\) retaunt=Some\((\d+)\)",text)
 valid={int(i):target for i,target,expected,restored,effective,taunt in rows if int(i)in cases and int(expected)==1-int(i)%2 and restored=="true"and float(effective)==0 and int(taunt)==int(i)%2 and target in("None",f"Some({1-int(i)%2})")}
 checks=dict(
  four_immunity_entries=all(f"immunity fixture begin: case={i} slot={i%2} spell={spell} active=true old=Some({i%2}) cleared=None"in text for i,spell in enumerate([642,20580,7744,1856])),
  ordinary_target_excludes_immune=len(rows)==4 and set(valid)==set(cases),
  both_directions_retargeted=all(any(i%2==slot and target==f"Some({1-slot})"for i,target in valid.items())for slot in range(2)),
  native_pursuit_override=all(re.search(rf"official taunt pursuit: enemy=\d+ caster={slot} ",text)is not None for slot in range(2)),
  retaunt_held=all(f"immunity fixture held: case={i} owner=Some({i%2}) expected={i%2}"in text for i in cases),
  native_expiry=all(f"immunity fixture expired: case={i} active=false owner=Some({i%2}) expected={i%2}"in text for i in cases))
 return dict(checks=checks,passed=all(checks.values()),targets=valid)
