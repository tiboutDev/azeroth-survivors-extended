-- Native HUD presentation. No gameplay commands, native pointers or outgoing messages.
local controller=CreateFrame("Frame","AZSXUIHudController",UIParent)
local root,base,counters,clock,drawer,equipment,rows,icons
local ready,enabled=false,false
local statsButton,sheetVisible
local sectionNames={"Offense","Specialization","Survival","Utility"}
local sectionKeys={{"dmg","cd","crit","cm","area","cnt","echo","bp"},{"auto","pet","dots","doth","big","exe","cc","dotd","shd"},{"hp","arm","reg","spd","lch","heal","thn","liv"},{"xp","gold","luck","pick"}}
local preferences={}
local pinned,lastSession,lastClock,equipmentHovered
local progress,elapsed=0,0
local nativeMore,replaceMore
local font="Fonts\\FRIZQT__.TTF"
local defs={
 combat={{"dmg","Damage","bonus"},{"cd","Haste","haste"},{"crit","Crit","decimalPercent"},{"cm","Crit dmg","percent"},{"area","Area","bonus"},{"cnt","Targets","targets"},{"echo","Echo","percent"},{"bp","Buff power","signedPercent"},{"dots","DoT stacks","integer"},{"doth","DoT haste","signedPercent"},{"big","Vs elites","signedPercent"},{"exe","Execute","signedPercent"},{"cc","Vs controlled","signedPercent"},{"dotd","Vs afflicted","signedPercent"},{"shd","Shielded","signedPercent"},{"auto","Melee","signedPercent"},{"pet","Pet damage","signedPercent"}},
 character={{"hp","Max health","integer"},{"arm","Reduction","percent"},{"reg","Regen","decimal"},{"spd","Move speed","bonus"},{"pick","Pickup","yards"},{"lch","Lifesteal","decimalPercent"},{"heal","Heal / kill","integer"},{"thn","Thorns","percent"},{"liv","Lives","integer"},{"xp","XP bonus","bonus"},{"gold","Gold bonus","bonus"},{"luck","Luck","signedPercent"}}
}
-- Keep complete terms for hover help while the XP-width panel uses short labels.
local shortLabels={dmg="DMG",bp="Buff pow",dots="DoT stk",big="Vs elite",cc="Vs CC",dotd="Vs DoT",pet="Pet dmg",hp="Max HP",arm="Reduction",spd="Speed",lch="Leech",heal="HP / kill",xp="XP",gold="Gold"}
local allStats={}
for _,kind in ipairs({"combat","character"})do for _,def in ipairs(defs[kind])do allStats[def[1]]=def end end
local function finite(v)return type(v)=="number"and v==v and math.abs(v)<1e12 end
local function text(parent,size)
 local t=parent:CreateFontString(nil,"OVERLAY","GameFontNormalSmall");t:SetFont(font,size,"");return t
end
local function db()
 if type(AZSXUIDB)~="table"then AZSXUIDB={}end
 if type(AZSXUIDB.hud)~="boolean"then AZSXUIDB.hud=true end
 return AZSXUIDB
end
local function format(def,stats)
 local v=stats and stats[def[1]];if not finite(v)then return nil end
 local kind=def[3]
 if kind=="bonus"then v=v-1 elseif kind=="haste"then v=1-v end
 local value
 if kind=="bonus"or kind=="haste"or kind=="signedPercent"then value=string.format("%+.0f%%",v*100)
 elseif kind=="decimalPercent"then value=string.format("%.1f%%",v*100)
 elseif kind=="percent"then value=string.format("%.0f%%",v*100)
 elseif kind=="decimal"then value=string.format("%.1f",v)
 elseif kind=="yards"then value=string.format("%.0f yd",v)
 elseif kind=="targets"then value=string.format("%+d",v)
 else value=string.format("%.0f",v)end
 local _,_,n=string.find(value,"^([%+%-]?[%d%.]+)")
 return value,tonumber(n)==0
