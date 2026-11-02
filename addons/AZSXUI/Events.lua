-- AZSXUI owns event widgets and preferences. All displayed wording comes from AZSX.
local controller=CreateFrame("Frame","AZSXUIEventsController",UIParent)
local model
local function supplyOwner()return model and model.owner end
local function blessingFor()return model and model.blessing end
local blessingToast,blessingSeen,blessingUntil=nil,nil,0
-- Classic WoW tooltip/quickslot assets; gameplay wording remains AZSX-owned.
local asGold={1,0.82,0}
local asMuted={0.75,0.75,0.75}
local function eventFont(parent,template,font,size)
    local text=parent:CreateFontString(nil,"OVERLAY",template);text:SetFont("Fonts\\"..font..".TTF",size,"");return text
end
-- Keep numeric/effect text intact; only split an authored duration suffix.
local function durationLines(text)
 local value=text or""
 value=string.gsub(value," %- (%d+ seconds)$","\nDuration: %1")
 value=string.gsub(value," %- (lasts until the end of the run%.)$","\nDuration: until the end of the run.")
 return value
end
local function discoveryToast(name,heading,texture)
    local toast=CreateFrame("Frame",name,UIParent)
    toast:SetWidth(320);toast:SetHeight(80);toast.azsxCompactToast=true;toast:SetPoint("TOP",UIParent,"TOP",0,-140);toast:EnableMouse(false)
    toast:SetBackdrop({bgFile="Interface\\Tooltips\\UI-Tooltip-Background",edgeFile="Interface\\Tooltips\\UI-Tooltip-Border",tile=true,tileSize=16,edgeSize=16,insets={left=4,right=4,top=4,bottom=4}})
    toast:SetBackdropColor(0.04,0.04,0.10,1);toast:SetBackdropBorderColor(0.75,0.75,0.75,1)
    if AZSXUI_StylePanel then AZSXUI_StylePanel(toast,true)end
    local iconFrame=CreateFrame("Frame",nil,toast);iconFrame:SetWidth(36);iconFrame:SetHeight(36);iconFrame:SetPoint("LEFT",toast,"LEFT",14,0)
    toast.icon=iconFrame:CreateTexture(nil,"ARTWORK");toast.icon:SetWidth(32);toast.icon:SetHeight(32);toast.icon:SetPoint("CENTER",iconFrame,"CENTER",0,0);toast.icon:SetTexture(texture)
    local border=iconFrame:CreateTexture(nil,"OVERLAY");border:SetTexture("Interface\\Buttons\\UI-Quickslot2");border:SetWidth(52);border:SetHeight(52);border:SetPoint("CENTER",iconFrame,"CENTER",0,0)
    toast.content=CreateFrame("Frame",nil,toast);toast.content:SetWidth(246);toast.content:SetHeight(44);toast.content:SetPoint("LEFT",toast,"LEFT",60,0)
    toast.title=eventFont(toast.content,"GameFontNormal","FRIZQT__",12);toast.title:SetPoint("TOPLEFT",toast.content,"TOPLEFT",0,0);toast.title:SetWidth(246);toast.title:SetJustifyH("LEFT");toast.title:SetText(heading);toast.title:SetTextColor(asGold[1],asGold[2],asGold[3])
    toast.reward=eventFont(toast.content,"GameFontHighlight","FRIZQT__",15);toast.reward:SetPoint("TOPLEFT",toast.title,"BOTTOMLEFT",0,-4);toast.reward:SetWidth(246);toast.reward:SetJustifyH("LEFT")
    toast.note=eventFont(toast.content,"GameFontHighlightSmall","FRIZQT__",12);toast.note:SetPoint("TOPLEFT",toast.reward,"BOTTOMLEFT",0,-4);toast.note:SetWidth(246);toast.note:SetJustifyH("LEFT");toast.note:SetTextColor(asMuted[1],asMuted[2],asMuted[3])
    return toast
