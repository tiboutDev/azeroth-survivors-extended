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
local observer = CreateFrame("Frame", "ASXOfficialPartyPresentation", UIParent)
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
            if a[1]~="F" and a[1]~="C" and a[1]~="R" and a[1]~="A" and a[1]~="P" and a[1]~="D" then table.insert(signature,lines[n])end
            local id=identity(a[2])
            if not id then return nil end
            if a[1]=="M" and table.getn(a)==4 then
                local slot=integer(a[3],0,4)
                local name=unhex(a[4],256)
                if not slot or slot<=lastSlot or result.members[id] or not name or name=="" or string.find(name,"[%c]") then return nil end
                lastSlot=slot
                local member={id=id,slot=slot,name=name,spells={},items={},consumables={},markers={}}
                result.members[id]=member;table.insert(result.ordered,member)
            elseif a[1]=="B" and table.getn(a)==7 and result.members[id] then
                local boss,category,rarity=integer(a[3],1,4294967295),integer(a[4],0,4),integer(a[5],0,4)
                local amount,best=integer(a[6],1,14000),integer(a[7],0,14000)
                if result.members[id].blessing or not boss or not category or not rarity or not amount or not best then return nil end
                result.members[id].blessing={boss=boss,category=category,rarity=rarity,amount=amount,best=best}
            elseif a[1]=="R" and (table.getn(a)==5 or table.getn(a)==6) and result.members[id] then
                local event,held,inside=integer(a[3],1,4294967295),integer(a[4],0,2000),integer(a[5],0,1)
                local allInside;if table.getn(a)==6 then allInside=integer(a[6],0,1)end
                if result.members[id].ritual or not event or not held or not inside or (table.getn(a)==6 and allInside==nil) then return nil end
                result.members[id].ritual={event=event,held=held,inside=inside==1,allInside=allInside and allInside==1}
            elseif a[1]=="C" and table.getn(a)==9 and result.members[id] then
                local family,tier,left,index=integer(a[3],0,6),integer(a[4],1,3),integer(a[5],1,60),integer(a[6],0,31)
                local name,icon,text=unhex(a[7],128),unhex(a[8],96),unhex(a[9],256)
                if not family or not tier or not left or not index or not name or name=="" or string.find(name,"[%c]") or not icon or not string.find(icon,"^INV_[%w_]+$") or not text or string.find(text,"[%c]") or result.members[id].consumables[family] then return nil end
                for _,buff in pairs(result.members[id].consumables)do if buff.index==index then return nil end end
                result.members[id].consumables[family]={family=family,tier=tier,remaining=left,index=index,name=name,icon=icon,text=text}
            elseif a[1]=="D" and (table.getn(a)==8 or table.getn(a)==9) and result.members[id] then
                local event,rarity,gold=integer(a[3],1,1e12),integer(a[4],0,3),integer(a[5],0,1000000)
                local quality=1
                if table.getn(a)==9 then quality=integer(a[9],1,2)end
                if not quality then return nil end
                local name,icon,text=unhex(a[6],128),unhex(a[7],96),unhex(a[8],256)
                if result.members[id].discovery or not event or not rarity or not gold or not name or name=="" or string.find(name,"[%c]") or not icon or not string.find(icon,"^INV_[%w_]+$") or not text or string.find(text,"[%c]") then return nil end
                result.members[id].discovery={event=event,rarity=rarity,quality=quality,gold=gold,name=name,icon=icon,text=text}
            elseif a[1]=="A" and table.getn(a)==12 and result.members[id] then
                local x,y=integer(a[3],-1e8,1e8),integer(a[4],-1e8,1e8)
                local radii={}
                for n=1,6 do local v=integer(a[4+n],1,1e9);if not v then return nil end;radii[n]=v/1000000 end
                local qx,qy=integer(a[11],-1e8,1e8),integer(a[12],-1e8,1e8)
                if not x or not y or not qx or not qy or result.members[id].minimap then return nil end
                result.members[id].minimap={x=x/100,y=y/100,radii=radii,pingX=qx/100,pingY=qy/100}
            elseif a[1]=="P" and table.getn(a)==7 and result.members[id] then
                local event,kind=integer(a[3],1,1e12),integer(a[4],0,1)
                local x,y,left=integer(a[5],-1e8,1e8),integer(a[6],-1e8,1e8),integer(a[7],0,90)
                if not event or not kind or not x or not y or not left or table.getn(result.members[id].markers)>=9 then return nil end
                for _,pin in ipairs(result.members[id].markers)do if pin.event==event and pin.kind==kind then return nil end end
                table.insert(result.members[id].markers,{event=event,kind=kind,x=x/100,y=y/100,left=left})
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
    local wire=AZS_RUN and AZS_RUN.asx
    local updated=AZS_RUN_UPDATED
    if type(wire)~="string" or type(updated)~="number" or GetTime()-updated>4 then model=nil;modelWire=nil;return end
    if wire~=modelWire then
        local nextModel=decode(wire)
        if not nextModel or not model or model.epoch~=nextModel.epoch or model.own~=nextModel.own or model.signature~=nextModel.signature then model=nextModel
        else
            for id,member in pairs(model.members)do member.fatigue=nextModel.members[id].fatigue;member.consumables=nextModel.members[id].consumables;member.discovery=nextModel.members[id].discovery;member.ritual=nextModel.members[id].ritual;member.minimap=nextModel.members[id].minimap;member.markers=nextModel.members[id].markers end
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
        local name="ASXOfficialParty"..index..kind..n
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
        local name = "ASXOfficialParty" .. index .. "Buff" .. n
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
    tooltip:ClearLines()
    tooltip:AddDoubleLine("Sayge's Dark Fortune of "..blessingNames[b.category+1],"",1,1,1,1,1,1)
    tooltip:AddLine("+"..blessingPercent(b.amount).." "..blessingLabels[b.category+1],0.12,1,0,true)
    if (b.category==1 or b.category==2) and model and model.ordered[2] then
        if b.amount<b.best then
            tooltip:AddDoubleLine("Your fortune","+"..blessingPercent(b.amount),0.75,0.75,0.75,0.75,0.75,0.75)
            tooltip:AddDoubleLine("Active party bonus","+"..blessingPercent(b.best),1,0.82,0,0.12,1,0)
            tooltip:AddLine("Only the strongest party fortune applies.",0.75,0.75,0.75,true)
        else tooltip:AddLine("Party bonus. Only the strongest fortune of this type applies.",1,0.82,0,true)end
    else tooltip:AddLine("Personal bonus.",0.75,0.75,0.75,true)end
    tooltip:AddLine("Until replaced by another fortune.",1,1,1,true)
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
local nativeSupplyTexture=GetPlayerBuffTexture
local function supplyOwner()
    return model and GetTime()-modelTime<=4 and model.members[model.own] or nil
