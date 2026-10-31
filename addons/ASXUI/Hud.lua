-- Native HUD presentation. No gameplay commands, native pointers or outgoing messages.
local controller=CreateFrame("Frame","ASXUIHudController",UIParent)
local root,base,counters,clock,mask,drawer,equipment,buttons,rows,icons
local ready,enabled=false,false
local owned,original,hiddenArt,widths={},{},{},{}
local preferences={}
local selected,pinned,visibleDefs,lastSession,lastClock
local progress,target,elapsed,leaveAt=0,0,0,nil
local duration=.30
local font="Fonts\\FRIZQT__.TTF"
local defs={
 combat={{"dmg","Damage","bonus"},{"cd","Haste","haste"},{"crit","Crit","decimalPercent"},{"cm","Crit dmg","percent"},{"area","Area","bonus"},{"cnt","Targets","targets"},{"echo","Echo","percent"},{"bp","Buff power","signedPercent"},{"dots","DoT stacks","integer"},{"doth","DoT haste","signedPercent"},{"big","Vs elites","signedPercent"},{"exe","Execute","signedPercent"},{"cc","Vs controlled","signedPercent"},{"dotd","Vs afflicted","signedPercent"},{"shd","Shielded","signedPercent"},{"auto","Melee","signedPercent"},{"pet","Pet damage","signedPercent"}},
 character={{"hp","Max health","integer"},{"arm","Reduction","percent"},{"reg","Regen","decimal"},{"spd","Move speed","bonus"},{"pick","Pickup","yards"},{"lch","Lifesteal","decimalPercent"},{"heal","Heal / kill","integer"},{"thn","Thorns","percent"},{"liv","Lives","integer"},{"xp","XP bonus","bonus"},{"gold","Gold bonus","bonus"},{"luck","Luck","signedPercent"}}
}
local function finite(v)return type(v)=="number"and v==v and math.abs(v)<1e12 end
local function text(parent,size)
 local t=parent:CreateFontString(nil,"OVERLAY","GameFontNormalSmall");t:SetFont(font,size,"");return t
end
local function db()
 if type(ASXUIDB)~="table"then ASXUIDB={}end
 if type(ASXUIDB.hud)~="boolean"then ASXUIDB.hud=true end
 return ASXUIDB
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
local function points(widget)
 local out={};for i=1,widget:GetNumPoints()do table.insert(out,{widget:GetPoint(i)})end;return out
end
local function remember(widget)
 if not original[widget]then original[widget]={points=points(widget),alpha=widget:GetAlpha(),width=widget:GetWidth()}end
end
local function anchor(widget,point,parent,relative,x,y)
 remember(widget);widget:ClearAllPoints();widget:SetPoint(point,parent,relative,x,y);owned[widget]={point,parent,relative,x,y}
end
local function stillOwned(widget,expect)
 if widget:GetNumPoints()~=1 then return false end
 local a,b,c,d,e=widget:GetPoint(1)
 return a==expect[1]and b==expect[2]and c==expect[3]and math.abs((d or 0)-expect[4])<.01 and math.abs((e or 0)-expect[5])<.01
end
local function restore()
 for widget,expect in pairs(owned)do
  if stillOwned(widget,expect)then widget:ClearAllPoints();for _,a in ipairs(original[widget].points)do widget:SetPoint(unpack(a))end end
 end
 for widget,value in pairs(widths)do if math.abs(widget:GetWidth()-value)<.1 then widget:SetWidth(original[widget].width)end end
 for widget in pairs(hiddenArt)do if widget:GetAlpha()==0 then widget:SetAlpha(original[widget].alpha)end end
 owned,hiddenArt,original,widths={},{},{},{}
 for _,part in ipairs({"build","tally","clock"})do if preferences[part]~=nil then ASXUI_RestoreNativeHud(part)end end
 preferences={}
end
local function replacePart(part,active)
 if not AZS_IsHudShown or not AZS_SetHudShown then return end
 if active then
  local value=ASXUI_ClaimNativeHud(part)
  if preferences[part]==nil then preferences[part]=value end
 else ASXUI_RestoreNativeHud(part);preferences[part]=nil end
end
local function close()
 pinned=nil;target=0;leaveAt=nil
