"""Build receipts bind a compiler invocation to exact inputs and DLL bytes. No deployment."""
import argparse,hashlib,json,os,re,subprocess
from pathlib import Path
try:
 from .client_registry import ROOT,VERSION,SHA256
except ImportError:
 from client_registry import ROOT,VERSION,SHA256

def digest(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def inputs(root=ROOT,legacy=False):
 root=Path(root);files={}
 for directory in (("src","crates","native-ui","resources/windows")if legacy else("src","crates","resources/windows")):
  for p in sorted((root/directory).rglob("*")):
   if p.is_file()and"target"not in p.parts:files[p.relative_to(root).as_posix()]=digest(p)
 for name in ("Cargo.toml","Cargo.lock","rust-toolchain.toml","build.rs"):
  p=root/name
  if p.is_file():files[name]=digest(p)
 if not legacy:
  pending=[root/name for name in files if name.endswith('.rs')];seen=set()
  while pending:
   source=pending.pop()
   if source in seen:continue
   seen.add(source);text=source.read_text()
   calls=list(re.finditer(r'include_(?:str|bytes)\s*!\s*\(',text))
   literal=list(re.finditer(r'include_(?:str|bytes)\s*!\s*\(\s*"([^"\\]+)"\s*\)',text))
   if len(calls)!=len(literal):raise ValueError('Unresolved embedded compilation input requires review: '+str(source.relative_to(root)))
   for match in literal:
    target=(source.parent/match[1]).resolve()
    if not target.is_relative_to(root.resolve())or not target.is_file():raise ValueError('Embedded compilation input escapes checkout or is absent')
    files[target.relative_to(root.resolve()).as_posix()]=digest(target)
    if target.suffix=='.rs':pending.append(target)
 return files

def ui_inputs(root=ROOT):
 root=Path(root)
 return {p.relative_to(root).as_posix():digest(p)for p in sorted((root/'native-ui').rglob('*'))if p.is_file()}

def ui_fingerprint(root=ROOT):return fingerprint(ui_inputs(root))

def fingerprint(files):return hashlib.sha256(json.dumps(files,sort_keys=True,separators=(",",":")).encode()).hexdigest()
def verify(record,dll,root=ROOT,kind=None):
 if record.get("schema")not in(1,2) or record.get("version")!=VERSION or record.get("native_sha256")!=SHA256:raise ValueError("Artifact client identity mismatch")
 if kind is not None and record.get("kind")!=kind:raise ValueError("Wrong artifact kind")
 if record.get("dll_sha256")!=digest(dll):raise ValueError("Artifact DLL bytes changed")
 current=inputs(root,legacy=record.get("schema")==1)
 if record.get("schema")==2 and record.get("compilation_fingerprint")!=record.get("source_fingerprint"):raise ValueError("Compilation identity mismatch")
 if record.get("inputs")!=current or record.get("source_fingerprint")!=fingerprint(current):raise ValueError("Artifact inputs differ from staged source")
 raw=Path(dll).read_bytes();diagnostic=b"AZCOOP_TEST_EXTENSIONS"in raw
 if diagnostic!=(record.get("kind")=="diagnostic"):raise ValueError("Artifact normal/diagnostic marker mismatch")
 return record

def build(kind,dll,command):
 if command[:3]!=["cargo","xwin","build"]:raise ValueError("Only the explicit project cargo-xwin build is accepted")
 for flag,value in (("--target","x86_64-pc-windows-msvc"),("-p","azeroth-coop-native")):
  if command.count(flag)!=1 or command[command.index(flag)+1]!=value:raise ValueError("Unexpected compiler target/package")
 if "--release"not in command or"--locked"not in command:raise ValueError("Release locked build required")
 target=Path(command[command.index("--target-dir")+1])if"--target-dir"in command else Path(os.environ.get("CARGO_TARGET_DIR","target"))
 target=target if target.is_absolute()else ROOT/target
 expected=target/"x86_64-pc-windows-msvc/release/avrt.dll"
 if Path(dll).resolve()!=expected.resolve():raise ValueError("Selected DLL does not match compiler output directory")
 before=inputs();source=fingerprint(before)
 subprocess.run(command,cwd=ROOT,check=True)
 if inputs()!=before:raise ValueError("Source changed during build; no receipt issued")
 dll=Path(dll)
 if not dll.is_file():raise ValueError("Compiler did not produce the selected DLL")
 record=dict(schema=2,kind=kind,version=VERSION,native_sha256=SHA256,dll_sha256=digest(dll),source_fingerprint=source,compilation_fingerprint=source,inputs=before,command=command,rustc=subprocess.check_output(["rustc","--version"],text=True).strip(),flag_fingerprint=fingerprint({k:os.environ.get(k,"")for k in("RUSTFLAGS","CARGO_ENCODED_RUSTFLAGS","CARGO_PROFILE_RELEASE_LTO")}))
 verify(record,dll,kind=kind)
 dest=dll.with_suffix('.artifact.json');temp=dest.with_suffix('.tmp');temp.write_text(json.dumps(record,indent=2)+"\n");temp.replace(dest)
 print("Artifact receipt: "+str(dest),flush=True)
 return record

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--kind',choices=('normal','diagnostic'),required=True);p.add_argument('--dll',type=Path,required=True);p.add_argument('command',nargs=argparse.REMAINDER);a=p.parse_args();command=a.command[1:]if a.command[:1]==['--']else a.command
 build(a.kind,a.dll,command)
if __name__=='__main__':main()
