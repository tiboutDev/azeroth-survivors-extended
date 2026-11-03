"""Crystal data/tooltip integration through the retained Lua 5.1 provider."""
import unittest
from test_official_party_lua import STUB,LuaRuntime,ROOT
@unittest.skipIf(LuaRuntime is None,"Lua 5.1 dependency unavailable")
class CrystalsLua(unittest.TestCase):
 def setUp(self):
  self.lua=LuaRuntime(unpack_returned_tuples=True);self.lua.execute(STUB)
  self.lua.execute('GetPlayerBuffTexture=function(i)if i==3 then return "Interface/Icons/Temp"end;return "Native"end;GetPlayerBuff=function(i)return i,0 end;GetPlayerBuffTimeLeft=function(i)return 9 end;GameTooltip.SetPlayerBuff=function()end')
  source=(ROOT/'native-ui/official-party.lua').read_text().replace('local azsxNativeActive = 0 -- AZSX_LOCAL_UI_GATE','local azsxNativeActive = 1 -- AZSX_LOCAL_UI_GATE');self.lua.execute(source)
 def publish(self,extra='G|11|2|1|3|1|3\n'):
  wire=('1|1|11|1\nM|11|0|506c61796572\n'+extra).encode().hex();self.lua.execute('AZS_RUN={azsx="'+wire+'"};AZS_RUN_UPDATED=now;objects.AZSXOfficialPartyPresentation.scripts.OnUpdate()')
 def test_counts_bonuses_native_slot_and_permanent_texture(self):
  self.publish();self.lua.execute(r'local s=AZSX_GetState();assert(s.api.minor>=6 and s.data.crystals.total==7);assert(s.data.crystals.bonuses.damage==24 and s.data.crystals.bonuses.cooldownRecovery==7 and s.data.crystals.bonuses.maximumHealth==45 and s.data.crystals.bonuses.areaOfEffect==0 and s.data.crystals.bonuses.criticalStrikeChance==0 and s.data.crystals.bonuses.luck==8);assert(s.texts.crystals.title=="Crystal Resonance");local slot,permanent=GetPlayerBuff(3);assert(slot==3 and permanent==1 and GetPlayerBuffTimeLeft(3)==0);assert(GetPlayerBuffTexture(3)=="Interface\\Icons\\INV_Misc_Gem_Variety_01","icon:"..tostring(GetPlayerBuffTexture(3)));assert(GetPlayerBuffTexture(2)=="Native")')
 def test_pickup_texts_are_azsx_owned_and_detached(self):
  self.publish();self.lua.execute('local s=AZSX_GetState();local p=s.texts.crystals.pickups;assert(p.red.title=="Red Crystal collected"and p.red.reward=="+12% damage");assert(p.blue.reward=="+7% cooldown recovery"and p.green.reward=="+15% maximum health"and p.yellow.reward=="+8% Luck");p.red.title="edited";p.red.titleColor[1]=0;assert(AZSX_GetState().texts.crystals.pickups.red.title=="Red Crystal collected"and AZSX_GetState().texts.crystals.pickups.red.titleColor[1]==1)')
 def test_duplicate_malformed_and_stale_data_do_not_override_native(self):
  for row in ['G|11|1|0|0|0|33\n','G|11|-1|0|0|0|3\n','G|11|1|0|0|0|3\nG|11|1|0|0|0|3\n']:
   self.publish(row);self.lua.execute('assert(not AZSX_GetState().available and GetPlayerBuffTimeLeft(3)==9)')
  self.publish();self.lua.execute('now=now+5;assert(GetPlayerBuffTimeLeft(3)==9 and not AZSX_GetState().available)')
 def test_independent_api_snapshots_and_tooltip_values(self):
  self.publish();self.lua.execute('local a=AZSX_GetState();a.data.crystals.red=999;assert(AZSX_GetState().data.crystals.red==2);lines={};GameTooltip.ClearLines=function()lines={}end;GameTooltip.AddLine=function(self,text)table.insert(lines,text)end;GameTooltip.Show=function()end;GameTooltip:SetPlayerBuff(3);assert(lines[1]=="Crystal Resonance" and lines[2]=="Crystals collected: 7" and string.find(lines[3],"+24%",1,true) and string.find(lines[3],"(2 red crystals)",1,true) and string.find(lines[3],"+24%",1,true)<string.find(lines[3],"(2 red crystals)",1,true) and string.find(lines[5],"+45%",1,true))')
 def test_zero_total_is_a_visible_permanent_buff_with_guidance(self):
  self.publish("G|11|0|0|0|0|3\n")
  self.lua.execute(r'local s=AZSX_GetState();assert(s.available and s.data.crystals.total==0 and s.texts.crystals.note=="Crystals collected: 0","state");assert(GetPlayerBuffTexture(3)=="Interface\\Icons\\INV_Misc_Gem_Variety_01","icon:"..tostring(GetPlayerBuffTexture(3)));lines={};GameTooltip.ClearLines=function()lines={}end;GameTooltip.AddLine=function(self,text)table.insert(lines,text)end;GameTooltip.Show=function()end;GameTooltip:SetPlayerBuff(3);assert(lines[3]=="Find crystals across the map to gain permanent bonuses.","line:"..tostring(lines[3]))')

if __name__=='__main__':unittest.main()
