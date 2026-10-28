"""ASX-only Ed25519 feed; old coop release feeds are a different product."""
import base64,io,os,re,urllib.error,urllib.parse,urllib.request,zipfile
import core
MAX_ZIP=256<<20
MAX_FEED=4<<20
def https(url):
    p=urllib.parse.urlsplit(url)
    if p.scheme!="https"or not p.hostname or p.username or p.password or p.fragment:raise ValueError("ASX updates require HTTPS without credentials")
    return url
def fetch(url,limit):
    class Redirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self,req,fp,code,msg,headers,newurl):
            https(newurl);return super().redirect_request(req,fp,code,msg,headers,newurl)
    with urllib.request.build_opener(Redirect()).open(https(url),timeout=8)as response:
        https(response.url);b=response.read(limit+1)
    if len(b)>limit:raise ValueError("ASX download exceeds its size limit")
    return b
def verify(raw,signature,keys,game_sha,minimum=0):
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    from cryptography.exceptions import InvalidSignature
    if len(raw)>MAX_FEED or len(signature)!=64:raise ValueError("Invalid ASX signature/feed size")
    accepted=False
    for encoded in keys:
        key=base64.b64decode(encoded,validate=True)
        if len(key)!=32:raise ValueError("Invalid ASX public key")
        try:Ed25519PublicKey.from_public_bytes(key).verify(signature,raw);accepted=True;break
        except InvalidSignature:pass
    if not accepted:raise ValueError("ASX update signature rejected")
    f=core.read_json(raw)
    if f.get("schema")!=1 or f.get("product")!=core.PRODUCT or type(f.get("sequence"))is not int or not 1<=f["sequence"]<=2**53:raise ValueError("Wrong ASX product or release sequence")
    m=core.validate(f.get("manifest",{}))
    if m["game"]["sha256"]!=game_sha:raise ValueError("This ASX release does not support the installed native game build")
    if f["sequence"]<minimum:raise ValueError("Older ASX update refused")
    spec=f.get("package",{})
    if spec.get("asset")!="asx-"+m["version"]+".zip"or type(spec.get("size"))is not int or not 1<=spec["size"]<=MAX_ZIP or not core.SHA.fullmatch(spec.get("sha256","")):raise ValueError("Invalid signed ASX archive descriptor")
    return f
def unpack(data,feed):
    if len(data)!=feed["package"]["size"]or core.digest(data)!=feed["package"]["sha256"]:raise ValueError("ASX archive checksum rejected")
    m=feed["manifest"];files={};seen=set();total=0
    with zipfile.ZipFile(io.BytesIO(data))as z:
        if len(z.infolist())!=len(m["files"])+1:raise ValueError("ASX archive inventory differs")
        for item in z.infolist():
            name=core.payload_name(item.filename)
            if item.is_dir()or name.lower()in seen or (item.external_attr>>16)&0o170000==0o120000 or item.flag_bits&1:raise ValueError("Duplicate/link/encrypted ASX archive entry")
            seen.add(name.lower());total+=item.file_size
            if total>MAX_ZIP or item.file_size>64<<20:raise ValueError("Oversized ASX archive entry")
            b=z.read(item)
            if name==core.MANIFEST:
                if core.read_json(b)!=m:raise ValueError("ASX archive manifest differs from its signature")
            else:
                spec=m["files"].get(name)
                if not spec or len(b)!=spec["size"]or core.digest(b)!=spec["sha256"]:raise ValueError("ASX archive file checksum rejected: "+name)
                files[name]=b
    if set(files)!=set(m["files"]):raise ValueError("Missing ASX archive files")
    return files
def configuration(root):
    name="ASX/Settings/channel.json";value=core.get(root,name)
    if value is None:
        value=core.get(root,"ASX/Defaults/channel.json");core.put(root,name,value)
    if not isinstance(value,dict)or value.get("schema")!=1 or not isinstance(value.get("keys"),list)or not 1<=len(value["keys"])<=8:raise ValueError("Invalid ASX update channel configuration")
    https(value["base"])
    if not value["base"].endswith("/"):raise ValueError("ASX update base must end with /")
    return value
def automatic(root,game_sha,download=fetch):
    cfg=configuration(root)
    if not cfg.get("enabled",True):return "disabled"
    state=core.get(root,"ASX/State/updates.json",{"sequence":0})
    try:raw=download(cfg["base"]+"asx-latest.json",MAX_FEED)
    except urllib.error.HTTPError as e:
        if e.code==404:return "no ASX release published"
        raise
    signature=download(cfg["base"]+"asx-latest.json.sig",64)
    feed=verify(raw,signature,cfg["keys"],game_sha,state["sequence"])
    if feed["sequence"]==state["sequence"]:return "current"if core.installed(root).get("version")==feed["manifest"]["version"]else "held after rollback or failed update"
    b=download(cfg["base"]+urllib.parse.quote(feed["package"]["asset"]),MAX_ZIP)
    files=unpack(b,feed)
    # High-water mark is outside version rollback; uncertain/failed updates do not loop.
    core.put(root,"ASX/State/updates.json",{"sequence":feed["sequence"]})
    return core.apply(root,feed["manifest"],files)