end
-- Native English UI receipts, not wallet-delta inference or gameplay commands.
local nativeQueue,nativeCurrent,nativeToast={}
local nativeUntil,nativeClock,nativeWall=0,nil,0
local dropScope,dropSequence=nil,0
local lootPreference
local nativeTrialScope,nativeAnnouncementUntil,nativeAnnouncementStart=nil,0,0
local function restoreLootPreference()
    if AZSXUI_RestoreNativeHud then AZSXUI_RestoreNativeHud("loot")elseif lootPreference~=nil and AZS_IsHudShown and AZS_SetHudShown and not AZS_IsHudShown("loot")then AZS_SetHudShown("loot",lootPreference)end
    lootPreference=nil
end
local function nativeNoticeActive()
    local locale=GetLocale and GetLocale()
    return AZSXUI_GetHudAnchors and AZSXUI_GetHudAnchors()and(locale=="enUS"or locale=="enGB")
end
local function noticeData(message,channel)
    if type(message)~="string"or string.len(message)>2048 then return nil end
    local plain=string.gsub(string.gsub(message,"|c%x%x%x%x%x%x%x%x",""),"|r","")
    local _,_,amount,total=string.find(plain,"^You earn (%d+%.?%d*) marks %((%d+%.?%d*) in all%)%.?$")
    if channel=="CHAT_MSG_SYSTEM"and amount and tonumber(amount)>0 and tonumber(amount)<=1e12 and tonumber(total)>=tonumber(amount)and tonumber(total)<=1e12 then
        return {kind="marks",amount=tonumber(amount),total=tonumber(total),reward="|cff1eff00+"..amount.."|r marks",note="Total: |cff1eff00"..total.."|r"}
    end
    if channel~="CHAT_MSG_LOOT"and channel~="CHAT_MSG_SYSTEM"then return nil end
    if not string.find(plain,"^You receive loot:")then return nil end
    local _,_,name=string.find(plain,"%[([^%]]+)%]");if not name then return nil end
    local _,_,count=string.find(plain,"[xX](%d+)%.?$");count=count or"1"
    if tonumber(count)<1 or tonumber(count)>100000 then return nil end
    local _,_,color=string.find(message,"|c(%x%x%x%x%x%x%x%x)|Hitem:")
    if not color then _,_,color=string.find(message,"|c(%x%x%x%x%x%x%x%x)%[")end
    return {kind="loot",reward=(color and("|c"..color..name.."|r")or name),note="Quantity: |cff1eff00"..count.."|r"}
end
local function noticeIcon(kind,message)
    if kind=="marks"then return"Interface\\Icons\\INV_Misc_Rune_01"end
    if GetItemInfo then
        local _,_,id=string.find(message,"|Hitem:(%d+)")
        local _,_,name=string.find(message,"%[([^%]]+)%]")
        local reference=id and tonumber(id)or name
        if reference then
            local values={pcall(GetItemInfo,reference)}
            -- Native builds differ in tuple length. Select a validated texture,
            -- never an equip location, price or a hard-coded slot index.
            if values[1]then for i=2,16 do local icon=values[i]
                if type(icon)=="string"and string.len(icon)<160 and not string.find(icon,"[^%w_./\\]")and(string.find(icon,"^Interface[\\/]")or string.find(icon,"^[Ii][Nn][Vv]_")or string.find(icon,"^[Aa]bility_")or string.find(icon,"^[Ss]pell_"))then
                    if not string.find(icon,"^Interface[\\/]")then icon="Interface\\Icons\\"..icon end
                    return icon
                end
            end end
        end
    end
    return"Interface\\Icons\\INV_Box_02"
end

local function claimLoot()
    if not AZS_IsHudShown or not AZS_SetHudShown then return false end
    if AZSXUI_ClaimNativeHud then local value=AZSXUI_ClaimNativeHud("loot");if lootPreference==nil then lootPreference=value end
    else if lootPreference==nil then lootPreference=AZS_IsHudShown("loot")and true or false end;if lootPreference then AZS_SetHudShown("loot",false)end end
    return lootPreference==true
