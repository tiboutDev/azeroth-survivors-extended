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
    toast:SetWidth(340);toast:SetHeight(88);toast.asxCompactToast=true;toast:SetPoint("TOP",UIParent,"TOP",0,-140);toast:EnableMouse(false)
    toast:SetBackdrop({bgFile="Interface\\Tooltips\\UI-Tooltip-Background",edgeFile="Interface\\Tooltips\\UI-Tooltip-Border",tile=true,tileSize=16,edgeSize=16,insets={left=4,right=4,top=4,bottom=4}})
    toast:SetBackdropColor(0.04,0.04,0.10,1);toast:SetBackdropBorderColor(0.75,0.75,0.75,1)
    if ASXUI_StylePanel then ASXUI_StylePanel(toast,true)end
    local iconFrame=CreateFrame("Frame",nil,toast);iconFrame:SetWidth(36);iconFrame:SetHeight(36);iconFrame:SetPoint("LEFT",toast,"LEFT",14,0)
    toast.icon=iconFrame:CreateTexture(nil,"ARTWORK");toast.icon:SetWidth(32);toast.icon:SetHeight(32);toast.icon:SetPoint("CENTER",iconFrame,"CENTER",0,0);toast.icon:SetTexture(texture)
    local border=iconFrame:CreateTexture(nil,"OVERLAY");border:SetTexture("Interface\\Buttons\\UI-Quickslot2");border:SetWidth(52);border:SetHeight(52);border:SetPoint("CENTER",iconFrame,"CENTER",0,0)
    toast.title=eventFont(toast,"GameFontNormal","FRIZQT__",12);toast.title:SetPoint("TOPLEFT",toast,"TOPLEFT",62,-12);toast.title:SetText(heading);toast.title:SetTextColor(asGold[1],asGold[2],asGold[3])
    toast.reward=eventFont(toast,"GameFontHighlight","FRIZQT__",15);toast.reward:SetPoint("TOPLEFT",toast.title,"BOTTOMLEFT",0,-4);toast.reward:SetWidth(264);toast.reward:SetJustifyH("LEFT")
    toast.note=eventFont(toast,"GameFontHighlightSmall","FRIZQT__",12);toast.note:SetPoint("TOPLEFT",toast.reward,"BOTTOMLEFT",0,-4);toast.note:SetWidth(264);toast.note:SetJustifyH("LEFT");toast.note:SetTextColor(asMuted[1],asMuted[2],asMuted[3])
    return toast
end
-- Native English UI receipts, not wallet-delta inference or gameplay commands.
local nativeQueue,nativeCurrent,nativeToast={}
local nativeUntil,nativeClock,nativeWall=0,nil,0
local lootPreference
local nativeTrialScope,nativeAnnouncementUntil,nativeAnnouncementStart=nil,0,0
local function restoreLootPreference()
    if ASXUI_RestoreNativeHud then ASXUI_RestoreNativeHud("loot")elseif lootPreference~=nil and AZS_IsHudShown and AZS_SetHudShown and not AZS_IsHudShown("loot")then AZS_SetHudShown("loot",lootPreference)end
    lootPreference=nil
end
local function nativeNoticeActive()
    local locale=GetLocale and GetLocale()
    return ASXUI_GetHudAnchors and ASXUI_GetHudAnchors()and(locale=="enUS"or locale=="enGB")
end
local function noticeKind(message,channel)
    if type(message)~="string"or string.len(message)>2048 then return nil end
    local plain=string.gsub(string.gsub(message,"|c%x%x%x%x%x%x%x%x",""),"|r","")
    if(channel=="CHAT_MSG_LOOT"or channel=="CHAT_MSG_SYSTEM")and string.find(plain,"^You receive loot:")then return"loot"end
    local _,_,amount=string.find(plain,"^You earn (%d+%.?%d*) marks %(")
    if channel=="CHAT_MSG_SYSTEM"and amount and tonumber(amount)>0 then return"marks"end
end
local function noticeIcon(kind,message)
    if kind=="marks"then return"Interface\\Icons\\INV_Misc_Rune_01"end
    if GetItemInfo then
        local _,_,name=string.find(message,"%[([^%]]+)%]")
        if name then
            local values={pcall(GetItemInfo,name)}
            -- Accept both retained Classic and extended item-info tuples.
            local icon=values[11]or values[10]
            local texture=type(icon)=="string"and(string.find(icon,"^Interface[\\/]")or string.find(icon,"^[Ii][Nn][Vv]_")or string.find(icon,"^[Aa]bility_")or string.find(icon,"^[Ss]pell_"))
            if values[1]and texture and string.len(icon)<160 and not string.find(icon,"[^%w_./\\]")then
                if not string.find(icon,"^Interface[\\/]")then icon="Interface\\Icons\\"..icon end
                return icon
            end
        end
    end
    return"Interface\\Icons\\INV_Box_02"