end
local function restore()
 if replaceMore and AZS_UpdateMore==replaceMore then
  AZS_UpdateMore=nativeMore
  if PaperDollFrame and PaperDollFrame:IsVisible() and AZS_STATS then nativeMore()end
 end
 nativeMore,replaceMore=nil,nil
 for _,part in ipairs({"build","tally","clock"})do if preferences[part]~=nil then AZSXUI_RestoreNativeHud(part)end end
 preferences={}
end
local function replacePart(part,active)
 if not AZS_IsHudShown or not AZS_SetHudShown then return end
 if active then
  local value=AZSXUI_ClaimNativeHud(part)
  if preferences[part]==nil then preferences[part]=value end
 else AZSXUI_RestoreNativeHud(part);preferences[part]=nil end
end
local function close()
 pinned=nil;progress=0
 if drawer then drawer:Hide()end
end
local function statsView()
 if AZS_GetPhase and AZS_GetPhase()=="ready"then return {} end
 local stats={};for key,value in pairs(AZS_STATS or{})do stats[key]=value end
 local view=AZSXUI_ModState()
 local common=view and view.available and view.data and(view.data.sharedEconomy or view.data.economy)
 local count=GetNumPartyMembers and GetNumPartyMembers()
 if common or(AZSXUI_HasModAPI() and finite(count)and count>0 and count<5)then stats.xp=nil;stats.gold=nil end
 if common then for _,key in ipairs({"xp","gold"})do if finite(common[key])and common[key]>=0 then stats[key]=common[key]end end end
 return stats
end
local function statOrder(a,b)
 if a.column~=b.column then return a.column<b.column end
 if a.group~=b.group then return a.group<b.group end
 return a.order<b.order
end
local function refreshStats()
 local stats=statsView()
 if drawer and drawer:IsShown()then
  local changed=false
  for _,row in ipairs(rows)do
   local value,zero=format(row.def,stats)
   local group=not value and 3 or zero and 2 or 1
   if row.group~=group then row.group=group;changed=true end
   row.value:SetText(value or "—")
   local brightness=value and not zero and 1 or .62
   row.value:SetTextColor(brightness,brightness,brightness)
  end
  if changed then
   table.sort(rows,statOrder)
   local column,line=0,0
   for _,row in ipairs(rows)do
    if row.column~=column then column=row.column;line=0 end
    row:ClearAllPoints();row:SetPoint("TOPLEFT",drawer,"TOPLEFT",10+(column-1)*117,-32-line*18)
    line=line+1
   end
  end
 end
end
local function hoverTip(owner,title,body)
 GameTooltip:SetOwner(owner,"ANCHOR_TOP");GameTooltip:SetText(title,1,1,1)
 if body and body~=""then GameTooltip:AddLine(body,1,.82,0,1)end;GameTooltip:Show()
end
local hoveredKills
local function showKills(button,state)
 local ranks=state and state.available and state.data and state.data.killRanking
 if not ranks then hoverTip(button,"Kills");return end
 GameTooltip:SetOwner(button,"ANCHOR_NONE");GameTooltip:ClearLines()
 GameTooltip:ClearAllPoints();GameTooltip:SetPoint("TOPRIGHT",button,"BOTTOMRIGHT",-4,-5)
 GameTooltip:AddLine("Kills by player",1,.82,0)
 local place=0
 for i,row in ipairs(ranks)do
  if i==1 or row.kills~=ranks[i-1].kills then place=i end
  GameTooltip:AddDoubleLine(place..".  "..row.name,tostring(row.kills),1,1,1,1,.82,0)
 end
 GameTooltip:Show()
end
local function setEquipment(button,item)
 local old=button.item
 if equipmentHovered==button and(not item or not old or old.name~=item.name or old.text~=item.text or old.rank~=item.rank)then GameTooltip:Hide();equipmentHovered=nil end
 button.item=item
