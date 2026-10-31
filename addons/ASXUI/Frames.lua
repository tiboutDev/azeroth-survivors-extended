-- Native health/aura values and retained ally children, presented beside the HUD.
local controller=CreateFrame("Frame","ASXUIFramesController",UIParent)
local vitals,health,healthText,auraRoot,objectives
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
 vitals=CreateFrame("Frame","ASXUIHudVitals",a.root);vitals:SetWidth(math.floor(a.width*2/3));vitals:SetHeight(28)
 local outline=ASXUI_CreatePill(vitals,"ASXUIHudHealthOutline",vitals:GetWidth(),26,.36,.34,.29);outline:SetPoint("CENTER",vitals,"CENTER",0,0)
 local background=ASXUI_CreatePill(outline,"ASXUIHudHealthBackground",vitals:GetWidth()-4,22,.045,.045,.035);background:SetPoint("CENTER",outline,"CENTER",0,0)
 health=CreateFrame("ScrollFrame","ASXUIHudHealth",background);health.fullWidth=background:GetWidth();health:SetWidth(health.fullWidth);health:SetHeight(22);health:SetPoint("LEFT",background,"LEFT",0,0)
 -- Clip a full-size capsule instead of shrinking its caps. Even one HP stays
 -- inside the rounded silhouette, and the filled width remains proportional.
 local fill=ASXUI_CreatePill(health,"ASXUIHudHealthFill",health.fullWidth,22,.12,.7,.22,"Interface\\TargetingFrame\\UI-StatusBar");health:SetScrollChild(fill)
 local label=CreateFrame("Frame","ASXUIHudHealthLabel",vitals);label:SetAllPoints(background);label:SetFrameLevel(fill:GetFrameLevel()+1);label:EnableMouse(false)
 healthText=font(label,12);healthText:SetPoint("CENTER",background,"CENTER",0,0);healthText:SetTextColor(1,1,1)
 auraRoot=CreateFrame("Frame","ASXUIHudAuras",a.root);auraRoot:SetWidth(8*43);auraRoot:SetHeight(80);auraRoot:SetPoint("BOTTOM",vitals,"TOP",0,8)
end
local function auraKind(f,state)
 local index=f.buffIndex
 if not state or not state.available or not state.data or not finite(index)or not GetPlayerBuffTexture then return nil end
 local texture=GetPlayerBuffTexture(index)
 if type(texture)~="string"then return nil end
 texture=string.lower(string.gsub(texture,"/","\\"))
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
 local b=this;local state=ASX_GetState and ASX_GetState(1)
 local key=b.kind=="crystal"and"crystals"or"fortune"
 if not state or not state.available or not state.data[key]or not state.texts or not state.texts[key]then GameTooltip:Hide();return end
 GameTooltip:SetOwner(b,"ANCHOR_TOP")
 if b.native and auraKind(b.native,state)==b.kind and GameTooltip.SetPlayerBuff then
  GameTooltip:SetPlayerBuff(b.native.buffIndex);GameTooltip:Show();return
 end
 local t=state.texts[key];GameTooltip:SetText(t.title,1,1,1)
 if t.reward then GameTooltip:AddLine(t.reward,.12,1,0,true)end
 if t.note then GameTooltip:AddLine(t.note,.8,.8,.8,true)end
 GameTooltip:Show()