end
local function supplyBuff(index)
    local own=supplyOwner()
    if not own or not nativeSupplyTexture then return nil end
    local raw=nativeSupplyTexture(index)
    if type(raw)~="string" then return nil end
    raw=string.gsub(string.lower(raw),string.char(92),"/");raw=string.gsub(raw,"%.blp$","")
    if raw~="interface/icons/temp" then return nil end
    for _,buff in pairs(own.consumables)do if buff.index==index then return buff end end
    return nil
end
if GetPlayerBuff then
    local native=GetPlayerBuff
    GetPlayerBuff=function(index,filter)
        local slot,permanent=native(index,filter)
        if slot and supplyBuff(slot) then return slot,0 end
        return slot,permanent
    end
end
if GetPlayerBuffTimeLeft then
    local native=GetPlayerBuffTimeLeft
    GetPlayerBuffTimeLeft=function(index)local buff=supplyBuff(index);if buff then return buff.remaining end;return native(index)end
end
if GetPlayerBuffTexture then
    local native=GetPlayerBuffTexture
    GetPlayerBuffTexture=function(index)local texture=native(index);local buff=supplyBuff(index);if buff then return "Interface\\Icons\\"..buff.icon end;return texture end
end
if GameTooltip and GameTooltip.SetPlayerBuff then
    local native=GameTooltip.SetPlayerBuff
    GameTooltip.SetPlayerBuff=function(tooltip,index)
        native(tooltip,index);local buff=supplyBuff(index)
        if buff then
            tooltip:ClearLines();tooltip:AddLine(buff.name,1,1,1)
            tooltip:AddLine(buff.text,1,1,1,true)
            local duration=SecondsToTime and SecondsToTime(buff.remaining) or (buff.remaining.." sec")
            tooltip:AddLine(duration.." remaining",1,1,1);tooltip:Show()
        end
    end
end
function ASXSupplyReadout()
    local own=supplyOwner();local result={}
    if own then for _,buff in pairs(own.consumables)do table.insert(result,{family=buff.family,tier=buff.tier,remaining=buff.remaining,index=buff.index,raw=nativeSupplyTexture and nativeSupplyTexture(buff.index),visible=supplyBuff(buff.index)~=nil})end end
    local shown,event=false,nil
    if ASXUI_SupplyVisualReadout then shown,event=ASXUI_SupplyVisualReadout()end
    return result,shown,event