end
local function structuredLoot()
    local snapshot=AZSX_GetState and AZSX_GetState(1)
    if snapshot and snapshot.available and snapshot.api and snapshot.api.major==1 and snapshot.api.minor>=6 and snapshot.data and type(snapshot.data.loot)=="table"then return snapshot end
end
local function refreshLootReceipts()
    local snapshot=structuredLoot()
    if not snapshot or not snapshot.session then
        local major,minor;if AZSX_GetAPIVersion then major,minor=AZSX_GetAPIVersion()end
        if major==1 and type(minor)=="number"and minor>=6 then restoreLootPreference()end
        return
    end
    -- A current structured provider covers local native item receipts. Hide
    -- native toasts before their arrival, including already-collected drops.
    local showLoot=claimLoot()
    local scope=snapshot.session.epoch..":"..snapshot.session.recipientGUID
    if scope~=dropScope then
        if dropScope then nativeQueue={};nativeCurrent=nil;if nativeToast then nativeToast:Hide()end end
        dropScope=scope;dropSequence=0
    end
    local colors={"ffffffff","ff1eff00","ff0070dd","ffa335ee","ffff8000","ffe6cc80"}
    for _,drop in ipairs(snapshot.data.loot)do
        if type(drop.event)=="number"and drop.event>dropSequence then
            dropSequence=drop.event
            local text=snapshot.texts and snapshot.texts.loot and snapshot.texts.loot[drop.event]
            if drop.isNew==true and showLoot and text and type(text.name)=="string"and type(text.icon)=="string"and string.find(text.icon,"^INV_[%w_]+$")and colors[drop.rarity+1]and table.getn(nativeQueue)<16 then
                table.insert(nativeQueue,{kind="loot",reward="|c"..colors[drop.rarity+1]..text.name.."|r",note=drop.isNew and"New in your collection!"or"Added to your collection",icon="Interface\\Icons\\"..text.icon,readyAt=GetTime()+.25})
            end
        end
    end
end
local function markNumber(value)return string.gsub(string.gsub(string.format("%.6f",value),"0+$",""),"%.$","")end
local function mergeMarks(receipt,now)
    if receipt.kind~="marks"then return false end
    local last=nativeQueue[table.getn(nativeQueue)]
    if not last and nativeCurrent and nativeCurrent.kind=="marks"and now<nativeUntil then last=nativeCurrent end
    if not last or last.kind~="marks"or not last.amount or not last.total or not last.lastAt or now-last.lastAt>3 then return false end
    -- A repeated/stale total is not a second gain. A discontinuity starts a new receipt.
    if receipt.total<=last.total then return true end
    if math.abs(receipt.total-last.total-receipt.amount)>.00001 then return false end
    last.amount=last.amount+receipt.amount;last.total=receipt.total;last.lastAt=now
    last.reward="|cff1eff00+"..markNumber(last.amount).."|r marks";last.note="Total: |cff1eff00"..markNumber(last.total).."|r"
    if last==nativeCurrent then nativeUntil=math.max(nativeUntil,now+3);if nativeToast then nativeToast.reward:SetText(last.reward);nativeToast.note:SetText(last.note)end end
    return true