end
local function badgeClick()
 local b=this;local state=ASX_GetState and ASX_GetState(1)
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
 local b=CreateFrame("Button",kind=="crystal"and"ASXUICrystalMedallion"or"ASXUIFortuneMedallion",vitals)
 b.kind=kind;b:SetWidth(104);b:SetHeight(36)
 if kind=="crystal"then b:SetPoint("RIGHT",vitals,"LEFT",-8,0)else b:SetPoint("LEFT",vitals,"RIGHT",8,0)end
 b.icon=b:CreateTexture(b:GetName().."Icon","ARTWORK");b.icon:SetWidth(30);b.icon:SetHeight(30)
 b.icon:SetPoint(kind=="crystal"and"RIGHT"or"LEFT",b,kind=="crystal"and"RIGHT"or"LEFT",0,0)
 local rim=b:CreateTexture(nil,"OVERLAY");rim:SetTexture("Interface\\Minimap\\MiniMap-TrackingBorder");rim:SetTexCoord(0,.625,0,.625);rim:SetWidth(40);rim:SetHeight(40);rim:SetPoint("CENTER",b.icon,"CENTER",0,0)
 if kind=="crystal"then
  b.counts={};for i,key in ipairs(crystalKeys)do local label=font(b,11);label:SetPoint("TOPLEFT",b,"TOPLEFT",math.mod(i-1,2)*32,-4-math.floor((i-1)/2)*16);b.counts[key]=label end
 else
  -- A small native portrait overlaps the orb without taking tooltip ownership.
  b.nature=b:CreateTexture(b:GetName().."Nature","OVERLAY");b.nature:SetWidth(14);b.nature:SetHeight(14)
  b.nature:SetPoint("BOTTOMRIGHT",b.icon,"BOTTOMRIGHT",2,-2)
  b.natureRim=b:CreateTexture(nil,"OVERLAY");b.natureRim:SetTexture("Interface\\Minimap\\MiniMap-TrackingBorder");b.natureRim:SetTexCoord(0,.625,0,.625);b.natureRim:SetWidth(20);b.natureRim:SetHeight(20);b.natureRim:SetPoint("CENTER",b.nature,"CENTER",0,0)
  b.value=font(b,13);b.value:SetPoint("LEFT",b.icon,"RIGHT",6,-6);b.value:SetTextColor(.12,1,0)
  b.category=font(b,10);b.category:SetPoint("BOTTOMLEFT",b.value,"TOPLEFT",0,3);b.category:SetTextColor(1,.82,0)
 end
 if b.RegisterForClicks then b:RegisterForClicks("RightButtonUp")end;b:SetScript("OnClick",badgeClick)
 b:SetScript("OnEnter",badgeTooltip);b:SetScript("OnLeave",function()GameTooltip:Hide()end);b:Hide();medallions[kind]=b;return b
end
local function updateMedallions(state,crystal,fortune)
 local wanted={};local active={}
 local data=state and state.available and state.data;local texts=state and state.available and state.texts
 for _,kind in ipairs({"crystal","fortune"})do
  local key=kind=="crystal"and"crystals"or"fortune";local value=data and data[key];local t=texts and texts[key]
  local valid=vitals and value and type(t)=="table"and type(t.title)=="string"and type(t.icon)=="string"
  if valid and kind=="crystal"then
   valid=finite(value.total)and value.total>0
   for _,color in ipairs(crystalKeys)do valid=valid and finite(value[color])and value[color]>=0 end
  elseif valid then valid=finite(value.amount)and value.amount>=0 and finite(value.category)and value.category>=0 and value.category<=4 and math.floor(value.category)==value.category end
  local b=medallions[kind]
  if valid then
   b=b or createMedallion(kind);active[kind]=true;if kind=="crystal"then b.native=crystal else b.native=fortune end
   if b.iconPath~=t.icon then SetPortraitToTexture(b.icon:GetName(),t.icon);b.iconPath=t.icon end
   if kind=="crystal"then
    for _,color in ipairs(crystalKeys)do
     local label=b.counts[color];label:SetText(string.format("%.0f",value[color]))
     local pickup=t.pickups and t.pickups[color];local rgb=pickup and pickup.titleColor
     if value[color]>0 and rgb then label:SetTextColor(rgb[1],rgb[2],rgb[3])else label:SetTextColor(.65,.65,.65)end
    end
   else
    local nature=fortuneIcons[value.category]
    if nature then
     if b.naturePath~=nature then SetPortraitToTexture(b.nature:GetName(),nature);b.naturePath=nature end
     b.nature:Show();b.natureRim:Show()
    else b.nature:Hide();b.natureRim:Hide()end
    local amount=value.amount
    if(value.category==1 or value.category==2)and finite(value.best)then amount=math.max(amount,value.best)end
    b.value:SetText(string.format("+%d%%",math.floor(amount/100+.5)));b.category:SetText(({"Luck","XP","Gold","Pickup","Speed"})[value.category+1])
   end
   if b.native then wanted[b.native]=true;local timer=getglobal(b.native:GetName().."Duration");if timer then wanted[timer]=true end end
   b:Show()
  elseif b then b.native=nil;b:Hide()end
 end
 badgeVisibility(wanted);return active.crystal,active.fortune
end

