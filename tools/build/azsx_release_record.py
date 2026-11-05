"""Private per-release identity, derived from the actual authenticated archive."""
import hashlib
import io
import json
import re
import zipfile
from pathlib import Path
from azsx_release_identity import REVISION
from azsx_reproducibility import verify_artifact

NAME="RELEASE-IDENTITY.json"

def digest(raw):return hashlib.sha256(raw).hexdigest()

def identity(folder,evidence,meta):
    # Historical preparations remain readable without rewriting their records.
    if not REVISION.fullmatch(meta["version"]):return None
    def read(name):
        path=folder/name if (folder/name).exists()else evidence/name
        return path.read_bytes()
    source=json.loads(read("SOURCE-COMMIT.json"))
    commit=source.get("release_source_commit",source.get("commit",""))
    payload_commit=source.get("payload_source_commit",commit)
    if not all(isinstance(c,str)and re.fullmatch(r"[0-9a-f]{40}",c)for c in(commit,payload_commit)):
        raise ValueError("Release identity needs exact source commits")
    validation_raw=read("VALIDATION.json");validation=json.loads(validation_raw)
    report_raw=read("REPRODUCIBILITY.json");report=json.loads(report_raw)
    feed=json.loads((folder/"UPLOAD/azsx-latest.json").read_bytes())
    manifest=feed["manifest"];asset=feed["package"]["asset"]
    archive=(folder/"UPLOAD"/asset).read_bytes()
    with zipfile.ZipFile(io.BytesIO(archive))as z:
        dll_name="AZSX/App/Bin/avrt.dll"
        if z.namelist().count(dll_name)!=1:raise ValueError("Expected one distributed DLL")
        raw=z.read(dll_name)
    actual={"sha256":digest(raw),"size":len(raw)}
    if manifest["files"].get(dll_name)!=actual:raise ValueError("DLL differs from signed manifest")
    verify_artifact(report,raw,manifest["game"]["version"])
    return {"schema_version":1,"product":feed["product"],"version":meta["version"],
            "source":source,"game":manifest["game"],"artifact":report["artifact"],
            "package":{"asset":asset,"sha256":digest(archive)},
            "reproducibility":{"report":"REPRODUCIBILITY.json","sha256":digest(report_raw),
                               "source":report.get("source"),"verification":report["verification"],
                               "source_fingerprint":report["source_fingerprint"]["sha256"],
                               "build_recipe_sha256":report["build_recipe_sha256"]},
            "validation":{"report":"VALIDATION.json","sha256":digest(validation_raw),"results":validation}}

def check_existing(evidence,record):
    path=evidence/NAME
    if record is not None and path.exists():
        if path.is_symlink()or json.loads(path.read_text())!=record:
            raise ValueError("Existing release identity differs; earlier records are preserved")
