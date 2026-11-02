local azsxNativeActive = 0 -- AZSX_LOCAL_UI_GATE
if azsxNativeActive~=1 then return end
-- Enrich existing official frames; native frame state and scripts remain owned by the game.
-- WoW 1.12-compatible Lua. Native aura templates; no gameplay writes.
local rows, built, hovered = {}, 0, nil
local interval, last = 0.25, -1
local function setShown(widget,wanted)
    if widget.azsxShown==wanted then return end
    widget.azsxShown=wanted
    if wanted then widget:Show()else widget:Hide()end
end
local limit = 16
local observer = CreateFrame("Frame", "AZSXOfficialPartyPresentation", UIParent)
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
    if table.getn(lines)>198 then return nil end
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
            elseif a[1]=="L" and table.getn(a)==3 and id==result.own and result.members[id] then
                local count=integer(a[3],0,5)
                if count==nil or result.members[id].equipment then return nil end
                result.members[id].equipment={expected=count,items={}}
            elseif a[1]=="Q" and table.getn(a)==7 and id==result.own and result.members[id] then
                local sequence,rarity,isNew=integer(a[3],1,1e9),integer(a[4],0,5),integer(a[5],0,1)
                local name,icon=unhex(a[6],128),unhex(a[7],96)
                local member=result.members[id];member.loot=member.loot or{}
                if not sequence or not rarity or isNew==nil or not name or name==""or string.find(name,"[%c]")or not icon or not string.find(icon,"^INV_[%w_]+$")or table.getn(member.loot)>=16 then return nil end
                for _,drop in ipairs(member.loot)do if drop.event==sequence then return nil end end
                table.insert(member.loot,{event=sequence,rarity=rarity,isNew=isNew==1,name=name,icon=icon})
            elseif a[1]=="G" and table.getn(a)==7 and result.members[id] then
                local red,blue,green,yellow=integer(a[3],0,1000000),integer(a[4],0,1000000),integer(a[5],0,1000000),integer(a[6],0,1000000)
                local index=integer(a[7],0,32)
                if red==nil or blue==nil or green==nil or yellow==nil or index==nil or result.members[id].crystals then return nil end
                result.members[id].crystals={red=red,blue=blue,green=green,yellow=yellow,index=index,total=red+blue+green+yellow}
            elseif a[1]=="O" and table.getn(a)==4 and id==result.own and result.members[id] then
                local xp,gold=integer(a[3],0,1e10),integer(a[4],0,1e10)
                if (not xp and a[3]~="-")or(not gold and a[4]~="-")or result.members[id].economy then return nil end
                result.members[id].economy={xp=xp and xp/1000000 or nil,gold=gold and gold/1000000 or nil}
            elseif a[1]=="B" and (table.getn(a)==7 or table.getn(a)==8) and result.members[id] then
                local boss,category,rarity=integer(a[3],1,4294967295),integer(a[4],0,4),integer(a[5],0,4)
                local amount,best=integer(a[6],1,14000),integer(a[7],0,14000)
                if result.members[id].blessing or not boss or not category or not rarity or not amount or not best then return nil end
                local receipt=a[8]and integer(a[8],1,1e9)or nil
                if a[8]and not receipt then return nil end
                result.members[id].blessing={boss=boss,category=category,rarity=rarity,amount=amount,best=best,event=receipt}
            elseif a[1]=="V" and (table.getn(a)==12 or table.getn(a)==14) and id==result.own and result.members[id] then
                local event,kind,stage=integer(a[3],1,1e9),integer(a[4],0,1),integer(a[5],0,4)
                local held,maximum,inside=integer(a[6],0,200),integer(a[7],200,200),integer(a[8],0,1)
                local charge=integer(a[9],0,4294967295)
                local progress=integer(a[10],0,kind==1 and 120 or 0)
                local total=integer(a[11],kind==1 and 8 or 0,kind==1 and 120 or 0)
                local remaining=integer(a[12],0,0)
                local x,y
                if table.getn(a)==14 then
                    x,y=tonumber(a[13]),tonumber(a[14])
                    if not x or not y or x~=x or y~=y or math.abs(x)>=1e8 or math.abs(y)>=1e8 then return nil end
                end
                if not event or kind==nil or stage==nil or not held or not maximum or inside==nil or charge==nil or progress==nil or total==nil or remaining==nil or progress>total or result.members[id].soloEvent or (kind==0 and stage~=3 and charge~=0) or (kind~=0 and charge~=0) then return nil end
                result.members[id].soloEvent={event=event,kind=kind==0 and "banished_demon" or "chicken_escort",stage=stage,held=held,maximum=maximum,inside=inside==1,banishes=charge,progress=progress,total=total,x=x,y=y}
            elseif a[1]=="W" and table.getn(a)==6 and id==result.own and result.members[id] then
                local revision,kind=integer(a[3],1,1e9),integer(a[4],0,5)
                local nextTrigger,effect=integer(a[5],0,60000),integer(a[6],0,60000)
                if not revision or kind==nil or not nextTrigger or not effect or result.members[id].engineering then return nil end
                local kinds={[1]="net",[2]="rocket_boots",[4]="grenade",[5]="shrink_ray"}
                if not kinds[kind+1] then return nil end
                result.members[id].engineering={event=revision,kind=kinds[kind+1],nextTrigger=nextTrigger,remaining=effect}
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
            elseif ((a[1]=="S" and (table.getn(a)==7 or table.getn(a)==11)) or (a[1]=="I" and table.getn(a)==7) or (a[1]=="E"and(table.getn(a)==7 or table.getn(a)==8))) and result.members[id] then
                local itemId,rank=integer(a[3],1,4294967295),integer(a[4],1,255)
                local name,icon,text=unhex(a[5],256),unhex(a[6],160),unhex(a[7],a[1]=="S" and 8192 or 2048)
                if not itemId or not rank or not name or name=="" or string.find(name,"[%c]") or not icon or icon=="" or string.find(icon,"[^%w_./\\]") or not text or string.find(text,"[%z\001-\009\011-\031\127]") then return nil end
                local localItems=result.members[id].equipment
                if a[1]=="E" and (id~=result.own or not localItems or itemId~=table.getn(localItems.items)+1 or itemId>localItems.expected)then return nil end
                local list=a[1]=="E" and localItems.items or(a[1]=="S" and result.members[id].spells or result.members[id].items)
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
                local category=a[1]=="E"and a[8]or nil
                if category and category~="amulet"and category~="ring"and category~="trinket"then return nil end
                table.insert(list,{id=itemId,rank=rank,name=name,icon=icon,text=text,owner=id,tip=tip,category=category})
            else return nil end
        end
    end
    for _,member in ipairs(result.ordered)do
        if member.equipment and table.getn(member.equipment.items)~=member.equipment.expected then return nil end
    end
    result.signature=table.concat(signature,"\n")
    if not result.members[result.own] then return nil end
    return result
