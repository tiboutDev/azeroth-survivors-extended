"""Crystal data/tooltip integration through the retained Lua 5.1 provider."""
import unittest
from test_official_party_lua import STUB,LuaRuntime,ROOT
@unittest.skipIf(LuaRuntime is None,"Lua 5.1 dependency unavailable")
class CrystalsLua(unittest.TestCase):
 def setUp(self):
  self.lua=LuaRuntime(unpack_returned_tuples=True);self.lua.execute(STUB)
  self.lua.execute('GetPlayerBuffTexture=function(i)if i==3 then return "Interface/Icons/Temp"end;return "Native"end;GetPlayerBuff=function(i)return i,0 end;GetPlayerBuffTimeLeft=function(i)return 9 end;GameTooltip.SetPlayerBuff=function()end')
  source=(ROOT/'native-ui/official-party.lua').read_text().replace('local asxNativeActive = 0 -- ASX_LOCAL_UI_GATE','local asxNativeActive = 1 -- ASX_LOCAL_UI_GATE');self.lua.execute(source)
 def publish(self,extra='G|11|2|1|3|1|3\n'):
  wire=('1|1|11|1\nM|11|0|506c61796572\n'+extra).encode().hex();self.lua.execute('AZS_RUN={asx="'+wire+'"};AZS_RUN_UPDATED=now;objects.ASXOfficialPartyPresentation.scripts.OnUpdate()')
 def test_counts_bonuses_native_slot_and_permanent_texture(self):
  self.publish();self.lua.execute(r'local s=ASX_GetState();assert(s.api.minor==4 and s.data.crystals.total==7);assert(s.data.crystals.bonuses.damage==20 and s.data.crystals.bonuses.maximumHealth==45);assert(s.texts.crystals.title=="Crystal Resonance");local slot,permanent=GetPlayerBuff(3);assert(slot==3 and permanent==1 and GetPlayerBuffTimeLeft(3)==0);assert(GetPlayerBuffTexture(3)=="Interface\\Icons\\INV_Misc_Gem_Variety_01");assert(GetPlayerBuffTexture(2)=="Native")')
 def test_pickup_texts_are_asx_owned_and_detached(self):
  self.publish();self.lua.execute('local s=ASX_GetState();local p=s.texts.crystals.pickups;assert(p.red.title=="Red Crystal collected"and p.red.reward=="+10% damage");assert(p.blue.reward=="+5% cooldown recovery"and p.green.reward=="+15% maximum health"and p.yellow.reward=="+10% area of effect");p.red.title="edited";p.red.titleColor[1]=0;assert(ASX_GetState().texts.crystals.pickups.red.title=="Red Crystal collected"and ASX_GetState().texts.crystals.pickups.red.titleColor[1]==1)')
 def test_duplicate_malformed_and_stale_data_do_not_override_native(self):
  for row in ['G|11|1|0|0|0|33\n','G|11|-1|0|0|0|3\n','G|11|0|0|0|0|3\n','G|11|1|0|0|0|3\nG|11|1|0|0|0|3\n']:
   self.publish(row);self.lua.execute('assert(not ASX_GetState().available and GetPlayerBuffTimeLeft(3)==9)')
  self.publish();self.lua.execute('now=now+5;assert(GetPlayerBuffTimeLeft(3)==9 and not ASX_GetState().available)')
 def test_independent_api_snapshots_and_tooltip_values(self):
  self.publish();self.lua.execute('local a=ASX_GetState();a.data.crystals.red=999;assert(ASX_GetState().data.crystals.red==2);lines={};GameTooltip.ClearLines=function()lines={}end;GameTooltip.AddLine=function(self,text)table.insert(lines,text)end;GameTooltip.Show=function()end;GameTooltip:SetPlayerBuff(3);assert(lines[1]=="Crystal Resonance" and lines[2]=="Crystals collected: 7" and lines[3]=="+20% damage" and lines[5]=="+45% maximum health")')
if __name__=='__main__':unittest.main()