end
local function build()
 local bar,art,xp=MainMenuBar,MainMenuBarArtFrame,MainMenuExpBar
 if not bar or not art or not xp or xp:GetWidth()<=0 then return false end
 root=CreateFrame("Frame","AZSXUIHudRoot",bar);root:SetAllPoints(bar)
 root:SetFrameStrata(art:GetFrameStrata());root:SetFrameLevel(math.max(0,art:GetFrameLevel()-2))
 base=CreateFrame("Frame","AZSXUIHudBase",root);base:SetWidth(xp:GetWidth());base:SetHeight(1);base:SetPoint("BOTTOM",xp,"TOP",0,0)
 counters=CreateFrame("Frame","AZSXUIHudCounters",root);counters:SetWidth(300);counters:SetHeight(46);counters:SetPoint("TOP",UIParent,"TOP",0,-8)
 clock=CreateFrame("Frame","AZSXUIHudClock",counters);clock:SetWidth(300);clock:SetHeight(62);clock:SetPoint("TOP",counters,"BOTTOM",0,-8)
 clock.time=text(clock,32);clock.time:SetFont("Fonts\\MORPHEUS.TTF",32,"");clock.time:SetPoint("TOP",clock,"TOP",0,0);clock.time:SetTextColor(1,.82,0)
 clock.next=text(clock,14);clock.next:SetPoint("TOP",clock.time,"BOTTOM",0,-3);clock.next:SetTextColor(.8,.75,.65)
 counters:SetBackdrop({bgFile="Interface\\DialogFrame\\UI-DialogBox-Background",edgeFile="Interface\\Tooltips\\UI-Tooltip-Border",tile=true,tileSize=16,edgeSize=8,insets={left=3,right=3,top=3,bottom=1}});counters:SetBackdropColor(.06,.055,.045,1);counters:SetBackdropBorderColor(.5,.46,.39,1)
 counters.values={}
 for i,entry in ipairs({{"Kills","Ability_DualWield","kills"},{"Gold","INV_Misc_Coin_01","gold"},{"Cache cost","INV_Box_02","chest"}})do
  local b=CreateFrame("Button",entry[3]=="kills"and"AZSXUIKillCounter"or nil,counters);b:SetWidth(counters:GetWidth()/3);b:SetHeight(36);b:SetPoint("LEFT",counters,"LEFT",(i-1)*counters:GetWidth()/3,0)
  local icon=b:CreateTexture(nil,"ARTWORK");icon:SetTexture("Interface\\Icons\\"..entry[2]);icon:SetWidth(20);icon:SetHeight(20);icon:SetPoint("LEFT",b,"LEFT",13,0)
  local label=text(b,12);label:SetPoint("LEFT",icon,"RIGHT",6,0);label:SetText("—");label:SetTextColor(1,.82,0)
  if entry[3]=="gold"then
   counters.goldGlow=b:CreateTexture(nil,"BACKGROUND");counters.goldGlow:SetTexture("Interface\\Cooldown\\star4")
   counters.goldGlow:SetPoint("CENTER",label,"CENTER",0,0);counters.goldGlow:SetWidth(64);counters.goldGlow:SetHeight(32)
   counters.goldGlow:SetVertexColor(1,.72,.16,.5);counters.goldGlow:SetBlendMode("ADD");counters.goldGlow:Hide()
  end
  if entry[3]=="kills"then
   b:SetScript("OnEnter",function()hoveredKills=this;showKills(this,AZSXUI_ModState())end)
   b:SetScript("OnLeave",function()hoveredKills=nil;GameTooltip:Hide()end)
  else
   b.azsxCounterTitle=entry[1];b:SetScript("OnEnter",function()hoverTip(this,this.azsxCounterTitle)end);b:SetScript("OnLeave",function()GameTooltip:Hide()end)
  end
  table.insert(counters.values,{key=entry[3],label=label,button=b,index=i})
 end
 rows={}
 if AZSXUI_StyleTopCounters then AZSXUI_StyleTopCounters(counters)end
 equipment=CreateFrame("Frame","AZSXUIHudEquipment",root);equipment:SetWidth(224);equipment:SetHeight(40);equipment:SetFrameLevel(math.max(0,MainMenuBarArtFrame:GetFrameLevel()-3));icons={}
 for i=1,5 do
  local b=CreateFrame("Button","AZSXUIHudEquipment"..i,equipment);b:SetWidth(36);b:SetHeight(36);b:SetFrameLevel(equipment:GetFrameLevel());b:SetPoint("LEFT",equipment,"LEFT",(i-1)*44,0)
  b.icon=b:CreateTexture("AZSXUIHudEquipmentIcon"..i,"ARTWORK");b.icon:SetWidth(32);b.icon:SetHeight(32);b.icon:SetPoint("CENTER",b,"CENTER",0,0)
  local rim=b:CreateTexture(nil,"OVERLAY");rim:SetTexture("Interface\\Minimap\\MiniMap-TrackingBorder");rim:SetTexCoord(0,.625,0,.625);rim:SetWidth(42);rim:SetHeight(42);rim:SetPoint("CENTER",b,"CENTER",0,0)
  b:SetScript("OnEnter",function()
   if not this.item then return end;equipmentHovered=this
   GameTooltip:SetOwner(this,"ANCHOR_TOP");GameTooltip:ClearLines()
   GameTooltip:AddDoubleLine(this.item.name,"Rank "..this.item.rank,1,.82,0,.7,.7,.7)
   local body=(this.item.text or"").."\n";local start=1
   while start<=string.len(body)do
    local stop=string.find(body,"\n",start,true);if not stop then break end
    local line=string.sub(body,start,stop-1);start=stop+1
    local enchant=string.find(line,"^Enchantment:")or string.find(line,"^Active enchantment:")
    -- Native item text joins separate effects with comma-space. Preserve its
    -- wording and the existing blank line before the enchantment block.
    if enchant or line==""then
     GameTooltip:AddLine(line,enchant and .12 or 1,1,enchant and 0 or 1,true)
    else
     local effects=line..", ";local from=1
     while from<=string.len(effects)do
      local last=string.find(effects,", ",from,true);if not last then break end
      GameTooltip:AddLine(string.sub(effects,from,last-1),1,1,1,true);from=last+2
     end
    end
   end
   GameTooltip:Show()
  end);b:SetScript("OnLeave",function()GameTooltip:Hide();equipmentHovered=nil end);table.insert(icons,b)
 end
 root:Hide();return true
