local asxNativeActive = 0 -- ASX_LOCAL_UI_GATE
if asxNativeActive~=1 then return end
-- Enrich existing official frames; native frame state and scripts remain owned by the game.
-- WoW 1.12-compatible Lua. Native aura templates; no gameplay writes.
local rows, built, hovered = {}, 0, nil
local interval, last = 0.25, -1
local function setShown(widget,wanted)
    if widget.asxShown==wanted then return end
    widget.asxShown=wanted
    if wanted then widget:Show()else widget:Hide()end
end
local limit = 16
local observer = CreateFrame("Frame", "AzCoopOfficialPartyPresentation", UIParent)
local function leave(button)
    if hovered == button then GameTooltip:Hide(); hovered = nil end
end
local function enter()
    local button = this
    if not button:IsVisible() or not UnitExists(button.unit)
        or not UnitIsConnected(button.unit) or UnitName(button.unit) ~= button.owner
        or UnitBuff(button.unit, button:GetID()) ~= button.texture then return end
    GameTooltip:SetOwner(button, "ANCHOR_RIGHT")
    GameTooltip:SetUnitBuff(button.unit, button:GetID())
    hovered = button
end
local model, modelWire, modelTime = nil, nil, -1
local ownBuildDefaultApplied=false
local function split(value, separator)
    local result = {}
    local start = 1
    while true do
        local at = string.find(value, separator, start, true)
        if not at then table.insert(result, string.sub(value,start)); break end
        table.insert(result,string.sub(value,start,at-1)); start=at+string.len(separator)
    end
    return result
end
local function unhex(value, max)
    if not value or string.len(value)>max*2 or math.mod(string.len(value),2)~=0 or string.find(value,"[^%x]") then return nil end
    return string.gsub(value,"%x%x",function(pair)return string.char(tonumber(pair,16))end)
end
local function integer(value, min, max)
    local n=tonumber(value)
    if not n or math.floor(n)~=n or n<min or n>max then return nil end
    return n
end
local function identity(value)
    return value and string.len(value)<=20 and string.find(value,"^[1-9]%d*$") and value or nil
end
local function expand(raw)
    if string.byte(raw,1)~=2 then return raw end
    if string.len(raw)<5 then return nil end
    local b1,b2,b3,b4=string.byte(raw,2),string.byte(raw,3),string.byte(raw,4),string.byte(raw,5)
    local count=b1+b2*256+b3*65536+b4*16777216
    if count<1 or count>131072 then return nil end
    local out,n,at={},0,6
    while n<count do
        local flags=string.byte(raw,at);if not flags then return nil end;at=at+1
        for bit=1,8 do
            if n==count then if flags~=0 then return nil end;break end
            if math.mod(flags,2)==0 then
                local value=string.sub(raw,at,at);if value==""then return nil end
                n=n+1;out[n]=value;at=at+1
            else
                local low,high=string.byte(raw,at),string.byte(raw,at+1);if not high then return nil end;at=at+2
                local word=low+high*256;local distance=math.mod(word,4096)+1;local length=math.floor(word/4096)+3
                if distance>n or n+length>count then return nil end
                for i=1,length do n=n+1;out[n]=out[n-distance]end
            end
            flags=math.floor(flags/2)
        end
    end
    if at~=string.len(raw)+1 then return nil end
    return table.concat(out)
