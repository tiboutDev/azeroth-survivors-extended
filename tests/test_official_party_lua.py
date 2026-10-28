"""Run with: uv run --with lupa==2.6 python3 tests/test_official_party_lua.py.
Uses Lua 5.1, matching the native addon API's legacy this/event callbacks.
"""
from pathlib import Path
import unittest
try:
 from lupa.lua51 import LuaRuntime
except ImportError:
 LuaRuntime=None
ROOT=Path(__file__).resolve().parents[1]
STUB=r"""
-- Native compatibility layer implements only string.byte(s, index).
local byte=string.byte
string.byte=function(s,i)return byte(s,i)end
cvars={};asxEnabled=true
function RegisterCVar(key,value)cvars[key]=cvars[key] or value end
function SetCVar(key,value)cvars[key]=(key=="asx_runtime_active" and asxEnabled)and"1"or value end
function GetCVar(key)return cvars[key]end
objects={}; now=1; allocations=0; visibilityWrites=0; tooltipCalls=0; buffs={party1={"buff-a","buff-b"},party2={"buff-c"}}; names={party1="Alice",party2="Bob"}; connected={party1=true,party2=true}
function getglobal(name)return objects[name]end
function GetTime()return now end
function GetPlayerBuff(index,filter)return index,0 end
function AZS_GetPhase()return phase end
function UnitExists(unit)return names[unit]~=nil end
function UnitIsConnected(unit)return connected[unit]end
function UnitName(unit)return names[unit]end
function UnitBuff(unit,index)return buffs[unit] and buffs[unit][index]end
local methods={}
function methods:SetID(id)self.id=id end
function methods:GetID()return self.id end
function methods:EnableMouse(v)self.mouse=v end
function methods:SetFrameStrata(v)self.strata=v end
function methods:SetFrameLevel(v)self.level=v end
function methods:SetOwner(v)self.owner=v end
function methods:ClearLines()self.lines={} end
function methods:AddLine(v)table.insert(self.lines,v)end
function methods:GetName()return self.name end
function methods:GetParent()return self.parent end
function methods:SetWidth(v)end
function methods:SetAllPoints(v)end
function methods:SetAlpha(v)self.alpha=v end
function methods:SetTextColor(...)end
function methods:CreateTexture()return CreateFrame("Texture",nil,self)end
function methods:SetHeight(v)end
function methods:ClearAllPoints()self.anchor=nil end
function methods:SetPoint(...)self.anchor={unpack(arg)}end
function methods:SetScript(event,fn)self.scripts[event]=fn end
function methods:RegisterEvent(e)end
function methods:SetTexture(t)self.texture=t end
function methods:SetText(t)self.text=t end
function methods:Hide()self.shown=false;visibilityWrites=visibilityWrites+1 end
function methods:Show()self.shown=true;visibilityWrites=visibilityWrites+1 end
function methods:IsVisible()return self.shown and (not self.parent or self.parent:IsVisible())end
function methods:CreateFontString()return CreateFrame("Font",nil,self)end
function CreateFrame(kind,name,parent,template)
 allocations=allocations+1
 local f=setmetatable({name=name,parent=parent,scripts={},shown=true},{__index=methods})
 if name then objects[name]=f end
 if template then CreateFrame("Texture",name.."Icon",f);CreateFrame("Texture",name.."Border",f)end
 return f
end
UIParent=CreateFrame("Frame","UIParent")
PlayerFrame=CreateFrame("Frame","PlayerFrame",UIParent)
for i=1,4 do local f=CreateFrame("Frame","PartyMemberFrame"..i,UIParent); f.scripts.OnClick=function()end; f.scripts.OnEvent=function()end end
nativeClick=objects.PartyMemberFrame1.scripts.OnClick
nativeEvent=objects.PartyMemberFrame1.scripts.OnEvent
GameTooltip={}
function GameTooltip:Hide()self.hidden=true end
function GameTooltip:SetOwner(owner)self.owner=owner end
function methods:GetScript(event)return self.scripts[event]end
function GameTooltip:ClearLines()self.title=nil;self.detail=nil end
function GameTooltip:AddDoubleLine(left,right)self.title=left;self.rank=right end
function GameTooltip:AddLine(text)self.detail=text end
function GameTooltip:Show()self.hidden=false end
function AZS_SetHudShown(part,shown)assert(part=="build" and not shown);buildDefaults=(buildDefaults or 0)+1 end
function GameTooltip:SetUnitBuff(unit,id)tooltipCalls=tooltipCalls+1;self.unit=unit;self.index=id end
nativeAura="Interface\\Icons\\Spell_Shadow_DeathScream"
function GetPlayerBuffTexture(index)return index==32 and nativeAura or "unrelated"end
function GetPlayerBuffTimeLeft(index)return 123 end
function UnitDebuff(unit,index)return index==1 and nativeAura or "unrelated",2 end
objects.GameTooltipTextLeft1={IsShown=function()return true end,GetText=function()return "Resurrection Sickness"end}
function GameTooltip:SetPlayerBuff(index)self.title="Native";self.detail="Original"end
function GameTooltip:SetUnitDebuff(unit,index,filter)self.title="Native";self.detail="Original"end
function update()objects.AzCoopOfficialPartyPresentation.scripts.OnUpdate()end
function enter(button)this=objects[button];this.scripts.OnEnter();this=nil end
"""
@unittest.skipUnless(LuaRuntime,"Use the official-party-ui profile for required Lua 5.1 execution")
class NativePartyLua(unittest.TestCase):
 def setUp(self):
  self.lua=LuaRuntime();self.lua.execute(STUB);self.lua.execute((ROOT/"native-ui/official-party.lua").read_text().replace("local asxNativeActive = 0 -- ASX_LOCAL_UI_GATE","local asxNativeActive = 1 -- ASX_LOCAL_UI_GATE",1));self.lua.execute("update();now=now+0.3;update()")
 def test_blessing_native_buff_and_single_retained_notification(self):
  text=bytes.fromhex(self.wire()).decode();out=[]
  for line in text.splitlines():
   out.append(line)
   if line.startswith("M|"):out.append("B|"+line.split("|")[1]+"|1|3|2|6300|6300")
  wire="\n".join(out)+"\n"
  self.publish(wire.encode().hex())
  self.lua.execute('nativeAura="Interface/Icons/INV_Misc_Orb_02.blp";assert(GetPlayerBuffTimeLeft(32)==0);local slot,permanent=GetPlayerBuff(32,"HELPFUL");assert(slot==32 and permanent==1);GameTooltip:SetPlayerBuff(32);assert(GameTooltip.title=="Sayge\'s Dark Fortune of Discovery" and GameTooltip.rank=="Rare");assert(objects.ASXDarkmoonBlessingToast.shown);local n=allocations;now=now+7;AZS_RUN_UPDATED=now;update();assert(not objects.ASXDarkmoonBlessingToast.shown and allocations==n)')
  self.publish(wire.encode().hex())
  self.lua.execute('assert(not objects.ASXDarkmoonBlessingToast.shown)')
  self.lua.execute('phase="choosing"')
  self.publish(wire.replace('|1|3|2|6300|6300','|2|1|0|1400|2400').encode().hex())
  self.lua.execute('assert(not objects.ASXDarkmoonBlessingToast.shown);phase="playing";now=now+1;AZS_RUN_UPDATED=now;update()')
  self.lua.execute('assert(objects.ASXDarkmoonBlessingToast.shown);GameTooltip:SetPlayerBuff(32);assert(GameTooltip.title=="Sayge\'s Dark Fortune of Wisdom");now=now+5;update();assert(GetPlayerBuffTimeLeft(32)==123)')
 def test_solo_wisdom_uses_native_buff_toast_and_personal_description(self):
  rows=bytes.fromhex(self.wire()).decode().splitlines()[:4];rows.insert(2,"B|11|1|1|0|1200|1200")
  self.publish(("\n".join(rows)+"\n").encode().hex())
  self.lua.execute("nativeAura='Interface/Icons/INV_Misc_Orb_02.blp';tooltipLines={};function GameTooltip:AddLine(s)table.insert(tooltipLines,s)end;GameTooltip:SetPlayerBuff(32)")
  self.assertEqual(self.lua.eval("GameTooltip.title"),"Sayge's Dark Fortune of Wisdom")
  self.lua.execute('assert(tooltipLines[1]=="+12% Experience");assert(objects.ASXDarkmoonBlessingToast.reward.text=="+12% Experience");assert(tooltipLines[2]=="Personal bonus.");local slot,permanent=GetPlayerBuff(32,"HELPFUL");assert(permanent==1 and GetPlayerBuffTimeLeft(32)==0);assert(objects.ASXDarkmoonBlessingToast.shown)')
 def test_swiftness_is_a_personal_permanent_native_fortune(self):
  text=bytes.fromhex(self.wire()).decode();out=[]
  for line in text.splitlines():
   out.append(line)
   if line.startswith("M|"):out.append("B|"+line.split("|")[1]+"|5|4|4|2000|2000")
  wire="\n".join(out)+"\n";self.publish(wire.encode().hex())
  self.lua.execute("nativeAura='Interface/Icons/INV_Misc_Orb_02.blp';GameTooltip:SetPlayerBuff(32)")
  self.assertEqual(self.lua.eval("GameTooltip.title"),"Sayge's Dark Fortune of Swiftness")
  self.lua.execute('assert(GetPlayerBuffTimeLeft(32)==0);local slot,permanent=GetPlayerBuff(32,"HELPFUL");assert(slot==32 and permanent==1);assert(objects.ASXDarkmoonBlessingToast.shown)')
  _,_,_,_,b=self.lua.eval("AzCoopOfficialPartyReadout")();self.assertEqual(b['category'],4)
  self.publish(wire.replace('|5|4|4|2000|2000','|6|0|0|2700|2700').encode().hex())
  self.lua.execute('GameTooltip:SetPlayerBuff(32)');self.assertEqual(self.lua.eval("GameTooltip.title"),"Sayge's Dark Fortune of Luck")
 def test_fatigue_uses_native_debuff_channel_title_and_frozen_countdown(self):
  text=bytes.fromhex(self.wire()).decode();out=[]
  for line in text.splitlines():
   out.append(line)
   if line.startswith("M|"):out.append("F|"+line.split("|")[1]+"|2|47")
  wire=("\n".join(out)+"\n")
  self.publish(wire.encode().hex())
  self.lua.execute('nativeAura="Interface/Icons/Spell_Shadow_DeathScream.blp";assert(objects.AzCoopResurrectionFatigue0==nil);assert(GetPlayerBuffTimeLeft(32)==47 and GetPlayerBuffTimeLeft(0)==123);GameTooltip:SetPlayerBuff(32);assert(GameTooltip.title=="Resurrection Sickness" and GameTooltip.rank=="2 / 3");GameTooltip:SetUnitDebuff("party1",1);assert(GameTooltip.title=="Resurrection Sickness");GameTooltip:SetUnitDebuff("party1",2);assert(GameTooltip.title=="Native");local n=allocations;now=now+1;AZS_RUN_UPDATED=now;update();assert(GetPlayerBuffTimeLeft(32)==47 and allocations==n);now=now+5;update();assert(GetPlayerBuffTimeLeft(32)==123)')
  self.publish(wire.encode().hex())
  self.lua.execute('enter("AzCoopOfficialParty1Item1");local r=AzCoopOfficialPartyReadout();oldIcon=r[1].items[1].data')
  self.publish(wire.replace("|2|47","|2|46").encode().hex())
  self.lua.execute('local r=AzCoopOfficialPartyReadout();assert(r[1].items[1].data==oldIcon and not GameTooltip.hidden);GameTooltip:SetPlayerBuff(32);assert(GameTooltip.detail=="Next stack expires in 46s of alive game time.")')
  self.publish(wire.replace("|2|47","|4|47").encode().hex())
  self.lua.execute('assert(GetPlayerBuffTimeLeft(32)==123)')
 def test_unused_slots_allocate_nothing_and_late_members_keep_retained_children(self):
  self.lua.execute('local r,b=AzCoopOfficialPartyReadout();assert(b==2 and r[3]==nil and r[4]==nil);local count=allocations;now=2;update();assert(allocations==count);names.party3="Later";connected.party3=true;buffs.party3={"new"};now=3;update();r,b=AzCoopOfficialPartyReadout();assert(b==3 and r[3].buffs==1);count=allocations;names.party3=nil;now=4;update();assert(not objects.AzCoopOfficialParty3Buff1:IsVisible());names.party3="Returned";now=5;update();assert(allocations==count and objects.AzCoopOfficialParty3Buff1.owner=="Returned")')
 def test_refreshes_are_bounded_and_latest_state_is_observed_without_event_listeners(self):
  self.lua.execute('local observer=objects.AzCoopOfficialPartyPresentation;assert(observer.scripts.OnEvent==nil);local reads=0;local original=UnitBuff;UnitBuff=function(u,n)reads=reads+1;return original(u,n)end;for i=1,600 do now=1.3+i/60;update()end;assert(reads>0 and reads<=40*64);buffs.party1={"replacement"};now=now+0.3;update();assert(objects.AzCoopOfficialParty1Buff1.texture=="replacement")')
 def test_tooltip_audit_bounds_unready_polling_and_retires_after_one_capture(self):
  self.publish(self.wire())
  self.lua.execute('local methods=getmetatable(UIParent).__index;function methods:GetWidth()return 800 end;function methods:GetHeight()return 600 end;function methods:SetJustifyH()end;function methods:SetJustifyV()end;function methods:SetFont()end;function methods:SetOwner()end;function methods:ClearLines()end;actionCalls=0;function methods:SetAction(slot)actionCalls=actionCalls+1 end;function HasAction(slot)return slot<=3 end;readouts=0;local read=AzCoopOfficialPartyReadout;function AzCoopOfficialPartyReadout()readouts=readouts+1;return read()end;AZS_RUN.t=0')
  self.lua.execute((ROOT/'tools/dev/fixtures/asx-tooltip-audit.lua').read_text())
  self.lua.execute('local f=objects.ASXTooltipAudit;for i=1,600 do now=2+i/60;f.scripts.OnUpdate()end;assert(readouts==0 and actionCalls==0 and objects.ASXTooltipOwnerTip==nil);AZS_RUN.t=10;local real=AzCoopOfficialPartyReadout;AzCoopOfficialPartyReadout=function()readouts=readouts+1;return {},0,nil,0 end;for i=1,600 do now=12+i/60;f.scripts.OnUpdate()end;assert(readouts>=4 and readouts<=5 and actionCalls==0);AzCoopOfficialPartyReadout=real;now=25;AZS_RUN_UPDATED=now;update();f.scripts.OnUpdate();assert(actionCalls==1 and f.scripts.OnUpdate==nil)')
 def test_native_launch_does_not_construct_asx_rows(self):
  lua=LuaRuntime();lua.execute(STUB);lua.execute("asxEnabled=false")
  lua.execute((ROOT/"native-ui/official-party.lua").read_text());lua.execute("assert(AzCoopOfficialPartyReadout==nil);assert(objects.AzCoopOfficialPartyPresentation==nil);assert(buildDefaults==nil);assert(objects.PartyMemberFrame1.scripts.OnClick==nativeClick)")
 def test_native_source_stays_inactive_even_with_asx_host_data(self):
  lua=LuaRuntime();lua.execute(STUB)
  lua.execute('cvars.asx_runtime_active="1";AZS_RUN={azcoop="host-data"};AZS_RUN_UPDATED=1')
  lua.execute((ROOT/"native-ui/official-party.lua").read_text());lua.execute('assert(AzCoopOfficialPartyReadout==nil);assert(objects.AzCoopOfficialPartyPresentation==nil);assert(objects.PartyMemberFrame1:IsVisible())')
 def test_unchanged_rows_do_not_enqueue_repeated_visibility_writes(self):
  self.lua.execute('local before=visibilityWrites;for i=1,16 do now=now+0.3;update()end;assert(visibilityWrites==before)')
 def test_retained_children_leave_native_parent_and_unit_scripts_untouched(self):
  self.lua.execute("assert(AzCoopOfficialPartyReadout); assert(objects.PartyMemberFrame1.scripts.OnClick==nativeClick);assert(objects.PartyMemberFrame1.scripts.OnEvent==nativeEvent); count=allocations; for i=1,40 do now=now+1;update()end;assert(allocations==count)")
 def test_buff_hover_uses_native_positive_unit_api_and_rejects_stale_owner(self):
  self.lua.execute('enter("AzCoopOfficialParty1Buff1");assert(GameTooltip.unit=="party1" and GameTooltip.index==1);names.party1="Other";enter("AzCoopOfficialParty1Buff1");assert(tooltipCalls==1);now=2;update();assert(GameTooltip.hidden);enter("AzCoopOfficialParty1Buff1");assert(tooltipCalls==2)')
 def test_disconnect_removal_hidden_parent_and_overflow(self):
  self.lua.execute('connected.party1=false;now=2;update();assert(not objects.AzCoopOfficialParty1Buff1:IsVisible()); connected.party1=true; buffs.party1={};now=3;update();assert(not objects.AzCoopOfficialParty1Buff1:IsVisible());for i=1,32 do buffs.party1[i]="buff"..i end;now=4;update();local rows,built=AzCoopOfficialPartyReadout();assert(rows[1].buffs==16 and built==2);objects.PartyMemberFrame1:Hide();now=5;update();assert(not objects.AzCoopOfficialParty1Buff1:IsVisible())')
 def wire(self, own="11", epoch="1", duplicate=False, item="+52% gold"):
  h=lambda s:s.encode().hex()
  rows=[f"1|{epoch}|{own}|5"]
  for slot,id,name in [(0,"11","Me"),(1,"22","Alice"),(2,"33","Alice" if duplicate else "Bob")]:
   rows.append(f"M|{id}|{slot}|{h(name)}")
   rows.append(f"S|{id}|845|2|{h('Cleave')}|{h('Ability_Warrior_Cleave')}|")
   rows.append(f"I|{id}|1|1|{h('Seal')}|{h('INV_Jewelry_Talisman_10')}|{h(item)}")
  return ("\n".join(rows)+"\n").encode().hex()
 def publish(self, wire):
  self.lua.globals().wire=wire
  self.lua.execute('now=now+0.3;AZS_RUN={azcoop=wire};AZS_RUN_UPDATED=now;update()')
 def test_native_identity_binding_owner_item_tooltips_and_one_time_build_default(self):
  self.publish(self.wire())
  self.lua.execute('local r=AzCoopOfficialPartyReadout();assert(r[1].member.id=="22" and r[2].member.id=="33");enter("AzCoopOfficialParty1Item1");assert(GameTooltip.title=="Seal" and GameTooltip.detail=="+52% gold");assert(buildDefaults==1);now=2;AZS_RUN_UPDATED=now;update();assert(buildDefaults==1)')
  self.publish(self.wire(epoch="2",item="+18% experience"))
  self.lua.execute('enter("AzCoopOfficialParty1Item1");assert(GameTooltip.detail=="+18% experience")')
 def test_duplicate_names_wrong_owner_malformed_data_and_expiry_hide_builds(self):
  self.publish(self.wire(duplicate=True))
  self.lua.execute('assert(not objects.AzCoopOfficialParty1Spell1:IsVisible())')
  self.publish(self.wire())
  self.lua.execute('assert(objects.AzCoopOfficialParty1Spell1:IsVisible());now=8;update();assert(not objects.AzCoopOfficialParty1Spell1:IsVisible())')
  for wire in (self.wire(own="44"),"xx","1|1|11|5".encode().hex(),self.wire().replace("7c32327c31","7c32327c30",1)):
   self.publish(wire)
   self.lua.execute('assert(not objects.AzCoopOfficialParty1Spell1:IsVisible())')
 def test_heartbeat_keeps_hover_and_build_identity_while_data_updates_retire_it(self):
  wire=self.wire();self.publish(wire)
  self.lua.execute('enter("AzCoopOfficialParty1Item1");local r=AzCoopOfficialPartyReadout();before=r[1].items[1].data;assert(not GameTooltip.hidden)')
  decoded=bytes.fromhex(wire).decode().replace("1|1|11|5\n","1|1|11|6\n",1)
  self.publish(decoded.encode().hex())
  self.lua.execute('local r=AzCoopOfficialPartyReadout();assert(r[1].items[1].data==before);assert(not GameTooltip.hidden)')
  self.publish(self.wire(item="Changed"))
  self.lua.execute('assert(GameTooltip.hidden)')
 def test_compressed_native_spell_headers_body_and_malformed_envelopes(self):
  wire=bytes.fromhex(self.wire()).decode()
  h=lambda s:s.encode().hex()
  wire=wire.replace(f"S|22|845|2|{h('Cleave')}|{h('Ability_Warrior_Cleave')}|",f"S|22|845|2|{h('Cleave')}|{h('Ability_Warrior_Cleave')}|{h('104 Physical damage. Your bonuses: +58% damage')}|{h('Rank 2')}|{h('7 yd range')}|{h('Automatic')}|{h('2.1 sec cooldown')}")
  raw=wire.encode();packed=bytes([2])+len(raw).to_bytes(4,'little')+b''.join(b'\0'+raw[i:i+8]for i in range(0,len(raw),8))
  self.publish(packed.hex())
  self.lua.execute('enter("AzCoopOfficialParty1Spell1");assert(GameTooltip.detail=="104 Physical damage. Your bonuses: +58% damage");assert(GameTooltip.title=="Automatic" and GameTooltip.rank=="2.1 sec cooldown")')
  for bad in (packed[:-1],packed+b'x',bytes([2])+bytes([255])*4,bytes([2,3,0,0,0,1,0,0])):
   self.publish(bad.hex());self.lua.execute('assert(not objects.AzCoopOfficialParty1Spell1:IsVisible())')
 def test_other_viewer_uses_its_native_frame_order(self):
  self.lua.execute('names.party1="Me";names.party2="Bob"')
  self.publish(self.wire(own="22"))
  self.lua.execute('local r=AzCoopOfficialPartyReadout();assert(r[1].member.id=="11" and r[2].member.id=="33")')
if __name__=="__main__":unittest.main()