end
controller:RegisterEvent("CHAT_MSG_LOOT");controller:RegisterEvent("CHAT_MSG_SYSTEM")
controller:SetScript("OnEvent",function()
    local message,channel=arg1,event
    if not nativeNoticeActive()then return end
    local receipt=noticeData(message,channel);if not receipt then return end;local kind=receipt.kind
    if kind=="loot"then
        -- The bundled provider owns exact icons even while its state is not
        -- ready. Never substitute a name-only chat guess during that gap.
        local major,minor;if AZSX_GetAPIVersion then major,minor=AZSX_GetAPIVersion()end
        if structuredLoot()or(major==1 and type(minor)=="number"and minor>=6)then return end
        if not claimLoot()then return end
    end
    local now=GetTime();if mergeMarks(receipt,now)then return end
    if table.getn(nativeQueue)<16 then table.insert(nativeQueue,{kind=kind,amount=receipt.amount,total=receipt.total,lastAt=now,reward=receipt.reward,note=receipt.note,icon=noticeIcon(kind,message),readyAt=now+.25})end
end)
local function refreshNativeRewards()
    local now=GetTime();local previousWall=nativeWall;local dt=math.max(0,now-nativeWall);nativeWall=now
    local clock=AZS_GetRunTime and AZS_GetRunTime()
    if not nativeNoticeActive()or(type(clock)=="number"and nativeClock and clock+.5<nativeClock)then
        nativeQueue={};nativeCurrent=nil;nativeTrialScope=nil;nativeAnnouncementUntil=0;nativeAnnouncementStart=0;restoreLootPreference();if nativeToast then nativeToast:Hide()end;nativeClock=clock;return
    end
    nativeClock=clock
    refreshLootReceipts()
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
        if not nativeToast then nativeToast=discoveryToast("AZSXUINativeRewardToast","",nativeCurrent.icon)end
        nativeToast.title:SetText(nativeCurrent.kind=="marks"and"Marks earned"or"Loot received")
        nativeToast.icon:SetTexture(nativeCurrent.icon);nativeToast.reward:SetText(nativeCurrent.reward);nativeToast.reward:SetTextColor(1,1,1);nativeToast.note:SetText(nativeCurrent.note);nativeToast.note:SetTextColor(1,1,1)
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
            if not supplyToast then supplyToast=discoveryToast("AZSXSupplyDiscoveryToast",d.title,d.icon)end
            supplyToast:SetAlpha(1)
            supplyToast.title:SetText(d.title)
            local c=d.color;supplyToast.icon:SetTexture(d.icon);supplyToast.reward:SetText(d.reward);supplyToast.reward:SetTextColor(c[1],c[2],c[3])
            local note=AZSXUI_FormatBonus(durationLines(d.note))
            -- AZSX gold is already expressed in whole gold, not WoW copper.
            -- Reuse the native tooltip coin artwork without changing the value.
            local sold=type(d.gold)=="number"and d.gold>0 and string.find(d.note or""," gold$")
            if sold then note=string.gsub(d.note," gold$","")end
            supplyToast.note:SetText(note);supplyToast.note:SetTextColor(1,1,1)
            if sold then
                if not supplyToast.coin then
                    supplyToast.coin=supplyToast.content:CreateTexture(nil,"OVERLAY");supplyToast.coin:SetTexture("Interface\\MoneyFrame\\UI-GoldIcon");supplyToast.coin:SetWidth(13);supplyToast.coin:SetHeight(13)
                end
                supplyToast.coin:ClearAllPoints();supplyToast.coin:SetPoint("TOPLEFT",supplyToast.note,"TOPLEFT",supplyToast.note:GetStringWidth()+3,0);supplyToast.coin:Show()
            elseif supplyToast.coin then supplyToast.coin:Hide()end
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
            if not crystalToast then crystalToast=discoveryToast("AZSXCrystalDiscoveryToast",d.title,d.icon)end
            crystalToast.title:SetText(d.title);crystalToast.icon:SetTexture(d.icon)
            local c=d.titleColor or d.color;crystalToast.title:SetTextColor(c[1],c[2],c[3])
            crystalToast.reward:SetText(AZSXUI_FormatBonus(d.reward));crystalToast.reward:SetTextColor(1,1,1)
            crystalToast.note:SetText(durationLines(d.note))
        end
    end
    if crystalCurrent then crystalToast:Show()elseif crystalToast then crystalToast:Hide()end