end
local function refreshModel()
    local wire=AZS_RUN and AZS_RUN.azsx
    local updated=AZS_RUN_UPDATED
    if type(wire)~="string" or type(updated)~="number" or GetTime()-updated>4 then model=nil;modelWire=nil;return end
    if wire~=modelWire then
        local nextModel=decode(wire)
        if not nextModel or not model or model.epoch~=nextModel.epoch or model.own~=nextModel.own or model.signature~=nextModel.signature then model=nextModel
        else
            for id,member in pairs(model.members)do member.fatigue=nextModel.members[id].fatigue;member.consumables=nextModel.members[id].consumables;member.discovery=nextModel.members[id].discovery;member.ritual=nextModel.members[id].ritual;member.soloEvent=nextModel.members[id].soloEvent;member.engineering=nextModel.members[id].engineering;member.minimap=nextModel.members[id].minimap;member.markers=nextModel.members[id].markers end
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
local function addItemEffects(body)
    local text=body.."\n";local start=1
    while start<=string.len(text)do
        local stop=string.find(text,"\n",start,true);if not stop then break end
        local line=string.sub(text,start,stop-1);start=stop+1
        if line=="" or string.find(line,"^Enchantment:") or string.find(line,"^Active enchantment:") then
            GameTooltip:AddLine(line,1,1,1,true)
        else
            local effects=line..", ";local from=1
            while from<=string.len(effects)do
                local last=string.find(effects,", ",from,true);if not last then break end
                GameTooltip:AddLine(string.sub(effects,from,last-1),1,1,1,true);from=last+2
            end
        end
    end
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
        if icon.text~="" then addItemEffects(icon.text)end
    end
    GameTooltip:Show();hovered=button
