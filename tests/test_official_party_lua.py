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
phase="playing";objects={}; now=1; allocations=0; visibilityWrites=0; tooltipCalls=0; buffs={party1={"buff-a","buff-b"},party2={"buff-c"}}; names={party1="Alice",party2="Bob"}; connected={party1=true,party2=true}
function getglobal(name)return objects[name]end
function GetTime()return now end
function GetPlayerBuff(index,filter)return index,nativeAuraPermanent or 0 end
function AZS_GetPhase()return phase end
function UnitExists(unit)return names[unit]~=nil end
function UnitIsConnected(unit)return connected[unit]end
function UnitName(unit)return names[unit]end
function UnitBuff(unit,index)return buffs[unit] and buffs[unit][index]end
local methods={}
function methods:SetBackdrop(v)self.backdrop=v end
function methods:SetBackdropColor(...)end
function methods:SetBackdropBorderColor(...)end
function methods:SetJustifyH(v)self.justify=v end
function methods:SetTexCoord(...)self.texcoords={unpack(arg)}end
function methods:SetFont(path,size,flags)self.font=path;self.fontSize=size end
function methods:SetVertexColor(...)end
function methods:SetGradientAlpha(...)end
function methods:GetAlpha()return self.alpha or 1 end
function SetPortraitToTexture(name,path)local t=type(name)=="string" and objects[name]or name;assert(t);t:SetTexture(path);t.portraitMasked=true end
function methods:SetStatusBarTexture(t)self.texture=t end
function methods:SetStatusBarColor(...)end
function methods:SetMinMaxValues(a,b)self.minimum=a;self.maximum=b end
function methods:SetValue(v)self.value=v end
function methods:IsShown()return self.shown end
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
function methods:SetScale(v)self.scale=v end
function methods:SetWidth(v)self.width=v end
function methods:GetWidth()return self.width or 140 end
function methods:GetHeight()return self.height or 140 end
function methods:SetAllPoints(v)end
function methods:SetAlpha(v)self.alpha=v end
function methods:SetTextColor(...)end
function methods:CreateTexture(name,layer)return CreateFrame("Texture",name,self)end
function methods:SetHeight(v)self.height=v end
function methods:ClearAllPoints()self.anchor=nil end
function methods:SetPoint(...)self.anchor={unpack(arg)}end
function methods:SetScript(event,fn)self.scripts[event]=fn end
function methods:RegisterEvent(e)self.events=self.events or {};self.events[e]=true end
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
function GameTooltip:ClearLines()self.title=nil;self.detail=nil;self.firstTitle=nil end
function GameTooltip:AddDoubleLine(left,right)if not self.firstTitle then self.firstTitle=left end;self.title=left;self.rank=right end
function GameTooltip:AddLine(text)self.detail=text end
function GameTooltip:Show()self.hidden=false end
function AZS_SetHudShown(part,shown)assert(part=="build" and not shown);buildDefaults=(buildDefaults or 0)+1 end
function GameTooltip:SetUnitBuff(unit,id)tooltipCalls=tooltipCalls+1;self.unit=unit;self.index=id end
nativeAura="Interface\\Icons\\Spell_Shadow_DeathScream"
nativeBuffTextures={}
function GetPlayerBuffTexture(index)return nativeBuffTextures[index] or (index==32 and nativeAura or "unrelated")end
function GetPlayerBuffTimeLeft(index)return 123 end
function UnitDebuff(unit,index)return index==1 and nativeAura or "unrelated",2 end
objects.GameTooltipTextLeft1={IsShown=function()return true end,GetText=function()return "Resurrection Sickness"end}
function GameTooltip:SetPlayerBuff(index)self.title="Native";self.detail="Original"end
function GameTooltip:SetUnitDebuff(unit,index,filter)self.title="Native";self.detail="Original"end
function update()objects.ASXOfficialPartyPresentation.scripts.OnUpdate();if objects.ASXUIMinimapController then objects.ASXUIMinimapController.scripts.OnUpdate()end;if objects.ASXUIEventsController then objects.ASXUIEventsController.scripts.OnUpdate()end end
function enter(button)this=objects[button];this.scripts.OnEnter();this=nil end
"""
@unittest.skipUnless(LuaRuntime,"Use the official-party-ui profile for required Lua 5.1 execution")
class NativePartyLua(unittest.TestCase):
 def setUp(self):
  self.lua=LuaRuntime();self.lua.execute(STUB);self.lua.execute((ROOT/"native-ui/official-party.lua").read_text().replace("local asxNativeActive = 0 -- ASX_LOCAL_UI_GATE","local asxNativeActive = 1 -- ASX_LOCAL_UI_GATE",1));self.lua.execute((ROOT/"addons/ASXUI/Minimap.lua").read_text());self.lua.execute((ROOT/"addons/ASXUI/Events.lua").read_text());self.lua.execute("update();now=now+0.3;update()")
 def test_blessing_native_buff_and_single_retained_notification(self):
  text=bytes.fromhex(self.wire()).decode();out=[]
  for line in text.splitlines():
   out.append(line)
   if line.startswith("M|"):out.append("B|"+line.split("|")[1]+"|1|3|2|6300|6300")
  wire="\n".join(out)+"\n"
  self.publish(wire.encode().hex())
  self.lua.execute('nativeAura="Interface/Icons/INV_Misc_Orb_02.blp";assert(GetPlayerBuffTimeLeft(32)==0);local slot,permanent=GetPlayerBuff(32,"HELPFUL");assert(slot==32 and permanent==1);GameTooltip:SetPlayerBuff(32);assert(GameTooltip.title=="Sayge\'s Dark Fortune of Discovery" and GameTooltip.rank=="");assert(objects.ASXDarkmoonBlessingToast.shown);local n=allocations;now=now+7;AZS_RUN_UPDATED=now;update();assert(not objects.ASXDarkmoonBlessingToast.shown and allocations==n)')
  self.publish(wire.encode().hex())
  self.lua.execute('assert(not objects.ASXDarkmoonBlessingToast.shown)')
  self.lua.execute('phase="choosing"')
  self.publish(wire.replace('|1|3|2|6300|6300','|2|1|0|1400|2400').encode().hex())
  self.lua.execute('assert(not objects.ASXDarkmoonBlessingToast.shown);phase="playing";now=now+1;AZS_RUN_UPDATED=now;update()')
  self.lua.execute('assert(objects.ASXDarkmoonBlessingToast.shown);GameTooltip:SetPlayerBuff(32);assert(GameTooltip.firstTitle=="Sayge\'s Dark Fortune of Wisdom");now=now+5;update();assert(GetPlayerBuffTimeLeft(32)==123)')
 def test_supply_families_are_retained_temporary_and_do_not_mask_native_buffs(self):
  h=lambda s:s.encode().hex()
  c=f"C|11|0|2|60|2|{h('Elixir of Greater Intellect')}|{h('INV_Potion_10')}|{h('+10% cooldown recovery')}\n"
  d=f"D|11|1|1|0|{h('Elixir of Greater Intellect')}|{h('INV_Potion_10')}|{h('+10% cooldown recovery')}\n"
  text="1|1|11|1\nM|11|0|506c61796572\n"+c+d
  self.lua.execute('nativeAuraPermanent=1;nativeBuffTextures[2]="Interface/Icons/Temp.blp"')
  self.publish(text.encode().hex())
  self.lua.execute('assert(objects.ASXSupplyDiscoveryToast.shown);assert(objects.ASXSupplyDiscoveryToast.reward.text=="Elixir of Greater Intellect");local slot,permanent=GetPlayerBuff(2,"HELPFUL");assert(permanent==0 and GetPlayerBuffTimeLeft(2)==60 and GetPlayerBuffTimeLeft(3)==123);local n=allocations;now=now+4;AZS_RUN_UPDATED=now;update();assert(not objects.ASXSupplyDiscoveryToast.shown and allocations==n)')
  self.lua.execute('tipLines={};function GameTooltip:AddLine(text,r,g,b) table.insert(tipLines,{text,r,g,b}) end;function SecondsToTime(n) return n.." seconds" end;GameTooltip:SetPlayerBuff(2);assert(table.getn(tipLines)==3 and tipLines[1][1]=="Elixir of Greater Intellect" and tipLines[1][2]==1 and tipLines[1][3]==1 and tipLines[1][4]==1 and tipLines[2][1]=="+10% cooldown recovery" and tipLines[3][1]=="60 seconds remaining")')
  self.publish(text.replace('|2|60|2|','|2|12|2|').encode().hex())
  self.lua.execute('assert(GetPlayerBuffTimeLeft(2)==12);nativeBuffTextures[2]="native-class-buff";assert(GetPlayerBuffTexture(2)=="native-class-buff" and GetPlayerBuffTimeLeft(2)==123)')
  c2=f"C|11|2|3|19|3|{h('Elixir of Superior Defense')}|{h('INV_Potion_66')}|{h('+7% damage reduction')}\n"
  self.lua.execute('nativeBuffTextures[2]="Interface/Icons/Temp.blp";nativeBuffTextures[3]="Interface/Icons/Temp.blp"')
  self.publish((text+c2).encode().hex())
  self.lua.execute('local buffs=ASXSupplyReadout();assert(table.getn(buffs)==2);assert(GetPlayerBuffTimeLeft(2)==60 and GetPlayerBuffTimeLeft(3)==19)')
  self.publish((text+c2.replace('|19|3|','|19|2|')).encode().hex())
  self.lua.execute('assert(GetPlayerBuffTimeLeft(2)==123)')
 def test_speed_family_and_reference_quality_do_not_invalidate_other_features(self):
  h=lambda s:s.encode().hex();base="1|1|11|1\nM|11|0|506c61796572\n"
  c=f"C|11|6|2|60|2|{h('Swiftness Potion')}|{h('INV_Potion_95')}|{h('+15% movement speed')}\n"
  d=f"D|11|1|2|0|{h('Swiftness Potion')}|{h('INV_Potion_95')}|{h('+15% movement speed')}|1\n"
  self.publish((base+c+d+"P|11|1|0|0|0|90\n").encode().hex())
  self.lua.execute('local major,minor=ASX_GetAPIVersion();assert(major==1 and minor>=2);local s=ASX_GetState(1);assert(s.available and s.data.consumables[6].remaining==60 and s.data.discovery.quality==1 and s.texts.supplies.color[1]==1 and s.data.markers[1].left==90)')
 def test_current_supply_duration_and_marker_wire_bounds(self):
  h=lambda s:s.encode().hex();base="1|1|11|1\nM|11|0|506c61796572\n"
  c=f"C|11|0|2|60|2|{h('Elixir')}|{h('INV_Potion_10')}|{h('+10% recovery')}\n"
  pin="P|11|1|0|100|200|90\n"
  self.publish((base+c+pin).encode().hex())
  self.lua.execute('local s=ASX_GetState(1);assert(s.available and s.data.consumables[0].remaining==60 and s.data.markers[1].left==90)')
  self.publish((base+c.replace('|60|2|','|61|2|')+pin).encode().hex());self.lua.execute('assert(not ASX_GetState(1).available)')
  self.publish((base+c+pin.replace('|90','|91')).encode().hex());self.lua.execute('assert(not ASX_GetState(1).available)')
 def test_supply_metadata_refreshes_native_cached_button_without_repeating_each_countdown(self):
  h=lambda s:s.encode().hex();base="1|1|11|1\nM|11|0|506c61796572\n"
  c=f"C|11|0|2|20|2|{h('Elixir of Greater Intellect')}|{h('INV_Potion_10')}|{h('+10% cooldown recovery')}\n"
  self.lua.execute('nativeBuffTextures[2]="Interface/Icons/Temp.blp";nativeUpdates=0;cachedIcon="Temp";BuffButton0=CreateFrame("Button","BuffButton0",UIParent);this={sentinel=true};function BuffButton_Update() nativeUpdates=nativeUpdates+1;cachedIcon=GetPlayerBuffTexture(2) end')
  self.publish((base+c).encode().hex())
  self.assertEqual(self.lua.eval("cachedIcon"),"Interface"+chr(92)+"Icons"+chr(92)+"INV_Potion_10")
  self.lua.execute('assert(nativeUpdates==1 and this.sentinel)')
  self.publish((base+c.replace('|20|2|','|19|2|')).encode().hex())
  self.lua.execute('assert(nativeUpdates==1)')
  self.publish(base.encode().hex())
  self.lua.execute('assert(nativeUpdates==2 and this.sentinel)')
 def test_supply_discoveries_queue_once_and_materials_use_actual_sale_receipt(self):
  h=lambda s:s.encode().hex();base="1|1|11|1\nM|11|0|506c61796572\n"
  d=lambda event,name,gold:f"D|11|{event}|3|{gold}|{h(name)}|{h('INV_Misc_Gem_Crystal_02')}|{h('Automatically sold for gold')}\n"
  self.publish((base+d(1,'Arcane Crystal',33)).encode().hex())
  self.publish((base+d(2,'Black Lotus',34)).encode().hex())
  self.lua.execute('assert(objects.ASXSupplyDiscoveryToast.reward.text=="Arcane Crystal");now=now+4;AZS_RUN_UPDATED=now;update();assert(objects.ASXSupplyDiscoveryToast.reward.text=="Black Lotus" and objects.ASXSupplyDiscoveryToast.note.text=="Sold for 34 gold");now=now+4;AZS_RUN_UPDATED=now;update();assert(not objects.ASXSupplyDiscoveryToast.shown)')
 def test_ritual_visual_fixture_runs_zoom_and_opacity_observations_without_lua_errors(self):
  self.lua.execute('local original=CreateFrame;function CreateFrame(kind,name,parent,template)local f=original(kind,name,parent,template);if kind=="Frame" then auditFrame=f elseif kind=="Font" then auditLabel=f end;return f end;local methods=getmetatable(UIParent).__index;function methods:SetJustifyH()end;Minimap={GetZoom=function()return 0 end,SetZoom=function()end};function ASXRitualReadout()return 1,700,true,true end;function ASXEventVisualReadout()return rewardVisible,1 end;function ASXEventPinsReadout()return {{event=1}}end;rewardVisible=true')
  self.lua.execute((ROOT/'tools/dev/fixtures/asx-ritual-ui-audit.lua').read_text())
  self.lua.execute('for i=1,100 do now=now+.1;auditFrame.scripts.OnUpdate()end;rewardVisible=false;now=now+3;auditFrame.scripts.OnUpdate();assert(string.find(auditLabel.text,"Zoom pins=true %(4%) / Full opacity=true / Removed=true"))')
 def test_discovery_keeps_full_opacity_until_instant_removal(self):
  h=lambda s:s.encode().hex();base="1|1|11|1\nM|11|0|506c61796572\n"
  d=f"D|11|1|0|0|{h('Elixir of Wisdom')}|{h('INV_Potion_06')}|{h('+6% cooldown recovery')}\n"
  self.publish((base+d).encode().hex())
  self.lua.execute('local start=now;for i=1,179 do now=start+i/60;AZS_RUN_UPDATED=now;update();assert(objects.ASXSupplyDiscoveryToast.shown and objects.ASXSupplyDiscoveryToast:GetAlpha()==1)end;now=start+3.4;AZS_RUN_UPDATED=now;update();assert(not objects.ASXSupplyDiscoveryToast.shown)')
 def test_minimap_bridge_expiry_hides_native_and_custom_pins_together(self):
  self.lua.execute('Minimap=CreateFrame("Frame","Minimap",UIParent);Minimap:SetWidth(200);Minimap:SetHeight(200);function Minimap:GetZoom()return 0 end;AZS_CHESTS={{id=754974735,x=10,y=20}}')
  text="1|1|11|1\nM|11|0|506c61796572\nA|11|1000|2000|100000000|80000000|60000000|40000000|20000000|10000000|0|0\nP|11|1|1|6000|2000|0\n"
  self.publish(text.encode().hex())
  self.lua.execute('assert(table.getn(ASXUI_MinimapReadout())==2);now=now+5;update();assert(table.getn(ASXUI_MinimapReadout())==0 and ASX_GetMinimapData()==nil)')
 def test_embedded_presentation_without_asxui_creates_no_minimap_widgets(self):
  lua=LuaRuntime();lua.execute(STUB)
  lua.execute((ROOT/"native-ui/official-party.lua").read_text().replace("local asxNativeActive = 0 -- ASX_LOCAL_UI_GATE","local asxNativeActive = 1 -- ASX_LOCAL_UI_GATE",1))
  lua.execute('update();assert(type(ASX_GetMinimapData)=="function");assert(objects.ASXEventPin1==nil and objects.ASXUIMinimapController==nil);assert(objects.ASXOfficialPartyPresentation.events==nil)')
 def test_old_presentation_field_is_not_an_activation_alias(self):
  self.lua.globals().wire=self.wire()
  self.lua.execute('AZS_RUN={azcoop=wire};AZS_RUN_UPDATED=now;now=now+.3;update();assert(not objects.ASXOfficialParty1Spell1 or not objects.ASXOfficialParty1Spell1:IsVisible())')
 def test_collective_sayge_pause_is_exposed_without_hiding_inside_player(self):
  text="1|1|11|1\nM|11|0|506c61796572\nR|11|1|700|1|0\n"
  self.publish(text.encode().hex())
  self.lua.execute('local s=ASX_GetState(1);assert(s.data.sayge.inside and s.data.sayge.allInside==false and s.data.sayge.held==700);assert(string.find(s.texts.sayge.note,"Waiting for all players",1,true));assert(objects.ASXSaygeTrialProgress.shown and objects.ASXSaygeTrialProgress.rendered==700)')
  self.publish(text.replace('|700|1|0','|700|1|1').encode().hex())
  self.lua.execute('assert(ASX_GetState(1).data.sayge.allInside);assert(not string.find(ASX_GetState(1).texts.sayge.note,"Waiting",1,true))')
 def test_legacy_minimap_zoom_field_is_rejected(self):
  base="1|1|11|1\nM|11|0|506c61796572\n"
  atlas="A|11|1000|2000|0|100000000|80000000|60000000|40000000|20000000|10000000|0|0\n"
  self.publish((base+atlas).encode().hex())
  self.lua.execute('local _,_,owner=ASXOfficialPartyReadout();assert(owner==nil)')
 def test_redundant_legacy_minimap_radius_is_rejected(self):
  base="1|1|11|1\nM|11|0|506c61796572\n"
  atlas="A|11|1000|2000|10000|0|100000000|80000000|60000000|40000000|20000000|10000000|0|0\n"
  self.publish((base+atlas).encode().hex())
  self.lua.execute('local _,_,owner=ASXOfficialPartyReadout();assert(owner==nil)')
 def test_event_minimap_projection_tracks_current_zoom_pan_ping_and_resize(self):
  self.lua.execute('Minimap=CreateFrame("Frame","Minimap",UIParent);Minimap:SetWidth(200);Minimap:SetHeight(200);nativeCx=10;nativeCy=20;nativeQx=0;nativeQy=0;nativeZoom=0;nativeRadii={100,80,60,40,20,10};function Minimap:GetPingPosition()local r=nativeRadii[nativeZoom+1];return -(nativeQy-nativeCy)/(2*r),(nativeQx-nativeCx)/(2*r)end;function Minimap:GetZoom()return nativeZoom end')
  base="1|1|11|1\nM|11|0|506c61796572\n"
  atlas="A|11|1000|2000|100000000|80000000|60000000|40000000|20000000|10000000|0|0\n"
  pins="P|11|1|0|6000|2000|60\nP|11|1|1|1000|-18000|0\n"
  self.publish((base+atlas+pins).encode().hex())
  self.lua.execute('assert(objects.ASXEventPin1.parent==Minimap);local x=objects.ASXEventPin1.anchor;assert(x[4]==0 and math.abs(x[5]-50)<.01);assert(objects.ASXEventPin1.icon.portraitMasked);local p=ASXEventPinsReadout();assert(table.getn(p)==2 and not p[1].outside and p[2].outside);local n=allocations;now=now+.3;AZS_RUN_UPDATED=now;update();assert(allocations==n)')
  self.lua.execute('Minimap:SetWidth(300);Minimap:SetHeight(300);now=now+.3;AZS_RUN_UPDATED=now;update();local x=objects.ASXEventPin1.anchor;assert(math.abs(x[5]-75)<.01);Minimap:Hide();now=now+.3;AZS_RUN_UPDATED=now;update();assert(not objects.ASXEventPin1:IsVisible());Minimap:Show();now=now+.3;AZS_RUN_UPDATED=now;update()')
  self.lua.execute('nativeCy=30;now=now+.01;update();local x=objects.ASXEventPin1.anchor;assert(math.abs(x[4]-15)<.01 and math.abs(x[5]-75)<.01);nativeZoom=1;now=now+.01;update();x=objects.ASXEventPin1.anchor;assert(objects.ASXEventPin1.shown and math.abs(x[4]-18.75)<.01 and math.abs(x[5]-93.75)<.01)')
  self.lua.execute('nativeQx=200;nativeQy=100;objects.ASXUIMinimapController.scripts.OnEvent();now=now+.01;update();local x=objects.ASXEventPin1.anchor;assert(objects.ASXEventPin1.shown and math.abs(x[4]-18.75)<.01 and math.abs(x[5]-93.75)<.01)')
  atlas=atlas.rsplit('|0|0',1)[0]+'|20000|10000\n'
  self.publish((base+atlas+pins).encode().hex())
  self.lua.execute('local x=objects.ASXEventPin1.anchor;assert(objects.ASXEventPin1.shown and math.abs(x[4]-18.75)<.01 and math.abs(x[5]-93.75)<.01);nativeZoom=0;now=now+.01;update();x=objects.ASXEventPin1.anchor;assert(math.abs(x[4]-15)<.01 and math.abs(x[5]-75)<.01)')
  self.publish(base.encode().hex())
  self.lua.execute('assert(not objects.ASXEventPin1.shown and not objects.ASXEventPin2.shown)')
 def test_guest_local_ping_rebases_without_waiting_for_host_projection_metadata(self):
  self.lua.execute('Minimap=CreateFrame("Frame","Minimap",UIParent);Minimap:SetWidth(200);Minimap:SetHeight(200);pingX=.1;pingY=-.05;function Minimap:GetPingPosition()return pingX,pingY end;function Minimap:GetZoom()return 0 end')
  base="1|1|11|1\nM|11|0|506c61796572\nA|11|1000|2000|100000000|80000000|60000000|40000000|20000000|10000000|0|0\nP|11|1|1|6000|2000|0\n"
  self.publish(base.encode().hex())
  self.lua.execute('arg2=.15;arg3=.1;pingX=arg2;pingY=arg3;objects.ASXUIMinimapController.scripts.OnEvent();arg2=nil;arg3=nil;now=now+.01;update();local x=objects.ASXEventPin1.anchor;assert(math.abs(x[4])<.01 and math.abs(x[5]-50)<.01)')
  self.publish(base.encode().hex())
  self.lua.execute('local x=objects.ASXEventPin1.anchor;assert(math.abs(x[4])<.01 and math.abs(x[5]-50)<.01)')
 def test_ritual_progress_only_shows_inside_without_losing_partial_presence(self):
  text=bytes.fromhex(self.wire()).decode().replace("M|11|0|4d65\n","M|11|0|4d65\nR|11|1|250|1\n")
  self.publish(text.encode().hex())
  self.lua.execute('local event,held,inside,shown=ASXRitualReadout();assert(event==1 and held==250 and inside and shown);local r=ASXOfficialPartyReadout();before=r[1].items[1].data;count=allocations;assert(objects.ASXSaygeTrialProgress.bar.value==250)')
  self.publish(text.replace("|250|1","|500|1").encode().hex())
  self.lua.execute('local r=ASXOfficialPartyReadout();assert(r[1].items[1].data==before and allocations==count);local _,held,inside=ASXRitualReadout();assert(held==500 and inside);assert(not string.find(objects.ASXSaygeTrialProgress.note.text,"remaining"))')
  self.publish(text.replace("|250|1","|500|0").encode().hex())
  self.lua.execute('local _,held,inside,shown=ASXRitualReadout();assert(held==500 and not inside and not shown)')
  self.publish(text.replace("|250|1","|500|1").encode().hex())
  self.lua.execute('local _,held,inside,shown=ASXRitualReadout();assert(held==500 and inside and shown and allocations==count)')
  self.publish(text.replace("R|11|1|250|1\n","").encode().hex())
  self.lua.execute('assert(not objects.ASXSaygeTrialProgress.shown)')
  for bad in ("|2001|1","|250|2"):
   self.publish(text.replace("|250|1",bad).encode().hex());self.lua.execute('assert(ASXRitualReadout()==nil)')

 def test_fortune_tooltip_native_header_green_effect_and_effective_party_bonus(self):
  text="1|1|11|1\nM|11|0|506c61796572\nB|11|1|1|0|1200|1800\nM|22|1|416c696365\n"
  self.publish(text.encode().hex())
  self.lua.execute('''nativeAura="Interface/Icons/INV_Misc_Orb_02.blp";tip={};function GameTooltip:AddDoubleLine(l,r,lr,lg,lb,rr,rg,rb)table.insert(tip,{l,r,lr,lg,lb,rr,rg,rb})end;function GameTooltip:AddLine(t,r,g,b)table.insert(tip,{t,r,g,b})end;GameTooltip:SetPlayerBuff(32);assert(tip[1][1]=="Sayge's Dark Fortune of Wisdom" and tip[1][2]=="" and tip[1][3]==1);assert(tip[2][1]=="+12% Experience" and tip[2][2]==.12 and tip[2][3]==1);assert(tip[3][1]=="Your fortune" and tip[3][2]=="+12%");assert(tip[4][1]=="Active party bonus" and tip[4][2]=="+18%");assert(tip[6][1]=="Until replaced by another fortune.")''')
 def test_solo_wisdom_uses_native_buff_toast_and_personal_description(self):
  rows=bytes.fromhex(self.wire()).decode().splitlines()[:2];rows.insert(2,"B|11|1|1|0|1200|1200")
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
  _,_,_,_,b=self.lua.eval("ASXOfficialPartyReadout")();self.assertEqual(b['category'],4)
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
  self.lua.execute('enter("ASXOfficialParty1Item1");local r=ASXOfficialPartyReadout();oldIcon=r[1].items[1].data')
  self.publish(wire.replace("|2|47","|2|46").encode().hex())
  self.lua.execute('local r=ASXOfficialPartyReadout();assert(r[1].items[1].data==oldIcon and not GameTooltip.hidden);GameTooltip:SetPlayerBuff(32);assert(GameTooltip.detail=="Next stack expires in 46s of alive game time.")')
  self.publish(wire.replace("|2|47","|4|47").encode().hex())
  self.lua.execute('assert(GetPlayerBuffTimeLeft(32)==123)')
 def test_unused_slots_allocate_nothing_and_late_members_keep_retained_children(self):
  self.lua.execute('local r,b=ASXOfficialPartyReadout();assert(b==2 and r[3]==nil and r[4]==nil);local count=allocations;now=2;update();assert(allocations==count);names.party3="Later";connected.party3=true;buffs.party3={"new"};now=3;update();r,b=ASXOfficialPartyReadout();assert(b==3 and r[3].buffs==1);count=allocations;names.party3=nil;now=4;update();assert(not objects.ASXOfficialParty3Buff1:IsVisible());names.party3="Returned";now=5;update();assert(allocations==count and objects.ASXOfficialParty3Buff1.owner=="Returned")')
 def test_refreshes_remain_bounded_with_only_the_minimap_ui_listener(self):
  self.lua.execute('local observer=objects.ASXOfficialPartyPresentation;assert(observer.scripts.OnEvent==nil and observer.events==nil);assert(objects.ASXUIMinimapController.events.MINIMAP_PING);local reads=0;local original=UnitBuff;UnitBuff=function(u,n)reads=reads+1;return original(u,n)end;for i=1,600 do now=1.3+i/60;update()end;assert(reads>0 and reads<=40*64);buffs.party1={"replacement"};now=now+0.3;update();assert(objects.ASXOfficialParty1Buff1.texture=="replacement")')
 def test_tooltip_audit_bounds_unready_polling_and_retires_after_one_capture(self):
  self.publish(self.wire())
  self.lua.execute('local methods=getmetatable(UIParent).__index;function methods:GetWidth()return 800 end;function methods:GetHeight()return 600 end;function methods:SetJustifyH()end;function methods:SetJustifyV()end;function methods:SetFont()end;function methods:SetOwner()end;function methods:ClearLines()end;actionCalls=0;function methods:SetAction(slot)actionCalls=actionCalls+1 end;function HasAction(slot)return slot<=3 end;readouts=0;local read=ASXOfficialPartyReadout;function ASXOfficialPartyReadout()readouts=readouts+1;return read()end;AZS_RUN.t=0')
  self.lua.execute((ROOT/'tools/dev/fixtures/asx-tooltip-audit.lua').read_text())
  self.lua.execute('local f=objects.ASXTooltipAudit;for i=1,600 do now=2+i/60;f.scripts.OnUpdate()end;assert(readouts==0 and actionCalls==0 and objects.ASXTooltipOwnerTip==nil);AZS_RUN.t=10;local real=ASXOfficialPartyReadout;ASXOfficialPartyReadout=function()readouts=readouts+1;return {},0,nil,0 end;for i=1,600 do now=12+i/60;f.scripts.OnUpdate()end;assert(readouts>=4 and readouts<=5 and actionCalls==0);ASXOfficialPartyReadout=real;now=25;AZS_RUN_UPDATED=now;update();f.scripts.OnUpdate();assert(actionCalls==1 and f.scripts.OnUpdate==nil)')
 def test_native_launch_does_not_construct_asx_rows(self):
  lua=LuaRuntime();lua.execute(STUB);lua.execute("asxEnabled=false")
  lua.execute((ROOT/"native-ui/official-party.lua").read_text());lua.execute("assert(ASXOfficialPartyReadout==nil);assert(objects.ASXOfficialPartyPresentation==nil);assert(buildDefaults==nil);assert(objects.PartyMemberFrame1.scripts.OnClick==nativeClick)")
 def test_native_source_stays_inactive_even_with_asx_host_data(self):
  lua=LuaRuntime();lua.execute(STUB)
  lua.execute('cvars.asx_runtime_active="1";AZS_RUN={asx="host-data"};AZS_RUN_UPDATED=1')
  lua.execute((ROOT/"native-ui/official-party.lua").read_text());lua.execute('assert(ASXOfficialPartyReadout==nil);assert(objects.ASXOfficialPartyPresentation==nil);assert(objects.PartyMemberFrame1:IsVisible())')
 def test_unchanged_rows_do_not_enqueue_repeated_visibility_writes(self):
  self.lua.execute('local before=visibilityWrites;for i=1,16 do now=now+0.3;update()end;assert(visibilityWrites==before)')
 def test_retained_children_leave_native_parent_and_unit_scripts_untouched(self):
  self.lua.execute("assert(ASXOfficialPartyReadout); assert(objects.PartyMemberFrame1.scripts.OnClick==nativeClick);assert(objects.PartyMemberFrame1.scripts.OnEvent==nativeEvent); count=allocations; for i=1,40 do now=now+1;update()end;assert(allocations==count)")
 def test_buff_hover_uses_native_positive_unit_api_and_rejects_stale_owner(self):
  self.lua.execute('enter("ASXOfficialParty1Buff1");assert(GameTooltip.unit=="party1" and GameTooltip.index==1);names.party1="Other";enter("ASXOfficialParty1Buff1");assert(tooltipCalls==1);now=2;update();assert(GameTooltip.hidden);enter("ASXOfficialParty1Buff1");assert(tooltipCalls==2)')
 def test_disconnect_removal_hidden_parent_and_overflow(self):
  self.lua.execute('connected.party1=false;now=2;update();assert(not objects.ASXOfficialParty1Buff1:IsVisible()); connected.party1=true; buffs.party1={};now=3;update();assert(not objects.ASXOfficialParty1Buff1:IsVisible());for i=1,32 do buffs.party1[i]="buff"..i end;now=4;update();local rows,built=ASXOfficialPartyReadout();assert(rows[1].buffs==16 and built==2);objects.PartyMemberFrame1:Hide();now=5;update();assert(not objects.ASXOfficialParty1Buff1:IsVisible())')
 def wire(self, own="11", epoch="1", duplicate=False, item="+52% gold"):
  h=lambda s:s.encode().hex()
  rows=[f"1|{epoch}|{own}|5"]
  for slot,id,name in [(0,"11","Me"),(1,"22","Alice"),(2,"33","Alice" if duplicate else "Bob")]:
   rows.append(f"M|{id}|{slot}|{h(name)}")
   if id==own:continue
   rows.append(f"S|{id}|845|2|{h('Cleave')}|{h('Ability_Warrior_Cleave')}|")
   rows.append(f"I|{id}|1|1|{h('Seal')}|{h('INV_Jewelry_Talisman_10')}|{h(item)}")
  return ("\n".join(rows)+"\n").encode().hex()
 def publish(self, wire):
  self.lua.globals().wire=wire
  self.lua.execute('now=now+0.3;AZS_RUN={asx=wire};AZS_RUN_UPDATED=now;update()')
 def test_native_identity_binding_owner_item_tooltips_and_one_time_build_default(self):
  self.publish(self.wire())
  self.lua.execute('local r=ASXOfficialPartyReadout();assert(r[1].member.id=="22" and r[2].member.id=="33");enter("ASXOfficialParty1Item1");assert(GameTooltip.title=="Seal" and GameTooltip.detail=="+52% gold");assert(buildDefaults==1);now=2;AZS_RUN_UPDATED=now;update();assert(buildDefaults==1)')
  self.publish(self.wire(epoch="2",item="+18% experience"))
  self.lua.execute('enter("ASXOfficialParty1Item1");assert(GameTooltip.detail=="+18% experience")')
 def test_duplicate_names_wrong_owner_malformed_data_and_expiry_hide_builds(self):
  self.publish(self.wire(duplicate=True))
  self.lua.execute('assert(not objects.ASXOfficialParty1Spell1:IsVisible())')
  self.publish(self.wire())
  self.lua.execute('assert(objects.ASXOfficialParty1Spell1:IsVisible());now=8;update();assert(not objects.ASXOfficialParty1Spell1:IsVisible())')
  for wire in (self.wire(own="44"),"xx","1|1|11|5".encode().hex(),self.wire().replace("7c32327c31","7c32327c30",1)):
   self.publish(wire)
   self.lua.execute('assert(not objects.ASXOfficialParty1Spell1:IsVisible())')
 def test_heartbeat_keeps_hover_and_build_identity_while_data_updates_retire_it(self):
  wire=self.wire();self.publish(wire)
  self.lua.execute('enter("ASXOfficialParty1Item1");local r=ASXOfficialPartyReadout();before=r[1].items[1].data;assert(not GameTooltip.hidden)')
  decoded=bytes.fromhex(wire).decode().replace("1|1|11|5\n","1|1|11|6\n",1)
  self.publish(decoded.encode().hex())
  self.lua.execute('local r=ASXOfficialPartyReadout();assert(r[1].items[1].data==before);assert(not GameTooltip.hidden)')
  self.publish(self.wire(item="Changed"))
  self.lua.execute('assert(GameTooltip.hidden)')
 def test_compressed_native_spell_headers_body_and_malformed_envelopes(self):
  wire=bytes.fromhex(self.wire()).decode()
  h=lambda s:s.encode().hex()
  wire=wire.replace(f"S|22|845|2|{h('Cleave')}|{h('Ability_Warrior_Cleave')}|",f"S|22|845|2|{h('Cleave')}|{h('Ability_Warrior_Cleave')}|{h('104 Physical damage. Your bonuses: +58% damage')}|{h('Rank 2')}|{h('7 yd range')}|{h('Automatic')}|{h('2.1 sec cooldown')}")
  raw=wire.encode();packed=bytes([2])+len(raw).to_bytes(4,'little')+b''.join(b'\0'+raw[i:i+8]for i in range(0,len(raw),8))
  self.publish(packed.hex())
  self.lua.execute('enter("ASXOfficialParty1Spell1");assert(GameTooltip.detail=="104 Physical damage. Your bonuses: +58% damage");assert(GameTooltip.title=="Automatic" and GameTooltip.rank=="2.1 sec cooldown")')
  for bad in (packed[:-1],packed+b'x',bytes([2])+bytes([255])*4,bytes([2,3,0,0,0,1,0,0])):
   self.publish(bad.hex());self.lua.execute('assert(not objects.ASXOfficialParty1Spell1:IsVisible())')
 def test_other_viewer_uses_its_native_frame_order(self):
  self.lua.execute('names.party1="Me";names.party2="Bob"')
  self.publish(self.wire(own="22"))
  self.lua.execute('local r=ASXOfficialPartyReadout();assert(r[1].member.id=="11" and r[2].member.id=="33")')
if __name__=="__main__":unittest.main()