end
-- Shared native casting-bar recipe. Progress and completion remain native/AZSX.
local function channelFrame(name)
    local f=CreateFrame("Frame",name,UIParent);f:SetWidth(320);f:SetHeight(54);f:EnableMouse(false)
    local track=CreateFrame("Frame",nil,f);track:SetWidth(300);track:SetHeight(18);track:SetPoint("CENTER",f,"CENTER",0,5)
    local bg=track:CreateTexture(nil,"BACKGROUND");bg:SetAllPoints(track);bg:SetTexture("Interface\\TargetingFrame\\UI-StatusBar");bg:SetVertexColor(.06,.05,.035,1)
    f.bar=CreateFrame("StatusBar",nil,track);f.bar:SetAllPoints(track);f.bar:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar")
    for _,part in ipairs({{22,40,"BOTTOMLEFT",-9},{40,216},{216,234,"BOTTOMRIGHT",9}})do
        local edge=f.bar:CreateTexture(nil,"OVERLAY");edge:SetTexture("Interface\\CastingBar\\UI-CastingBar-Border");edge:SetTexCoord(part[1]/256,part[2]/256,.25,.75);edge:SetHeight(32)
        if part[3]then edge:SetWidth(part[2]-part[1]);edge:SetPoint(part[3],track,part[3],part[4],-7)
        else edge:SetPoint("BOTTOMLEFT",track,"BOTTOMLEFT",9,-7);edge:SetPoint("BOTTOMRIGHT",track,"BOTTOMRIGHT",-9,-7)end
    end
    local labels=CreateFrame("Frame",nil,track);labels:SetAllPoints(track);labels:SetFrameLevel(f.bar:GetFrameLevel()+2)
    f.time=eventFont(labels,"GameFontHighlightSmall","FRIZQT__",11);f.time:SetPoint("RIGHT",track,"RIGHT",-5,0);f.time:SetWidth(78);f.time:SetJustifyH("RIGHT");f.time:SetTextColor(1,1,1)
    f.title=eventFont(labels,"GameFontNormal","FRIZQT__",11);f.title:SetPoint("LEFT",track,"LEFT",5,0);f.title:SetWidth(205);f.title:SetJustifyH("LEFT");f.title:SetTextColor(1,1,1)
    f.note=eventFont(f,"GameFontHighlightSmall","FRIZQT__",12);f.note:SetPoint("TOP",track,"BOTTOM",0,-8);f.note:SetWidth(300);f.note:SetJustifyH("CENTER");f.note:SetTextColor(1,.82,0)
    return f
end

local reviveFrame,revivePreference
local function restoreRevive()
    if AZSXUI_RestoreNativeHud then AZSXUI_RestoreNativeHud("revive")elseif revivePreference~=nil and AZS_IsHudShown and AZS_SetHudShown and not AZS_IsHudShown("revive")then AZS_SetHudShown("revive",revivePreference)end
    revivePreference=nil;if reviveFrame then reviveFrame:Hide()end
