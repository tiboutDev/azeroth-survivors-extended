-- Explicit real-AZSX HUD trial observer; no synthetic provider or item text.
local f=CreateFrame("Frame","AZSXUIHudLiveAudit",UIParent)
local last,count=-100,0
local chat={}
f:RegisterEvent("CHAT_MSG_LOOT");f:RegisterEvent("CHAT_MSG_SYSTEM")
f:SetScript("OnEvent",function()if type(arg1)=="string"and string.len(arg1)<2048 then local record={event=event,text=arg1};local _,_,name=string.find(arg1,"%[([^%]]+)%]");if name and GetItemInfo then local values={pcall(GetItemInfo,name)};record.itemInfo=values end;table.insert(chat,record);if table.getn(chat)>50 then table.remove(chat,1)end end end)
if seterrorhandler then seterrorhandler(function()end)end
local function quote(s)return '"'..string.gsub(tostring(s),'[%z\001-\031\\"]',function(c)return string.format("\\u%04x",string.byte(c))end)..'"'end
local function json(v)
 if type(v)=="string"then return quote(v)end
 if type(v)=="boolean"or type(v)=="number"then return tostring(v)end
 if type(v)~="table"then return "null"end
 local out={};for k,item in pairs(v)do table.insert(out,quote(k)..":"..json(item))end;return "{"..table.concat(out,",").."}"
end
local function click(name)local b=getglobal(name);local call=b and b:GetScript("OnClick");if call then local previous=this;this=b;call();this=previous end end
f:SetScript("OnUpdate",function()
 local now=GetTime();if now-last<2 then return end;last=now;count=count+1
 local hud=AZSXUI_HudReadout and AZSXUI_HudReadout();if not f.started and hud and hud.enabled then f.started=now end
 local age=f.started and now-f.started or 0
 local stage=math.floor(age/10)
 if f.stage~=stage then
  f.stage=stage
  if stage==1 then ToggleCharacter("PaperDollFrame")
  elseif stage==2 then ToggleCharacter("PaperDollFrame")
  elseif stage==3 then ToggleCharacter("PaperDollFrame")
  elseif stage==4 and AZSXUI_ConfigureHud then AZSXUI_ConfigureHud("off")
  elseif stage==5 and AZSXUI_ConfigureHud then AZSXUI_ConfigureHud("on")end
 end
 local s=AZSX_GetState and AZSX_GetState(1)
 local xp=MainMenuExpBar
 local frames=AZSXUI_FramesReadout and AZSXUI_FramesReadout()
 local out={pins=AZSXUI_MinimapReadout and AZSXUI_MinimapReadout(),nativeLairs=AZS_GetLairs and AZS_GetLairs(),nativeHP=UnitHealth("player"),nativeLevel=AZS_RUN and AZS_RUN.lvl,nativeStats=AZS_STATS,economy=s and s.data and s.data.economy,hudLevels={xp=MainMenuExpBar and MainMenuExpBar:GetFrameLevel(),panel=AZSXUIHudDrawer and AZSXUIHudDrawer:GetFrameLevel(),art=MainMenuBarArtFrame and MainMenuBarArtFrame:GetFrameLevel()},nativeSpecialUnits=AZS_UNITS,statsChecked=AZSXUIHudStats and AZSXUIHudStats:GetChecked(),statsWidth=AZSXUIHudDrawer and AZSXUIHudDrawer:GetWidth(),loot=s and s.data and s.data.loot,lootTexts=s and s.texts and s.texts.loot,plateLayout=GetCVar("nameplateLayout"),platesOn=NAMEPLATES_ON,plates=AZSXUI_Readout and AZSXUI_Readout(),locale=GetLocale and GetLocale(),crystals=s and s.data and s.data.crystals,fortune=s and s.data and s.data.fortune,chat=chat,reason=s and s.reason,apiPhase=s and s.phase,raw=(s and not s.available)and AZS_RUN and AZS_RUN.azsx or nil,rawUpdated=AZS_RUN_UPDATED,frames=frames,sample=count,time=now,age=age,phase=AZS_GetPhase and AZS_GetPhase(),api=s and s.api,available=s and s.available,equipment=s and s.data and s.data.equipment,equipmentTexts=s and s.texts and s.texts.equipment,hud=AZSXUI_HudReadout and AZSXUI_HudReadout(),xp={width=xp and xp:GetWidth(),height=xp and xp:GetHeight(),text=MainMenuBarExpText and MainMenuBarExpText:GetText()},shown={}}
 for _,name in ipairs({"AZSXUIHudRoot","AZSXUIHudDrawer","AZSXUIHudEquipment","AZSXUIHudStats"})do local o=getglobal(name);out.shown[name]=o and o:IsVisible()or false end
 out.counterCenters={};if AZSXUIHudCounters then for _,v in ipairs(AZSXUIHudCounters.values or{})do table.insert(out.counterCenters,((v.button:GetTop()+v.button:GetBottom())-(AZSXUIHudCounters:GetTop()+AZSXUIHudCounters:GetBottom()))/2)end end
 out.popups={}
 for _,name in ipairs({"AZSXSupplyDiscoveryToast","AZSXDarkmoonBlessingToast","AZSXCrystalDiscoveryToast","AZSXUINativeRewardToast"})do
  local panel=getglobal(name)
  if panel and panel:IsVisible()then
   local bottom=panel:GetBottom();local noteBottom=panel.note:GetBottom()
   table.insert(out.popups,{title=panel.title:GetText(),reward=panel.reward:GetText(),note=panel.note:GetText(),name=name,width=panel:GetWidth(),height=panel:GetHeight(),icon=panel.icon:GetWidth(),iconTexture=panel.icon:GetTexture(),inside=type(bottom)=="number"and type(noteBottom)=="number"and noteBottom>=bottom+4,titleHeight=panel.title:GetHeight(),rewardHeight=panel.reward:GetHeight(),noteHeight=panel.note:GetHeight()})
  end
 end
 out.badgeTooltips={};out.channels={}
 for _,name in ipairs({"AZSXSaygeTrialProgress","AZSXUIResurrectionProgress"})do local bar=getglobal(name);if bar and bar:IsVisible()then out.channels[name]={title=bar.title:GetText(),note=bar.note:GetText(),seconds=bar.time:GetText(),value=bar.bar:GetValue()}end end
 if stage>=6 then
  for _,name in ipairs({"AZSXUICrystalMedallion","AZSXUIFortuneMedallion"})do local b=getglobal(name);if b and b:IsVisible()then local call=b:GetScript("OnEnter");if call then local previous=this;this=b;call();this=previous;local title=getglobal("GameTooltipTextLeft1");out.badgeTooltips[name]=title and title:GetText()end end end
 end
 if s and s.available and s.texts.equipment and s.texts.equipment[1]and stage>=5 then
  local chosen=1;for i=2,3 do local candidate=getglobal("AZSXUIHudEquipment"..i);if candidate and candidate.item and string.find(candidate.item.text or"","Enchantment:",1,true)then chosen=i;break end end;local b=getglobal("AZSXUIHudEquipment"..chosen);local enter=b and b:GetScript("OnEnter");if enter then local previous=this;this=b;enter();this=previous;local t=getglobal("GameTooltipTextLeft1");out.tooltip=t and t:GetText();out.itemTooltipLines={};for i=1,16 do local line=getglobal("GameTooltipTextLeft"..i);if line and line:GetText()and line:GetText()~=""then table.insert(out.itemTooltipLines,line:GetText())end end end
 end
 error("AZSXUI_HUD_LIVE "..json(out),0)
end)