end
local function layoutStats(kind)
 selected=kind;visibleDefs={}
 for _,def in ipairs(defs[kind])do local value,zero=format(def,AZS_STATS);if value and not zero then table.insert(visibleDefs,def)end end
 local n=table.getn(visibleDefs);local h=18+math.ceil(n/3)*23
 drawer:SetHeight(h);mask:SetHeight(h);mask.content:SetHeight(h*2)
 drawer:ClearAllPoints();drawer:SetPoint("TOPLEFT",mask.content,"TOPLEFT",0,-h)
 drawer.heading:SetText(kind=="combat"and"Combat"or"Character")
 for i,row in ipairs(rows)do
  local def=visibleDefs[i]
  if def then
   row:ClearAllPoints();row:SetPoint("TOPLEFT",drawer,"TOPLEFT",12+math.mod(i-1,3)*((drawer:GetWidth()-24)/3),-12-math.floor((i-1)/3)*23)
   row:SetWidth((drawer:GetWidth()-24)/3-9);row.label:SetText(def[2]);row:Show()
  else row:Hide()end
 end
 drawer.height=h
end
local function open(kind,pin)
 if not enabled then return end
 if selected~=kind or target==0 then layoutStats(kind)end
 if pin then if pinned==kind then pinned=nil else pinned=kind end;target=pinned and 1 or 0 else target=1 end
 leaveAt=nil
end
local function hoverTip(owner,title,body)
 GameTooltip:SetOwner(owner,"ANCHOR_TOP");GameTooltip:SetText(title,1,1,1)
 if body and body~=""then GameTooltip:AddLine(body,1,.82,0,1)end;GameTooltip:Show()
