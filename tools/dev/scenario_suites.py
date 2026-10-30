"""Declarative bounded readiness. Completion never substitutes for acceptance checks."""
import json,re,time
from pathlib import Path
try:
 from .client_registry import ROOT
except ImportError:
 from client_registry import ROOT

def load_suites(root=ROOT):
 value=json.loads((Path(root)/"compat/suites.json").read_text())
 if value.get("schema")!=1:raise ValueError("Unsupported suite schema")
 for name,s in value["suites"].items():
  if s["phase"]not in("solo","coop")or s["kind"]not in("normal","diagnostic")or s["players"]!=(1 if s["phase"]=="solo"else 2):raise ValueError("Invalid scenario phase")
  if not 15<=s["seconds"]<=300 or not 0<=s["settle_seconds"]<=15:raise ValueError("Invalid scenario budget")
  for c in s["readiness"]:
   p=Path(c["source"])
   if p.is_absolute()or".."in p.parts or"\\"in c["source"]:raise ValueError("Readiness path escaped trial")
   if ("contains"in c)==("regex"in c)or not 1<=c["minimum"]<=1024:raise ValueError("Invalid readiness condition")
   if "regex"in c:re.compile(c["regex"])
 return value

class Readiness:
 def __init__(self,suite):self.suite=suite;self.ready_at=None;self.cache={}
 def complete(self,root,now=None):
  now=time.monotonic()if now is None else now
  if not self.suite["readiness"]:return False
  for condition in self.suite["readiness"]:
   p=Path(root)/condition["source"]
   if p.is_symlink()or not p.is_file():self.ready_at=None;return False
   stat=p.stat();key=(stat.st_mtime_ns,stat.st_size)
   old=self.cache.get(p)
   if old is None or old[0]!=key:
    with p.open('rb')as stream:
     stream.seek(max(0,stat.st_size-8*1024*1024));text=stream.read().decode('utf-8',errors='replace')
    self.cache[p]=(key,text)
   text=self.cache[p][1]
   count=text.count(condition["contains"])if"contains"in condition else len(re.findall(condition["regex"],text))
   if count<condition["minimum"]:self.ready_at=None;return False
  if self.ready_at is None:self.ready_at=now
  return now-self.ready_at>=self.suite["settle_seconds"]
