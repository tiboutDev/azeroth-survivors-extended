"""Read-only offline address/ABI candidates between official builds.
No runtime activation, source rewrite, historical-port imports or automatic approval.
Instruction equality keeps stack/member offsets and constants; relocations alone
are normalized. Changed owners/branches are evidence requiring explicit review.
"""
import argparse,bisect,collections,difflib,functools,hashlib,json,re,struct
from pathlib import Path
import capstone as cs,pefile
ROOT=Path(__file__).resolve().parents[2]
MD=cs.Cs(cs.CS_ARCH_X86,cs.CS_MODE_64);MD.detail=True
class Image:
 def __init__(self,path):
  self.path=Path(path);self.raw=self.path.read_bytes();self.sha=hashlib.sha256(self.raw).hexdigest();self.pe=pefile.PE(data=self.raw,fast_load=True)
  d=self.pe.OPTIONAL_HEADER.DATA_DIRECTORY[3];self.ends=dict((a,b)for a,b,_ in struct.iter_unpack("<III",self.pe.get_data(d.VirtualAddress,d.Size)));self.starts=sorted(self.ends)
  s=next(s for s in self.pe.sections if s.Characteristics&0x20);self.text=s.VirtualAddress;self.code=s.get_data();self.sizes=collections.defaultdict(list)
  for a,b in self.ends.items():self.sizes[b-a].append(a)
 def owner(self,rva):
  i=bisect.bisect_right(self.starts,rva)-1
  return self.starts[i]if i>=0 and rva<self.ends[self.starts[i]]else None
 @functools.lru_cache(None)
 def ins(self,start):return list(MD.disasm(self.pe.get_data(start,self.ends[start]-start),start))
 @functools.lru_cache(None)
 def tokens(self,start):
  out=[]
  for i in self.ins(start):
   b=bytearray(i.bytes)
   if any(o.type==cs.x86.X86_OP_MEM and o.mem.base==cs.x86.X86_REG_RIP for o in i.operands):b[i.disp_offset:i.disp_offset+4]=bytes(4)
   if (i.group(cs.CS_GRP_CALL)or i.group(cs.CS_GRP_JUMP))and i.operands and i.operands[0].type==cs.x86.X86_OP_IMM:
    b[i.imm_offset:i.imm_offset+i.imm_size]=bytes(i.imm_size)
   out.append(bytes(b))
  return out
 def signature(self,start):return b"".join(self.tokens(start))
def anchors(image,start):
 runs=[];current=b""
 for ins,token in zip(image.ins(start),image.tokens(start)):
  if bytes(ins.bytes)==token and not(ins.group(cs.CS_GRP_CALL)or ins.group(cs.CS_GRP_JUMP)):current+=token
  else:
   if len(current)>=8:runs.append(current)
   current=b""
 if len(current)>=8:runs.append(current)
 return sorted(set(runs),key=len,reverse=True)[:48]
def shape(i):
 def reg(r):return MD.reg_name(r)if r in(0,cs.x86.X86_REG_RIP,cs.x86.X86_REG_RSP,cs.x86.X86_REG_RBP)else"xmm"if MD.reg_name(r).startswith("xmm")else"gp"
 return(i.mnemonic,tuple(("r",reg(o.reg),o.size)if o.type==1 else("m",reg(o.mem.base),bool(o.mem.index),o.size)if o.type==3 else("i",o.size)for o in i.operands))
def grams(image,start):
 tokens=[shape(i)for i in image.ins(start)]
 return collections.Counter(tuple(tokens[i:i+4])for i in range(len(tokens)-3))
def coarse_candidate(old,new,start):
 size=old.ends[start]-start;wanted=grams(old,start);scores=[]
 for a,b in new.ends.items():
  if abs(a-start)>0x600000 or not .5<(b-a)/size<2:continue
  other=grams(new,a);score=2*sum((wanted&other).values())/(sum(wanted.values())+sum(other.values()))if wanted and other else 0
  scores.append((score,a))
 scores.sort(reverse=True)
 if scores and scores[0][0]>=.72 and(len(scores)<2 or scores[0][0]-scores[1][0]>=.08):return scores[0][1],scores[0][0],"coarse-candidate-ABI-review-required"
 raise ValueError("ambiguous coarse "+repr([(round(s,4),hex(a))for s,a in scores[:3]]))
def candidates(old,new,start):
 size=old.ends[start]-start;wanted=old.signature(start)
 exact=[a for a in new.sizes[size]if new.pe.get_data(a,3)==old.pe.get_data(start,3)and new.signature(a)==wanted]
 if len(exact)==1:return exact[0],1.0,"relocations-only"
 votes=collections.Counter()
 for anchor in anchors(old,start):
  pos=new.code.find(anchor);hits=[]
  while pos>=0 and len(hits)<65:
   at=new.owner(new.text+pos)
   if at is not None and .4<(new.ends[at]-at)/size<2.5:hits.append(at)
   pos=new.code.find(anchor,pos+1)
  if len(hits)<=64:votes.update(set(hits))
 scores=[]
 for at,count in votes.most_common(8):
  score=difflib.SequenceMatcher(None,old.tokens(start),new.tokens(at),autojunk=False).ratio();scores.append((score,at,count))
 scores.sort(reverse=True)
 if not scores or scores[0][0]<.65:return coarse_candidate(old,new,start)
 if len(scores)>1 and scores[0][0]-scores[1][0]<.08:raise ValueError("ambiguous "+repr([(round(s,4),hex(a),n)for s,a,n in scores[:3]]))
 return scores[0][1],scores[0][0],"changed-owner-candidate"