end
local function buildButtons(index,parent,kind,y)
    local buttons={}
    for n=1,16 do
        local name="AZSXOfficialParty"..index..kind..n
        local button=CreateFrame("Button",name,parent,"PartyBuffButtonTemplate")
        button:SetWidth(15);button:SetHeight(15)
        button:SetPoint("TOPLEFT",parent,"TOPRIGHT",4+(n-1)*17,y)
        button.icon=getglobal(name.."Icon")
        local border=getglobal(name.."Border");if border then border:Hide()end
        button.index=index;button.unit="party"..index
        button:SetScript("OnEnter",buildEnter)
        button:SetScript("OnLeave",function()leave(this)end)
        button:SetScript("OnHide",function()leave(this)end)
        button:Hide();button.azsxShown=false;buttons[n]=button
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
        local name = "AZSXOfficialParty" .. index .. "Buff" .. n
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
        button:Hide();button.azsxShown=false
        row.buttons[n] = button
    end
    row.overflow = parent:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
    row.overflow:SetPoint("LEFT", row.buttons[limit], "RIGHT", 2, 0)
    row.overflow:Hide();row.overflow.azsxShown=false
    built = built + 1
    rows[index] = row
    return row
end
-- The server projects spell 15007 into the real harmful-aura channel.
-- Keep native buttons, UnitDebuff/GetPlayerBuff, layout and stack rendering.
-- Only replace AZSX's description and freeze its native duration at host time.
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
-- Native positive aura with an AZSX description, plus one retained reward toast.
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
-- One reserved recipe-only native presentation row, never a gameplay aura.
local crystalIcon="Interface\\Icons\\INV_Misc_Gem_Variety_01"
local nativeCrystalTexture=GetPlayerBuffTexture
local function crystalOwner(unit)
    if not model or GetTime()-modelTime>4 then return nil end
    local member
    if unit=="player"then member=model.members[model.own]
    else local _,_,index=string.find(unit or "","^party([1-4])$");if index then member=memberFor(tonumber(index),UnitName(unit))end end
    return member and member.crystals
end
local function isCrystalPlaceholder(texture)
    if type(texture)~="string"then return false end
    local path=string.gsub(string.lower(texture),string.char(92),"/");path=string.gsub(path,"%.blp$","")
    return path=="interface/icons/temp"
end
local function crystalBuff(index)
    local data=crystalOwner("player")
    if data and data.index==index and index<32 and nativeCrystalTexture and isCrystalPlaceholder(nativeCrystalTexture(index))then return data end
end
local function crystalPresentation(c)
    local text={summary="Crystals collected: "..c.total,details={},footer="Persists through death. Ends with this run."}
    if c.total==0 then
        table.insert(text.details,"Find crystals across the map to gain permanent bonuses.")
    else
        for _,row in ipairs({
            {"red","Red",12,"Damage","ffff6060"},
            {"blue","Blue",7,"Cooldown recovery","ff609fff"},
            {"green","Green",15,"Maximum health","ff50d870"},
            {"yellow","Yellow",2,"Critical strike chance","ffffd050"},
        })do
            local count=c[row[1]]
            if count>0 then
                table.insert(text.details,"|cff1eff00+"..(count*row[3]).."%|r "..row[4].."  |c"..row[5].."("..count.." "..string.lower(row[2])..(count==1 and " crystal)" or " crystals)").."|r")
            end
        end
    end
    return text