end
-- Native buttons cache their texture on PLAYER_AURAS_CHANGED. Cosmetic
-- metadata arrives separately, so refresh the retained native buttons once per
-- changed family/slot/icon identity, never once per countdown tick.
local supplyBuffSignature=nil
local function refreshSupplyBuffButtons()
    local own=supplyOwner();local parts={}
    if own then for family=0,6 do local b=own.consumables[family];if b then table.insert(parts,family..":"..b.index..":"..b.icon)end end end
    local signature=table.concat(parts,"|")
    if signature==supplyBuffSignature then return end
    local previous=supplyBuffSignature;supplyBuffSignature=signature
    if signature=="" and (not previous or previous=="")then return end
    if not BuffButton_Update then return end
    local saved=this
    for n=0,23 do
        local button=getglobal("BuffButton"..n)
        if button then this=button;local ok=pcall(BuffButton_Update);if not ok then supplyBuffSignature=nil end end
    end
    this=saved
end
-- Read-only presentation bridge. ASXUI owns all minimap widgets.
-- No cache coordinates are copied here: ASXUI reads native AZS_CHESTS directly.
function ASX_GetMinimapData()
    refreshModel()
    local phase=AZS_GetPhase and AZS_GetPhase()
    if phase~="playing" and phase~="paused" and phase~="choosing" then return nil end
    local own=supplyOwner()
    if not own then return nil end
    if own.minimap and not own.minimap.identity then own.minimap.identity=model.epoch..":"..model.own end
    return own.minimap,own.markers
end
local function refresh()
    refreshModel()
    refreshSupplyBuffButtons()
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
-- Model/buff work stays at 4 Hz. ASXUI owns minimap geometry;
-- hover handlers still validate live owner/connection/buff identity immediately.
observer:SetScript("OnUpdate", function()
    local now = GetTime()
    if now-last >= interval then
        last=now
    refresh()
    end
end)
-- ASX owns all semantic strings, reward values, colors and authoritative progress.
function ASX_GetEventPresentation()
    refreshModel()
    local phase=AZS_GetPhase and AZS_GetPhase()
    if phase~="playing"and phase~="choosing"and phase~="paused"and phase~="dead"then return nil end
    local own=supplyOwner()
    if not own then return nil end
    local view={epoch=model.epoch,own=model.own,owner={}}
    local d=own.discovery
    if d then view.owner.discovery={event=d.event,title="Lost Supplies",icon="Interface\\Icons\\"..d.icon,reward=d.name,note=d.gold>0 and ("Sold for "..d.gold.." gold")or(d.text.." - 60 seconds"),color=({{1,1,1},{0.12,1,0}})[d.quality or 1]}end
    local b=blessingFor("player")
    if b then view.blessing={boss=b.boss,title="Sayge's Dark Fortune",icon=blessingIcon,reward="+"..blessingPercent(b.amount).." "..blessingLabels[b.category+1],note="Fortune of "..blessingNames[b.category+1],color={0.12,1,0}}end
    local ritual=own.ritual
    if ritual then view.owner.ritual={event=ritual.event,held=ritual.held,inside=ritual.inside,allInside=ritual.allInside,maximum=2000,title="Sayge's Trial",note=(ritual.allInside==false and "Waiting for all players - "or "")..math.floor(ritual.held/100).." / 20 sec"}end
    return view
end
-- Public ASX addon API v1. Snapshots are detached; callbacks carry no shared tables.
local apiRevision,apiSignature,apiCache,apiInput=0,nil,nil,nil
local apiHandlers,apiNextToken,apiNotified,apiDispatching={},0,0,false
local eventPresentation=ASX_GetEventPresentation
local function apiCopy(value)
    if type(value)~="table"then return value end
    local result={};for key,item in pairs(value)do result[key]=apiCopy(item)end;return result
end
local function apiEncode(value)
    if type(value)~="table"then local encoded=type(value)=="number"and string.format("%.17g",value)or tostring(value);return type(value)..":"..string.len(encoded)..":"..encoded end
    local keys={};for key in pairs(value)do table.insert(keys,key)end
    table.sort(keys,function(a,b)return tostring(a)<tostring(b)end)
    local parts={"{"};for _,key in ipairs(keys)do table.insert(parts,apiEncode(key));table.insert(parts,apiEncode(value[key]))end
    table.insert(parts,"}");return table.concat(parts)
