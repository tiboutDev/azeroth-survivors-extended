"""Verify prepared ASX assets locally or after manual GitHub publication; never uploads."""
import argparse,base64,hashlib,json,urllib.request,urllib.parse
from pathlib import Path
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

def digest(data):return hashlib.sha256(data).hexdigest()
def download(url,size):
    with urllib.request.urlopen(url,timeout=30)as response:
        if urllib.parse.urlsplit(response.url).scheme!="https":raise ValueError("Non-HTTPS release redirect")
        raw=response.read(size+1)
    if len(raw)!=size:raise ValueError("Published asset size differs")
    return raw

def verify(root,local=False):
    meta=json.loads((root/"PUBLICATION.json").read_text());data={}
    base="https://github.com/"+meta["repository"]+"/releases/"
    for name,spec in meta["assets"].items():
        if Path(name).name!=name or "/"in name or "\\"in name or not 0<spec["size"]<=64<<20:raise ValueError("Invalid local publication inventory")
        raw=(root/"UPLOAD"/name).read_bytes()if local else download(base+"download/"+meta["tag"]+"/"+urllib.parse.quote(name),spec["size"])
        if digest(raw)!=spec["sha256"]:raise ValueError("Release checksum differs: "+name)
        data[name]=raw;print("OK "+name,flush=True)
    pub=Ed25519PublicKey.from_public_bytes(base64.b64decode(meta["public_key"],validate=True))
    pub.verify(data["asx-latest.json.sig"],data["asx-latest.json"])
    legacy={"latest.json","latest.json.sig"}
    if legacy & data.keys() and not legacy <= data.keys():raise ValueError("Incomplete legacy signed feed")
    if legacy <= data.keys():pub.verify(data["latest.json.sig"],data["latest.json"])
    feed=json.loads(data["asx-latest.json"])
    if feed["product"]!="azeroth-survivors-extended"or feed["sequence"]!=meta["sequence"]or feed["manifest"]["version"]!=meta["version"]:raise ValueError("Signed ASX release identity differs")
    asset=feed["package"]["asset"]
    if digest(data[asset])!=feed["package"]["sha256"]or data["ASX.zip"]!=data[asset]:raise ValueError("Signed archive / download alias differs")
    if not local:
        for name in ("ASX.zip","asx-latest.json","asx-latest.json.sig","latest.json","latest.json.sig"):
            if name not in data:continue
            spec=meta["assets"][name];raw=download(base+"latest/download/"+urllib.parse.quote(name),spec["size"])
            if digest(raw)!=spec["sha256"]:raise ValueError("Latest-release link differs: "+name)
            print("OK latest/download/"+name,flush=True)
    result={"passed":True,"published_checked":not local,"tag":meta["tag"],"signed_feeds":1+int(legacy <= data.keys()),"assets":len(data),"game_launched":False}
    (root/("LOCAL-VERIFICATION.json"if local else"PUBLIC-VERIFICATION.json")).write_text(json.dumps(result,indent=2)+"\n")
    print("Verified release signatures and archive alias",flush=True)
    return result
if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("--folder",type=Path,default=Path(__file__).resolve().parent);parser.add_argument("--local",action="store_true")
    args=parser.parse_args();verify(args.folder.resolve(),args.local)