end
controller:RegisterEvent("CHAT_MSG_LOOT");controller:RegisterEvent("CHAT_MSG_SYSTEM")
controller:SetScript("OnEvent",function()
    local message,channel=arg1,event
    if not nativeNoticeActive()then return end
    local kind=noticeKind(message,channel);if not kind then return end
    if kind=="loot"then
        -- Claim the native notice only after receiving a compatible local item
        -- receipt. Unknown locale/absence of signals leaves the original UI.
        if not AZS_IsHudShown or not AZS_SetHudShown then return end
        if ASXUI_ClaimNativeHud then local value=ASXUI_ClaimNativeHud("loot");if lootPreference==nil then lootPreference=value end
        else if lootPreference==nil then lootPreference=AZS_IsHudShown("loot")and true or false end;if lootPreference then AZS_SetHudShown("loot",false)end end
        if not lootPreference then return end
    end
    if table.getn(nativeQueue)<16 then table.insert(nativeQueue,{kind=kind,text=message,icon=noticeIcon(kind,message),readyAt=GetTime()+.25})end
end)
local function refreshNativeRewards()
    local now=GetTime();local previousWall=nativeWall;local dt=math.max(0,now-nativeWall);nativeWall=now
    local clock=AZS_GetRunTime and AZS_GetRunTime()
    if not nativeNoticeActive()or(type(clock)=="number"and nativeClock and clock+.5<nativeClock)then
        nativeQueue={};nativeCurrent=nil;nativeTrialScope=nil;nativeAnnouncementUntil=0;nativeAnnouncementStart=0;restoreLootPreference();if nativeToast then nativeToast:Hide()end;nativeClock=clock;return
    end
    nativeClock=clock
    if model and model.owner then
        local ritual=model.owner.ritual
        local scope=model.epoch..":"..model.own..":"..tostring(ritual and ritual.event or "none")
        if nativeTrialScope and scope~=nativeTrialScope and ritual then
            -- Leave the native ritual arrival banner readable. Receipt data and
            -- already-earned rewards remain unchanged; only display is delayed.
            nativeAnnouncementStart=now;nativeAnnouncementUntil=now+6
        end
        nativeTrialScope=scope
    end
    local phase=AZS_GetPhase and AZS_GetPhase()
    if phase~="playing"and phase~="dead"then if nativeCurrent then nativeUntil=nativeUntil+dt end;if nativeToast then nativeToast:Hide()end;return end
    if nativeCurrent then nativeUntil=nativeUntil+math.max(0,math.min(now,nativeAnnouncementUntil)-math.max(previousWall,nativeAnnouncementStart))end
    if now<nativeAnnouncementUntil then if nativeToast then nativeToast:Hide()end;return end
    if nativeCurrent and now>=nativeUntil then nativeCurrent=nil end
    if not nativeCurrent and table.getn(nativeQueue)>0 and now>=nativeQueue[1].readyAt then
        nativeCurrent=table.remove(nativeQueue,1);nativeUntil=now+3
        if not nativeToast then nativeToast=discoveryToast("ASXUINativeRewardToast","",nativeCurrent.icon)end
        nativeToast.title:SetText(nativeCurrent.kind=="marks"and"Marks earned"or"Loot received")
        nativeToast.icon:SetTexture(nativeCurrent.icon);nativeToast.reward:SetText(nativeCurrent.text);nativeToast.reward:SetTextColor(1,1,1);nativeToast.note:SetText("")
    end
    if nativeToast then if nativeCurrent then nativeToast:Show()else nativeToast:Hide()end end
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
-- Snapshot deltas announce only newly observed personal pickups, never a restored total.
local crystalToast,crystalScope,crystalCounts,crystalClock,crystalCurrent
local crystalQueue,crystalUntil={},0
local crystalKeys={"red","blue","green","yellow"}
local function clearCrystalToast()
    crystalQueue={};crystalCurrent=nil
    if crystalToast then crystalToast:Hide()end
