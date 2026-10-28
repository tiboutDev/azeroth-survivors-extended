"""Evidence for native retained-bonus tooltips; no Windows cursor manipulation."""
import re

def evidence(host,logs,native):
 accepted=re.findall(r"glyph accepted: slot=(\d+)(?: guid=\d+)? spell=(\d+) rank=(\d+) glyph=([^\r\n]+)",host)
 kept=re.findall(r"glyph retained native acceptance: slot=(\d+) before=(\d+) after=(\d+)",host)
 return dict(checks=dict(no_low_rank_glyphs=bool(accepted)and all(int(rank)>=5 for slot,spell,rank,glyph in accepted),both_players_apply=all(any(int(s)==slot and name!="Unglyphed"for s,sp,r,name in accepted)for slot in range(2)),both_players_keep=all(any(int(s)==slot for s,b,a in kept)for slot in range(2)),both_players_remove=all(any(int(s)==slot and name=="Unglyphed"for s,sp,r,name in accepted)for slot in range(2)),compact_rows_rendered=all("kept bonus row:"in text for text in logs),native_tooltip_attached=all("native kept tooltip attached: category=0"in text for text in logs),native_tooltip_body=all("native kept tooltip body: category=0"in text for text in logs),controlled_native_hover=all("category=0 entered=true"in text and "category=0 entered=false"in text for text in logs),no_tooltip_errors=all("Native kept tooltip rejected:"not in text and "Native kept tooltip body rejected:"not in text for text in logs)),accepted=accepted,kept=kept)
