-- ASXUI owns event widgets and preferences. All displayed wording comes from ASX.
local controller=CreateFrame("Frame","ASXUIEventsController",UIParent)
local model
local function supplyOwner()return model and model.owner end
local function blessingFor()return model and model.blessing end
local blessingToast,blessingSeen,blessingUntil=nil,nil,0
local blessingClock
-- Classic WoW tooltip/quickslot assets; gameplay wording remains ASX-owned.
local asGold={1,0.82,0}
local asMuted={0.75,0.75,0.75}
local function eventFont(parent,template,font,size)
    local text=parent:CreateFontString(nil,"OVERLAY",template);text:SetFont("Fonts\\"..font..".TTF",size,"");return text
end
local function discoveryToast(name,heading,texture)
    local toast=CreateFrame("Frame",name,UIParent)
    toast:SetWidth(388);toast:SetHeight(106);toast:SetPoint("TOP",UIParent,"TOP",0,-140);toast:EnableMouse(false)
    toast:SetBackdrop({bgFile="Interface\\Tooltips\\UI-Tooltip-Background",edgeFile="Interface\\Tooltips\\UI-Tooltip-Border",tile=true,tileSize=16,edgeSize=16,insets={left=4,right=4,top=4,bottom=4}})
    toast:SetBackdropColor(0.04,0.04,0.10,1);toast:SetBackdropBorderColor(0.75,0.75,0.75,1)
    local iconFrame=CreateFrame("Frame",nil,toast);iconFrame:SetWidth(52);iconFrame:SetHeight(52);iconFrame:SetPoint("LEFT",toast,"LEFT",19,0)
    toast.icon=iconFrame:CreateTexture(nil,"ARTWORK");toast.icon:SetWidth(46);toast.icon:SetHeight(46);toast.icon:SetPoint("CENTER",iconFrame,"CENTER",0,0);toast.icon:SetTexture(texture)
    local border=iconFrame:CreateTexture(nil,"OVERLAY");border:SetTexture("Interface\\Buttons\\UI-Quickslot2");border:SetWidth(76);border:SetHeight(76);border:SetPoint("CENTER",iconFrame,"CENTER",0,0)
    toast.title=eventFont(toast,"GameFontNormal","FRIZQT__",14);toast.title:SetPoint("TOPLEFT",toast,"TOPLEFT",86,-16);toast.title:SetText(heading);toast.title:SetTextColor(asGold[1],asGold[2],asGold[3])
    toast.reward=eventFont(toast,"GameFontHighlight","FRIZQT__",17);toast.reward:SetPoint("TOPLEFT",toast.title,"BOTTOMLEFT",0,-7);toast.reward:SetWidth(286);toast.reward:SetJustifyH("LEFT")
    toast.note=eventFont(toast,"GameFontHighlightSmall","FRIZQT__",13);toast.note:SetPoint("TOPLEFT",toast.reward,"BOTTOMLEFT",0,-7);toast.note:SetWidth(286);toast.note:SetJustifyH("LEFT");toast.note:SetTextColor(asMuted[1],asMuted[2],asMuted[3])
    return toast
end
local supplyToast,supplyScope,supplyUntil,supplyCurrent=nil,nil,0,nil
local supplySeen,supplyQueue={},{}

