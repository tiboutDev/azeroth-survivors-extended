-- Native health/aura values and retained ally children, presented beside the HUD.
local controller=CreateFrame("Frame","AZSXUIFramesController",UIParent)
local vitals,health,healthText,healthPercent,levelText,auraRoot,objectives
local hiddenRoots,auraOwned,timerOwned,allyOwned,allyPanels={},{},{},{},{}
local rows={};local elapsed=0;local auraRows=0
local function finite(n)return type(n)=="number"and n==n and math.abs(n)<1e12 end
local function getPoints(f)local out={};for i=1,f:GetNumPoints()do table.insert(out,{f:GetPoint(i)})end;return out end
local function match(f,p)
 if f:GetNumPoints()~=1 then return false end
 local a,b,c,d,e=f:GetPoint(1);return a==p[1]and b==p[2]and c==p[3]and math.abs((d or 0)-p[4])<.02 and math.abs((e or 0)-p[5])<.02
end
local function move(store,f,point,parent,relative,x,y)
 local entry=store[f]
 if not entry then entry={original=getPoints(f)};store[f]=entry end
 local p={point,parent,relative,x,y}
 if not match(f,p)then f:ClearAllPoints();f:SetPoint(unpack(p))end;entry.applied=p
end
local function restore(store)
 for f,e in pairs(store)do if match(f,e.applied)then f:ClearAllPoints();for _,p in ipairs(e.original)do f:SetPoint(unpack(p))end end end
 for f in pairs(store)do store[f]=nil end
end
local function rootVisibility(active)
 for _,name in ipairs({"PlayerFrame","TargetFrame","GameTimeFrame","MinimapBorderTop","MinimapZoneTextButton","MinimapToggleButton"})do
  local f=getglobal(name)
  if f then
   if active then
    if not hiddenRoots[f]then local mouse=true;if f.IsMouseEnabled then mouse=f:IsMouseEnabled()end;hiddenRoots[f]={alpha=f:GetAlpha(),mouse=mouse}end
    f:SetAlpha(0);if f.EnableMouse then f:EnableMouse(false)end
   elseif hiddenRoots[f]then
    local old=hiddenRoots[f];if f:GetAlpha()==0 then f:SetAlpha(old.alpha)end
    if f.EnableMouse and old.mouse~=nil then f:EnableMouse(old.mouse)end;hiddenRoots[f]=nil
   end
  end
 end
end
local function font(parent,size)
 local t=parent:CreateFontString(nil,"OVERLAY","GameFontNormalSmall");t:SetFont("Fonts\\FRIZQT__.TTF",size,"");return t
end
local function createVitals(a)
 vitals=CreateFrame("Frame","AZSXUIHudVitals",a.root);vitals:SetWidth(291);vitals:SetHeight(28)
 local outline=AZSXUI_CreatePill(vitals,"AZSXUIHudHealthOutline",vitals:GetWidth(),26,.36,.34,.29);outline:SetPoint("CENTER",vitals,"CENTER",0,0)
 local background=AZSXUI_CreatePill(outline,"AZSXUIHudHealthBackground",vitals:GetWidth()-4,22,.045,.045,.035);background:SetPoint("CENTER",outline,"CENTER",0,0)
 health=CreateFrame("ScrollFrame","AZSXUIHudHealth",background);health.fullWidth=background:GetWidth();health:SetWidth(health.fullWidth);health:SetHeight(22);health:SetPoint("LEFT",background,"LEFT",0,0)
 -- Clip a full-size capsule instead of shrinking its caps. Even one HP stays
 -- inside the rounded silhouette, and the filled width remains proportional.
 local fill=AZSXUI_CreatePill(health,"AZSXUIHudHealthFill",health.fullWidth,22,.12,.7,.22,"Interface\\TargetingFrame\\UI-StatusBar");health:SetScrollChild(fill)
 local label=CreateFrame("Frame","AZSXUIHudHealthLabel",vitals);label:SetAllPoints(background);label:SetFrameLevel(fill:GetFrameLevel()+1);label:EnableMouse(false)
 healthText=font(label,12);healthText:SetPoint("LEFT",background,"LEFT",10,0);healthText:SetTextColor(1,1,1)
 healthPercent=font(label,12);healthPercent:SetPoint("RIGHT",background,"RIGHT",-10,0);healthPercent:SetTextColor(1,1,1)
 levelText=MainMenuExpBar:CreateFontString("AZSXUIExperienceLevel","OVERLAY","GameFontNormalSmall")
 levelText:SetFont("Fonts\\FRIZQT__.TTF",12,"OUTLINE")
 levelText:SetPoint("CENTER",MainMenuExpBar,"CENTER",0,2);levelText:SetTextColor(1,1,1)
 auraRoot=CreateFrame("Frame","AZSXUIHudAuras",a.root);auraRoot:SetWidth(8*43);auraRoot:SetHeight(80);auraRoot:SetPoint("BOTTOM",vitals,"TOP",0,8)
