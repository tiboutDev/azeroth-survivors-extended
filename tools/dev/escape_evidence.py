"""Guest native-Escape evidence; synthetic menu state alone is insufficient."""
import json

def assess(host, guest):
    views=[]
    for line in guest.splitlines():
        if "official pause UI: " not in line:continue
        try:views.append(json.loads(line.split("official pause UI: ",1)[1]).get("view",{}))
        except (ValueError,AttributeError):continue
    checks={"guest_escape_native_request":"official menu request: open=true role=Guest" in guest,
            "host_accepts_guest_menu":"official peer menu: slot=1 open=true" in host,
            "guest_menu_ownership_projected":any(v.get("active") is True and v.get("local_menu") is True for v in views)}
    return {"checks":checks,"passed":all(checks.values()),"scope":"Actual native keyboard producer and menu ownership; rendered visibility requires a separate observation."}