local function refreshSupplyToast()
    local own=supplyOwner()
    if not own then if supplyToast then supplyToast:Hide()end;return end
    local scope=model.epoch..":"..model.own
    if scope~=supplyScope then supplyScope=scope;supplySeen={};supplyQueue={};supplyCurrent=nil end
    local d=own.discovery
    if d and not supplySeen[d.event]then
        supplySeen[d.event]=true;table.insert(supplyQueue,d)
        -- Native discovery IDs increase; bound cosmetic history and unread notifications.
        for id,_ in pairs(supplySeen)do if id<d.event-32 then supplySeen[id]=nil end end
        if table.getn(supplyQueue)>16 then table.remove(supplyQueue,1)end
    end
    local phase=AZS_GetPhase and AZS_GetPhase()
    if phase and phase~="playing" and phase~="dead" then if supplyCurrent then supplyUntil=GetTime()+3 end;if supplyToast then supplyToast:Hide()end;return end
    if not supplyCurrent or GetTime()>=supplyUntil then
        supplyCurrent=table.remove(supplyQueue,1)
        if supplyCurrent then
            d=supplyCurrent;supplyUntil=GetTime()+3
            if not supplyToast then supplyToast=discoveryToast("ASXSupplyDiscoveryToast",d.title,d.icon)end
            supplyToast:SetAlpha(1)
            supplyToast.title:SetText(d.title)
            local c=d.color;supplyToast.icon:SetTexture(d.icon);supplyToast.reward:SetText(d.reward);supplyToast.reward:SetTextColor(c[1],c[2],c[3])
            supplyToast.note:SetText(d.note)
        end
    end
    if not supplyCurrent then if supplyToast then supplyToast:Hide()end;return end
    supplyToast:Show()
end
-- Retained display only: native simulation owns presence, progress and completion.
local ritualFrame
local function refreshRitual()
    local own=supplyOwner();local r=own and own.ritual
    if not r or not r.inside then if ritualFrame then ritualFrame:Hide();ritualFrame.rendered=r and r.held or 0 end;return end
    if not ritualFrame then
        local f=CreateFrame("Frame","ASXSaygeTrialProgress",UIParent)
        f:SetWidth(306);f:SetHeight(70);f:EnableMouse(false)
        local icon=f:CreateTexture(nil,"ARTWORK");icon:SetWidth(32);icon:SetHeight(32);icon:SetPoint("LEFT",f,"LEFT",2,-1);icon:SetTexture("Interface\\Icons\\INV_Misc_Orb_02")
        local track=CreateFrame("Frame",nil,f);track:SetWidth(266);track:SetHeight(22);track:SetPoint("RIGHT",f,"RIGHT",0,-1)
        local bg=track:CreateTexture(nil,"BACKGROUND");bg:SetAllPoints(track);bg:SetTexture(0,0,0,.7)
        f.bar=CreateFrame("StatusBar",nil,track);f.bar:SetAllPoints(track);f.bar:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar");f.bar:SetStatusBarColor(1,.7,0);f.bar:SetMinMaxValues(0,r.maximum);f.rendered=r.held
        local border=track:CreateTexture(nil,"OVERLAY");border:SetTexture("Interface\\CastingBar\\UI-CastingBar-Border");border:SetWidth(350);border:SetHeight(88);border:SetPoint("CENTER",track,"CENTER",0,0)
        f.title=eventFont(f.bar,"GameFontNormal","FRIZQT__",14);f.title:SetPoint("CENTER",track,"CENTER",0,1);f.title:SetText(r.title);f.title:SetTextColor(1,1,1)
        f.note=eventFont(f,"GameFontHighlightSmall","FRIZQT__",12);f.note:SetPoint("TOP",track,"BOTTOM",0,-7);f.note:SetWidth(276);f.note:SetJustifyH("CENTER");f.note:SetTextColor(1,.82,0)
        ritualFrame=f
    end
    local identity=model.epoch..":"..model.own..":"..r.event
    if ritualFrame.identity~=identity then ritualFrame.identity=identity;ritualFrame.rendered=r.held end
    if r.allInside==false then ritualFrame.rendered=r.held end
    ritualFrame.target=r.held;ritualFrame.title:SetText(r.title);ritualFrame.note:SetText(r.note)
    ritualFrame:Show()
end
function ASXRitualReadout()
    local own=supplyOwner();local r=own and own.ritual
    return r and r.event,r and r.held,r and r.inside,ritualFrame and ritualFrame:IsShown() or false