end
local function auraKind(f,state)
 local index=f.buffIndex
 if not state or not state.available or not state.data or not finite(index)or not GetPlayerBuffTexture then return nil end
 local texture=GetPlayerBuffTexture(index)
 if type(texture)~="string"then return nil end
 texture=string.lower(string.gsub(texture,"/","\\"))
 local charge=state.data.consumables and state.data.consumables[7]
 local text=state.texts and state.texts.consumables and state.texts.consumables[7]
 if charge and charge.index==index and text and type(text.icon)=="string" and texture==string.lower(text.icon)then return "potion"end
 if state.data.fortune and string.find(texture,"inv_misc_orb_02",1,true)then return"fortune"end
 if state.data.crystals and state.data.crystals.index==index and string.find(texture,"inv_misc_gem_variety_01",1,true)then return"crystal"end
end
local medallions,badgeHidden={},{}
local crystalKeys={"red","blue","green","yellow"}
local function badgeVisibility(wanted)
 for f,old in pairs(badgeHidden)do if not wanted[f]then
  if f:GetAlpha()==0 then f:SetAlpha(old.alpha)end
  if old.mouse~=nil and type(f.EnableMouse)=="function"then f:EnableMouse(old.mouse)end
  badgeHidden[f]=nil
 end end
 for f in pairs(wanted)do
  if not badgeHidden[f]then badgeHidden[f]={alpha=f:GetAlpha(),mouse=type(f.IsMouseEnabled)=="function"and f:IsMouseEnabled()or nil}end
  f:SetAlpha(0);if type(f.EnableMouse)=="function"then f:EnableMouse(false)end
 end
end
local function badgeTooltip()
 local b=this;local state=AZSXUI_ModState()
 local key=b.kind=="crystal"and"crystals"or"fortune"
 if not state or not state.available or not state.data[key]or not state.texts or not state.texts[key]then GameTooltip:Hide();return end
 GameTooltip:SetOwner(b,"ANCHOR_TOP")
 if b.native and auraKind(b.native,state)==b.kind and GameTooltip.SetPlayerBuff then
  GameTooltip:SetPlayerBuff(b.native.buffIndex);GameTooltip:Show();return
 end
 local t=state.texts[key];GameTooltip:SetText(t.title,1,1,1)
 if t.reward then GameTooltip:AddLine(t.reward,.12,1,0,true)end
 if t.note then GameTooltip:AddLine(t.note,.8,.8,.8,true)end
 if key=="crystals"then
  for _,line in ipairs(t.details or{})do GameTooltip:AddLine(line,1,1,1,true)end
  if t.footer then GameTooltip:AddLine(t.footer,.8,.8,.8,true)end
 end
 GameTooltip:Show()
end
local function badgeClick()
 local b=this;local state=AZSXUI_ModState()
 if not state or not state.available or not b.native or auraKind(b.native,state)~=b.kind then return end
 local callback=b.native.GetScript and b.native:GetScript("OnClick")
 if callback then local previous=this;this=b.native;local ok,err=pcall(callback);this=previous;if not ok then error(err)end end
end
local fortuneIcons={
 [0]="Interface\\Icons\\INV_Misc_Ticket_Tarot_Stack_01",
 [1]="Interface\\Icons\\INV_Misc_Book_09",
 [2]="Interface\\Icons\\INV_Misc_Coin_01",
}
local function createMedallion(kind)
 local b=CreateFrame("Button",kind=="crystal"and"AZSXUICrystalMedallion"or"AZSXUIFortuneMedallion",vitals)
 b.kind=kind
 AZSXUI_CreateStatusIcon(b)
 if kind=="crystal"then
  b.total=font(b,12);b.total:SetFont("Fonts\\FRIZQT__.TTF",12,"OUTLINE");b.total:SetPoint("BOTTOMRIGHT",b.icon,"BOTTOMRIGHT",2,-1);b.total:SetTextColor(1,1,1)
 else
  -- A small native portrait overlaps the orb without taking tooltip ownership.
  b.nature=b:CreateTexture(b:GetName().."Nature","OVERLAY");b.nature:SetWidth(14);b.nature:SetHeight(14)
  b.nature:SetPoint("BOTTOMRIGHT",b.icon,"BOTTOMRIGHT",2,-2)
  b.natureRim=b:CreateTexture(nil,"OVERLAY");b.natureRim:SetTexture("Interface\\Minimap\\MiniMap-TrackingBorder");b.natureRim:SetTexCoord(0,.625,0,.625);b.natureRim:SetWidth(20);b.natureRim:SetHeight(20);b.natureRim:SetPoint("CENTER",b.nature,"CENTER",0,0)
 end
 if b.RegisterForClicks then b:RegisterForClicks("RightButtonUp")end;b:SetScript("OnClick",badgeClick)
 b:SetScript("OnEnter",badgeTooltip);b:SetScript("OnLeave",function()GameTooltip:Hide()end);b:Hide();medallions[kind]=b;return b