end
local function attachStats()
 if not PaperDollFrame then return end
 if not replaceMore and type(AZS_UpdateMore)=="function" then
  nativeMore=AZS_UpdateMore
  replaceMore=function() if AZSMoreStats then AZSMoreStats:Hide()end end
  AZS_UpdateMore=replaceMore;replaceMore()
 end
 if statsButton then return end
 statsButton=CreateFrame("Button","AZSXUIHudStats",PaperDollFrame,"UIPanelButtonTemplate")
 statsButton:SetWidth(68);statsButton:SetHeight(22);statsButton:SetPoint("TOPRIGHT",PaperDollFrame,"TOPRIGHT",-38,-43);statsButton:SetText("Stats")
 drawer=CreateFrame("Frame","AZSXUIHudDrawer",statsButton)
 drawer:SetWidth(485);drawer:SetHeight(205);drawer:SetPoint("TOPLEFT",PaperDollFrame,"TOPRIGHT",-28,-18)
 if AZSXUI_StylePanel then AZSXUI_StylePanel(drawer)end
 for column,keys in ipairs(sectionKeys)do
  local x=10+(column-1)*117
  local heading=text(drawer,12);heading:SetPoint("TOPLEFT",drawer,"TOPLEFT",x,-12);heading:SetText(sectionNames[column]);heading:SetTextColor(1,.82,0)
  for line,key in ipairs(keys)do
   local def=allStats[key];local row=CreateFrame("Frame",nil,drawer);row.def=def;row.order=line;row.column=column
   row:SetWidth(108);row:SetHeight(18);row:SetPoint("TOPLEFT",drawer,"TOPLEFT",x,-32-(line-1)*18)
   row.label=text(row,11);row.label:SetPoint("LEFT",row,"LEFT",0,0);row.label:SetText(shortLabels[key] or def[2])
   row.value=text(row,11);row.value:SetPoint("RIGHT",row,"RIGHT",0,0)
   row:EnableMouse(true);row:SetScript("OnEnter",function()hoverTip(row,def[2])end);row:SetScript("OnLeave",function()GameTooltip:Hide()end)
   table.insert(rows,row)
  end
 end
 drawer:Hide()
 local function opened()
  sheetVisible=true;drawer:Show();pinned="stats";progress=1;refreshStats()
 end
 statsButton:SetScript("OnHide",function()sheetVisible=false;close()end)
 statsButton:SetScript("OnShow",opened)
 statsButton:SetScript("OnClick",function()
  if drawer:IsShown()then close()else drawer:Show();pinned="stats";progress=1;refreshStats()end
 end)
