"""Use only a freshly extracted private kit to test its signed release assets."""
from pathlib import Path
import base64,json,sys
root=Path(sys.argv[1]).resolve(strict=True);assets=Path(sys.argv[2]).resolve(strict=True)
assert root.name=="AzerothSurvivorsCoop" and root.parent.name.startswith("release1125-") and "test-results" in root.parts
sys.path.insert(0,str(root/"Coop/Updates/client"))
import updater,channel
manifest,_=updater.inventory(root)
raw=(assets/"latest.json").read_bytes();signature=(assets/"latest.json.sig").read_bytes()
config=updater.configuration(root)[0];public=base64.b64decode(config["public_key"])
feed=channel.verify_feed(raw,signature,public,"stable",manifest["game_sha256"],0)
package=(assets/feed["package"]["asset"]).read_bytes()
original=(root/"Coop/COOP.json").read_bytes()
# Private previous revision: change one managed file, add one retired file,
# and leave one new-release component absent. No player save is read.
readme=root/"README.md";readme.write_text("Disposable previous release fixture\n")
retired=root/"Coop/reconstruction/retired-release-test.txt";retired.write_text("retire me")
added="Coop/INSTALLATION-V3.md";(root/added).unlink();manifest["components"].pop(added)
manifest["version"]="0.112.5-test-baseline"
(root/"VERSION.txt").write_bytes(channel.version_text(manifest).encode("utf-8"))
for name in ("README.md","VERSION.txt","Coop/reconstruction/retired-release-test.txt"):manifest["components"][name]=updater.sha(root/name)
updater.put_json(root/"Coop/COOP.json",manifest)
keep=root/"UserData/Saves/update-preservation-fixture.txt";keep.parent.mkdir(parents=True,exist_ok=True);keep.write_text("keep progress")
unmanaged=root/"Interface/AddOns/PersonalFixture/keep.txt";unmanaged.parent.mkdir(parents=True,exist_ok=True);unmanaged.write_text("keep addon")
with updater.locked(root):
    bad=bytes([signature[0]^1])+signature[1:]
    try:updater.install(root,raw,bad,package,public,"stable",running=lambda _:False)
    except ValueError:pass
    else:raise AssertionError("Tampered signature accepted")
    assert retired.exists() and not (root/added).exists()
    def download(url,limit):
        value=(assets/url.rsplit("/",1)[-1]).read_bytes();assert len(value)<=limit;return value
    assert updater.automatic(root,download=download,running=lambda _:False)=="installed"
    assert (root/"Coop/COOP.json").read_bytes()==original
    assert not retired.exists() and (root/added).is_file()
    assert updater.automatic(root,download=download,running=lambda _:False)=="current"
    assert updater.rollback(root,running=lambda _:False)=="restored"
    assert retired.exists() and not (root/added).exists() and readme.read_text()=="Disposable previous release fixture\n"
    assert updater.automatic(root,download=download,running=lambda _:False)=="held"
    # Restore the exact release before the menu launch check.
    assert updater.install(root,raw,signature,package,public,"stable",force=True,running=lambda _:False)=="installed"
    assert (root/"Coop/COOP.json").read_bytes()==original
assert keep.read_text()=="keep progress" and unmanaged.read_text()=="keep addon"
keep.unlink();unmanaged.unlink()
print("Signed automatic update, additions/replacements/deletions, rollback, held release and personal-file preservation passed.")