end
local function updateMedallions(state,crystal,fortune,potion)
 local wanted={};local active={}
 local data=state and state.available and state.data;local texts=state and state.available and state.texts
 for _,kind in ipairs({"crystal","fortune"})do
  local key=kind=="crystal"and"crystals"or"fortune";local value=data and data[key];local t=texts and texts[key]
  local valid=vitals and value and type(t)=="table"and type(t.title)=="string"and type(t.icon)=="string"
  if valid and kind=="crystal"then
   valid=finite(value.total)and value.total>=0
   for _,color in ipairs(crystalKeys)do valid=valid and finite(value[color])and value[color]>=0 end
  elseif valid then valid=finite(value.amount)and value.amount>=0 and finite(value.category)and value.category>=0 and value.category<=4 and math.floor(value.category)==value.category end
  local b=medallions[kind]
  if valid then
   b=b or createMedallion(kind);active[kind]=true;if kind=="crystal"then b.native=crystal else b.native=fortune end
   if b.iconPath~=t.icon then SetPortraitToTexture(b.icon:GetName(),t.icon);b.iconPath=t.icon end
   if kind=="crystal"then
    b.total:SetText(value.total>0 and string.format("%.0f",value.total) or "")
   else
    local nature=fortuneIcons[value.category]
    if nature then
     if b.naturePath~=nature then SetPortraitToTexture(b.nature:GetName(),nature);b.naturePath=nature end
     b.nature:Show();b.natureRim:Show()
    else b.nature:Hide();b.natureRim:Hide()end
   end
   if b.native then wanted[b.native]=true;local timer=getglobal(b.native:GetName().."Duration");if timer then wanted[timer]=true end end
   b:Show()
  elseif b then b.native=nil;b:Hide()end
 end
 if potion then wanted[potion]=true;local timer=getglobal(potion:GetName().."Duration");if timer then wanted[timer]=true end end
 badgeVisibility(wanted);return active.crystal,active.fortune
end

local potionFrame
local function potionBadge(state,native)
 local charge=state and state.available and state.data and state.data.consumables and state.data.consumables[7]
 local t=charge and state.texts and state.texts.consumables and state.texts.consumables[7]
 if not t then
  if potionFrame then potionFrame:Hide()end
  return nil
 end
 if not potionFrame then
  local b=CreateFrame("Button","AZSXUIPotionBadge",vitals)
  AZSXUI_CreateStatusIcon(b)
  b.kind="potion";if b.RegisterForClicks then b:RegisterForClicks("RightButtonUp")end;b:SetScript("OnClick",badgeClick)
  b:SetScript("OnEnter",function()
   GameTooltip:SetOwner(b,"ANCHOR_TOP");GameTooltip:SetText(b.text.name,1,1,1)
   GameTooltip:AddLine(b.text.description,1,1,1,true);GameTooltip:Show()
  end)
  b:SetScript("OnLeave",function()GameTooltip:Hide()end)
  potionFrame=b
 end
 potionFrame.text=t;potionFrame.native=native
 if potionFrame.iconPath~=t.icon then SetPortraitToTexture(potionFrame.icon:GetName(),t.icon);potionFrame.iconPath=t.icon end
 potionFrame:Show()
 return potionFrame
end
local function dockStatusRow(crystal,fortune,gadget,potion)
 local visible={}
 local width=0
 for _,frame in ipairs({crystal or false,fortune or false,gadget or false,potion or false})do
  if frame and frame:IsShown()then
   table.insert(visible,frame);width=width+frame:GetWidth()+8
  end
 end
 local x=-(width-8)/2
 for _,frame in ipairs(visible)do
  local point={"LEFT",vitals,"BOTTOM",x,-24}
   if not match(frame,point)then frame:ClearAllPoints();frame:SetPoint(unpack(point))end
  x=x+frame:GetWidth()+8
 end
end