end
local function refreshBlessingToast()
    local b=blessingFor("player")
    local runPhase=AZS_GetPhase and AZS_GetPhase()
    if runPhase and runPhase~="playing" and runPhase~="choosing" and runPhase~="paused" and runPhase~="dead" then blessingSeen=nil;blessingClock=nil end
    if not b then
        -- Missing/stale data hides the widget without replaying an acquired Fortune.
        if blessingToast then blessingToast:Hide()end
        return
    end
    local key=model.own..":"..b.boss
    local clock=AZS_GetRunTime and AZS_GetRunTime()
    if type(clock)=="number" then
        if blessingClock and clock+0.5<blessingClock then blessingSeen=nil end
        blessingClock=clock
    end
    -- Boss XP commonly opens the native choice overlay. Announce when readable.
    local phase=AZS_GetPhase and AZS_GetPhase()
    if phase and phase~="playing" and phase~="dead" then
        if blessingToast then
            if blessingUntil>GetTime()then blessingUntil=GetTime()+6 end
            blessingToast:Hide()
        end
        return
    end
    if key~=blessingSeen then
        blessingSeen=key;blessingUntil=GetTime()+6
        if not blessingToast then
            blessingToast=discoveryToast("ASXDarkmoonBlessingToast",b.title,b.icon)
        end
        blessingToast:SetAlpha(1)
        blessingToast.title:SetText(b.title);blessingToast.icon:SetTexture(b.icon)
        local c=b.color
        blessingToast.reward:SetText(b.reward)
        blessingToast.reward:SetTextColor(c[1],c[2],c[3])
        blessingToast.note:SetText(b.note)
        blessingToast:SetAlpha(1);blessingToast:Show()
    end
    local remaining=blessingUntil-GetTime()
    if remaining<=0 then blessingToast:Hide()
    else blessingToast:Show()end
end
local eventAnimationTime=GetTime()
local function updateRitualBar(now)
    local dt=math.max(0,math.min(0.1,now-eventAnimationTime));eventAnimationTime=now
    if ritualFrame and ritualFrame:IsShown()then
        local value=ritualFrame.rendered or 0;local target=ritualFrame.target or value
        value=value+(target-value)*(1-math.exp(-dt/0.08));ritualFrame.rendered=value;ritualFrame.bar:SetValue(value)
    end
end
-- Read-only visual diagnostics; never used for gameplay or reward authority.
function ASXEventVisualReadout()
    return blessingToast and blessingToast:IsShown()or false,blessingToast and blessingToast:GetAlpha()or 0,supplyToast and supplyToast:IsShown()or false,supplyToast and supplyToast:GetAlpha()or 0
end

local defaults={supplies={x=0,y=0},fortune={x=0,y=0},sayge={x=0,y=0}}
local anchor=CreateFrame("Frame","ASXUIEventAnchor",UIParent)
anchor:SetWidth(1);anchor:SetHeight(1);anchor:EnableMouse(false)
local function layoutSettings()
    if type(ASXUIDB)~="table"then ASXUIDB={}end
    if type(ASXUIDB.widgets)~="table"then ASXUIDB.widgets={}end
    if ASXUIDB.widgetLayoutVersion~=2 then
        local old={supplies=-140,fortune=-260,sayge=-75}
        for kind,y in pairs(old)do local p=ASXUIDB.widgets[kind];if type(p)=="table"and type(p.y)=="number"then p.y=p.y-y end end
        ASXUIDB.widgetLayoutVersion=2
    end
    if type(ASXUIDB.eventAnchor)~="table"then ASXUIDB.eventAnchor={}end
    return ASXUIDB.eventAnchor
end
local function number(value,minimum,maximum,fallback)
    return type(value)=="number"and value==value and value>=minimum and value<=maximum and value or fallback