function ASXUI_DockAuras(state)
 local a=ASXUI_GetHudAnchors and ASXUI_GetHudAnchors()
 if not a or not auraRoot then restore(auraOwned);restore(timerOwned);updateMedallions(nil);return false end
 local helpful,harmful,order={},{},{}
 local fortune,crystal;local fortuneCount,crystalCount=0,0
 for i=0,31 do
  local f=getglobal("BuffButton"..i)
  if f and f:IsShown()then
   if f.buffFilter=="HARMFUL"then table.insert(harmful,f)
   elseif f.buffFilter=="HELPFUL"then
    table.insert(helpful,f);local kind=auraKind(f,state);if kind=="fortune"then fortune=f;fortuneCount=fortuneCount+1 elseif kind=="crystal"then crystal=f;crystalCount=crystalCount+1 end
   end
  end
 end
 if fortuneCount>1 then fortune=nil end;if crystalCount>1 then crystal=nil end
 local crystalBadge,fortuneBadge=updateMedallions(state,crystal,fortune)
 if fortune and not fortuneBadge then table.insert(order,fortune)end;if crystal and not crystalBadge then table.insert(order,crystal)end
 for _,f in ipairs(helpful)do if not(f==fortune and fortuneBadge)and not(f==crystal and crystalBadge)and f~=fortune and f~=crystal then table.insert(order,f)end end
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
  if parent then local above=index>1 and getglobal("PartyMemberFrame"..(index-1))or(objectives and objectives:IsShown()and objectives or nil);move(allyOwned,parent,"TOPLEFT",above or UIParent,above and"BOTTOMLEFT"or"TOPLEFT",above and 0 or 20,above and -12 or -20)end
 local spells,items,buffs={},{},{}
  for i=1,16 do
   for _,entry in ipairs({{"Spell",spells},{"Item",items}})do local b=getglobal("ASXOfficialParty"..index..entry[1]..i);if b and b:IsShown()and b.data then table.insert(entry[2],b)end end
  end
  for i=1,16 do local b=getglobal("ASXOfficialParty"..index.."Buff"..i);if b and b:IsShown()then table.insert(buffs,b)end end
  local count=math.max(table.getn(spells),table.getn(items));local panel=allyPanels[index]
  if parent and parent:IsVisible()and(count>0 or table.getn(buffs)>0) then
   if not panel then panel=CreateFrame("Frame","ASXUIAllyBuild"..index,parent);panel:SetPoint("LEFT",parent,"RIGHT",6,0);panel:SetHeight(49);ASXUI_StylePanel(panel,true);panel.title=font(panel,10);panel.title:SetPoint("TOPLEFT",panel,"TOPLEFT",8,-8);panel.title:SetText("Build");panel.title:Hide();panel:SetFrameLevel(math.max(0,parent:GetFrameLevel()-1));allyPanels[index]=panel end
   panel:SetWidth(math.max(100,count*19+16));panel:Show()
   for row,list in ipairs({spells,items})do for i,b in ipairs(list)do move(allyOwned,b,"TOPLEFT",panel,"TOPLEFT",8+(i-1)*19,-8-(row-1)*18)end end
   for i,b in ipairs(buffs)do move(allyOwned,b,"TOPLEFT",panel,"TOPRIGHT",6+math.mod(i-1,8)*19,-8-math.floor((i-1)/8)*18)end
  elseif panel then panel:Hide()end
 end
end
local function updateObjectives(active,state)
 if not active then if objectives then objectives:Hide()end;return end
 local goals,guardianNames,guardianOrder,seen={},{},{},{}
 -- Held lairs include distant guardians outside the local unit interest set.
 local lairs=AZS_GetLairs and AZS_GetLairs()
 local guardians
 if type(lairs)=="table"then
  guardians={};for i=1,math.min(128,table.getn(lairs))do local g=lairs[i];if type(g)=="table"and g.active==true then table.insert(guardians,g)end end
 else guardians=AZS_GetGuardians and AZS_GetGuardians()end
 if type(guardians)=="table"then
  for i=1,math.min(12,table.getn(guardians))do local g=guardians[i];if type(g)=="table"and type(g.name)=="string"and(g.hp==nil or finite(g.hp)and g.hp>0)then local id=(type(g.id)=="string"or finite(g.id))and g.id or i;if not seen[id]then seen[id]=true;if not guardianNames[g.name]then table.insert(guardianOrder,g.name);guardianNames[g.name]=0 end;guardianNames[g.name]=guardianNames[g.name]+1 end end end
  for _,name in ipairs(guardianOrder)do local count=guardianNames[name];table.insert(goals,"Defeat "..name..(count>1 and(" x"..count)or""))end
 end
 if state and state.available and state.data and state.data.sayge then table.insert(goals,"Complete Sayge's Trial")end
 if table.getn(goals)==0 then if objectives then objectives:Hide()end;return end
 if not objectives then objectives=CreateFrame("Frame","ASXUIObjectives",UIParent);objectives:SetWidth(216);objectives:SetPoint("TOPLEFT",UIParent,"TOPLEFT",20,-20);ASXUI_StylePanel(objectives,true);objectives.title=font(objectives,12);objectives.title:SetPoint("TOPLEFT",objectives,"TOPLEFT",10,-11);objectives.title:SetText("Objectives");objectives.title:SetTextColor(1,.82,0)end
 for i=1,13 do
  if goals[i]then if not rows[i]then rows[i]=font(objectives,11);rows[i]:SetWidth(196);rows[i]:SetHeight(28);rows[i]:SetJustifyH("LEFT");rows[i]:SetPoint("TOPLEFT",objectives,"TOPLEFT",10,-31-(i-1)*28)end;rows[i]:SetText(goals[i]);rows[i]:Show()
  elseif rows[i]then rows[i]:Hide()end
 end
 objectives:SetHeight(37+table.getn(goals)*28);objectives:Show()