end
local function refreshRevive()
    local active=AZSXUI_GetHudAnchors and AZSXUI_GetHudAnchors()
    local r=active and AZS_GetRevive and AZS_GetRevive()
    if type(r)~="table"or not AZS_IsHudShown or not AZS_SetHudShown then restoreRevive();return end
    local down=r.down==1 or r.down==true
    local raising=type(r.reviving)=="string"and r.reviving~=""
    local fraction=down and r.share or r.rshare
    if(down or raising)and(type(fraction)~="number"or fraction~=fraction or fraction<0 or fraction>1)then restoreRevive();return end
    if AZSXUI_ClaimNativeHud then local value=AZSXUI_ClaimNativeHud("revive");if revivePreference==nil then revivePreference=value end
    else if revivePreference==nil then revivePreference=AZS_IsHudShown("revive")and true or false end;if AZS_IsHudShown("revive")then AZS_SetHudShown("revive",false)end end
    if not revivePreference or not(down or raising)then if reviveFrame then reviveFrame:Hide()end;return end
    if not reviveFrame then reviveFrame=channelFrame("AZSXUIResurrectionProgress");reviveFrame.bar:SetStatusBarColor(.3,.7,1);reviveFrame.bar:SetMinMaxValues(0,1)end
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
        local f=channelFrame("AZSXSaygeTrialProgress")
        f.bar:SetStatusBarColor(1,.7,0);f.bar:SetMinMaxValues(0,r.maximum);f.rendered=r.held
        ritualFrame=f
    end
    local identity=model.epoch..":"..model.own..":"..r.event
    if ritualFrame.identity~=identity then ritualFrame.identity=identity;ritualFrame.rendered=r.held end
    if r.allInside==false then ritualFrame.rendered=r.held end
    ritualFrame.target=r.held;ritualFrame.title:SetText(r.title)
    ritualFrame.time:SetText(string.format("%.1f s",math.max(0,r.maximum-r.held)/100))
    ritualFrame.note:SetText(r.allInside==false and "Waiting for all players"or"")
    ritualFrame:Show()
end
function AZSXRitualReadout()
    local own=supplyOwner();local r=own and own.ritual
    return r and r.event,r and r.held,r and r.inside,ritualFrame and ritualFrame:IsShown() or false
end
local function refreshBlessingToast()
    local b=blessingFor("player")
    local runPhase=AZS_GetPhase and AZS_GetPhase()
    if runPhase and runPhase~="playing" and runPhase~="choosing" and runPhase~="paused" and runPhase~="dead" then blessingSeen=nil end
    if not b then
        -- Missing/stale data hides the widget without replaying an acquired Fortune.
        if blessingToast then blessingToast:Hide()end
        return
    end
    local key=model.own..":"..tostring(b.event or(b.boss..":"..tostring(b.category)..":"..tostring(b.amount)))
    local phase=AZS_GetPhase and AZS_GetPhase()
    if phase and phase~="playing"and phase~="dead"then
        if blessingToast then blessingToast:Hide()end
        return
    end
    if key~=blessingSeen then
        blessingSeen=key;blessingUntil=GetTime()+6
        if not blessingToast then
            blessingToast=discoveryToast("AZSXDarkmoonBlessingToast",b.title,b.icon)
        end
        blessingToast:SetAlpha(1)
        blessingToast.title:SetText(b.title);blessingToast.icon:SetTexture(b.icon)
        blessingToast.reward:SetText(AZSXUI_FormatBonus(b.reward))
        blessingToast.reward:SetTextColor(1,1,1)
        blessingToast.note:SetText(durationLines(b.note))
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
function AZSXEventVisualReadout()
    return blessingToast and blessingToast:IsShown()or false,blessingToast and blessingToast:GetAlpha()or 0,supplyToast and supplyToast:IsShown()or false,supplyToast and supplyToast:GetAlpha()or 0
end

local defaults={supplies={x=0,y=0},fortune={x=0,y=0},crystals={x=0,y=0},sayge={x=0,y=0},revive={x=0,y=0},loot={x=0,y=0},marks={x=0,y=0}}
local anchor=CreateFrame("Frame","AZSXUIEventAnchor",UIParent)
anchor:SetWidth(1);anchor:SetHeight(1);anchor:EnableMouse(false)
local function layoutSettings()
    if type(AZSXUIDB)~="table"then AZSXUIDB={}end
    if type(AZSXUIDB.widgets)~="table"then AZSXUIDB.widgets={}end
    if AZSXUIDB.widgetLayoutVersion~=2 then
        local old={supplies=-140,fortune=-260,sayge=-75}
        for kind,y in pairs(old)do local p=AZSXUIDB.widgets[kind];if type(p)=="table"and type(p.y)=="number"then p.y=p.y-y end end
        AZSXUIDB.widgetLayoutVersion=2
    end
    if type(AZSXUIDB.eventAnchor)~="table"then AZSXUIDB.eventAnchor={}end
    if AZSXUIDB.eventAnchorLayoutVersion~=1 then
        local p=AZSXUIDB.eventAnchor
        -- Migrate only the previous centered default; retain custom placements.
        if (p.x==nil or p.x==0)and p.y==-85 then p.y=-125 end
        AZSXUIDB.eventAnchorLayoutVersion=1
    end
    return AZSXUIDB.eventAnchor