end
local function decode(wire)
    local raw=unhex(wire,16384)
    if not raw then return nil end
    local text=expand(raw)
    if not text then return nil end
    local newline=string.find(text,"\n",1,true)
    if not newline then return nil end
    local lines=split(text,"\n")
    if table.getn(lines)>176 then return nil end
    local head=split(lines[1],"|")
    if table.getn(head)~=4 or head[1]~="1" or not integer(head[2],0,1e12) or not identity(head[3]) or not integer(head[4],0,1e9) then return nil end
    local result={epoch=head[2],own=head[3],members={},ordered={},signature=string.sub(text,newline+1)}
    local signature={}
    local lastSlot=-1
    for n=2,table.getn(lines) do
        if lines[n]~="" then
            local a=split(lines[n],"|")
            if a[1]~="F" then table.insert(signature,lines[n])end
            local id=identity(a[2])
            if not id then return nil end
            if a[1]=="M" and table.getn(a)==4 then
                local slot=integer(a[3],0,4)
                local name=unhex(a[4],256)
                if not slot or slot<=lastSlot or result.members[id] or not name or name=="" or string.find(name,"[%c]") then return nil end
                lastSlot=slot
                local member={id=id,slot=slot,name=name,spells={},items={}}
                result.members[id]=member;table.insert(result.ordered,member)
            elseif a[1]=="B" and table.getn(a)==7 and result.members[id] then
                local boss,category,rarity=integer(a[3],1,4294967295),integer(a[4],0,4),integer(a[5],0,4)
                local amount,best=integer(a[6],1,14000),integer(a[7],0,14000)
                if result.members[id].blessing or not boss or not category or not rarity or not amount or not best then return nil end
                result.members[id].blessing={boss=boss,category=category,rarity=rarity,amount=amount,best=best}
            elseif a[1]=="F" and table.getn(a)==4 and result.members[id] then
                local member=result.members[id]
                local stacks,seconds=integer(a[3],0,3),integer(a[4],0,60)
                if member.fatigue or not stacks or not seconds or (stacks==0 and seconds~=0) or (stacks>0 and seconds==0) then return nil end
                member.fatigue={stacks=stacks,seconds=seconds}
            elseif ((a[1]=="S" and (table.getn(a)==7 or table.getn(a)==11)) or (a[1]=="I" and table.getn(a)==7)) and result.members[id] then
                local itemId,rank=integer(a[3],1,4294967295),integer(a[4],1,255)
                local name,icon,text=unhex(a[5],256),unhex(a[6],160),unhex(a[7],a[1]=="S" and 8192 or 2048)
                if not itemId or not rank or not name or name=="" or string.find(name,"[%c]") or not icon or icon=="" or string.find(icon,"[^%w_./\\]") or not text or string.find(text,"[%z\001-\009\011-\031\127]") then return nil end
                local list=a[1]=="S" and result.members[id].spells or result.members[id].items
                if table.getn(list)>=16 then return nil end
                local tip=nil
                if a[1]=="S" and table.getn(a)==11 then
                    tip={}
                    for n=8,11 do
                        local value=unhex(a[n],512)
                        if not value or string.find(value,"[%c]")then return nil end
                        tip[n-7]=value
                    end
                end
                table.insert(list,{id=itemId,rank=rank,name=name,icon=icon,text=text,owner=id,tip=tip})
            else return nil end
        end
    end
    result.signature=table.concat(signature,"\n")
    if not result.members[result.own] then return nil end
    return result
end
local function refreshModel()
    local wire=AZS_RUN and AZS_RUN.azcoop
    local updated=AZS_RUN_UPDATED
    if type(wire)~="string" or type(updated)~="number" or GetTime()-updated>4 then model=nil;modelWire=nil;return end
    if wire~=modelWire then
        local nextModel=decode(wire)
        if not nextModel or not model or model.epoch~=nextModel.epoch or model.own~=nextModel.own or model.signature~=nextModel.signature then model=nextModel
        else
            for id,member in pairs(model.members)do member.fatigue=nextModel.members[id].fatigue end
        end
        modelWire=wire
    end
    modelTime=updated
    -- Apply the agreed coop default once. Subsequent manual native toggles are respected.
    if model and table.getn(model.ordered)>1 and not ownBuildDefaultApplied and AZS_SetHudShown then
        AZS_SetHudShown("build",false);ownBuildDefaultApplied=true
    end
end
local function memberFor(index, name)
    if not model or GetTime()-modelTime>4 then return nil end
    local remote={};local names={}
    for _,member in ipairs(model.ordered)do
        names[member.name]=(names[member.name] or 0)+1
        if member.id~=model.own then table.insert(remote,member)end
    end
    local member=remote[index]
    if member and member.name==name and names[name]==1 then return member end
end
local function buildEnter()
    local button=this
    local member=memberFor(button.index,UnitName(button.unit))
    if not button:IsVisible() or not member or member.id~=button.owner or not UnitIsConnected(button.unit) then return end
    local icon=button.data
    if not icon then return end
    GameTooltip:SetOwner(button,"ANCHOR_RIGHT");GameTooltip:ClearLines()
    if icon.tip and icon.tip[1]~="" then
        GameTooltip:AddDoubleLine(icon.name,icon.tip[1],1,1,1,0.5,0.5,0.5)
        if icon.tip[2]~=""then GameTooltip:AddLine(icon.tip[2],1,1,1)end
        GameTooltip:AddDoubleLine(icon.tip[3],icon.tip[4],1,1,1,1,1,1)
        if icon.text~=""then GameTooltip:AddLine(icon.text,1,210/255,0,true)end
    else
        GameTooltip:AddDoubleLine(icon.name,"Rank "..icon.rank,1,0.82,0,1,0.82,0)
        if icon.text~="" then GameTooltip:AddLine(icon.text,1,1,1,true)end
    end
    GameTooltip:Show();hovered=button
