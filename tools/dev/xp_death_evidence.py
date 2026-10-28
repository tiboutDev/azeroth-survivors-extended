"""Read-only XP conservation across two native deaths/resurrections."""
import ast,collections,re

def assess(text):
 credits=collections.defaultdict(int);replays=collections.defaultdict(int)
 for slot,guid,amount in re.findall(r"official XP deferred: slot=(\d+) guid=(\d+) amount=(\d+) pending=\d+",text):credits[(int(slot),int(guid))]+=int(amount)
 for slot,guid,amount in re.findall(r"official XP replayed: slot=(\d+) guid=(\d+) amount=(\d+)",text):replays[(int(slot),int(guid))]+=int(amount)
 deaths={int(s)for s in re.findall(r"XP death confirmed: slot=(\d+)",text)};revived={int(s)for s in re.findall(r"XP death revived: slot=(\d+)",text)}
 final=[]
 for stage,clock,raw in re.findall(r"XP death state: stage=(\d+) clock=([\d.]+) actors=(\[[^\r\n]+\])",text):
  if int(stage)<6:continue
  actors=ast.literal_eval(re.sub(r"\b(Playing|Choosing|Paused|Downed|Inactive)\b",r'"\1"',raw))
  if len(actors)==2 and all(a[2]=="Playing"for a in actors):final.append(actors[0][3:]==actors[1][3:])
 checks={"both_native_deaths":deaths=={0,1},"both_native_revivals":revived=={0,1},"host_and_guest_shared_xp_captured":{s for s,g in credits}=={0,1},"all_deferred_xp_replayed_exactly_once":bool(credits)and dict(credits)==dict(replays),"post_revival_progression_equal":len(final)>=3 and all(final),"xp_recovery_clean":not any("rejected"in line and("deferred XP"in line or"XP replay"in line)for line in text.splitlines())}
 return {"checks":checks,"deferred":[dict(slot=s,guid=g,amount=a)for(s,g),a in credits.items()],"replayed":[dict(slot=s,guid=g,amount=a)for(s,g),a in replays.items()],"post_revival_samples":len(final)}