end
local function build()
 local bar,art,xp=MainMenuBar,MainMenuBarArtFrame,MainMenuExpBar
 if not bar or not art or not xp or not AZS_MAIN_BAR_SLICES or not AZS_MAIN_BAR_ROWS or not AZS_MainBar_Slice then return false end
 if math.abs(bar:GetWidth()-397)>.1 then return false end
 root=CreateFrame("Frame","ASXUIHudRoot",UIParent);root:SetWidth(485);root:SetHeight(66);root:SetPoint("BOTTOM",bar,"BOTTOM",0,0)
 root:SetFrameStrata(art:GetFrameStrata());root:SetFrameLevel(math.max(0,art:GetFrameLevel()-2))
 for i,slice in ipairs({{0,0,256,0},{2,18,39,256},{1,1,45,277},{2,28,66,321},{2,66,28,359},{0,1,45,397},{0,1,45,441}})do
  local t=root:CreateTexture(nil,"ARTWORK");t:SetHeight(43)
  AZS_MainBar_Slice(t,"Interface\\MainMenuBar\\UI-MainMenuBar-Dwarf",AZS_MAIN_BAR_ROWS,slice,root:GetName(),"BOTTOMLEFT",0)
 end
 base=CreateFrame("Frame","ASXUIHudBase",root);base:SetWidth(481);base:SetHeight(1);base:SetPoint("BOTTOM",xp,"TOP",0,0)
 counters=CreateFrame("Frame","ASXUIHudCounters",root);counters:SetWidth(360);counters:SetHeight(46);counters:SetPoint("TOP",UIParent,"TOP",0,0)
 clock=CreateFrame("Frame","ASXUIHudClock",counters);clock:SetWidth(360);clock:SetHeight(62);clock:SetPoint("TOP",counters,"BOTTOM",0,-8)
 clock.time=text(clock,32);clock.time:SetFont("Fonts\\MORPHEUS.TTF",32,"");clock.time:SetPoint("TOP",clock,"TOP",0,0);clock.time:SetTextColor(1,.82,0)
 clock.next=text(clock,14);clock.next:SetPoint("TOP",clock.time,"BOTTOM",0,-3);clock.next:SetTextColor(.8,.75,.65)
 counters:SetBackdrop({bgFile="Interface\\DialogFrame\\UI-DialogBox-Background",edgeFile="Interface\\Tooltips\\UI-Tooltip-Border",tile=true,tileSize=16,edgeSize=8,insets={left=3,right=3,top=3,bottom=1}});counters:SetBackdropColor(.06,.055,.045,1);counters:SetBackdropBorderColor(.5,.46,.39,1)
 counters.values={}
 for i,entry in ipairs({{"Kills","Ability_DualWield","kills"},{"Gold","INV_Misc_Coin_01","gold"},{"Cache cost","INV_Box_02","chest"}})do
  local b=CreateFrame("Button",nil,counters);b:SetWidth(counters:GetWidth()/3);b:SetHeight(36);b:SetPoint("TOPLEFT",counters,"TOPLEFT",(i-1)*counters:GetWidth()/3,0)
  local icon=b:CreateTexture(nil,"ARTWORK");icon:SetTexture("Interface\\Icons\\"..entry[2]);icon:SetWidth(20);icon:SetHeight(20);icon:SetPoint("LEFT",b,"LEFT",13,0)
  local label=text(b,12);label:SetPoint("LEFT",icon,"RIGHT",6,0);label:SetText("—");label:SetTextColor(1,.82,0)
  b.asxCounterTitle=entry[1];b:SetScript("OnEnter",function()hoverTip(this,this.asxCounterTitle)end);b:SetScript("OnLeave",function()GameTooltip:Hide()end)
  table.insert(counters.values,{key=entry[3],label=label,button=b,index=i})
 end
 mask=CreateFrame("ScrollFrame","ASXUIHudDrawerMask",root);mask:SetWidth(xp:GetWidth());mask:SetHeight(200);mask:SetPoint("BOTTOM",counters,"TOP",0,0)
 mask.content=CreateFrame("Frame",nil,mask);mask.content:SetWidth(mask:GetWidth());mask.content:SetHeight(400);mask:SetScrollChild(mask.content)
 drawer=CreateFrame("Frame","ASXUIHudDrawer",mask.content);drawer:SetWidth(mask:GetWidth());drawer:SetHeight(200);drawer.height=200
 drawer:SetBackdrop({bgFile="Interface\\Tooltips\\UI-Tooltip-Background",edgeFile="Interface\\Tooltips\\UI-Tooltip-Border",tile=true,tileSize=16,edgeSize=12,insets={left=3,right=3,top=3,bottom=0}});drawer:SetBackdropColor(.035,.03,.025,1);drawer:SetBackdropBorderColor(.5,.46,.39,1)
 counters:SetFrameLevel(root:GetFrameLevel());mask:SetFrameLevel(root:GetFrameLevel());mask.content:SetFrameLevel(root:GetFrameLevel());drawer:SetFrameLevel(root:GetFrameLevel())
 drawer.heading=text(drawer,13);drawer.heading:SetPoint("TOPLEFT",drawer,"TOPLEFT",12,-10);drawer.heading:SetTextColor(1,.82,0);drawer.heading:Hide();if ASXUI_StylePanel then ASXUI_StylePanel(drawer)end;if ASXUI_StyleTopCounters then ASXUI_StyleTopCounters(counters)elseif ASXUI_StylePanel then ASXUI_StylePanel(counters)end
 drawer:EnableMouse(true);drawer:SetScript("OnEnter",function()leaveAt=nil end);drawer:SetScript("OnLeave",function()if not pinned then leaveAt=GetTime()+.2 end end)
 rows={};for i=1,17 do local r=CreateFrame("Frame",nil,drawer);r:SetHeight(18);r.label=text(r,12);r.label:SetPoint("LEFT",r,"LEFT",0,0);r.label:SetTextColor(.75,.72,.65);r.value=text(r,12);r.value:SetPoint("RIGHT",r,"RIGHT",0,0);r.value:SetTextColor(1,1,1);r:Hide();table.insert(rows,r)end
 equipment=CreateFrame("Frame","ASXUIHudEquipment",root);equipment:SetWidth(224);equipment:SetHeight(40);icons={}
 for i=1,5 do
  local b=CreateFrame("Button","ASXUIHudEquipment"..i,equipment);b:SetWidth(36);b:SetHeight(36);b:SetPoint("LEFT",equipment,"LEFT",(i-1)*44,0)
  b.icon=b:CreateTexture("ASXUIHudEquipmentIcon"..i,"ARTWORK");b.icon:SetWidth(32);b.icon:SetHeight(32);b.icon:SetPoint("CENTER",b,"CENTER",0,0)
  local rim=b:CreateTexture(nil,"OVERLAY");rim:SetTexture("Interface\\Minimap\\MiniMap-TrackingBorder");rim:SetTexCoord(0,.625,0,.625);rim:SetWidth(42);rim:SetHeight(42);rim:SetPoint("CENTER",b,"CENTER",0,0)
  b:SetScript("OnEnter",function()if this.item then hoverTip(this,this.item.name,this.item.text)end end);b:SetScript("OnLeave",function()GameTooltip:Hide()end);table.insert(icons,b)
 end
 buttons={};for i,kind in ipairs({"combat","character"})do
  local b=CreateFrame("Button","ASXUIHud"..(i==1 and"Combat"or"Character"),root);b:SetWidth(36);b:SetHeight(36);b:SetPoint("BOTTOMLEFT",root,"BOTTOMLEFT",403+(i-1)*44,4)
  b.icon=b:CreateTexture(nil,"ARTWORK");b.icon:SetAllPoints(b);b.icon:SetTexture("Interface\\Icons\\"..(i==1 and"INV_Sword_04"or"INV_Shield_05"))
  b:SetNormalTexture("Interface\\Buttons\\UI-Quickslot2");b:SetHighlightTexture("Interface\\Buttons\\ButtonHilight-Square")
  b.asxHudKind=kind;b:SetScript("OnEnter",function()open(this.asxHudKind,false)end);b:SetScript("OnLeave",function()if not pinned then leaveAt=GetTime()+.2 end end);b:SetScript("OnClick",function()open(this.asxHudKind,true)end);buttons[kind]=b
 end
 root:Hide();return true