function AZSXUI_DockAuras(state)
 local a=AZSXUI_GetHudAnchors and AZSXUI_GetHudAnchors()
 if not a or not auraRoot then
  restore(auraOwned);restore(timerOwned);updateMedallions(nil);potionBadge(nil,nil)
  if AZSXUI_HasModAPI() and AZSX_SetCooperativeRulesDocked then AZSX_SetCooperativeRulesDocked("AZSXUI",false)end
  return false
 end
 local helpful,harmful,order={},{},{}
 if AZSXUI_HasModAPI() and AZSX_SetCooperativeRulesDocked then
  local info=AZSX_SetCooperativeRulesDocked("AZSXUI",true)
  if info then info:ClearAllPoints();info:SetPoint("LEFT",a.counters,"RIGHT",5,0)end
 end
 local fortune,crystal,potion;local fortuneCount,crystalCount=0,0
 for i=0,31 do
  local f=getglobal("BuffButton"..i)
  if f and f:IsShown()then
   if f.buffFilter=="HARMFUL"then table.insert(harmful,f)
   elseif f.buffFilter=="HELPFUL"then
    table.insert(helpful,f);local kind=auraKind(f,state);if kind=="fortune"then fortune=f;fortuneCount=fortuneCount+1 elseif kind=="crystal"then crystal=f;crystalCount=crystalCount+1 elseif kind=="potion"then potion=f end
   end
  end
 end
 if fortuneCount>1 then fortune=nil end;if crystalCount>1 then crystal=nil end
 local crystalBadge,fortuneBadge=updateMedallions(state,crystal,fortune,potion)
 dockStatusRow(crystalBadge and medallions.crystal,fortuneBadge and medallions.fortune,getglobal("AZSXUIEngineeringBadge"),potionBadge(state,potion))
 if fortune and not fortuneBadge then table.insert(order,fortune)end;if crystal and not crystalBadge then table.insert(order,crystal)end
 for _,f in ipairs(helpful)do if f~=potion and not(f==fortune and fortuneBadge)and not(f==crystal and crystalBadge)and f~=fortune and f~=crystal then table.insert(order,f)end end
 -- Native button IDs, tooltips, timers, flashing and cancellation remain native.
 -- Native button size stays unchanged; eight columns keep duration labels readable.
 local base=math.ceil(table.getn(order)/8)*46
 local function dock(list,y)
  local count=table.getn(list)
  for i,f in ipairs(list)do
   local row=math.floor((i-1)/8);local column=math.mod(i-1,8)
   local columns=math.min(8,count-row*8)
   -- Center each occupied row, including its last partial row. Preserve native
   -- icon widths and duration space; priority still starts at the right.
   local width=(columns-1)*43+f:GetWidth()
   move(auraOwned,f,"BOTTOMRIGHT",auraRoot,"BOTTOM",width/2-column*43,y+row*46)
   local timer=getglobal(f:GetName().."Duration")
   if timer then move(timerOwned,timer,"BOTTOM",f,"TOP",0,2)end
  end
 end
 dock(order,0);dock(harmful,base)
 auraRows=math.ceil(table.getn(order)/8)+math.ceil(table.getn(harmful)/8)
 return true
end
local function updateAllies(active)
 if not active then restore(allyOwned);for _,p in pairs(allyPanels)do p:Hide()end;return end
 for index=1,4 do
  local parent=getglobal("PartyMemberFrame"..index);
  if parent then local above=index>1 and getglobal("PartyMemberFrame"..(index-1))or nil;move(allyOwned,parent,"TOPLEFT",above or UIParent,above and"BOTTOMLEFT"or"TOPLEFT",above and 0 or 20,above and -12 or -20)end
 local spells,items,buffs={},{},{}
  for i=1,16 do
   for _,entry in ipairs({{"Spell",spells},{"Item",items}})do local b=getglobal("AZSXOfficialParty"..index..entry[1]..i);if b and b:IsShown()and b.data then table.insert(entry[2],b)end end
  end
  for i=1,16 do local b=getglobal("AZSXOfficialParty"..index.."Buff"..i);if b and b:IsShown()then table.insert(buffs,b)end end
  local count=math.max(table.getn(spells),table.getn(items));local panel=allyPanels[index]
  if parent and parent:IsVisible()and(count>0 or table.getn(buffs)>0) then
   if not panel then panel=CreateFrame("Frame","AZSXUIAllyBuild"..index,parent);panel:SetPoint("LEFT",parent,"RIGHT",6,0);panel:SetHeight(49);AZSXUI_StylePanel(panel,true);panel.title=font(panel,10);panel.title:SetPoint("TOPLEFT",panel,"TOPLEFT",8,-8);panel.title:SetText("Build");panel.title:Hide();panel:SetFrameLevel(math.max(0,parent:GetFrameLevel()-1));allyPanels[index]=panel end
   panel:SetWidth(math.max(100,count*19+16));panel:Show()
   for row,list in ipairs({spells,items})do for i,b in ipairs(list)do move(allyOwned,b,"TOPLEFT",panel,"TOPLEFT",8+(i-1)*19,-8-(row-1)*18)end end
   for i,b in ipairs(buffs)do move(allyOwned,b,"TOPLEFT",panel,"TOPRIGHT",6+math.mod(i-1,8)*19,-8-math.floor((i-1)/8)*18)end
  elseif panel then panel:Hide()end
 end