end
local function apiState()
    refreshModel()
    local phase=AZS_GetPhase and AZS_GetPhase()or "unknown"
    local allowed=phase=="playing"or phase=="choosing"or phase=="paused"or phase=="dead"
    local own=allowed and supplyOwner()or nil
    local input=phase..":"..tostring(own~=nil)..":"..tostring(AZS_RUN_UPDATED)..":"..tostring(modelWire)
    if apiCache and input==apiInput then return apiCache end
    local state={api={major=1,minor=2},available=own~=nil,phase=phase,data={},texts={}}
    if own then
        state.session={epoch=model.epoch,recipientGUID=model.own}
        state.data={fortune=apiCopy(own.blessing),fatigue=apiCopy(own.fatigue),consumables=apiCopy(own.consumables),discovery=apiCopy(own.discovery),sayge=apiCopy(own.ritual),minimap=apiCopy(own.minimap),markers=apiCopy(own.markers)}
        if state.data.sayge then state.data.sayge.maximum=2000 end
        if state.data.discovery then local d=state.data.discovery;state.data.discovery={event=d.event,rarity=d.rarity,quality=d.quality,gold=d.gold}end
        local consumableTexts={}
        for family,c in pairs(state.data.consumables or{})do
            consumableTexts[family]={name=c.name,description=c.text,icon="Interface\\Icons\\"..c.icon}
            state.data.consumables[family]={family=c.family,tier=c.tier,remaining=c.remaining,index=c.index}
        end
        if state.data.minimap then state.data.minimap.identity=model.epoch..":"..model.own end
        local formatted=eventPresentation()
        local function textOnly(value)
            if not value then return nil end
            return {title=value.title,icon=value.icon,reward=value.reward,note=value.note,color=apiCopy(value.color)}
        end
        state.texts={supplies=textOnly(formatted.owner.discovery),fortune=textOnly(formatted.blessing),sayge=textOnly(formatted.owner.ritual),consumables=consumableTexts}
        if state.texts.supplies then state.texts.supplies.name=own.discovery.name;state.texts.supplies.description=own.discovery.text end
    else
        state.reason=not allowed and "outside_run"or(type(AZS_RUN_UPDATED)=="number"and GetTime()-AZS_RUN_UPDATED>4 and "stale"or"missing_or_invalid_payload")
    end
    local signature=apiEncode(state)
    if signature~=apiSignature then apiRevision=apiRevision+1;apiSignature=signature end
    state.revision=apiRevision
    state.updatedAt=own and modelTime or nil
    apiCache=state;apiInput=input;return state
end
function ASX_GetAPIVersion()return 1,2 end
function ASX_GetState(major)
    if major~=nil and major~=1 then return nil,"unsupported_version"end
    return apiCopy(apiState())
end
function ASX_RegisterStateHandler(callback)
    if type(callback)~="function"then return nil,"invalid_callback"end
    local count=0
    for _,existing in pairs(apiHandlers)do if existing==callback then return true end;count=count+1 end
    if count>=64 then return nil,"subscriber_limit"end
    apiNextToken=apiNextToken+1;apiHandlers[apiNextToken]=callback;return true
end
function ASX_UnregisterStateHandler(callback)
    for token,existing in pairs(apiHandlers)do if existing==callback then apiHandlers[token]=nil;return true end end
    return false
end
local apiPublishedRevision,apiPublishedReceived
local function apiNotify()
    if apiDispatching then return end
    local state=apiState()
    if state.revision~=apiPublishedRevision or state.updatedAt~=apiPublishedReceived then
        ASX_STATE=apiCopy(state);ASX_STATE_UPDATED=GetTime()
        apiPublishedRevision=state.revision;apiPublishedReceived=state.updatedAt
    end
    if state.revision==apiNotified then return end
    local revision=state.revision
    apiNotified=revision;apiDispatching=true
    local pending={};for token,callback in pairs(apiHandlers)do table.insert(pending,{token,callback})end
    table.sort(pending,function(a,b)return a[1]<b[1]end)
    for _,entry in ipairs(pending)do if apiHandlers[entry[1]]==entry[2]then pcall(entry[2],apiCopy(state))end end
    apiDispatching=false
end
local originalUpdate=observer:GetScript("OnUpdate")
observer:SetScript("OnUpdate",function()originalUpdate();apiNotify()end)
-- Private pre-v1 adapters remain compatible, but no longer return model aliases.
function ASX_GetEventPresentation()return apiCopy(eventPresentation())end
local oldMinimapData=ASX_GetMinimapData
function ASX_GetMinimapData()local map,pins=oldMinimapData();return apiCopy(map),apiCopy(pins)end
-- Read-only validation surface; never used as a simulation or membership authority.
function ASXOfficialPartyReadout()
    local result = {}
    for index, row in pairs(rows) do
        result[index] = {owner=row.owner, buffs=row.count, parent=row.parent:GetName(), attached=row.buttons[1]:GetParent()==row.parent, member=row.member, spells=row.spells, items=row.items}
    end
    return result, built, model and model.own, asxNativeActive, blessingFor("player")
end