end
local function activate()
 if enabled then return end
 if not root and not build()then return end
 for _,c in ipairs(counters.values)do c.button:SetWidth(counters:GetWidth()/3);c.button:ClearAllPoints();c.button:SetPoint("LEFT",counters,"LEFT",(c.index-1)*counters:GetWidth()/3,0)end
 enabled=true;root:Show()
end
local function deactivate()
 if hoveredKills then hoveredKills=nil;GameTooltip:Hide()end
 local wasEnabled=enabled;enabled=false;if wasEnabled then restore()end;close();progress=0
 if equipmentHovered then GameTooltip:Hide();equipmentHovered=nil end
 if root then root:Hide();if statsButton then statsButton:Hide()end;equipment:Hide();clock:Hide()end
end
local receivedEquipment=false
local lastPhase
local function runPhase(phase)
 return phase=="ready"or phase=="playing"or phase=="choosing"or phase=="paused"or phase=="dead"
end
local function refresh()
 local phase=AZS_GetPhase and AZS_GetPhase()
 if phase=="ready" or not enabled then receivedEquipment=false end
 local transition=lastPhase~=phase or not enabled
 lastPhase=phase
 if not ready or not db().hud or not runPhase(phase) or not MainMenuBar then
  local wasEnabled=enabled;deactivate()
  if wasEnabled and AZSXUI_SyncHudFrames then AZSXUI_SyncHudFrames()end
  return
 end
 activate();if not enabled then return end;attachStats();if statsButton then statsButton:Show()end
 local run=phase~="ready"and AZS_RUN or nil;local valid=type(run)=="table"
 for _,c in ipairs(counters.values)do local v=valid and run[c.key];if finite(v)and v>=0 then c.label:SetText(string.format("%.0f",v))else c.label:SetText("—");valid=false end end
 local affordable=valid and run.chest>0 and run.gold>=run.chest
 if affordable then counters.goldGlow:Show()else counters.goldGlow:Hide()end
 replacePart("tally",valid)
 -- The native clock is not a movable Lua frame. Replace its presentation only
 -- while native run time is available; pause/choice interpolation stays native.
 local time=AZS_GetRunTime and AZS_GetRunTime()
 local showClock=phase~="ready"and finite(time)and time>=0 and(AZS_IsHudShown and AZS_SetHudShown)and(preferences.clock==true or preferences.clock==nil and AZS_IsHudShown("clock"))
 if showClock then
  local function timestamp(t)t=math.max(0,math.floor(t));return string.format("%02d:%02d",math.floor(t/60),math.mod(t,60))end
  clock.time:SetText(timestamp(time))
  local upcoming=AZS_GetUpcomingBosses and AZS_GetUpcomingBosses();local nextTime=type(upcoming)=="table"and upcoming[1]and upcoming[1].left
  if not finite(nextTime)then nextTime=run and run.nb end
  clock.next:SetText(finite(nextTime)and nextTime>=0 and("next boss in "..timestamp(nextTime))or"")
  replacePart("clock",true);clock:Show()
 else replacePart("clock",false);clock:Hide()end
 local view=phase~="ready"and AZSXUI_ModState();local items=view and view.available and view.data and view.data.equipment;local texts=view and view.texts and view.texts.equipment
 if hoveredKills and (not GameTooltip.IsOwned or GameTooltip:IsOwned(hoveredKills))then showKills(hoveredKills,view)end
 local session=view and view.available and view.session and(view.session.epoch..":"..view.session.recipientGUID)or nil
 if session~=lastSession or(lastClock and finite(run and run.t)and run.t<lastClock-.5)then sheetVisible=false;close();lastSession=session end
 if statsButton then
  local visible=PaperDollFrame:IsVisible()
  if visible and not sheetVisible then statsButton:GetScript("OnShow")()
  elseif not visible and sheetVisible then sheetVisible=false;close()end
 end
 lastClock=run and run.t
 local available=type(items)=="table"and type(texts)=="table"and table.getn(items)<=5 and table.getn(items)==table.getn(texts)
 if available then receivedEquipment=true end
 local waiting=view and not view.available and not receivedEquipment and view.reason=="missing_or_invalid_payload"
 if waiting then items={};texts={}end
 if available or waiting then
  local display,counts={}, {amulet=0,ring=0,trinket=0};local typed=true
  for _,item in ipairs(items)do if not counts[item.category]then typed=false end end
  if typed then
   for i,item in ipairs(items)do
    local category=item.category;counts[category]=counts[category]+1
    local maximum=category=="amulet"and 1 or 2
    if counts[category]>maximum then available=false;break end
    local slot=category=="amulet"and 1 or(category=="ring"and(1+counts[category])or(3+counts[category]))
    display[slot]={name=texts[i].name,icon=texts[i].icon,text=texts[i].text,rank=item.rank}
   end
  else for i,item in ipairs(items)do local t=texts[i];display[i]={name=t.name,icon=t.icon,text=t.text,rank=item.rank}end end
  for i,b in ipairs(icons)do
   setEquipment(b,available and display[i]or nil)
   local empty=i==1 and"Neck"or(i<=3 and"Finger"or"Trinket")
   local path=b.item and b.item.icon or("Interface\\PaperDoll\\UI-PaperDoll-Slot-"..empty)
   if not string.find(path,"^Interface[\\/]")then path="Interface\\Icons\\"..path end
   SetPortraitToTexture(b.icon:GetName(),path)
  end
  if available or waiting then equipment:Show()else equipment:Hide()end
 else for _,b in ipairs(icons)do setEquipment(b,nil)end;equipment:Hide()end
 replacePart("build",available or waiting)
 refreshStats()
 if transition and AZSXUI_SyncHudFrames then AZSXUI_SyncHudFrames()end