end
local function updateObjectives(active,state,bosses)
 if not active then if objectives then objectives:Hide()end;return end
 local goals,guardianNames,guardianOrder,seen={},{},{},{};local titles,details={},{};local arriving={}
 local bossLines,bossIDs={},{}
 if type(bosses)=="table"then
  for _,boss in ipairs(bosses)do
   if type(boss)=="table"and boss.id~=nil and not bossIDs[boss.id]and type(boss.name)=="string"and boss.name~=""and finite(boss.hp)and boss.hp>0 then
    bossIDs[boss.id]=true;table.insert(bossLines,"Defeat "..boss.name)
   end
  end
 end
 if table.getn(bossLines)>0 then
  table.insert(goals,table.concat(bossLines,"; "));table.insert(titles,"Bosses");table.insert(details,table.concat(bossLines,"\n  - "))
 end
 -- Held lairs include distant guardians outside the local unit interest set.
 local lairs=AZS_GetLairs and AZS_GetLairs()
 local guardians
 if type(lairs)=="table"then
  guardians={};for i=1,math.min(128,table.getn(lairs))do local g=lairs[i]
   if type(g)=="table"then
    if g.state=="empty"and type(g.name)=="string"and finite(g.left)and g.left>=0 and g.left<=60 then table.insert(arriving,g)
    elseif g.active==true then table.insert(guardians,g)end
   end
  end
 else guardians=AZS_GetGuardians and AZS_GetGuardians()end
 if type(guardians)=="table"then
  for i=1,math.min(12,table.getn(guardians))do local g=guardians[i];if type(g)=="table"and type(g.name)=="string"and(g.hp==nil or finite(g.hp)and g.hp>0)then local id=(type(g.id)=="string"or finite(g.id))and g.id or i;if not seen[id]then seen[id]=true;if not guardianNames[g.name]then table.insert(guardianOrder,g.name);guardianNames[g.name]=0 end;guardianNames[g.name]=guardianNames[g.name]+1 end end end
 end
 local guardianLines,guardianDisplay={},{}
 for _,name in ipairs(guardianOrder)do local count=guardianNames[name];local line="Defeat "..name..(count>1 and(" x"..count)or"");table.insert(guardianLines,line);table.insert(guardianDisplay,line)end
 table.sort(arriving,function(a,b)return a.left<b.left end)
 for i=1,math.min(12,table.getn(arriving))do local g=arriving[i]
  local line=g.left>0 and("Arrives in "..math.ceil(g.left).." s")or"Arriving soon"
  table.insert(guardianLines,g.name..": "..(g.left<=30 and line or"Incoming"))
  table.insert(guardianDisplay,"Await "..g.name..(g.left<=30 and(" ("..(g.left>0 and (math.ceil(g.left).." s") or "soon")..")")or""))
 end
 if table.getn(guardianLines)>0 then table.insert(goals,table.concat(guardianLines,"; "));table.insert(titles,"Guardians");table.insert(details,table.concat(guardianDisplay,"\n  - "))end
 if state and state.available and state.data and state.data.sayge then table.insert(goals,"Complete Sayge's Trial");table.insert(titles,"Sayge's Trial");table.insert(details,"Complete the ritual")end
 if state and state.available and state.data and state.data.soloEvent and state.texts and state.texts.soloEvent then local e=state.texts.soloEvent;table.insert(goals,e.note);table.insert(titles,e.title);table.insert(details,e.note)end
 if table.getn(goals)==0 then table.insert(goals,"No active objectives");table.insert(titles,"No active objectives");table.insert(details,"")end
 if not objectives then
  objectives=CreateFrame("Frame","AZSXUIObjectives",UIParent);objectives:SetWidth(216)
  local band=CreateFrame("Frame",nil,objectives);band:SetWidth(216);band:SetHeight(40);band:SetPoint("TOP",objectives,"TOP",0,0);AZSXUI_StyleHeader(band)
  objectives.title=font(band,12);objectives.title:SetPoint("LEFT",band,"LEFT",10,0);objectives.title:SetText("Objectives");objectives.title:SetTextColor(1,.82,0)
 end
 -- Reanchor after minimap replacement/resize; hidden maps still provide geometry.
 local map=MinimapCluster or Minimap
 objectives:ClearAllPoints();if map then objectives:SetPoint("TOPRIGHT",map,"BOTTOMRIGHT",-8,-12)else objectives:SetPoint("TOPRIGHT",UIParent,"TOPRIGHT",-20,-180)end
 local offset=44
 for i=1,13 do
  if goals[i]then
   if not rows[i]then rows[i]=font(objectives,11);rows[i]:SetWidth(200);rows[i]:SetJustifyH("LEFT")end
   local text=(details[i]==""and"|cffaaaaaa"or"|cffffd100")..titles[i].."|r"..(details[i]~=""and("\n|cffffffff  - "..details[i].."|r")or"")
   local plain=string.gsub(string.gsub(text,"|c%x%x%x%x%x%x%x%x",""),"|r","");local height=0
   -- Native GetStringHeight can report already-clipped bounds. Reserve a
   -- conservative wrap budget as well, without dropping names or timers.
   local at=1
   while true do local stop=string.find(plain,"\n",at,true);local line=string.sub(plain,at,(stop or(string.len(plain)+1))-1);height=height+math.max(1,math.ceil(string.len(line)/28))*11;if not stop then break end;at=stop+1 end
   rows[i].summary=goals[i];rows[i]:SetHeight(0);rows[i]:SetText(text)
   local measured=rows[i].GetStringHeight and rows[i]:GetStringHeight();if finite(measured)then height=math.max(height,measured)end
   rows[i]:SetHeight(height);rows[i]:ClearAllPoints();rows[i]:SetPoint("TOPLEFT",objectives,"TOPLEFT",8,-offset);rows[i]:Show();offset=offset+height+3
  elseif rows[i]then rows[i]:Hide()end
 end
 objectives:SetHeight(offset);objectives:Show()