end
local function buildButtons(index,parent,kind,y)
    local buttons={}
    for n=1,16 do
        local name="AzCoopOfficialParty"..index..kind..n
        local button=CreateFrame("Button",name,parent,"PartyBuffButtonTemplate")
        button:SetWidth(15);button:SetHeight(15)
        button:SetPoint("TOPLEFT",parent,"TOPRIGHT",4+(n-1)*17,y)
        button.icon=getglobal(name.."Icon")
        local border=getglobal(name.."Border");if border then border:Hide()end
        button.index=index;button.unit="party"..index
        button:SetScript("OnEnter",buildEnter)
        button:SetScript("OnLeave",function()leave(this)end)
        button:SetScript("OnHide",function()leave(this)end)
        button:Hide();button.asxShown=false;buttons[n]=button
    end
    return buttons
end
local function updateBuild(row,index,owner)
    local member=owner and memberFor(index,owner) or nil
    row.member=member
    for _,kind in ipairs({"spells","items"})do
        for n,button in ipairs(row[kind])do
            local data=member and member[kind][n] or nil
            if button.data~=data then
                leave(button);button.data=data;button.owner=data and data.owner or nil
                if data then
                    local icon=data.icon
                    if not string.find(icon,"[\\/]")then icon="Interface\\Icons\\"..icon end
                    button.icon:SetTexture(icon)
                end
            end
            setShown(button,data~=nil)
        end
    end
end
local function construct(index, parent)
    local row = { parent = parent, buttons = {}, owner = nil, count = 0 }
    row.spells=buildButtons(index,parent,"Spell",0)
    row.items=buildButtons(index,parent,"Item",-18)
    -- Native debuffs and parent layout remain untouched.
    for n = 1, limit do
        local name = "AzCoopOfficialParty" .. index .. "Buff" .. n
        local button = CreateFrame("Button", name, parent, "PartyBuffButtonTemplate")
        button:SetID(n)
        button:SetWidth(15); button:SetHeight(15)
        button:SetPoint("TOPLEFT", parent, "TOPRIGHT", 4 + (n-1)*17, -36)
        button.unit = "party" .. index
        button.icon = getglobal(name .. "Icon")
        local border = getglobal(name .. "Border")
        if border then border:Hide() end -- native debuff-type border does not describe positive buffs
        button:SetScript("OnEnter", enter)
        button:SetScript("OnLeave", function() leave(this) end)
        button:SetScript("OnHide", function() leave(this) end)
        button:Hide();button.asxShown=false
        row.buttons[n] = button
    end
    row.overflow = parent:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
    row.overflow:SetPoint("LEFT", row.buttons[limit], "RIGHT", 2, 0)
    row.overflow:Hide();row.overflow.asxShown=false
    built = built + 1
    rows[index] = row
    return row
end
-- The server projects spell 15007 into the real harmful-aura channel.
-- Keep native buttons, UnitDebuff/GetPlayerBuff, layout and stack rendering.
-- Only replace ASX's description and freeze its native duration at host time.
local fatigueTexture="interface\\icons\\spell_shadow_deathscream"
local function isFatigueTexture(texture)
    if type(texture)~="string" then return false end
    local path=string.gsub(string.lower(texture),"/","\\")
    path=string.gsub(path,"%.blp$","")
    return path==fatigueTexture
end
local function fatigueFor(unit)
    if not model or GetTime()-modelTime>4 then return nil end
    local member
    if unit=="player" then member=model.members[model.own]
    else
        local _,_,index=string.find(unit or "","^party([1-4])$")
        if index then member=memberFor(tonumber(index),UnitName(unit))end
    end
    local data=member and member.fatigue
    if data and data.stacks>0 then return data end
end
local function fatigueDescription(tooltip,data)
    -- Title comes from the game's own spell data (and localization).
    local line=getglobal("GameTooltipTextLeft1")
    local title=line and line:GetText()
    if not title or title=="" then return end
    tooltip:ClearLines()
    tooltip:AddDoubleLine(title,data.stacks.." / 3",1,0.2,0.2,1,1,1)
    tooltip:AddLine("Damage and healing done reduced by "..(data.stacks*5).."%.",1,1,1,true)
    tooltip:AddLine("Next stack expires in "..data.seconds.."s of alive game time.",1,0.82,0,true)
    tooltip:Show()
end
if GetPlayerBuffTimeLeft and GetPlayerBuffTexture then
    local nativeTime=GetPlayerBuffTimeLeft
    GetPlayerBuffTimeLeft=function(index)
        local data=fatigueFor("player")
        if data and isFatigueTexture(GetPlayerBuffTexture(index))then return data.seconds end
        return nativeTime(index)
    end