end
local function activate()
 if enabled then return end
 if not root and not build()then return end
 -- Preserve native XP artwork, height, text and handlers; widen only its geometry.
 anchor(MainMenuExpBar,"BOTTOMLEFT",root,"BOTTOMLEFT",2,43)
 MainMenuExpBar:SetWidth(root:GetWidth()-4);widths[MainMenuExpBar]=root:GetWidth()-4
 local width=MainMenuExpBar:GetWidth();base:SetWidth(width);mask:SetWidth(width);mask.content:SetWidth(width);drawer:SetWidth(width)
 for _,c in ipairs(counters.values)do c.button:SetWidth(counters:GetWidth()/3);c.button:ClearAllPoints();c.button:SetPoint("TOPLEFT",counters,"TOPLEFT",(c.index-1)*counters:GetWidth()/3,0)end
 for i=0,4 do local t=getglobal(i==4 and"AZS_MainMenuBarTexture4"or("MainMenuBarTexture"..i));if t then remember(t);t:SetAlpha(0);hiddenArt[t]=true end end
 for i=1,6 do local b=getglobal("ActionButton"..i);if b then anchor(b,"BOTTOMLEFT",root,"BOTTOMLEFT",8+(i-1)*42,4)end end
 if ActionButton12 then anchor(ActionButton12,"BOTTOMLEFT",root,"BOTTOMLEFT",280,4)end
 if CharacterMicroButton then anchor(CharacterMicroButton,"BOTTOMLEFT",root,"BOTTOMLEFT",332,2)end
 if MainMenuBarLeftEndCap then anchor(MainMenuBarLeftEndCap,"BOTTOM",root,"BOTTOMLEFT",-32,0)end
 if MainMenuBarRightEndCap then anchor(MainMenuBarRightEndCap,"BOTTOM",root,"BOTTOMRIGHT",32,0)end
 enabled=true;root:Show()
end
local function deactivate()
 local wasEnabled=enabled;enabled=false;if wasEnabled then restore()end;close();progress=0;selected=nil;visibleDefs=nil
 if root then root:Hide();mask:Hide();equipment:Hide();clock:Hide()end