end
-- Native boss/awake-guardian data, positioned below our top ribbon and clock.
-- The native canvas exposes visibility but no Lua anchor. Fail open if the
-- complete bounded health snapshot is unavailable; never fabricate HP.
local encounters,encounterRows,encounterHeight=nil,{},0
local function updateEncounters(active,bosses)
 local guardians=active and AZS_GetGuardians and AZS_GetGuardians()
 -- The AZSX native creature definition/query uses this exact fixed name and
 -- elite rank. Read HP/max and spawn identity from the native special-unit API,
 -- not a nameplate or a name-based target-token guess.
 local goblins={}
 if active and AZSXUI_HasModAPI()and type(AZS_UNITS)=="table"and table.getn(AZS_UNITS)<=40 then for _,unit in ipairs(AZS_UNITS)do
  if type(unit)=="table"and unit.kind=="elite"and unit.name=="Venture Co. Treasurer"then
   table.insert(goblins,{id=unit.id,name=unit.name,hp=unit.hp,hpm=unit.hpm,
    debuffs=unit.id and AZS_GetDebuffs and AZS_GetDebuffs(unit.id)})
  end
 end end
 local entries,seen={},{}
 local valid=type(bosses)=="table"and type(guardians)=="table"
 if valid then
  for category,list in ipairs({bosses,guardians,goblins})do
   for _,b in ipairs(list)do
    if type(b)~="table"then valid=false;break end
    if category~=2 or b.awake==true then
     if type(b.name)~="string"or not finite(b.hp)or not finite(b.hpm)or b.hpm<=0 or b.id==nil then valid=false;break end
     if b.hp>0 and not seen[b.id]then seen[b.id]=true;table.insert(entries,b)end
    end
   end
  end
  if table.getn(entries)>8 then valid=false end
 end
 if not valid or not AZSXUI_ClaimNativeHud or not AZS_IsHudShown or not AZS_SetHudShown then
  encounterHeight=0;if encounters then encounters:Hide()end
  if AZSXUI_RestoreNativeHud then AZSXUI_RestoreNativeHud("bosses")end;return
 end
 local shown=AZSXUI_ClaimNativeHud("bosses")
 if not shown then encounterHeight=0;if encounters then encounters:Hide()end;return end
 if not encounters then
  encounters=CreateFrame("Frame","AZSXUIEncounters",UIParent);encounters:SetPoint("TOP",UIParent,"TOP",0,-132)
 end
 local barWidth=AZSXUIHudCounters:GetWidth()
 local gap=10
 -- Keep the centered grid clear of the minimap and objective panel on the right.
 local usable=UIParent:GetWidth()-2*210
 local columns=math.max(1,math.min(2,math.floor((usable+gap)/(barWidth+gap))))
 local total=table.getn(entries)
 local used=math.max(1,math.min(total,columns))
 encounters:SetWidth(used*barWidth+(used-1)*gap)
 local offset,lineHeight=0,0
 for i=1,math.max(table.getn(entries),table.getn(encounterRows))do
  local b=entries[i];local row=encounterRows[i]
  if b then
   if not row then
    row=CreateFrame("Frame","AZSXUIEncounter"..i,encounters);row:SetHeight(27)
    AZSXUI_StylePanel(row);row.bar=CreateFrame("StatusBar",nil,row);row.bar:SetPoint("TOPLEFT",row,"TOPLEFT",5,-5);row.bar:SetPoint("BOTTOMRIGHT",row,"BOTTOMRIGHT",-5,5)
    row.bar:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar");row.bar:SetStatusBarColor(.65,.08,.05);row.bar:SetMinMaxValues(0,1)
    local labels=CreateFrame("Frame",nil,row);labels:SetAllPoints(row);labels:SetFrameLevel(row.bar:GetFrameLevel()+2)
    row.title=font(labels,11);row.title:SetPoint("CENTER",row,"CENTER",0,0);row.title:SetJustifyH("CENTER");row.title:SetTextColor(1,1,1)
    row.debuffs={};row.more=font(row,10);row.more:SetTextColor(1,.82,0);row.more:Hide();encounterRows[i]=row
   end
   local column=math.mod(i-1,columns)
   local lineCount=math.min(columns,total-i+1+column)
   local x=(column-(lineCount-1)/2)*(barWidth+gap)
   row:SetWidth(barWidth);row.title:SetWidth(barWidth-20)
   row:ClearAllPoints();row:SetPoint("TOP",encounters,"TOP",x,-offset)
   row.bar:SetStatusBarColor(b.enraged and .9 or .65,b.enraged and .25 or .08,.05)
   row.title:SetText(b.name);row.bar:SetValue(math.max(0,math.min(1,b.hp/b.hpm)));row:Show()
   local count=0;local debuffs=type(b.debuffs)=="table"and b.debuffs or nil
   if debuffs then for j=1,math.min(64,table.getn(debuffs))do
    local d=debuffs[j]
    if type(d)=="table"and type(d.name)=="string"and d.name~=""and type(d.icon)=="string"and d.icon~=""and(d.permanent or not finite(d.remaining)or d.remaining>0)then
     count=count+1
     if count<=16 then
      local icon=row.debuffs[count]
      if not icon then
       local name="AZSXUIEncounter"..i.."Debuff"..count
       icon=CreateFrame("Button",name,row,"TargetDebuffButtonTemplate");icon:SetID(count)
       icon.icon=getglobal(name.."Icon")
       icon.border=getglobal(name.."Border")
       icon.stack=getglobal(name.."Count")
       local function iconKey(texture)
        if type(texture)~="string"then return nil end
        local path=string.gsub(string.lower(texture),string.char(92),"/")
        return (string.gsub(string.gsub(path,"^.*/",""),"%.blp$",""))
       end
       local function nativeIndex(button)
        if not button.effect or not AZS_TARGET or tostring(AZS_TARGET.id)~=tostring(button.unit)or not UnitDebuff then return nil end
        local wanted=iconKey(button.effect.icon)
        if not wanted then return nil end
        local found
        for index=1,16 do
         local ok,texture=pcall(UnitDebuff,"target",index)
         if not ok then return nil end
         if iconKey(texture)==wanted then
          if found then return nil end;found=index
         end
        end
        return found
       end
       local function showEffect(button)
        local effect=button.effect
        if not effect then GameTooltip:Hide();return end
        GameTooltip:SetOwner(button,"ANCHOR_RIGHT")
        local index=nativeIndex(button)
        if not (index and GameTooltip.SetUnitDebuff and pcall(GameTooltip.SetUnitDebuff,GameTooltip,"target",index)) then
         GameTooltip:SetText(effect.name or "Debuff")
         if finite(effect.stacks)and effect.stacks>1 then GameTooltip:AddLine("Stacks: "..math.floor(effect.stacks))end
         if finite(effect.remaining)then GameTooltip:AddLine(math.ceil(effect.remaining).." sec remaining")end
        end
        GameTooltip:Show()
       end
       icon.refreshTip=showEffect
       icon:SetScript("OnEnter",function()showEffect(icon)end)
       icon:SetScript("OnLeave",function()GameTooltip:Hide()end)
       row.debuffs[count]=icon
      end
      icon.effect=d;icon.unit=b.id
      if GameTooltip.IsOwned and GameTooltip:IsOwned(icon)then icon.refreshTip(icon)end
      local native=getglobal("TargetFrameDebuff"..count)
      local size=native and native:GetWidth()or(count<=6 and 21 or 17)
      icon:SetWidth(size);icon:SetHeight(size)
      if icon.border then
       local color=DebuffTypeColor and(DebuffTypeColor[d.dispelType or "none"]or DebuffTypeColor.none)
       if color then icon.border:SetVertexColor(color.r,color.g,color.b)end
       icon.border:Show()
      end
      icon:ClearAllPoints();icon:SetPoint("TOPLEFT",row,"BOTTOMLEFT",8+math.mod(count-1,12)*23,-3-math.floor((count-1)/12)*22)
      if icon.stack then icon.stack:SetText(finite(d.stacks)and d.stacks>1 and tostring(math.floor(d.stacks))or"")end
      icon.icon:SetTexture(string.find(d.icon,"^Interface[\\/]")and d.icon or("Interface\\Icons\\"..d.icon));icon:Show()
     end
    end
   end end
   for j=count+1,table.getn(row.debuffs)do row.debuffs[j].effect=nil;row.debuffs[j]:Hide()end
   row.debuffCount=count
   if count>16 then
    row.more:ClearAllPoints();row.more:SetPoint("TOPRIGHT",row,"BOTTOMRIGHT",-8,-25)
    row.more:SetText("+"..(count-16));row.more:Show()
   else row.more:Hide()end
   lineHeight=math.max(lineHeight,31+math.ceil(math.min(count,16)/12)*22)
   if column+1==lineCount then offset=offset+lineHeight;lineHeight=0 end
  elseif row then row:Hide();row.more:Hide();for _,icon in ipairs(row.debuffs)do icon.effect=nil;icon:Hide()end end
 end
 encounterHeight=offset;encounters:SetHeight(math.max(1,encounterHeight))
 if encounterHeight>0 then encounters:Show()else encounters:Hide()end