end
local function preference(kind)
    layoutSettings()
    if type(ASXUIDB.widgets[kind])~="table"then ASXUIDB.widgets[kind]={}end
    local p=ASXUIDB.widgets[kind];local d=defaults[kind]
    p.x=number(p.x,-4096,4096,d.x);p.y=number(p.y,-4096,4096,d.y)
    p.scale=number(p.scale,.5,2,1);p.opacity=number(p.opacity,0,1,1)
    if type(p.visible)~="boolean"then p.visible=true end
    return p
end
local function layout()
    local p=layoutSettings();p.x=number(p.x,-4096,4096,0);p.y=number(p.y,-4096,4096,-85)
    local signature=p.x..":"..p.y
    if anchor.asxPosition~=signature then anchor:ClearAllPoints();anchor:SetPoint("TOP",UIParent,"TOP",p.x,p.y);anchor.asxPosition=signature end
    local offset=0
    for _,entry in ipairs({{ritualFrame,"sayge",70},{supplyToast,"supplies",106},{blessingToast,"fortune",106}})do
        local frame,kind,height=entry[1],entry[2],entry[3];local pref=preference(kind)
        if frame then
            frame:SetScale(pref.scale);frame:SetAlpha(pref.opacity)
            if not pref.visible then frame:Hide()end
            if frame:IsShown()then
                local x,y=pref.x/pref.scale,(-offset+pref.y)/pref.scale
                local key=x..":"..y
                if frame.asxPosition~=key then frame:ClearAllPoints();frame:SetPoint("TOP",anchor,"BOTTOM",x,y);frame.asxPosition=key end
                offset=offset+height*pref.scale+12
            end
        end
    end
end
local function update()
    local snapshot=type(ASX_GetAPIVersion)=="function"and ASX_GetAPIVersion()==1 and type(ASX_GetState)=="function"and ASX_GetState(1)or nil
    model=nil
    if snapshot and snapshot.available then
        local data,texts=snapshot.data,snapshot.texts
        local function present(raw,text)
            if not raw or not text then return nil end
            local value={};for key,item in pairs(raw)do value[key]=item end;for key,item in pairs(text)do value[key]=item end
            return value
        end
        model={epoch=snapshot.session.epoch,own=snapshot.session.recipientGUID,owner={discovery=present(data.discovery,texts.supplies),ritual=present(data.sayge,texts.sayge)},blessing=present(data.fortune,texts.fortune)}
    end
    refreshSupplyToast();refreshBlessingToast();refreshRitual()
    layout()
    updateRitualBar(GetTime())
end
function ASXUI_ConfigureWidget(kind,property,first,second)
    if kind=="anchor"then
        local p=layoutSettings()
        if property=="reset"then p.x=0;p.y=-85
        elseif property=="position"then
            local x,y=tonumber(first),tonumber(second)
            if not number(x,-4096,4096,nil)or not number(y,-4096,4096,nil)then return false end;p.x=x;p.y=y
        else return false end
        update();return true
    end
    if not defaults[kind]then return false end
    local p=preference(kind)
    if property=="position"then
        local x,y=tonumber(first),tonumber(second)
        if not number(x,-4096,4096,nil)or not number(y,-4096,4096,nil)then return false end
        p.x=x;p.y=y
    elseif property=="scale"or property=="opacity"then
        local v=tonumber(first);local minimum=property=="scale"and .5 or 0;local maximum=property=="scale"and 2 or 1
        if not number(v,minimum,maximum,nil)then return false end;p[property]=v
    elseif property=="show"then
        if first~="on"and first~="off"then return false end;p.visible=first=="on"
    elseif property=="reset"then ASXUIDB.widgets[kind]={}
    else return false end
    update();return true
end
function ASXUI_SupplyVisualReadout()return supplyToast and supplyToast:IsShown()or false,supplyCurrent and supplyCurrent.event end
local last=-1
controller:SetScript("OnUpdate",function()
    local now=GetTime()
    if now-last>=.1 then last=now;update()else updateRitualBar(now)end
end)