end
if GameTooltip and GameTooltip.SetPlayerBuff and GetPlayerBuffTexture then
    local nativeTip=GameTooltip.SetPlayerBuff
    GameTooltip.SetPlayerBuff=function(tooltip,index)
        nativeTip(tooltip,index)
        local data=fatigueFor("player")
        if data and isFatigueTexture(GetPlayerBuffTexture(index))then fatigueDescription(tooltip,data)end
    end
end
if GameTooltip and GameTooltip.SetUnitDebuff and UnitDebuff then
    local nativeTip=GameTooltip.SetUnitDebuff
    GameTooltip.SetUnitDebuff=function(tooltip,unit,index,filter)
        nativeTip(tooltip,unit,index,filter)
        local data=fatigueFor(unit)
        if data and isFatigueTexture(UnitDebuff(unit,index,filter))then fatigueDescription(tooltip,data)end
    end
end
-- Native positive aura with an ASX description, plus one retained reward toast.
local blessingLabels={"Luck","Experience","Gold","Pickup Radius","Movement Speed"}
local blessingNames={"Luck","Wisdom","Wealth","Discovery","Swiftness"}
local blessingRarities={"Common","Uncommon","Rare","Epic","Legendary"}
local blessingColors={{1,1,1},{0.12,1,0},{0,0.44,0.87},{0.64,0.21,0.93},{1,0.5,0}}
local blessingIcon="Interface\\Icons\\INV_Misc_Orb_02"
local function isBlessingTexture(texture)
    if type(texture)~="string"then return false end
    return string.gsub(string.gsub(string.lower(texture),"/","\\"),"%.blp$","")==string.lower(blessingIcon)
end
local function blessingFor(unit)
    if not model or GetTime()-modelTime>4 then return nil end
    local member
    if unit=="player" then member=model.members[model.own]
    else
        local _,_,index=string.find(unit or "","^party([1-4])$")
        if index then member=memberFor(tonumber(index),UnitName(unit))end
    end
    return member and member.blessing
end
local function blessingPercent(amount)
    return string.format("%.0f%%",math.floor(amount/100+0.5))
end
local function blessingDescription(tooltip,b)
    local color=blessingColors[b.rarity+1]
    tooltip:ClearLines()
    tooltip:AddDoubleLine("Sayge's Dark Fortune of "..blessingNames[b.category+1],blessingRarities[b.rarity+1],color[1],color[2],color[3],color[1],color[2],color[3])
    tooltip:AddLine("+"..blessingPercent(b.amount).." "..blessingLabels[b.category+1],1,1,1,true)
    if (b.category==1 or b.category==2) and model and model.ordered[2] then
        if b.amount<b.best then tooltip:AddLine("A stronger party fortune takes precedence: +"..blessingPercent(b.best)..".",1,0.82,0,true)
        else tooltip:AddLine("Party bonus. Only the strongest fortune of this type applies.",1,0.82,0,true)end
    else tooltip:AddLine("Personal bonus.",1,0.82,0,true)end
    tooltip:AddLine("Lasts until replaced by the next boss fortune.",0.8,0.8,0.8,true)
    tooltip:AddLine("Persists through death. Does not stack.",0.8,0.8,0.8,true)
    tooltip:Show()
end
-- Native BuffButton_OnUpdate hides duration and blinking for untilCancelled=1.
if GetPlayerBuff and GetPlayerBuffTexture then
    local native=GetPlayerBuff
    GetPlayerBuff=function(index,filter)
        local slot,permanent=native(index,filter)
        if slot and slot>=0 and blessingFor("player")and isBlessingTexture(GetPlayerBuffTexture(slot))then return slot,1 end
        return slot,permanent
    end
end
if GetPlayerBuffTimeLeft and GetPlayerBuffTexture then
    local native=GetPlayerBuffTimeLeft
    GetPlayerBuffTimeLeft=function(index)
        if blessingFor("player")and isBlessingTexture(GetPlayerBuffTexture(index))then return 0 end
        return native(index)
    end
end
if GameTooltip and GameTooltip.SetPlayerBuff and GetPlayerBuffTexture then
    local native=GameTooltip.SetPlayerBuff
    GameTooltip.SetPlayerBuff=function(tooltip,index)
        native(tooltip,index)
        local b=blessingFor("player")
        if b and isBlessingTexture(GetPlayerBuffTexture(index))then blessingDescription(tooltip,b)end
    end
end
if GameTooltip and GameTooltip.SetUnitBuff and UnitBuff then
    local native=GameTooltip.SetUnitBuff
    GameTooltip.SetUnitBuff=function(tooltip,unit,index,filter)
        native(tooltip,unit,index,filter)
        local b=blessingFor(unit)
        if b and isBlessingTexture(UnitBuff(unit,index,filter))then blessingDescription(tooltip,b)end
    end