end
function AZSXUI_GetEncounterOffset()return encounterHeight>0 and 132+encounterHeight+12 or 137 end
local originalXPAlpha
local function updateExperience(level)
 local native=MainMenuBarExpText
 local progress=native and native:GetText()
 if finite(level)and level>=1 and level==math.floor(level)and progress and progress~="" and native:IsShown()then
  if originalXPAlpha==nil then originalXPAlpha=native:GetAlpha();native:SetAlpha(0)end
  levelText:SetText("|cffffd578Level "..level.."|r · "..progress)
  levelText:SetAlpha(originalXPAlpha);levelText:Show()
 else
  if originalXPAlpha~=nil then
   if native and native:GetAlpha()==0 then native:SetAlpha(originalXPAlpha)end
   originalXPAlpha=nil
  end
  if levelText then levelText:SetText("");levelText:Hide()end
 end
end
local function refreshFrames(a)
 if a then
  if not vitals then createVitals(a)end
  vitals:ClearAllPoints();vitals:SetPoint("BOTTOM",UIParent,"BOTTOM",0,216);vitals:Show()
  updateExperience(AZS_RUN and AZS_RUN.lvl)
  local hp=UnitHealth and UnitHealth("player");local max=UnitHealthMax and UnitHealthMax("player")
  if finite(hp)and finite(max)and max>0 then
   local fraction=math.max(0,math.min(hp,max))/max
   health.fraction=fraction;health:SetWidth(math.max(.01,health.fullWidth*fraction))
   if fraction>0 then health:Show()else health:Hide()end
   healthText:SetText(string.format("%.0f",hp));healthPercent:SetText(string.format("%.0f%%",fraction*100))
  else health.fraction=nil;health:Hide();healthText:SetText("—");healthPercent:SetText("—")end
  auraRoot:ClearAllPoints();auraRoot:SetPoint("BOTTOM",vitals,"TOP",0,8)
  rootVisibility(true)
 else
  if vitals then vitals:Hide();updateExperience(nil)end;restore(auraOwned);restore(timerOwned);updateMedallions(nil);if AZSXUI_HasModAPI() and AZSX_SetCooperativeRulesDocked then AZSX_SetCooperativeRulesDocked("AZSXUI",false)end;rootVisibility(false)
 end
 local state=AZSXUI_ModState()
 local bosses=a and AZS_GetBosses and AZS_GetBosses()
 updateObjectives(a~=nil,state,bosses);updateAllies(a~=nil);updateEncounters(a~=nil,bosses)
