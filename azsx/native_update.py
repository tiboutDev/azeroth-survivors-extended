"""Bounded official update announcement; native updater owns the game download."""
import json,re,urllib.request
import updates
URL="https://wow.aethus.pl/exe/latest.json"
LIMIT=16384
SHA=re.compile(r"[0-9a-f]{64}\Z")
VERSION=re.compile(r"[0-9]+(?:\.[0-9]+){2}\Z")
def announcement(download=None):
    if download is None:
        def download():
            request=urllib.request.Request(URL,headers={"User-Agent":"AzerothSurvivors/0.138.0","Accept":"application/json"})
            with urllib.request.urlopen(request,timeout=8) as response:return response.read(LIMIT+1)
    raw=download()
    if len(raw)>LIMIT:raise ValueError("Official update announcement is too large")
    value=json.loads(raw)
    if not isinstance(value,dict) or not isinstance(value.get("version"),str) or not VERSION.fullmatch(value["version"]) or not isinstance(value.get("sha256"),str) or not SHA.fullmatch(value["sha256"]):
        raise ValueError("Invalid official update announcement")
    return value["version"],value["sha256"]
def compatible(root,current,download=None,offer=None):
    version,sha=announcement(download)
    if sha==current:return None
    if offer is None:offer=updates.offer
    result=offer(root,sha)
    if result is None or result[3]["manifest"]["game"]["version"]!=version:
        return None
    return sha