def inventory(old,paths):
 out=collections.defaultdict(list)
 for path in paths:
  for n,line in enumerate(path.read_text().splitlines(),1):
   for m in re.finditer(r"0x[0-9a-fA-F]+",line):
    v=int(m[0],16)
    if 0x1000<=v<old.pe.OPTIONAL_HEADER.SizeOfImage and v not in(0x10000,0x8000000,0x1000000,0x100000,0x10000000,0x10000000):out[v].append(f"{path.relative_to(ROOT)}:{n}")
 return out
def descriptor_inventory(path):
 """Inventory versioned contacts, named tables and native tooltip seams.
 Member offsets, stack slots and byte guards are deliberately not addresses.
 """
 path=Path(path);source=path.read_text();out={}
 def add(value,label):out.setdefault(int(value,16),[]).append(label)
 for n,line in enumerate(source.splitlines(),1):
  pair=re.search(r"\(0x([0-9a-f]+),\s*0x([0-9a-f]+)\)",line)
  if pair:
   baseline,native=pair.groups();add(native,f"{path.name}:{n} baseline={baseline}")
  named=re.search(r"pub const ([A-Z0-9_]+(?:_RVA|_TABLE))\s*:\s*u64\s*=\s*0x([0-9a-f]+)",line)
  if named:add(named[2],f"{path.name}:{n} {named[1]}")
 for name in ('KEPT_TOOLTIP_BINDINGS','KEPT_TOOLTIP_FOCUS'):
  match=re.search(r'pub const '+name+r'\s*:.*?=(.*?);',source,re.S)
  if match:
   for value in re.findall(r'\(0x([0-9a-f]+),\s*&\[',match[1]):add(value,f"{path.name} {name}")
 if not out:raise ValueError("No explicit native address pairs in descriptor")
 return out
def compare(old,new,needed,limit):
 owners=sorted({a for v in needed if(a:=old.owner(v))is not None});mapping={};refs=collections.defaultdict(set);pairs={};failures={};changes=[];branches=[]
 for start in owners[:limit]:
  try:
   dest,score,kind=candidates(old,new,start);a=old.ins(start);b=new.ins(dest)
   if kind=="relocations-only":blocks=[difflib.Match(0,0,len(a))]
   else:blocks=difflib.SequenceMatcher(None,old.tokens(start),new.tokens(dest),autojunk=False).get_matching_blocks()
   local={a[x.a+j].address:b[x.b+j].address for x in blocks for j in range(x.size)}
   for x in blocks:
    for j in range(x.size):
     oi,ni=a[x.a+j],b[x.b+j]
     for offset in range(min(oi.size,ni.size)):mapping[oi.address+offset]=ni.address+offset
     for o,n in zip(oi.operands,ni.operands):
      if o.type==n.type==cs.x86.X86_OP_MEM and o.mem.base==n.mem.base==cs.x86.X86_REG_RIP:refs[oi.address+oi.size+o.mem.disp].add(ni.address+ni.size+n.mem.disp)
      if o.type==n.type==cs.x86.X86_OP_IMM and(oi.group(cs.CS_GRP_CALL)or oi.group(cs.CS_GRP_JUMP)):
       refs[o.imm].add(n.imm)
       if oi.group(cs.CS_GRP_JUMP)and o.imm in local and local[o.imm]!=n.imm:branches.append([oi.address,ni.address,o.imm,n.imm])
   if kind!="relocations-only":
    for tag,i,j,k,l in difflib.SequenceMatcher(None,old.tokens(start),new.tokens(dest),autojunk=False).get_opcodes():
     if tag!="equal":changes.append(dict(owner=start,new_owner=dest,kind=tag,old=[f"{x.address:x} {x.mnemonic} {x.op_str}"for x in a[i:j]],new=[f"{x.address:x} {x.mnemonic} {x.op_str}"for x in b[k:l]]))
   pairs[start]=dict(new=dest,score=score,kind=kind,old_bytes=old.ends[start]-start,new_bytes=new.ends[dest]-dest)
   print(f"{start:x}->{dest:x} {score:.4f} {kind}",flush=True)
  except ValueError as e:failures[start]=str(e);print(f"PENDING {start:x}: {e}",flush=True)
  evidence=dict(old_sha256=old.sha,new_sha256=new.sha,needed={str(k):v for k,v in needed.items()},functions=pairs,mapping=mapping,references={str(k):sorted(v)for k,v in refs.items()},changes=changes,branch_conflicts=branches,failures=failures,approved=False,offline_only=True)
  for k,v in refs.items():
   if len(v)==1 and k not in mapping:mapping[k]=next(iter(v))
  evidence['unresolved']=[v for v in needed if v not in mapping]
  yield evidence
if __name__=="__main__":
 p=argparse.ArgumentParser(description=__doc__);p.add_argument("--old",type=Path,required=True);p.add_argument("--new",type=Path,required=True);p.add_argument("--out",type=Path,required=True);p.add_argument("--limit",type=int,default=200);p.add_argument("--descriptor",type=Path,help="Previous exact-build descriptor; inventory its native addresses")
 a=p.parse_args();old,new=Image(a.old),Image(a.new)
 paths=[ROOT/"crates/asx-game/src/official_native.rs",*(ROOT/"crates/asx-game/src/official_native").rglob("*.rs")]
 needed=descriptor_inventory(a.descriptor)if a.descriptor else inventory(old,paths)
 a.out.parent.mkdir(parents=True,exist_ok=True)
 for result in compare(old,new,needed,a.limit):a.out.write_text(json.dumps(result,indent=2)+"\n")
 print("Unresolved",[hex(x)for x in result['unresolved']],flush=True)
