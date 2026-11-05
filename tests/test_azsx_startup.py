"""Startup transactions with signed fake packages; never launch a real game."""
import copy
import io
import os
from contextlib import redirect_stdout
from pathlib import Path
import unittest
from unittest.mock import patch
import test_azsx as fixture
import core,updates,housekeeping,native_update,update_worker
launch=fixture.launch

class StartupTests(unittest.TestCase):
    # Reuse fixture builders without inheriting and rerunning their test cases.
    for name in ("setUp","release","extract","install","signed","configure","snapshot","bundled_ui","intact"):
        locals()[name]=getattr(fixture.AZSXTests,name)

    def setUp(self):
        fixture.AZSXTests.setUp(self);self.install();self.configure()
        quiet=redirect_stdout(io.StringIO());quiet.__enter__();self.addCleanup(quiet.__exit__,None,None,None)
        env=patch.dict(os.environ,{"AZSX_LAUNCH_ID":"test-start"});env.start();self.addCleanup(env.stop)

    def network(self,m,data,sequence=2):
        raw,sig,package=self.signed(m,data,sequence);calls=[]
        def fetch(url,limit):
            calls.append(url)
            return raw if url.endswith("azsx-latest.json")else sig if url.endswith(".sig")else package
        return fetch,calls

    def future(self):
        m,data=self.release(2);m["version"]="0.126.0-r1"
        self.new_exe=b"MZ next official native game"
        m["game"]=dict(m["game"],version="0.126.0",sha256=core.digest(self.new_exe))
        data["AZSX/App/Launcher/supported-games.json"]=core.encoded({"schema":1,"games":{"0.126.0":{"sha256":core.digest(self.new_exe)}}})
        m["files"]={n:{"size":len(b),"sha256":core.digest(b)}for n,b in data.items()}
        return m,data

    def staged(self,sequence=2):
        m,data=self.future();fetch,calls=self.network(m,data,sequence)
        with patch.object(native_update,"announcement",return_value=("0.126.0",core.digest(self.new_exe))):
            self.assertEqual(native_update.synchronize(self.root,core.digest(self.exe),download=fetch),"native update ready")
        return m,data,fetch,calls

    def game(self,command,**kwargs):
        self.assertTrue(kwargs["env"].get("AZSX_ACTIVE")=="1")
        core.atomic(Path(kwargs["env"]["AZSX_READY_FILE"]),core.encoded({"product":"azsx","game_sha256":core.inspect(self.root)}))
        class Child:
            pid=1234
            finished=False
            def poll(self):return 0 if self.finished else None
            def wait(self,timeout=None):self.finished=True;return 0
        return Child()

    def test_native_transition_stages_before_updater_then_installs_offline_from_cache(self):
        m,data,fetch,calls=self.staged()
        self.assertEqual(len(calls),3)
        self.assertEqual(core.installed(self.root)["version"],self.m["version"])
        self.assertEqual((self.root/"avrt.dll").read_bytes(),self.data["AZSX/App/Bin/avrt.dll"])
        old_exe=self.exe;root=self.root;new_exe=self.new_exe
        def updater(command,**kwargs):
            self.assertNotIn("AZSX_ACTIVE",kwargs["env"])
            self.assertNotIn("AZCOOP_ENGINE",kwargs["env"])
            self.assertEqual(kwargs["env"]["WOW_ROGUE_UPDATE_AUTO"],"1")
            self.assertEqual((root/"AzerothSurvivors.exe").read_bytes(),old_exe)
            self.assertEqual((root/"avrt.dll").read_bytes(),self.data["AZSX/App/Bin/avrt.dll"])
            class Child:
                def wait(self,timeout=None):core.atomic(root/"AzerothSurvivors.exe",new_exe);return 0
                def poll(self):return 0
            return Child()
        with patch.object(native_update.subprocess,"Popen",side_effect=updater):
            self.assertEqual(native_update.run(self.root,os.environ),75)
        with patch.object(updates,"fetch",side_effect=AssertionError("network on cached resume")):
            self.assertEqual(native_update.synchronize(self.root,core.digest(new_exe),offline=True),"installed")
        self.assertEqual(core.installed(self.root)["version"],m["version"])
        self.assertEqual((root/"avrt.dll").read_bytes(),data["AZSX/App/Bin/avrt.dll"])
        self.assertFalse(core.path(root,native_update.PENDING).exists())

    def test_launch_update_only_runs_updater_then_restarts_without_launching_game(self):
        m,data,_,_=self.staged()
        with patch.object(core,"native_paths",return_value=(self.root/"progression.txt",self.root/"benilla-config")),patch.object(native_update,"announcement",return_value=("0.126.0",core.digest(self.new_exe))),patch.object(native_update,"run",return_value=75)as run,patch.object(launch.subprocess,"Popen",side_effect=AssertionError("normal game launch during update")):
            self.assertEqual(launch.run(self.root,update_only=True),75)
        run.assert_called_once()

    def test_bad_package_prevents_native_update_and_preserves_installation(self):
        m,data=self.future();fetch,calls=self.network(m,data)
        def bad(url,limit):return b"bad ZIP"if url.endswith(".zip")else fetch(url,limit)
        before=self.snapshot()
        with patch.object(native_update,"announcement",return_value=("0.126.0",core.digest(self.new_exe))),self.assertRaises(ValueError):
            native_update.synchronize(self.root,core.digest(self.exe),download=bad)
        self.assertFalse(core.path(self.root,native_update.PENDING).exists())
        self.assertEqual(before,self.snapshot())

    def test_official_version_must_match_and_no_downgrade_is_attempted(self):
        m,data=self.future();fetch,calls=self.network(m,data)
        with patch.object(native_update,"announcement",return_value=("0.127.0","a"*64)):
            self.assertEqual(native_update.synchronize(self.root,core.digest(self.exe),download=fetch),"no compatible native update")
        self.assertEqual(len(calls),2)
        self.assertFalse(core.path(self.root,native_update.PENDING).exists())

    def test_native_updater_unexpected_result_cannot_activate_old_mod(self):
        self.staged();core.atomic(self.root/"AzerothSurvivors.exe",b"unexpected native version")
        with patch.object(launch.subprocess,"Popen",side_effect=AssertionError("wrong game launched")),self.assertRaisesRegex(ValueError,"unexpected client"):
            launch.run(self.root,offline=True)
        self.assertEqual(core.installed(self.root)["version"],self.m["version"])

    def test_native_update_failure_keeps_package_for_next_launch(self):
        self.staged()
        class Child:
            def wait(self,timeout=None):return 1
            def poll(self):return 1
        with patch.object(native_update.subprocess,"Popen",return_value=Child()),self.assertRaisesRegex(ValueError,"expected client"):
            native_update.run(self.root,os.environ)
        self.assertEqual(native_update.synchronize(self.root,core.digest(self.exe)),"deferred until next launch")
        with patch.dict(os.environ,{"AZSX_LAUNCH_ID":"next-start"}),patch.object(native_update,"announcement",return_value=("0.126.0",core.digest(self.new_exe))):
            self.assertEqual(native_update.synchronize(self.root,core.digest(self.exe)),"native update ready")

    def test_offline_launch_uses_existing_installation_without_network(self):
        with patch.object(updates,"fetch",side_effect=AssertionError("offline network")),patch.object(core,"native_paths",return_value=(self.root/"progression.txt",self.root/"benilla-config")),patch.object(launch.subprocess,"Popen",side_effect=self.game):
            self.assertEqual(launch.run(self.root,offline=True),0)
        self.intact()

    def test_missing_or_modified_owned_file_is_repaired_from_signed_cache_without_question(self):
        raw,sig,package=self.signed(self.m,self.data,1)
        updates.cache(self.root,raw,sig,package,updates.verify(raw,sig,[self.public],core.digest(self.exe)))
        (self.root/"AZSX/App/Bin/avrt.dll").write_bytes(b"damaged owned file")
        with patch.object(launch,"confirm_repair",side_effect=AssertionError("repair asked")),patch.object(core,"native_paths",return_value=(self.root/"progression.txt",self.root/"benilla-config")),patch.object(launch.subprocess,"Popen",side_effect=AssertionError("game before repair")):
            self.assertEqual(launch.run(self.root,offline=True),75)
        self.assertEqual((self.root/"AZSX/App/Bin/avrt.dll").read_bytes(),self.data["AZSX/App/Bin/avrt.dll"])
        self.assertTrue(any(p.read_bytes()==b"damaged owned file"for p in(self.root/"AZSX/Data/Backups/Files").rglob("avrt.dll")))

    def test_automatic_repair_preserves_foreign_loader_and_addon(self):
        self.m,self.data=self.bundled_ui(1);self.extract(self.m,self.data)
        raw,sig,archive=self.signed(self.m,self.data,1);updates.cache(self.root,raw,sig,archive,updates.verify(raw,sig,[self.public],core.digest(self.exe)))
        target=self.root/(core.AZSXUI_TARGET+"Events.lua");core.atomic(target,b"foreign addon")
        self.assertEqual(launch.maintenance.repair(self.root,lambda _:self.fail("question"),offline=True,automatic=True),"installed")
        self.assertEqual(target.read_bytes(),b"foreign addon")
        # A lost ownership receipt must not let automatic repair adopt a foreign loader.
        (self.root/core.STATE).unlink();(self.root/"avrt.dll").write_bytes(b"foreign loader")
        with self.assertRaisesRegex(ValueError,"Foreign loader"):
            launch.maintenance.repair(self.root,lambda _:True,offline=True,automatic=True)
        self.assertEqual((self.root/"avrt.dll").read_bytes(),b"foreign loader")

    def test_recovery_is_automatic_and_restarts_before_using_restored_sources(self):
        tx="a"*32;base="AZSX/Data/Updates/Transactions/"+tx
        core.atomic(self.root/(base+"/files/avrt.dll"),self.data["AZSX/App/Bin/avrt.dll"])
        core.put(self.root,base+"/journal.json",{"phase":"applying","state":core.installed(self.root),"before":{"avrt.dll":core.digest(self.data["AZSX/App/Bin/avrt.dll"])}})
        (self.root/"avrt.dll").write_bytes(b"partial")
        with patch.object(launch,"confirm_repair",side_effect=AssertionError("question")),patch.object(launch.subprocess,"Popen",side_effect=AssertionError("game during recovery")):
            self.assertEqual(launch.run(self.root,offline=True),75)
        self.assertFalse(launch.pending_recovery(self.root))

    def test_cleanup_keeps_five_distinct_saves_and_five_sessions(self):
        for i in range(12):
            name="20261105-1300%02d-%08x"%(i,i)
            core.atomic(self.root/("AZSX/Data/Backups/Progression/"+name+".txt"),str(i//2).encode())
            core.put(self.root,"AZSX/Data/Logs/"+name+"/session.json",{"product":core.PRODUCT})
        core.atomic(self.root/"AZSX/Data/Backups/Files/personal.txt",b"unique work")
        core.atomic(self.root/"AZSX/Data/Backups/Progression/user-copy.txt",b"user backup")
        housekeeping.clean(self.root)
        saves=list((self.root/"AZSX/Data/Backups/Progression").glob("2026*.txt"))
        self.assertEqual(len(saves),5);self.assertEqual({p.read_bytes()for p in saves},{str(i).encode()for i in range(1,6)})
        self.assertEqual(len(list((self.root/"AZSX/Data/Logs").iterdir())),5)
        self.assertIsNone(launch.backup(self.root,saves[-1]))
        self.assertTrue((self.root/"AZSX/Data/Backups/Files/personal.txt").exists())
        self.assertTrue((self.root/"AZSX/Data/Backups/Progression/user-copy.txt").exists())

    def test_cleanup_preserves_current_previous_and_pending_packages_then_prunes_old_history(self):
        for i in range(2,5):
            m,data=self.release(i);self.assertEqual(self.update_fixture(m,data,i),"installed")
        self.staged(sequence=5)
        housekeeping.clean(self.root)
        versions={p.name for p in(self.root/"AZSX/Data/Updates/Packages").iterdir()}
        self.assertEqual(versions,{"0.125.0-azsx.3","0.125.0-azsx.4","0.126.0-r1"})
        transactions=list((self.root/"AZSX/Data/Updates/Transactions").iterdir())
        self.assertEqual(len(transactions),1)
        core.rollback(self.root);housekeeping.clean(self.root)
        self.assertIsNone(core.installed(self.root).get("rollback"))
        self.assertFalse(list((self.root/"AZSX/Data/Updates/Transactions").iterdir()))

    def update_fixture(self,m,data,sequence):
        fetch,_=self.network(m,data,sequence)
        return updates.automatic(self.root,core.digest(self.exe),fetch)

    def test_cleanup_removes_only_exited_worker_and_preserves_links_unknown_content(self):
        for i,pid in ((1,100),(2,200)):
            work="AZSX/Data/Updates/Workers/"+str(i)*32
            core.put(self.root,work+"/job.json",{"worker_pid":pid,"version":self.m["version"]})
            core.atomic(self.root/(work+"/Python/python.exe"),b"temporary interpreter")
        with patch.object(core,"process_alive",side_effect=lambda pid:pid==200):housekeeping.clean(self.root)
        self.assertFalse((self.root/("AZSX/Data/Updates/Workers/"+"1"*32)).exists())
        self.assertTrue((self.root/("AZSX/Data/Updates/Workers/"+"2"*32)).exists())
        outside=self.root/"outside";outside.mkdir();(outside/"personal").write_bytes(b"keep")
        linked=self.root/("AZSX/Data/Updates/Workers/"+"3"*32);linked.symlink_to(outside,target_is_directory=True)
        with self.assertRaisesRegex(ValueError,"links"):housekeeping.clean(self.root)
        self.assertEqual((outside/"personal").read_bytes(),b"keep")

    def test_rollback_hold_survives_automatic_retry(self):
        m,data=self.release(2);fetch,_=self.network(m,data)
        updates.automatic(self.root,core.digest(self.exe),fetch);core.rollback(self.root)
        self.assertEqual(updates.automatic(self.root,core.digest(self.exe),fetch,retry=True),"held after rollback")

    def test_detached_worker_reauthenticates_applies_and_supervises_restart(self):
        m,data=self.release(2);raw,sig,archive=self.signed(m,data,2)
        updates.cache(self.root,raw,sig,archive,updates.verify(raw,sig,[self.public],core.digest(self.exe)))
        job={"root":str(self.root),"version":m["version"],"sequence":2,"keys":[self.public],"arguments":["--update-only"]}
        from unittest.mock import Mock
        with patch.object(update_worker.subprocess,"Popen",side_effect=[Mock(wait=Mock(return_value=75)),Mock(wait=Mock(return_value=0))])as start:
            self.assertEqual(update_worker.finish(job),0)
        self.assertEqual(start.call_count,2)
        self.assertEqual(start.call_args.args[0][-1],"--update-only")
        self.assertEqual(core.installed(self.root)["version"],m["version"])
        self.intact()

    def test_detached_worker_refuses_tampered_cache_before_restarting(self):
        m,data=self.release(2);raw,sig,archive=self.signed(m,data,2)
        updates.cache(self.root,raw,sig,archive,updates.verify(raw,sig,[self.public],core.digest(self.exe)))
        core.atomic(self.root/("AZSX/Data/Updates/Packages/"+m["version"]+"/package.zip"),b"tampered")
        with patch.object(update_worker.subprocess,"Popen",side_effect=AssertionError("restart before verification")),self.assertRaises(ValueError):
            update_worker.finish({"root":str(self.root),"version":m["version"],"sequence":2,"keys":[self.public],"arguments":[]})
        self.assertEqual(core.installed(self.root)["version"],self.m["version"])

    def test_cleanup_preserves_applying_transactions_and_unknown_transaction_files(self):
        for i,phase in ((1,"applying"),(2,"committed")):
            base="AZSX/Data/Updates/Transactions/"+str(i)*32
            core.put(self.root,base+"/journal.json",{"phase":phase,"state":{},"before":{}})
            core.atomic(self.root/(base+"/personal-note.txt"),b"keep")
        housekeeping.clean(self.root)
        self.assertEqual(len(list((self.root/"AZSX/Data/Updates/Transactions").iterdir())),2)

    def test_native_upgrade_blocks_incompatible_rollback_without_changing_files(self):
        self.staged()
        core.atomic(self.root/"AzerothSurvivors.exe",self.new_exe)
        native_update.synchronize(self.root,core.digest(self.new_exe),offline=True)
        before=self.snapshot()
        with self.assertRaisesRegex(ValueError,"different native client"):core.rollback(self.root)
        self.assertEqual(before,self.snapshot())

    def test_bad_signature_prevents_staging_native_update(self):
        m,data=self.future();fetch,calls=self.network(m,data)
        def invalid(url,limit):return bytes(64)if url.endswith(".sig")else fetch(url,limit)
        with patch.object(native_update,"announcement",side_effect=AssertionError("official updater considered before authentication")),self.assertRaises(ValueError):
            native_update.synchronize(self.root,core.digest(self.exe),download=invalid)
        self.assertFalse(core.path(self.root,native_update.PENDING).exists())

    def test_owned_obsolete_files_removed_but_untracked_files_preserved(self):
        core.atomic(self.root/"AZSX/App/Launcher/personal-note.txt",b"keep")
        m,data=self.release(2,extra=False);fetch,_=self.network(m,data)
        updates.automatic(self.root,core.digest(self.exe),fetch)
        self.assertFalse((self.root/"AZSX/App/Launcher/old.py").exists())
        self.assertTrue((self.root/"AZSX/App/Launcher/personal-note.txt").exists())

    def test_running_python_repair_uses_detached_worker_without_question(self):
        import types
        m,data=self.release(1);name="AZSX/App/Python/python.exe";data[name]=b"MZ replacement runtime"
        m["files"][name]={"size":len(data[name]),"sha256":core.digest(data[name])}
        self.extract(m,{n:b for n,b in data.items()if n!=name})
        raw,sig,archive=self.signed(m,data,1);updates.cache(self.root,raw,sig,archive,updates.verify(raw,sig,[self.public],core.digest(self.exe)))
        with patch.object(launch.maintenance,"os",types.SimpleNamespace(name="nt")),patch.object(update_worker,"spawn",return_value=100)as spawn:
            self.assertEqual(launch.maintenance.repair(self.root,lambda _:self.fail("question"),automatic=True,offline=True),"handoff")
        self.assertTrue(spawn.call_args.kwargs["repair"])
        self.assertTrue(spawn.call_args.kwargs["automatic_repair"])

    def test_superseded_pending_transition_adopts_new_matching_release(self):
        self.staged()
        m,data=self.future();m["version"]="0.127.0-r1"
        new_exe=b"MZ another official client";m["game"].update(version="0.127.0",sha256=core.digest(new_exe))
        registry="AZSX/App/Launcher/supported-games.json"
        data[registry]=core.encoded({"schema":1,"games":{"0.127.0":{"sha256":core.digest(new_exe)}}})
        m["files"][registry]={"size":len(data[registry]),"sha256":core.digest(data[registry])}
        fetch,_=self.network(m,data,3)
        with patch.object(native_update,"announcement",return_value=("0.127.0",core.digest(new_exe))):
            self.assertEqual(native_update.synchronize(self.root,core.digest(self.exe),download=fetch),"native update ready")
        self.assertEqual(core.get(self.root,native_update.PENDING)["version"],"0.127.0-r1")
        self.assertEqual(core.installed(self.root)["version"],self.m["version"])

    def test_corrupt_staged_cache_can_be_refreshed_online_after_native_update(self):
        m,data,fetch,calls=self.staged()
        core.atomic(self.root/("AZSX/Data/Updates/Packages/"+m["version"]+"/package.zip"),b"corrupt cache")
        core.atomic(self.root/"AzerothSurvivors.exe",self.new_exe)
        self.assertEqual(native_update.synchronize(self.root,core.digest(self.new_exe),download=fetch),"installed")
        self.assertEqual(core.installed(self.root)["version"],m["version"])
        self.assertFalse(core.path(self.root,native_update.PENDING).exists())