end
local blessingToast,blessingSeen,blessingUntil=nil,nil,0
local function refreshBlessingToast()
    local b=blessingFor("player")
    if not b then
        if not model then blessingSeen=nil end
        if blessingToast then blessingToast:Hide()end
        return
    end
    local key=model.epoch..":"..model.own..":"..b.boss
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
            blessingToast=CreateFrame("Frame","ASXDarkmoonBlessingToast",UIParent)
            blessingToast:SetWidth(390);blessingToast:SetHeight(94)
            blessingToast:SetPoint("TOP",UIParent,"TOP",0,-145)
            local bg=blessingToast:CreateTexture(nil,"BACKGROUND")
            bg:SetAllPoints(blessingToast);bg:SetTexture(0.04,0.02,0.08,0.9)
            local icon=blessingToast:CreateTexture(nil,"ARTWORK")
            icon:SetWidth(48);icon:SetHeight(48);icon:SetPoint("LEFT",blessingToast,"LEFT",16,0);icon:SetTexture(blessingIcon)
            blessingToast.title=blessingToast:CreateFontString(nil,"OVERLAY","GameFontNormal")
            blessingToast.title:SetPoint("TOPLEFT",blessingToast,"TOPLEFT",78,-13)
            blessingToast.title:SetText("SAYGE'S DARK FORTUNE")
            blessingToast.reward=blessingToast:CreateFontString(nil,"OVERLAY","GameFontNormalLarge")
            blessingToast.reward:SetPoint("TOPLEFT",blessingToast.title,"BOTTOMLEFT",0,-7)
            blessingToast.note=blessingToast:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
            blessingToast.note:SetPoint("TOPLEFT",blessingToast.reward,"BOTTOMLEFT",0,-7)
        end
        local c=blessingColors[b.rarity+1]
        blessingToast.reward:SetText("+"..blessingPercent(b.amount).." "..blessingLabels[b.category+1])
        blessingToast.reward:SetTextColor(c[1],c[2],c[3])
        blessingToast.note:SetText(blessingRarities[b.rarity+1].." - "..blessingNames[b.category+1].." - Replaces your previous fortune")
        blessingToast:SetAlpha(1);blessingToast:Show()
    end
    local remaining=blessingUntil-GetTime()
    if remaining<=0 then blessingToast:Hide()
    else blessingToast:SetAlpha(math.min(1,remaining));blessingToast:Show()end
end
local function refresh()
    refreshModel()
    refreshBlessingToast()
    for index = 1, 4 do
        local parent = getglobal("PartyMemberFrame" .. index)
        local row = rows[index]
        -- Allocate retained children only when this native party slot is in use.
        if parent and not row and parent:IsVisible() and UnitExists("party" .. index) then
            row = construct(index, parent)
        end
        if row then
            local unit = "party" .. index
            local active = parent:IsVisible() and UnitExists(unit) and UnitIsConnected(unit)
            local owner = active and UnitName(unit) or nil
            row.owner = owner; row.count = 0
            updateBuild(row,index,owner)
            for n = 1, limit do
                local button = row.buttons[n]
                local texture = active and UnitBuff(unit, n) or nil
                if button.owner ~= owner or button.texture ~= texture then
                    leave(button)
                    button.owner = owner; button.texture = texture
                    if texture then button.icon:SetTexture(texture) end
                end
                if texture then row.count = row.count + 1 end
                setShown(button,texture~=nil)
            end
            local extra = 0
            if active then
                for n = limit+1, 32 do if UnitBuff(unit, n) then extra = extra + 1 end end
            end
            if extra>0 and row.extra~=extra then row.overflow:SetText("+"..extra)end
            row.extra=extra;setShown(row.overflow,extra>0)
        end
    end
end
-- Native events can arrive every frame. Poll the latest state at a strict 4 Hz;
-- hover handlers still validate live owner/connection/buff identity immediately.
observer:SetScript("OnUpdate", function()
    local now = GetTime()
    if now-last < interval then return end
    last = now
    refresh()
end)
-- Read-only validation surface; never used as a simulation or membership authority.
function AzCoopOfficialPartyReadout()
    local result = {}
    for index, row in pairs(rows) do
        result[index] = {owner=row.owner, buffs=row.count, parent=row.parent:GetName(), attached=row.buttons[1]:GetParent()==row.parent, member=row.member, spells=row.spells, items=row.items}
    end
    return result, built, model and model.own, asxNativeActive, blessingFor("player")
end