end
local function refreshCrystalToast()
    if not model then
        clearCrystalToast();crystalScope=nil;crystalCounts=nil;crystalClock=nil;return
    end
    local scope=model.epoch..":"..model.own
    local counts={};local data=model.crystals
    for _,key in ipairs(crystalKeys)do
        local n=data and data[key]or 0
        if type(n)~="number"or n~=n or n<0 or n>1000000 or math.floor(n)~=n then
            clearCrystalToast();crystalScope=nil;crystalCounts=nil;return
        end
        counts[key]=n
    end
    local clock=AZS_GetRunTime and AZS_GetRunTime()
    local reset=scope~=crystalScope or not crystalCounts or(type(clock)=="number"and crystalClock and clock+.5<crystalClock)
    if crystalCounts then for _,key in ipairs(crystalKeys)do if counts[key]<crystalCounts[key]then reset=true end end end
    if reset then
        clearCrystalToast();crystalScope=scope;crystalCounts=counts;crystalClock=clock;return
    end
    for _,key in ipairs(crystalKeys)do
        local text=model.crystalTexts and model.crystalTexts[key]
        if text then
            for n=1,math.min(16,counts[key]-crystalCounts[key])do
                table.insert(crystalQueue,text)
                if table.getn(crystalQueue)>16 then table.remove(crystalQueue,1)end
            end
        end
    end
    crystalCounts=counts;crystalClock=clock
    local phase=AZS_GetPhase and AZS_GetPhase()
    if phase and phase~="playing"and phase~="dead"then
        if crystalCurrent then crystalUntil=GetTime()+3 end
        if crystalToast then crystalToast:Hide()end
        return
    end
    if not crystalCurrent or GetTime()>=crystalUntil then
        crystalCurrent=table.remove(crystalQueue,1)
        if crystalCurrent then
            local d=crystalCurrent;crystalUntil=GetTime()+3
            if not crystalToast then crystalToast=discoveryToast("ASXCrystalDiscoveryToast",d.title,d.icon)end
            crystalToast.title:SetText(d.title);crystalToast.icon:SetTexture(d.icon)
            local c=d.titleColor or d.color;crystalToast.title:SetTextColor(c[1],c[2],c[3])
            crystalToast.reward:SetText(d.reward);crystalToast.reward:SetTextColor(d.color[1],d.color[2],d.color[3])
            crystalToast.note:SetText(d.note)
        end
    end
    if crystalCurrent then crystalToast:Show()elseif crystalToast then crystalToast:Hide()end
end
-- Shared native casting-bar recipe. Progress and completion remain native/ASX.
local function channelFrame(name)
    local f=CreateFrame("Frame",name,UIParent);f:SetWidth(320);f:SetHeight(54);f:EnableMouse(false)
    local track=CreateFrame("Frame",nil,f);track:SetWidth(300);track:SetHeight(13);track:SetPoint("CENTER",f,"CENTER",0,5)
    local bg=track:CreateTexture(nil,"BACKGROUND");bg:SetAllPoints(track);bg:SetTexture("Interface\\TargetingFrame\\UI-StatusBar");bg:SetVertexColor(.06,.05,.035,1)
    f.bar=CreateFrame("StatusBar",nil,track);f.bar:SetAllPoints(track);f.bar:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar")
    for _,part in ipairs({{22,40,"BOTTOMLEFT",-9},{40,216},{216,234,"BOTTOMRIGHT",9}})do
        local edge=f.bar:CreateTexture(nil,"OVERLAY");edge:SetTexture("Interface\\CastingBar\\UI-CastingBar-Border");edge:SetTexCoord(part[1]/256,part[2]/256,.25,.75);edge:SetHeight(32)
        if part[3]then edge:SetWidth(part[2]-part[1]);edge:SetPoint(part[3],track,part[3],part[4],-7)
        else edge:SetPoint("BOTTOMLEFT",track,"BOTTOMLEFT",9,-7);edge:SetPoint("BOTTOMRIGHT",track,"BOTTOMRIGHT",-9,-7)end
    end
    f.title=eventFont(f.bar,"GameFontNormal","FRIZQT__",11);f.title:SetPoint("CENTER",track,"CENTER",0,0);f.title:SetTextColor(1,1,1)
    f.note=eventFont(f,"GameFontHighlightSmall","FRIZQT__",12);f.note:SetPoint("TOP",track,"BOTTOM",0,-8);f.note:SetWidth(300);f.note:SetJustifyH("CENTER");f.note:SetTextColor(1,.82,0)
    return f
end

local reviveFrame,revivePreference
local function restoreRevive()
    if ASXUI_RestoreNativeHud then ASXUI_RestoreNativeHud("revive")elseif revivePreference~=nil and AZS_IsHudShown and AZS_SetHudShown and not AZS_IsHudShown("revive")then AZS_SetHudShown("revive",revivePreference)end
    revivePreference=nil;if reviveFrame then reviveFrame:Hide()end