end
local function crystalDescription(tooltip,c)
    local text=crystalPresentation(c)
    tooltip:ClearLines();tooltip:AddLine("Crystal Resonance",1,1,1)
    tooltip:AddLine(text.summary,.75,.75,.75)
    for _,line in ipairs(text.details)do tooltip:AddLine(line,1,1,1,true)end
    tooltip:AddLine(text.footer,.75,.75,.75,true);tooltip:Show()
end
if GetPlayerBuff then
    local native=GetPlayerBuff
    GetPlayerBuff=function(index,filter)local slot,permanent=native(index,filter);if slot and crystalBuff(slot)then return slot,1 end;return slot,permanent end
end
if GetPlayerBuffTimeLeft then
    local native=GetPlayerBuffTimeLeft
    GetPlayerBuffTimeLeft=function(index)if crystalBuff(index)then return 0 end;return native(index)end
end
if GetPlayerBuffTexture then
    local native=GetPlayerBuffTexture
    GetPlayerBuffTexture=function(index)if crystalBuff(index)then return crystalIcon end;return native(index)end
end
if UnitBuff then
    local native=UnitBuff
    UnitBuff=function(unit,index,filter)
        local texture,count=native(unit,index,filter);local data=crystalOwner(unit)
        if data and index==data.index+1 and isCrystalPlaceholder(texture)then return crystalIcon,count end
        return native(unit,index,filter)
    end
end
if GameTooltip and GameTooltip.SetPlayerBuff then
    local native=GameTooltip.SetPlayerBuff
    GameTooltip.SetPlayerBuff=function(tooltip,index)native(tooltip,index);local c=crystalBuff(index);if c then crystalDescription(tooltip,c)end end
end
if GameTooltip and GameTooltip.SetUnitBuff then
    local native=GameTooltip.SetUnitBuff
    GameTooltip.SetUnitBuff=function(tooltip,unit,index,filter)
        native(tooltip,unit,index,filter);local c=crystalOwner(unit)
        if c and index==c.index+1 and UnitBuff(unit,index,filter)==crystalIcon then crystalDescription(tooltip,c)end
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
function AZSXSupplyReadout()
    local own=supplyOwner();local result={}
    if own then for _,buff in pairs(own.consumables)do table.insert(result,{family=buff.family,tier=buff.tier,remaining=buff.remaining,index=buff.index,raw=nativeSupplyTexture and nativeSupplyTexture(buff.index),visible=supplyBuff(buff.index)~=nil})end end
    local shown,event=false,nil
    if AZSXUI_SupplyVisualReadout then shown,event=AZSXUI_SupplyVisualReadout()end
    return result,shown,event
end
-- Native buttons cache their texture on PLAYER_AURAS_CHANGED. Cosmetic
-- metadata arrives separately, so refresh the retained native buttons once per
-- changed family/slot/icon identity, never once per countdown tick.
local supplyBuffSignature=nil
local function refreshSupplyBuffButtons()
    local own=supplyOwner();local parts={}
    if own then for family=0,6 do local b=own.consumables[family];if b then table.insert(parts,family..":"..b.index..":"..b.icon)end end end
    if own and own.crystals then local c=own.crystals;table.insert(parts,"G:"..c.index..":"..c.total)end
    local signature=table.concat(parts,"|")
    if signature==supplyBuffSignature then return end
    local previous=supplyBuffSignature;supplyBuffSignature=signature
    if signature=="" and (not previous or previous=="")then return end
    if not BuffButton_Update then return end
    local saved=this
    for n=0,31 do
        local button=getglobal("BuffButton"..n)
        if button then this=button;local ok=pcall(BuffButton_Update);if not ok then supplyBuffSignature=nil end end
    end
    this=saved