end
function AZSXUI_SyncHudFrames()
 local a=AZSXUI_GetHudAnchors and AZSXUI_GetHudAnchors()
 refreshFrames(a)
 AZSXUI_DockAuras(AZSXUI_ModState())
 elapsed=0
end
controller:SetScript("OnUpdate",function()
 elapsed=elapsed+(arg1 or 0)
 if elapsed>=.1 then elapsed=0;refreshFrames(AZSXUI_GetHudAnchors and AZSXUI_GetHudAnchors())end
end)

function AZSXUI_FramesReadout()
 local result={level=levelText and levelText:GetText(),health=healthText and healthText:GetText(),healthPercent=healthPercent and healthPercent:GetText(),vitalsShown=vitals and vitals:IsVisible(),playerAlpha=PlayerFrame and PlayerFrame:GetAlpha(),targetAlpha=TargetFrame and TargetFrame:GetAlpha(),objectives={},objectiveTexts={},allies=0,allyBuffsOutside=0}
 if objectives and objectives:IsVisible()then for _,row in ipairs(rows)do if row:IsShown()then table.insert(result.objectives,row.summary);table.insert(result.objectiveTexts,row:GetText())end end end
 for index,panel in pairs(allyPanels)do if panel:IsVisible()then
  result.allies=result.allies+1
  for i=1,16 do local b=getglobal("AZSXOfficialParty"..index.."Buff"..i);if b and b:IsVisible()then local _,parent,relative=b:GetPoint(1);if parent==panel and relative=="TOPRIGHT"then result.allyBuffsOutside=result.allyBuffsOutside+1 end end end
 end end
 result.auras={helpful=0,harmful=0,aboveHealth=true,durationsAbove=true}
 for frame in pairs(auraOwned)do if frame:IsVisible()and frame:GetAlpha()>0 then
  local key=frame.buffFilter=="HARMFUL"and"harmful"or"helpful";result.auras[key]=result.auras[key]+1
  if vitals and finite(frame:GetBottom())and finite(vitals:GetTop())and frame:GetBottom()<vitals:GetTop()then result.auras.aboveHealth=false end
  local timer=getglobal(frame:GetName().."Duration")
  if timer and timer:IsVisible()and finite(timer:GetBottom())and finite(frame:GetTop())and timer:GetBottom()<frame:GetTop()then result.auras.durationsAbove=false end
 end end
 result.encounters={visible=encounters and encounters:IsVisible()or false,height=encounterHeight,rows={}}
 for _,row in ipairs(encounterRows)do if row:IsVisible()then table.insert(result.encounters.rows,{name=row.title:GetText(),health=row.bar:GetValue(),debuffs=row.debuffCount})end end
 result.medallions={};for kind,b in pairs(medallions)do if b:IsVisible()then result.medallions[kind]={visible=true,value=b.value and b.value:GetText(),total=b.total and b.total:GetText(),native=b.native~=nil};if b.counts then local counts={};for key,label in pairs(b.counts)do counts[key]=label:GetText()end;result.medallions[kind].counts=counts end end end
 if vitals then result.vitalsTop=vitals:GetTop()end
 return result
end

function AZSXUI_GetDrawerAnchor()
 local a=AZSXUI_GetHudAnchors and AZSXUI_GetHudAnchors();if a then return a.base,0 end
end

-- Drawer layering is owned by Hud.lua: equipment < panel < native gryphons.