end
local function refreshRevive()
    local active=ASXUI_GetHudAnchors and ASXUI_GetHudAnchors()
    local r=active and AZS_GetRevive and AZS_GetRevive()
    if type(r)~="table"or not AZS_IsHudShown or not AZS_SetHudShown then restoreRevive();return end
    local down=r.down==1 or r.down==true
    local raising=type(r.reviving)=="string"and r.reviving~=""
    local fraction=down and r.share or r.rshare
    if(down or raising)and(type(fraction)~="number"or fraction~=fraction or fraction<0 or fraction>1)then restoreRevive();return end
    if ASXUI_ClaimNativeHud then local value=ASXUI_ClaimNativeHud("revive");if revivePreference==nil then revivePreference=value end
    else if revivePreference==nil then revivePreference=AZS_IsHudShown("revive")and true or false end;if AZS_IsHudShown("revive")then AZS_SetHudShown("revive",false)end end
    if not revivePreference or not(down or raising)then if reviveFrame then reviveFrame:Hide()end;return end
    if not reviveFrame then reviveFrame=channelFrame("ASXUIResurrectionProgress");reviveFrame.bar:SetStatusBarColor(.3,.7,1);reviveFrame.bar:SetMinMaxValues(0,1)end
    reviveFrame.bar:SetValue(fraction)
    reviveFrame.title:SetText(down and "Resurrection"or("Reviving "..r.reviving))
    if down then reviveFrame.note:SetText(type(r.by)=="string"and r.by~=""and("Being revived by "..r.by)or"Waiting for an ally")
    else reviveFrame.note:SetText(string.format("%.0f%%",fraction*100))end
    reviveFrame:Show()
end

-- Retained display only: native simulation owns presence, progress and completion.
local ritualFrame
local function refreshRitual()
    local own=supplyOwner();local r=own and own.ritual
    if not r or not r.inside then if ritualFrame then ritualFrame:Hide();ritualFrame.rendered=r and r.held or 0 end;return end
    if not ritualFrame then
        local f=channelFrame("ASXSaygeTrialProgress")
        f.bar:SetStatusBarColor(1,.7,0);f.bar:SetMinMaxValues(0,r.maximum);f.rendered=r.held
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

local defaults={supplies={x=0,y=0},fortune={x=0,y=0},crystals={x=0,y=0},sayge={x=0,y=0},revive={x=0,y=0},loot={x=0,y=0},marks={x=0,y=0}}
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
    if ASXUIDB.eventAnchorLayoutVersion~=1 then
        local p=ASXUIDB.eventAnchor
        -- Migrate only the previous centered default; retain custom placements.
        if (p.x==nil or p.x==0)and p.y==-85 then p.y=-125 end
        ASXUIDB.eventAnchorLayoutVersion=1
    end
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
    local p=layoutSettings();p.x=number(p.x,-4096,4096,0);p.y=number(p.y,-4096,4096,-125)
    local signature=p.x..":"..p.y
    if anchor.asxPosition~=signature then anchor:ClearAllPoints();anchor:SetPoint("TOP",UIParent,"TOP",p.x,p.y);anchor.asxPosition=signature end
    local offset=0
    for _,entry in ipairs({{reviveFrame,"revive",54},{ritualFrame,"sayge",54},{supplyToast,"supplies",88},{blessingToast,"fortune",88},{crystalToast,"crystals",88},{nativeToast,nativeCurrent and nativeCurrent.kind or "loot",88}})do
        local frame,kind,height=entry[1],entry[2],entry[3];local pref=preference(kind)
        if frame then
            if frame.asxCompactToast then
                local measured=32
                for _,label in ipairs({frame.title,frame.reward,frame.note})do
                    local value=label.GetStringHeight and label:GetStringHeight()or label:GetHeight()
                    if type(value)=="number"and value==value and value>0 and value<1024 then measured=measured+value end
                end
                height=math.max(88,math.ceil(measured));frame:SetHeight(height)
            end
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
        model={epoch=snapshot.session.epoch,own=snapshot.session.recipientGUID,owner={discovery=present(data.discovery,texts.supplies),ritual=present(data.sayge,texts.sayge)},blessing=present(data.fortune,texts.fortune),crystals=data.crystals,crystalTexts=texts.crystals and texts.crystals.pickups}
    end
    refreshSupplyToast();refreshBlessingToast();refreshRitual();refreshCrystalToast();refreshRevive();refreshNativeRewards()
    if reviveFrame and reviveFrame:IsShown()and ritualFrame then ritualFrame:Hide()end
    layout()
    updateRitualBar(GetTime())
end
function ASXUI_ConfigureWidget(kind,property,first,second)
    if kind=="anchor"then
        local p=layoutSettings()
        if property=="reset"then p.x=0;p.y=-125
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