end
-- Read-only presentation bridge. AZSXUI owns all minimap widgets.
-- No cache coordinates are copied here: AZSXUI reads native AZS_CHESTS directly.
function AZSX_GetMinimapData()
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
-- Model/buff work stays at 4 Hz. AZSXUI owns minimap geometry;
-- hover handlers still validate live owner/connection/buff identity immediately.
observer:SetScript("OnUpdate", function()
    local now = GetTime()
    if now-last >= interval then
        last=now
    refresh()
    end
end)
-- AZSX owns all semantic strings, reward values, colors and authoritative progress.
function AZSX_GetEventPresentation()
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
-- Public AZSX addon API v1. Snapshots are detached; callbacks carry no shared tables.
local apiRevision,apiSignature,apiCache,apiInput=0,nil,nil,nil
local apiHandlers,apiNextToken,apiNotified,apiDispatching={},0,0,false
local eventPresentation=AZSX_GetEventPresentation
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
    local state={api={major=1,minor=13},available=own~=nil,phase=phase,data={},texts={}}
    if own then
        state.session={epoch=model.epoch,recipientGUID=model.own}
        state.data={sharedEconomy=apiCopy(own.economy),crystals=apiCopy(own.crystals),fortune=apiCopy(own.blessing),fatigue=apiCopy(own.fatigue),consumables=apiCopy(own.consumables),discovery=apiCopy(own.discovery),sayge=apiCopy(own.ritual),soloEvent=apiCopy(own.soloEvent),engineering=apiCopy(own.engineering),minimap=apiCopy(own.minimap),markers=apiCopy(own.markers)}
        -- Retain v1.7's complete-pair guarantee for existing consumers.
        if own.economy and own.economy.xp~=nil and own.economy.gold~=nil then state.data.economy=apiCopy(own.economy)end
        state.data.loot={};local lootTexts={}
        for _,drop in ipairs(own.loot or{})do
            table.insert(state.data.loot,{event=drop.event,rarity=drop.rarity,isNew=drop.isNew})
            lootTexts[drop.event]={name=drop.name,icon=drop.icon}
        end
        local equipmentTexts
        if own.equipment then
            state.data.equipment={};equipmentTexts={}
            for slot,item in ipairs(own.equipment.items)do
                state.data.equipment[slot]={slot=slot,rank=item.rank,category=item.category}
                equipmentTexts[slot]={name=item.name,icon=item.icon,text=item.text}
            end
        end
        if state.data.sayge then state.data.sayge.maximum=2000 end
        if state.data.discovery then local d=state.data.discovery;state.data.discovery={event=d.event,rarity=d.rarity,quality=d.quality,gold=d.gold}end
        local crystalText
        if state.data.crystals then
            local c=state.data.crystals
            c.bonuses={damage=c.red*12,cooldownRecovery=c.blue*7,maximumHealth=c.green*15,areaOfEffect=0,criticalStrikeChance=c.yellow*2}
            local presentation=crystalPresentation(c)
            crystalText={title="Crystal Resonance",icon=crystalIcon,note=presentation.summary,details=presentation.details,footer=presentation.footer}
            crystalText.pickups={}
            local names={"Red","Blue","Green","Yellow"}
            local keys={"red","blue","green","yellow"}
            local gains={"+12% damage","+7% cooldown recovery","+15% maximum health","+2% critical strike chance"}
            local colors={{1,.25,.2},{.3,.65,1},{.12,1,0},{1,.82,0}}
            for i,key in ipairs(keys)do
                crystalText.pickups[key]={title=names[i].." Crystal collected",reward=gains[i],note="Crystal Resonance - lasts until the end of the run.",icon=crystalIcon,color={.12,1,0},titleColor=colors[i]}
            end
        end
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
        state.texts={loot=lootTexts,equipment=equipmentTexts,crystals=crystalText,supplies=textOnly(formatted.owner.discovery),fortune=textOnly(formatted.blessing),sayge=textOnly(formatted.owner.ritual),consumables=consumableTexts}
        if state.data.soloEvent then
            local e=state.data.soloEvent
            if e.kind=="banished_demon" then
                local notes={"Find the bound demon","Break the binding together","Defeat the released demon","Demon defeated","The binding faded"}
                local note=notes[e.stage+1]
                if e.stage==1 and not e.inside then note=note.." (waiting for all players)" end
                state.texts.soloEvent={title="Banished Demon",note=note,icon="Interface\\Icons\\Spell_Shadow_Cripple",reward="+1 Banish",duration="Until the end of the run"}
            else
                local notes={"Find the damaged chicken","Restart the chicken","Accompany the chicken to its teleporter","Extraction complete","Escort ended"}
                local note=notes[e.stage+1]
                if e.stage==2 and not e.inside then note=note.." (waiting for all players)" end
                state.texts.soloEvent={title="Mechanical Chicken Escort",note=note,icon="Interface\\Icons\\INV_Misc_EngGizmos_06",reward="Engineering gadget equipped",duration="Until the end of the run"}
            end
        end
        if state.data.engineering then
            local e=state.data.engineering
            local names={net="Gnomish Net",rocket_boots="Goblin Rocket Boots",grenade="Goblin Stun Grenade",shrink_ray="Gnomish Shrink Ray"}
            local icons={net="Ability_Ensnare",rocket_boots="Spell_Fire_Fire",grenade="Spell_Fire_SelfDestruct",shrink_ray="Spell_Ice_MagicDamage"}
            local descriptions={net="Roots up to three nearby ordinary enemies.",rocket_boots="A two-second movement speed burst.",grenade="Stuns up to three ordinary enemies within three yards for two seconds. Deals no damage.",shrink_ray="Changes appearance for six seconds. Cosmetic only."}
            state.texts.engineering={title=names[e.kind],icon="Interface\\Icons\\"..icons[e.kind],description=descriptions[e.kind],reward=names[e.kind],duration="Until the end of the run"}
        end
        if own.economy then
            state.data.cooperativeRules={id="cooperative_rules",permanent=true,priority=1,areaBonusShare=0.5}
            state.texts.cooperativeRules={title="Cooperative Rules",icon="Interface\\Icons\\Spell_Holy_PrayerofFortitude",
                rule="Area and range bonuses are 50% effective.",
                notes={"The stats panel shows effective values.","Item and reward descriptions may show base bonuses."}}
        end
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
function AZSX_GetAPIVersion()return 1,13 end
function AZSX_GetState(major)
    if major~=nil and major~=1 then return nil,"unsupported_version"end
    return apiCopy(apiState())