end
controller:SetScript("OnUpdate",function()
 local a=ASXUI_GetHudAnchors and ASXUI_GetHudAnchors()
 if a and vitals then vitals:ClearAllPoints();vitals:SetPoint("BOTTOM",a.base,"TOP",0,a.topOffset+9)end
 elapsed=elapsed+(arg1 or 0);if elapsed<.1 then return end;elapsed=0
 if a then
  if not vitals then createVitals(a)end
  vitals:ClearAllPoints();vitals:SetPoint("BOTTOM",a.base,"TOP",0,a.topOffset+9);vitals:Show()
  local hp=UnitHealth and UnitHealth("player");local max=UnitHealthMax and UnitHealthMax("player")
  if finite(hp)and finite(max)and max>0 then
   local fraction=math.max(0,math.min(hp,max))/max
   health.fraction=fraction;health:SetWidth(math.max(.01,health.fullWidth*fraction))
   if fraction>0 then health:Show()else health:Hide()end
   healthText:SetText(string.format("%.0f / %.0f",hp,max))
  else health.fraction=nil;health:Hide();healthText:SetText("—")end
  auraRoot:ClearAllPoints();auraRoot:SetPoint("BOTTOM",vitals,"TOP",0,8)
  rootVisibility(true)
 else
  if vitals then vitals:Hide()end;restore(auraOwned);restore(timerOwned);updateMedallions(nil);rootVisibility(false)
 end
 local state=ASX_GetState and ASX_GetState(1)
 updateObjectives(a~=nil,state);updateAllies(a~=nil)
end)

function ASXUI_FramesReadout()
 local result={health=healthText and healthText:GetText(),vitalsShown=vitals and vitals:IsVisible(),playerAlpha=PlayerFrame and PlayerFrame:GetAlpha(),targetAlpha=TargetFrame and TargetFrame:GetAlpha(),objectives={},allies=0,allyBuffsOutside=0}
 if objectives and objectives:IsVisible()then for _,row in ipairs(rows)do if row:IsShown()then table.insert(result.objectives,row:GetText())end end end
 for index,panel in pairs(allyPanels)do if panel:IsVisible()then
  result.allies=result.allies+1
  for i=1,16 do local b=getglobal("ASXOfficialParty"..index.."Buff"..i);if b and b:IsVisible()then local _,parent,relative=b:GetPoint(1);if parent==panel and relative=="TOPRIGHT"then result.allyBuffsOutside=result.allyBuffsOutside+1 end end end
 end end
 result.auras={helpful=0,harmful=0,aboveHealth=true,durationsAbove=true}
 for frame in pairs(auraOwned)do if frame:IsVisible()and frame:GetAlpha()>0 then
  local key=frame.buffFilter=="HARMFUL"and"harmful"or"helpful";result.auras[key]=result.auras[key]+1
  if vitals and finite(frame:GetBottom())and finite(vitals:GetTop())and frame:GetBottom()<vitals:GetTop()then result.auras.aboveHealth=false end
  local timer=getglobal(frame:GetName().."Duration")
  if timer and timer:IsVisible()and finite(timer:GetBottom())and finite(frame:GetTop())and timer:GetBottom()<frame:GetTop()then result.auras.durationsAbove=false end
 end end
 result.medallions={};for kind,b in pairs(medallions)do if b:IsVisible()then result.medallions[kind]={visible=true,value=b.value and b.value:GetText(),native=b.native~=nil};if b.counts then local counts={};for key,label in pairs(b.counts)do counts[key]=label:GetText()end;result.medallions[kind].counts=counts end end end
 if vitals then result.vitalsTop=vitals:GetTop()end
 return result
end

function ASXUI_GetDrawerAnchor()
 if vitals and vitals:IsVisible()then return vitals,12 end
end

-- The drawer intentionally covers auras during consultation. Keep native aura
-- strata/levels unchanged; only the owned clipped panel rises above them.
function ASXUI_GetDrawerLevel()
 local level=0
 for f in pairs(auraOwned)do if f:IsVisible()then level=math.max(level,f:GetFrameLevel())end end
 return level+2
end
