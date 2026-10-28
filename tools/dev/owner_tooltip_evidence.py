"""Compare rendered remote tooltips against actual native owner captures."""
import re
from collections import defaultdict

def assess(folder,native_menu=False):
    folder=__import__('pathlib').Path(folder)
    paths=[folder/role/('ingame-game/stderr.log' if native_menu else 'stderr.log') for role in (('host','guest') if native_menu else ('player-1','player-2'))]
    sources=defaultdict(set);items=defaultdict(set);remote=[];errors=[];samples=defaultdict(int)
    for path in paths:
        for line in path.read_text(encoding='utf-8',errors='replace').splitlines():
            if 'runtime error:' in line and not any(k in line for k in ('AZCOOP_','OnInputLanguageChanged')):errors.append(line)
            if 'AZCOOP_OWNER_TOOLTIP own=' not in line:continue
            own=int(re.search(r'own=(\d+)',line)[1]);samples[own]+=1
            match=re.search(r'source=([0-9a-f]*)',line)
            if match and match[1]:
                wire=bytes.fromhex(match[1]).decode();batch=defaultdict(list)
                for record in wire.splitlines()[1:]:
                    a=record.split('|');key=(own,'spells',int(a[0]),int(a[1]),a[2]);batch[key].append((a[3],a[4],a[5],a[6]))
                for key,rows in batch.items():sources[key].add(tuple(rows))
            for rank,name,body in re.findall(r'item_source=(\d+)/([0-9a-f]+)/([0-9a-f]+)',line):items[(own,int(rank),name)].add(body)
            for data in re.findall(r'remote=([^ ]+)',line):
                slot,kind,spell,rank,name,body=data.split('/',5)
                # Each rendered row uses text:text:rgb:rgb, with '/' excluded by hex.
                pattern=r'([0-9a-f]*):([0-9a-f]*):(\d+,\d+,\d+):(\d+,\d+,\d+)'
                rows=tuple(re.findall(pattern,body))
                remote.append(((int(slot),kind,int(spell),int(rank),name),rows))
    matches=defaultdict(int);mismatches=[];details=[]
    for key,rows in remote:
        if key[1]=='spells':
            normalize=lambda lines:tuple((l,r,c,rc if r else '') for l,r,c,rc in lines)
            ok=normalize(rows) in {normalize(candidate) for candidate in sources[key]}
        else:
            expected=items[(key[0],key[3],key[4])]
            ok=bool(rows) and rows[0][0]==key[4] and any(r[0] in expected for r in rows[1:])
        if ok:matches[(key[0],key[1])]+=1
        else:mismatches.append({'key':key,'rendered':rows})
    for key,variants in sources.items():
        bodies=[' | '.join(bytes.fromhex(row[0]).decode() for row in rows) for rows in variants]
        details.append({'owner':key[0],'id':key[2],'rank':key[3],'name':bytes.fromhex(key[4]).decode(),'native_variants':len(variants),'bonus_lines':any('bonus' in body.lower() or 'upgrade' in body.lower() for body in bodies),'examples':bodies[:2]})
    counts={str(slot):{kind:matches[(slot,kind)] for kind in ('spells','items')} for slot in (1,2)}
    passed=not errors and not mismatches and all(row['spells']>2 for row in counts.values())
    return {'passed':passed,'samples':dict(samples),'matches':counts,'mismatch_count':len(mismatches),'mismatches':mismatches[:4],'ui_errors':errors,'native_spell_variants':details,'item_sources':len(items),'remote_samples':len(remote)}
