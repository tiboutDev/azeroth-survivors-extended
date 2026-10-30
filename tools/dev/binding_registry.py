"""Generate exact native literals from reviewed manifests; never discovers or approves a build."""
import argparse,ast,json,re
from pathlib import Path
try:
 from .client_registry import ROOT,selected_client
except ImportError:
 from client_registry import ROOT,selected_client

def owned_path(value):
 if not isinstance(value,str)or"\\"in value or Path(value).is_absolute():raise ValueError("Invalid registry source path")
 path=(ROOT/value).resolve()
 if not path.is_relative_to(ROOT):raise ValueError("Registry path escapes source checkout")
 return path

def parse_literal(value):
 value=re.sub(r'"(?:\\.|[^"\\])*"|&\s*\[',lambda m:m[0]if m[0].startswith('"')else'[',value)
 if re.fullmatch(r"Some\(.*\)",value,re.S):value=value[5:-1]
 try:return ast.literal_eval(value)
 except (ValueError,SyntaxError)as e:raise ValueError("Binding values must be reviewed literals")from e

def load_binding():
 version,client=selected_client();path=owned_path(client["binding_manifest"]);data=json.loads(path.read_text())
 if data.get("schema")!=1 or data.get("version")!=version:raise ValueError("Binding/client identity mismatch")
 seen=set()
 for c in data["constants"]:
  name=c["name"]
  if not re.fullmatch(r"[A-Z][A-Z0-9_]*",name)or name in seen or name in("SHA256","VERSION"):raise ValueError("Invalid/duplicate binding constant")
  seen.add(name)
  if c["visibility"]not in("pub","pub(crate)",""):raise ValueError("Invalid constant visibility")
  if not re.fullmatch(r"[a-zA-Z0-9_\s&;\[\](),<>]+",c["type"]):raise ValueError("Invalid constant type")
  parse_literal(c["value"])
 return version,client,data

def render():
 version,client,data=load_binding();template=(ROOT/"compat/binding-template.rs").read_text()
 constants={c["name"]:((c["visibility"]+" ")if c["visibility"]else"")+"const "+c["name"]+": "+c["type"]+" = "+c["value"]+";"for c in data["constants"]}
 constants.update(SHA256='pub const SHA256: &str = '+json.dumps(client['sha256'])+';',VERSION='pub const VERSION: &str = '+json.dumps(version)+';',CLIENT_VERSION=version)
 placeholders=set(re.findall(r"\{\{([A-Z0-9_]+)\}\}",template))
 if placeholders!=set(constants):raise ValueError("Binding template/manifest contracts differ: "+str(sorted(placeholders^set(constants))))
 def replace(m):
  if m[1]not in constants:raise ValueError("Missing binding template contract: "+m[1])
  return constants[m[1]]
 return re.sub(r"\{\{([A-Z0-9_]+)\}\}",replace,template)

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument("--write",action="store_true");a=p.parse_args();_,client,_=load_binding();dest=owned_path(client["descriptor"]);source=render()
 if a.write:dest.write_text(source)
 elif dest.read_text()!=source:raise SystemExit("Generated native binding differs; review manifest and run binding_registry.py --write")
 print("Exact binding manifest verified")
if __name__=="__main__":main()