end
local function number(value,minimum,maximum,fallback)
    return type(value)=="number"and value==value and value>=minimum and value<=maximum and value or fallback
end
local function preference(kind)
    layoutSettings()
    if type(AZSXUIDB.widgets[kind])~="table"then AZSXUIDB.widgets[kind]={}end
    local p=AZSXUIDB.widgets[kind];local d=defaults[kind]
    p.x=number(p.x,-4096,4096,d.x);p.y=number(p.y,-4096,4096,d.y)
    p.scale=number(p.scale,.5,2,1);p.opacity=number(p.opacity,0,1,1)
    if type(p.visible)~="boolean"then p.visible=true end
    return p
end
local function layout()
    local p=layoutSettings();p.x=number(p.x,-4096,4096,0);p.y=number(p.y,-4096,4096,-125)
    local top=p.y;if p.x==0 and p.y==-125 and AZSXUI_GetEncounterOffset then top=-AZSXUI_GetEncounterOffset()end
    local signature=p.x..":"..top
    if anchor.azsxPosition~=signature then anchor:ClearAllPoints();anchor:SetPoint("TOP",UIParent,"TOP",p.x,top);anchor.azsxPosition=signature end
    local offset=0
    for _,entry in ipairs({{reviveFrame,"revive",54},{ritualFrame,"sayge",54},{supplyToast,"supplies",88},{blessingToast,"fortune",88},{crystalToast,"crystals",88},{nativeToast,nativeCurrent and nativeCurrent.kind or "loot",88}})do
        local frame,kind,height=entry[1],entry[2],entry[3];local pref=preference(kind)
        if frame then
            if frame.azsxCompactToast then
                local measured=24;local contentHeight=0;local count=0
                for _,label in ipairs({frame.title,frame.reward,frame.note})do
                    local value=label.GetStringHeight and label:GetStringHeight()or label:GetHeight()
                    if label:GetText()~=""and type(value)=="number"and value==value and value>0 and value<1024 then contentHeight=contentHeight+value;count=count+1;label:Show()else label:Hide()end
                end
                contentHeight=contentHeight+math.max(0,count-1)*4;frame.content:SetHeight(contentHeight);height=math.max(80,math.ceil(measured+contentHeight));frame:SetHeight(height)
            end
            frame:SetScale(pref.scale);frame:SetAlpha(pref.opacity)
            if not pref.visible then frame:Hide()end
            if frame:IsShown()then
                local x,y=pref.x/pref.scale,(-offset+pref.y)/pref.scale
                local key=x..":"..y
                if frame.azsxPosition~=key then frame:ClearAllPoints();frame:SetPoint("TOP",anchor,"BOTTOM",x,y);frame.azsxPosition=key end
                offset=offset+height*pref.scale+12
            end
        end
    end
end
local function update()
    local snapshot=type(AZSX_GetAPIVersion)=="function"and AZSX_GetAPIVersion()==1 and type(AZSX_GetState)=="function"and AZSX_GetState(1)or nil
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
function AZSXUI_ConfigureWidget(kind,property,first,second)
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
    elseif property=="reset"then AZSXUIDB.widgets[kind]={}
    else return false end
    update();return true
end
function AZSXUI_SupplyVisualReadout()return supplyToast and supplyToast:IsShown()or false,supplyCurrent and supplyCurrent.event end
local last=-1
controller:SetScript("OnUpdate",function()
    local now=GetTime()
    if now-last>=.1 then last=now;update()else updateRitualBar(now)end
end)