end
local function refresh()
 local phase=AZS_GetPhase and AZS_GetPhase()
 local active=phase=="playing"or phase=="choosing"or phase=="paused"or phase=="dead"
 if not ready or not db().hud or not active or not MainMenuBar or not MainMenuBar:IsVisible()then deactivate();return end
 activate();if not enabled then return end
 local run=AZS_RUN;local valid=type(run)=="table"
 for _,c in ipairs(counters.values)do local v=valid and run[c.key];if finite(v)and v>=0 then c.label:SetText(string.format("%.0f",v))else c.label:SetText("—");valid=false end end
 replacePart("tally",valid)
 -- The native clock is not a movable Lua frame. Replace its presentation only
 -- while native run time is available; pause/choice interpolation stays native.
 local time=AZS_GetRunTime and AZS_GetRunTime()
 local showClock=finite(time)and time>=0 and(AZS_IsHudShown and AZS_SetHudShown)and(preferences.clock==true or preferences.clock==nil and AZS_IsHudShown("clock"))
 if showClock then
  local function timestamp(t)t=math.max(0,math.floor(t));return string.format("%02d:%02d",math.floor(t/60),math.mod(t,60))end
  clock.time:SetText(timestamp(time))
  local upcoming=AZS_GetUpcomingBosses and AZS_GetUpcomingBosses();local nextTime=type(upcoming)=="table"and upcoming[1]and upcoming[1].left
  if not finite(nextTime)then nextTime=run and run.nb end
  clock.next:SetText(finite(nextTime)and nextTime>=0 and("next boss in "..timestamp(nextTime))or"")
  replacePart("clock",true);clock:Show()
 else replacePart("clock",false);clock:Hide()end
 local view=ASX_GetState and ASX_GetState(1);local items=view and view.available and view.data and view.data.equipment;local texts=view and view.texts and view.texts.equipment
 local session=view and view.available and view.session and(view.session.epoch..":"..view.session.recipientGUID)or nil
 if session~=lastSession or(lastClock and finite(run and run.t)and run.t<lastClock-.5)then close();lastSession=session end
 lastClock=run and run.t
 local available=type(items)=="table"and type(texts)=="table"and table.getn(items)<=5 and table.getn(items)==table.getn(texts)
 if available then
  for i,b in ipairs(icons)do
   b.item=texts[i]
   local path=b.item and b.item.icon or"Interface\\PaperDoll\\UI-PaperDoll-Slot-Trinket"
   if not string.find(path,"^Interface[\\/]")then path="Interface\\Icons\\"..path end
   SetPortraitToTexture(b.icon:GetName(),path)
  end
  equipment:Show()
 else for _,b in ipairs(icons)do b.item=nil end;equipment:Hide()end
 replacePart("build",available)
 if visibleDefs and selected then for i,def in ipairs(visibleDefs)do rows[i].value:SetText(format(def,AZS_STATS)or"—")end end
end
controller:RegisterEvent("ADDON_LOADED")
controller:SetScript("OnEvent",function()if arg1=="ASXUI"then ready=true;db()end end)
controller:SetScript("OnUpdate",function()
 local dt=arg1 or 0;if not finite(dt)or dt<0 then dt=0 end;dt=math.min(dt,.1)
 elapsed=elapsed+dt;if elapsed>=.25 then elapsed=0;refresh()end
 if not enabled then return end
 if leaveAt and GetTime()>=leaveAt and not pinned then target=0;leaveAt=nil end
 if progress<target then progress=math.min(target,progress+dt/duration)elseif progress>target then progress=math.max(target,progress-dt/duration)end
 -- Scroll a constant-size child through the viewport. Never animate its height.
 local eased=progress*progress*(3-2*progress)
 local level=math.max(root:GetFrameLevel(),ASXUI_GetDrawerLevel and ASXUI_GetDrawerLevel()or 0)
 mask:SetFrameLevel(level);mask.content:SetFrameLevel(level);drawer:SetFrameLevel(level)
 for _,row in ipairs(rows)do row:SetFrameLevel(level+1)end
 local frame,y;if ASXUI_GetDrawerAnchor then frame,y=ASXUI_GetDrawerAnchor()end
 mask:ClearAllPoints();mask:SetPoint("BOTTOM",frame or base,"TOP",0,y or 0)
 mask:SetVerticalScroll(eased*drawer.height);mask:Show()
 equipment:ClearAllPoints();equipment:SetPoint("BOTTOM",base,"TOP",0,8)
end)
function ASXUI_ConfigureHud(value)
 if value~="on"and value~="off"then return false end
 db().hud=value=="on";if not db().hud then deactivate()else refresh()end;return true
end
function ASXUI_HudReadout()
 local out={enabled=enabled,selected=selected,pinned=pinned,progress=progress,height=drawer and drawer.height or 0,rows={},equipment={}}
 if rows then for i,def in ipairs(visibleDefs or{})do table.insert(out.rows,{key=def[1],label=rows[i].label:GetText(),value=rows[i].value:GetText()})end end
 for i,b in ipairs(icons or{})do if b.item then out.equipment[i]={name=b.item.name,icon=b.item.icon,text=b.item.text}end end
 return out
end

-- Internal retained-frame anchors for ASXUI's persistent native-value views.
function ASXUI_GetHudAnchors()
 if not enabled or not root then return nil end
 local eased=progress*progress*(3-2*progress)
 return {root=root,base=base,counters=counters,width=base:GetWidth(),topOffset=8+(equipment:IsShown()and 40 or 0)}
end