end
function AZSX_RegisterStateHandler(callback)
    if type(callback)~="function"then return nil,"invalid_callback"end
    local count=0
    for _,existing in pairs(apiHandlers)do if existing==callback then return true end;count=count+1 end
    if count>=64 then return nil,"subscriber_limit"end
    apiNextToken=apiNextToken+1;apiHandlers[apiNextToken]=callback;return true
end
function AZSX_UnregisterStateHandler(callback)
    for token,existing in pairs(apiHandlers)do if existing==callback then apiHandlers[token]=nil;return true end end
    return false
end
local apiPublishedRevision,apiPublishedReceived
local function apiNotify()
    if apiDispatching then return end
    local state=apiState()
    if state.revision~=apiPublishedRevision or state.updatedAt~=apiPublishedReceived then
        AZSX_STATE=apiCopy(state);AZSX_STATE_UPDATED=GetTime()
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
function AZSX_GetEventPresentation()return apiCopy(eventPresentation())end
local oldMinimapData=AZSX_GetMinimapData
function AZSX_GetMinimapData()local map,pins=oldMinimapData();return apiCopy(map),apiCopy(pins)end
-- Read-only validation surface; never used as a simulation or membership authority.
function AZSXOfficialPartyReadout()
    local result = {}
    for index, row in pairs(rows) do
        result[index] = {owner=row.owner, buffs=row.count, parent=row.parent:GetName(), attached=row.buttons[1]:GetParent()==row.parent, member=row.member, spells=row.spells, items=row.items}
    end
    return result, built, model and model.own, azsxNativeActive, blessingFor("player")
end

