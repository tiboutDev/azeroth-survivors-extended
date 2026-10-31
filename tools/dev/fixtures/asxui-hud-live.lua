-- Explicit real-ASX HUD trial observer; no synthetic provider or item text.
local f=CreateFrame("Frame","ASXUIHudLiveAudit",UIParent)
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
 local hud=ASXUI_HudReadout and ASXUI_HudReadout();if not f.started and hud and hud.enabled then f.started=now end
 local age=f.started and now-f.started or 0
 local stage=math.floor(age/10)
 if f.stage~=stage then
  f.stage=stage
  if stage==1 then ToggleCharacter("PaperDollFrame")
  elseif stage==2 then ToggleCharacter("PaperDollFrame")
  elseif stage==3 then click("ASXUIHudStats")
  elseif stage==4 and ASXUI_ConfigureHud then ASXUI_ConfigureHud("off")
  elseif stage==5 and ASXUI_ConfigureHud then ASXUI_ConfigureHud("on");click("ASXUIHudStats")end
 end
 local s=ASX_GetState and ASX_GetState(1)
 local xp=MainMenuExpBar
 local frames=ASXUI_FramesReadout and ASXUI_FramesReadout()
 local out={pins=ASXUI_MinimapReadout and ASXUI_MinimapReadout(),nativeLairs=AZS_GetLairs and AZS_GetLairs(),nativeHP=UnitHealth("player"),hudLevels={xp=MainMenuExpBar and MainMenuExpBar:GetFrameLevel(),panel=ASXUIHudDrawer and ASXUIHudDrawer:GetFrameLevel(),art=MainMenuBarArtFrame and MainMenuBarArtFrame:GetFrameLevel()},nativeSpecialUnits=AZS_UNITS,statsChecked=ASXUIHudStats and ASXUIHudStats:GetChecked(),statsWidth=ASXUIHudDrawer and ASXUIHudDrawer:GetWidth(),loot=s and s.data and s.data.loot,lootTexts=s and s.texts and s.texts.loot,plateLayout=GetCVar("nameplateLayout"),platesOn=NAMEPLATES_ON,plates=ASXUI_Readout and ASXUI_Readout(),locale=GetLocale and GetLocale(),crystals=s and s.data and s.data.crystals,fortune=s and s.data and s.data.fortune,chat=chat,reason=s and s.reason,apiPhase=s and s.phase,raw=(s and not s.available)and AZS_RUN and AZS_RUN.asx or nil,rawUpdated=AZS_RUN_UPDATED,frames=frames,sample=count,time=now,age=age,phase=AZS_GetPhase and AZS_GetPhase(),api=s and s.api,available=s and s.available,equipment=s and s.data and s.data.equipment,equipmentTexts=s and s.texts and s.texts.equipment,hud=ASXUI_HudReadout and ASXUI_HudReadout(),xp={width=xp and xp:GetWidth(),height=xp and xp:GetHeight(),text=MainMenuBarExpText and MainMenuBarExpText:GetText()},shown={}}
 for _,name in ipairs({"ASXUIHudRoot","ASXUIHudDrawerMask","ASXUIHudEquipment","ASXUIHudStats"})do local o=getglobal(name);out.shown[name]=o and o:IsVisible()or false end
 out.counterCenters={};if ASXUIHudCounters then for _,v in ipairs(ASXUIHudCounters.values or{})do table.insert(out.counterCenters,((v.button:GetTop()+v.button:GetBottom())-(ASXUIHudCounters:GetTop()+ASXUIHudCounters:GetBottom()))/2)end end
 out.popups={}
 for _,name in ipairs({"ASXSupplyDiscoveryToast","ASXDarkmoonBlessingToast","ASXCrystalDiscoveryToast","ASXUINativeRewardToast"})do
  local panel=getglobal(name)
  if panel and panel:IsVisible()then
   local bottom=panel:GetBottom();local noteBottom=panel.note:GetBottom()
   table.insert(out.popups,{title=panel.title:GetText(),reward=panel.reward:GetText(),note=panel.note:GetText(),name=name,width=panel:GetWidth(),height=panel:GetHeight(),icon=panel.icon:GetWidth(),iconTexture=panel.icon:GetTexture(),inside=type(bottom)=="number"and type(noteBottom)=="number"and noteBottom>=bottom+4,titleHeight=panel.title:GetHeight(),rewardHeight=panel.reward:GetHeight(),noteHeight=panel.note:GetHeight()})
  end
 end
 out.badgeTooltips={};out.channels={}
 for _,name in ipairs({"ASXSaygeTrialProgress","ASXUIResurrectionProgress"})do local bar=getglobal(name);if bar and bar:IsVisible()then out.channels[name]={title=bar.title:GetText(),note=bar.note:GetText(),seconds=bar.time:GetText(),value=bar.bar:GetValue()}end end
 if stage>=6 then
  for _,name in ipairs({"ASXUICrystalMedallion","ASXUIFortuneMedallion"})do local b=getglobal(name);if b and b:IsVisible()then local call=b:GetScript("OnEnter");if call then local previous=this;this=b;call();this=previous;local title=getglobal("GameTooltipTextLeft1");out.badgeTooltips[name]=title and title:GetText()end end end
 end
 if s and s.available and s.texts.equipment and s.texts.equipment[1]and stage==5 then
  local b=getglobal("ASXUIHudEquipment1");local enter=b and b:GetScript("OnEnter");if enter then local previous=this;this=b;enter();this=previous;local t=getglobal("GameTooltipTextLeft1");out.tooltip=t and t:GetText()end
 end
 error("ASXUI_HUD_LIVE "..json(out),0)
end)
