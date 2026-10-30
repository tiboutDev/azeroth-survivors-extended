import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools/dev"))
from client_registry import load_clients, selected_client, VERSION, SHA256, NAMES

class ClientRegistryTests(unittest.TestCase):
 def fixture(self):
  return {"schema":1,"games":{"1.2.3":{"sha256":"a"*64,"archives":["Data/patch.MPQ"]}}}
 def read(self,data,select=False):
  with tempfile.TemporaryDirectory() as folder:
   path=Path(folder)/"clients.json";path.write_text(json.dumps(data))
   return (selected_client if select else load_clients)(path)
 def test_exact_identity_and_inventory_are_shared(self):
  import native_runtime_import as importer
  self.assertEqual((importer.SHA256,importer.NAMES),(SHA256,NAMES))
  self.assertEqual(selected_client()[0],VERSION)
  self.assertEqual(self.read(self.fixture(),True)[0],"1.2.3")
 def test_invalid_identity_and_paths_fail_closed(self):
  for field,value in (("sha256","unknown"),("archives",["../patch.MPQ"]),("archives",["Data/../patch.MPQ"]),("archives",["C:/patch.MPQ"]),("archives",["Data/patch.MPQ"]*2),("archives",[])):
   with self.subTest(field=field,value=value):
    data=self.fixture();data["games"]["1.2.3"][field]=value
    with self.assertRaises(ValueError):self.read(data)
 def test_multiple_builds_require_explicit_selection(self):
  data=self.fixture();data["games"]["1.2.4"]=copy.deepcopy(data["games"]["1.2.3"])
  with self.assertRaises(ValueError):self.read(data,True)