-- Informational aura: no native buff slot, cancellation or gameplay mutation.
-- Exists without AZSXUI. Other consumers can render the detached API record.
local cooperativeButton,cooperativeDock,cooperativeAnchor
local function cooperativeRestore()
    local e=cooperativeAnchor
    if e then
        local a,b,c,x,y=e.frame:GetPoint(1)
        if a=="TOPRIGHT"and b==cooperativeButton and c=="TOPLEFT"and x==-5 and y==0 then
            e.frame:ClearAllPoints();for _,point in ipairs(e.points)do e.frame:SetPoint(unpack(point))end
        end
    end
    cooperativeAnchor=nil
end
local function cooperativeRefresh()
    local state=apiState()
    local text=state.available and state.data.cooperativeRules and state.texts.cooperativeRules
    if not text then
        cooperativeRestore()
        if cooperativeButton then
            setShown(cooperativeButton,false)
            if cooperativeButton.hovered then GameTooltip:Hide();cooperativeButton.hovered=false end
        end
        return nil
    end
    if not cooperativeButton then
        local b=CreateFrame("Button","AZSXCooperativeRulesBuff",UIParent);cooperativeButton=b
        b:SetWidth(32);b:SetHeight(32)
        b.icon=b:CreateTexture(nil,"ARTWORK");b.icon:SetAllPoints(b);b.icon:SetTexture(text.icon)
        b:SetScript("OnEnter",function()
            local current=apiState()
            local t=current.available and current.data.cooperativeRules and current.texts.cooperativeRules
            if not t then GameTooltip:Hide();return end
            b.hovered=true;GameTooltip:SetOwner(b,"ANCHOR_RIGHT");GameTooltip:ClearLines()
            GameTooltip:AddLine(t.title,1,1,1);GameTooltip:AddLine(t.rule,1,1,1,true)
            GameTooltip:AddLine(" ")
            for _,note in ipairs(t.notes)do GameTooltip:AddLine(note,.65,.65,.65,true)end
            GameTooltip:Show()
        end)
        b:SetScript("OnLeave",function()b.hovered=false;GameTooltip:Hide()end)
    end
    if not cooperativeDock then
        local first=getglobal("BuffButton0")
        -- Follow a cosmetic reorder to its leading native button. Anchoring
        -- to an interior button would create a cycle with reordered auras.
        local seen={}
        while first and type(first.GetPoint)=="function"do
            if seen[first]then first=nil;break end
            seen[first]=true
            local _,parent=first:GetPoint(1)
            if type(parent)=="string"then parent=getglobal(parent)end
            local name=parent and parent.GetName and parent:GetName()
            if type(name)~="string"or not string.find(name,"^BuffButton%d+$")then break end
            first=parent
        end
        if first and first:IsShown()and type(first.GetNumPoints)=="function"and first:GetNumPoints()>0 then
            local _,parent=first:GetPoint(1)
            if parent~=cooperativeButton then
                cooperativeRestore()
                local points={};for i=1,first:GetNumPoints()do points[i]={first:GetPoint(i)}end
                cooperativeAnchor={frame=first,points=points}
                cooperativeButton:ClearAllPoints();for _,point in ipairs(points)do cooperativeButton:SetPoint(unpack(point))end
                first:ClearAllPoints();first:SetPoint("TOPRIGHT",cooperativeButton,"TOPLEFT",-5,0)
            end
        else
            cooperativeRestore();cooperativeButton:ClearAllPoints();cooperativeButton:SetPoint("TOPRIGHT",UIParent,"TOPRIGHT",-205,-13)
        end
    end
    setShown(cooperativeButton,true);return cooperativeButton
end
-- A single explicit owner borrows placement; release restores native fallback.
function AZSX_SetCooperativeRulesDocked(owner,docked)
    if type(owner)~="string"or owner==""then return nil end
    if docked then
        if cooperativeDock and cooperativeDock~=owner then return nil end
        cooperativeRestore();cooperativeDock=owner;return cooperativeRefresh()
    elseif cooperativeDock==owner then
        cooperativeDock=nil;return cooperativeRefresh()
    end
end
local cooperativeUpdate=observer:GetScript("OnUpdate")
observer:SetScript("OnUpdate",function()cooperativeUpdate();cooperativeRefresh()end)
