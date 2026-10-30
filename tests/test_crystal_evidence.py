import unittest,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools/dev'))
from asx_api_trial import assess_crystals,crystal_stat_checks
class CrystalEvidence(unittest.TestCase):
 def test_presence_without_native_buff_or_tooltip_is_not_success(self):
  sample={'available':True,'copyIsolated':True,'structured':{'crystals':{'red':1,'blue':2,'green':1,'yellow':1,'total':5,'index':3}}}
  self.assertFalse(all(assess_crystals([sample]).values()))
  sample['crystalBuff']={'visible':True,'tooltip':True,'permanent':True};self.assertTrue(all(assess_crystals([sample]).values()))
  sample['structured']['markers']={'1':{'kind':0}};self.assertFalse(assess_crystals([sample])['no_custom_minimap_pin'])
 def test_stats_require_real_owner_and_values_and_restored_inputs(self):
  row='crystal native stats: slot=1 owner=22 counts=[1, 2, 1, 1] base=[1, 1, 100, 0, 0, 0, 0, 1] actual=Ok([1.1, 0.9090909, 115, 0, 0, 0, 0, 1.1]) stable=true restored=true refresh=Ok(())'
  self.assertTrue(crystal_stat_checks(row,1)['native_personal_stats']);self.assertFalse(crystal_stat_checks(row,0)['native_personal_stats'])
  self.assertFalse(crystal_stat_checks(row.replace('restored=true','restored=false'),1)['native_personal_stats'])
  self.assertFalse(crystal_stat_checks(row.replace('1.1, 0.9090909','1.2, 0.9090909'),1)['native_personal_stats'])
if __name__=='__main__':unittest.main()
