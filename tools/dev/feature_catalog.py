"""Canonical ASX feature catalog and deterministic engineering inventory."""
import argparse,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
try:
 from .client_registry import VERSION
except ImportError:
 from client_registry import VERSION
CATALOG=ROOT/"compat/features.json"
def load_features(path=CATALOG):
 data=json.loads(Path(path).read_text());features=data.get("features",[])
 if data.get("schema")!=1 or not features:raise ValueError("Invalid feature catalog")
 seen=set()
 for f in features:
  if not re.fullmatch(r"ASX-[0-9]{2}",f.get("id",""))or f["id"]in seen:raise ValueError("Duplicate/invalid feature ID")
  seen.add(f["id"])
  if f.get("disposition")not in("active","disabled","upstream"):raise ValueError("Missing feature disposition")
  if not f.get("contract")or not f.get("mode")or not f.get("reference")or not f.get("native_domains"):raise ValueError("Incomplete feature contract")
 return features

def render(features=None):
 features=load_features()if features is None else features
 lines=["| ID | Current contract / disposition | Mode | Reference |","| --- | --- | --- | --- |"]
 lines.extend("| "+" | ".join(f[k]for k in("id","contract","mode","reference"))+" |"for f in features)
 return(ROOT/"compat/feature-doc-template.md").read_text().replace("{{FEATURE_TABLE}}","\n".join(lines)).replace("{{CLIENT_VERSION}}",VERSION)

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument("--write-doc",action="store_true");a=p.parse_args()
 content=render();dest=ROOT/"docs/ASX-FEATURES.md"
 if a.write_doc:dest.write_text(content)
 elif dest.read_text()!=content:raise SystemExit("Feature documentation is stale; run feature_catalog.py --write-doc")
 print("Feature catalog and documentation verified")
if __name__=="__main__":main()