end
controller:RegisterEvent("ADDON_LOADED")
controller:RegisterEvent("PLAYER_LOGIN")
controller:RegisterEvent("PLAYER_ENTERING_WORLD")
controller:SetScript("OnEvent",function()
 if arg1=="AZSXUI"then ready=true;db()
 elseif event~="PLAYER_ENTERING_WORLD"and event~="PLAYER_LOGIN"then return end
 if ready then refresh()end
end)
controller:SetScript("OnUpdate",function()
 local dt=arg1 or 0;if not finite(dt)or dt<0 then dt=0 end
 elapsed=elapsed+dt
 local phase=AZS_GetPhase and AZS_GetPhase()
 if phase~=lastPhase or elapsed>=.25 or (not enabled and ready and runPhase(phase) and db().hud and MainMenuBar)then
  elapsed=0;refresh()
 end
 if not enabled then return end
 equipment:ClearAllPoints();equipment:SetPoint("BOTTOM",base,"TOP",0,8)
end)
function AZSXUI_ConfigureHud(value)
 if value~="on"and value~="off"then return false end
 db().hud=value=="on";refresh();return true
end
function AZSXUI_HudReadout()
 local out={enabled=enabled,pinned=pinned,progress=progress,height=drawer and drawer:GetHeight() or 0,rows={},equipment={},affordable=enabled and counters and counters.goldGlow:IsShown()or false}
 if pinned then for _,row in ipairs(rows)do table.insert(out.rows,{key=row.def[1],label=row.label:GetText(),value=row.value:GetText()})end end
 for i,b in ipairs(icons or{})do if b.item then out.equipment[i]={name=b.item.name,icon=b.item.icon,text=b.item.text}end end
 return out
end

-- Internal retained-frame anchors for AZSXUI's persistent native-value views.
function AZSXUI_GetHudAnchors()
 if not enabled or not root then return nil end
 return {root=root,base=base,counters=counters,width=base:GetWidth(),topOffset=8+(equipment:IsShown()and 40 or 0)}
end
