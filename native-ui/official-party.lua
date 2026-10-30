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
            elseif a[1]=="R" and table.getn(a)==5 and result.members[id] then
                local event,held,inside=integer(a[3],1,4294967295),integer(a[4],0,2000),integer(a[5],0,1)
                if result.members[id].ritual or not event or not held or not inside then return nil end
                result.members[id].ritual={event=event,held=held,inside=inside==1}
            elseif a[1]=="C" and table.getn(a)==9 and result.members[id] then
                local family,tier,left,index=integer(a[3],0,5),integer(a[4],1,3),integer(a[5],1,20),integer(a[6],0,31)
                local name,icon,text=unhex(a[7],128),unhex(a[8],96),unhex(a[9],256)
                if not family or not tier or not left or not index or not name or name=="" or string.find(name,"[%c]") or not icon or not string.find(icon,"^INV_[%w_]+$") or not text or string.find(text,"[%c]") or result.members[id].consumables[family] then return nil end
                for _,buff in pairs(result.members[id].consumables)do if buff.index==index then return nil end end
                result.members[id].consumables[family]={family=family,tier=tier,remaining=left,index=index,name=name,icon=icon,text=text}
            elseif a[1]=="D" and table.getn(a)==8 and result.members[id] then
                local event,rarity,gold=integer(a[3],1,1e12),integer(a[4],0,3),integer(a[5],0,1000000)
                local name,icon,text=unhex(a[6],128),unhex(a[7],96),unhex(a[8],256)
                if result.members[id].discovery or not event or not rarity or not gold or not name or name=="" or string.find(name,"[%c]") or not icon or not string.find(icon,"^INV_[%w_]+$") or not text or string.find(text,"[%c]") then return nil end
                result.members[id].discovery={event=event,rarity=rarity,gold=gold,name=name,icon=icon,text=text}
            elseif a[1]=="A" and table.getn(a)==14 and result.members[id] then
                local x,y,radius=integer(a[3],-1e8,1e8),integer(a[4],-1e8,1e8),integer(a[5],1,1e6)
                local zoom=integer(a[6],0,5);local radii={}
                for n=1,6 do local v=integer(a[6+n],1,1e9);if not v then return nil end;radii[n]=v/1000000 end
                local qx,qy=integer(a[13],-1e8,1e8),integer(a[14],-1e8,1e8)
                if not x or not y or not radius or not zoom or not qx or not qy or result.members[id].minimap then return nil end
                result.members[id].minimap={x=x/100,y=y/100,radius=radius/100,zoom=zoom,radii=radii,pingX=qx/100,pingY=qy/100}
            elseif a[1]=="P" and table.getn(a)==7 and result.members[id] then
                local event,kind=integer(a[3],1,1e12),integer(a[4],0,1)
                local x,y,left=integer(a[5],-1e8,1e8),integer(a[6],-1e8,1e8),integer(a[7],0,60)
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
    local wire=AZS_RUN and AZS_RUN.azcoop
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
    tooltip:AddLine("Lasts until replaced by another fortune.",0.8,0.8,0.8,true)
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
-- Common retained discovery widget. Feature data never owns its construction.
-- Azeroth Survivors' actual native panel recipe: panel gradient, thin
-- border, 10px radius, and its heading/muted palette. Native portrait masks
-- supply rounded retained regions; no stock WoW dialog or quickslot skins.
local asPanelTop={0.078,0.059,0.035,1}
local asPanelBottom={0.031,0.024,0.016,1}
local asBorder={0.427,0.333,0.145,1}
local asGold={0.973,0.824,0.416}
local asMuted={0.725,0.663,0.510}
local eventRegionSerial=0
local function panelRect(parent,w,h,x,y,top,bottom,layer)
    local t=parent:CreateTexture(nil,layer or "BACKGROUND");t:SetWidth(w);t:SetHeight(h);t:SetPoint("CENTER",parent,"CENTER",x,y);t:SetTexture(top[1],top[2],top[3],top[4])
    if bottom then t:SetTexture(1,1,1,1);t:SetGradientAlpha("VERTICAL",bottom[1],bottom[2],bottom[3],bottom[4],top[1],top[2],top[3],top[4])end
end
local function roundedPanelShape(parent,w,h,r,x,y,top,bottom,layer)
    panelRect(parent,w-2*r,h,x,y,top,bottom,layer)
    panelRect(parent,w,h-2*r,x,y,top,bottom,layer)
    for _,side in ipairs({-1,1})do for _,vertical in ipairs({-1,1})do
        eventRegionSerial=eventRegionSerial+1;local name="ASXEventCorner"..eventRegionSerial
        local t=parent:CreateTexture(name,layer or "BACKGROUND");t:SetWidth(2*r);t:SetHeight(2*r);t:SetPoint("CENTER",parent,"CENTER",x+side*(w/2-r),y+vertical*(h/2-r))
        SetPortraitToTexture(name,"Interface\\Buttons\\WHITE8X8")
        local c=vertical>0 and top or (bottom or top);t:SetVertexColor(c[1],c[2],c[3],1)
    end end
end
local function eventPanel(frame,w,h,r)
    roundedPanelShape(frame,w,h,r,0,0,asBorder)
    roundedPanelShape(frame,w-2,h-2,r-1,0,0,asPanelBottom,nil,"BORDER")
end
local function eventFont(parent,template,font,size)
    local text=parent:CreateFontString(nil,"OVERLAY",template);text:SetFont("Fonts\\"..font..".TTF",size,"");return text
end
local function discoveryToast(name,heading,texture)
    local toast=CreateFrame("Frame",name,UIParent)
    toast:SetWidth(388);toast:SetHeight(106);toast:SetPoint("TOP",UIParent,"TOP",0,-140);toast:EnableMouse(false);eventPanel(toast,388,106,10)
    local iconFrame=CreateFrame("Frame",nil,toast);iconFrame:SetWidth(52);iconFrame:SetHeight(52);iconFrame:SetPoint("LEFT",toast,"LEFT",19,0);eventPanel(iconFrame,52,52,6)
    toast.icon=iconFrame:CreateTexture(nil,"ARTWORK");toast.icon:SetWidth(46);toast.icon:SetHeight(46);toast.icon:SetPoint("CENTER",iconFrame,"CENTER",0,0);toast.icon:SetTexture(texture)
    toast.title=eventFont(toast,"GameFontNormal","MORPHEUS",23);toast.title:SetPoint("TOPLEFT",toast,"TOPLEFT",86,-13);toast.title:SetText(heading);toast.title:SetTextColor(asGold[1],asGold[2],asGold[3])
    toast.reward=eventFont(toast,"GameFontHighlight","FRIZQT__",17);toast.reward:SetPoint("TOPLEFT",toast.title,"BOTTOMLEFT",0,-7);toast.reward:SetWidth(286);toast.reward:SetJustifyH("LEFT")
    toast.note=eventFont(toast,"GameFontHighlightSmall","FRIZQT__",13);toast.note:SetPoint("TOPLEFT",toast.reward,"BOTTOMLEFT",0,-7);toast.note:SetWidth(286);toast.note:SetJustifyH("LEFT");toast.note:SetTextColor(asMuted[1],asMuted[2],asMuted[3])
    return toast
end
local supplyToast,supplyScope,supplyUntil,supplyCurrent=nil,nil,0,nil
local supplySeen,supplyQueue={},{}
local supplyColors={{1,1,1},{0.12,1,0},{0,0.44,0.87},{0.64,0.21,0.93}}
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
    return result,supplyToast and supplyToast:IsShown() or false,supplyCurrent and supplyCurrent.event
end
-- Native buttons cache their texture on PLAYER_AURAS_CHANGED. Cosmetic
-- metadata arrives separately, so refresh the retained native buttons once per
-- changed family/slot/icon identity, never once per countdown tick.
local supplyBuffSignature=nil
local function refreshSupplyBuffButtons()
    local own=supplyOwner();local parts={}
    if own then for family=0,5 do local b=own.consumables[family];if b then table.insert(parts,family..":"..b.index..":"..b.icon)end end end
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
            if not supplyToast then supplyToast=discoveryToast("ASXSupplyDiscoveryToast","Lost Supplies","Interface\\Icons\\"..d.icon)end
            supplyToast:SetAlpha(1)
            local c=supplyColors[d.rarity+1];supplyToast.icon:SetTexture("Interface\\Icons\\"..d.icon);supplyToast.reward:SetText(d.name);supplyToast.reward:SetTextColor(c[1],c[2],c[3])
            supplyToast.note:SetText(d.gold>0 and ("Sold for "..d.gold.." gold")or(d.text.." - 20 seconds"))
        end
    end
    if not supplyCurrent then if supplyToast then supplyToast:Hide()end;return end
    supplyToast:Show()
end
-- Retained minimap children use the exact native minimap center and view radius.
local eventPins,hoveredEventPin={},nil
local eventPinCount,mapSample,viewCenterX,viewCenterY,pendingNativePing=0,nil,nil,nil,false
local localPingX,localPingY,referencePingX,referencePingY
local function refreshEventPins()
    local own=supplyOwner();local map=own and own.minimap;local parent=getglobal("Minimap");local used=0
    if parent and parent:IsVisible() and map then
        for _,data in ipairs(own.markers)do
            if data.kind==1 or data.left>0 then
                used=used+1;local pin=eventPins[used]
                if not pin then
                    pin=CreateFrame("Frame","ASXEventPin"..used,parent);pin:SetWidth(20);pin:SetHeight(20);pin:EnableMouse(true)
                    pin.icon=pin:CreateTexture("ASXEventPin"..used.."Icon","ARTWORK");pin.icon:SetAllPoints(pin)
                    local rim=pin:CreateTexture(nil,"OVERLAY");rim:SetTexture("Interface\\Minimap\\MiniMap-TrackingBorder");rim:SetTexCoord(0,0.625,0,0.625);rim:SetWidth(26);rim:SetHeight(26);rim:SetPoint("CENTER",pin,"CENTER",0,0)
                    pin:SetScript("OnEnter",function()
                        local v=this.data;if not v then return end;hoveredEventPin=this;GameTooltip:SetOwner(this,"ANCHOR_LEFT");GameTooltip:ClearLines()
                        GameTooltip:AddLine(v.kind==0 and "Lost Supplies"or"Sayge's Trial",1,0.82,0)
                        GameTooltip:AddLine(v.kind==1 and "Available until completed"or(v.left.."s remaining"),1,1,1)
                        if this.outside then GameTooltip:AddLine("Beyond minimap range",0.8,0.8,0.8)end;GameTooltip:Show()
                    end)
                    pin:SetScript("OnLeave",function()if hoveredEventPin==this then hoveredEventPin=nil;GameTooltip:Hide()end end);eventPins[used]=pin
                end
                pin.data=data
                if pin.kind~=data.kind then
                    SetPortraitToTexture(pin.icon:GetName(),data.kind==0 and "Interface\\Icons\\INV_Crate_01"or blessingIcon);pin.kind=data.kind
                end
                setShown(pin,true)
            end
        end
    end
    eventPinCount=used
    for n=used+1,table.getn(eventPins)do local pin=eventPins[n];if hoveredEventPin==pin then hoveredEventPin=nil;GameTooltip:Hide()end;pin.data=nil;setShown(pin,false)end
end
-- Native projection at the current zoom. A supplies the native world ping
-- and all six radii; GetPingPosition supplies its live normalized projection.
-- Their inverse gives the live center without waiting for a metadata refresh.
local function layoutEventPins()
    if eventPinCount==0 then return end
    local own=supplyOwner();local map=own and own.minimap;local parent=getglobal("Minimap")
    if not map or not parent or not parent:IsVisible()then
        for n=1,eventPinCount do setShown(eventPins[n],false)end;mapSample=nil;return
    end
    if mapSample~=map then
        if referencePingX~=map.pingX or referencePingY~=map.pingY then localPingX=nil;localPingY=nil end
        referencePingX,referencePingY=map.pingX,map.pingY;mapSample=map;pendingNativePing=false
    end
    local zoom=parent.GetZoom and parent:GetZoom()or map.zoom
    if type(zoom)~="number"or zoom<0 or zoom>5 or zoom~=math.floor(zoom)then return end
    local radius=map.radii[zoom+1];local cx,cy=map.x,map.y
    if pendingNativePing and viewCenterX then cx,cy=viewCenterX,viewCenterY
    elseif parent.GetPingPosition then
        local px,py=parent:GetPingPosition()
        if type(px)=="number"and type(py)=="number"and px==px and py==py and math.abs(px)<1e8 and math.abs(py)<1e8 then cx=(localPingX or map.pingX)-2*radius*py;cy=(localPingY or map.pingY)+2*radius*px end
    end
    viewCenterX,viewCenterY=cx,cy
    local halfW,halfH=parent:GetWidth()/2,parent:GetHeight()/2
    local edge=math.max(0,1-11/math.max(1,math.min(halfW,halfH)))
    for n=1,eventPinCount do
        local pin=eventPins[n];local data=pin.data
        local x,y=-(data.y-cy)/radius,(data.x-cx)/radius
        local length=math.sqrt(x*x+y*y);pin.outside=length>1
        if length>edge and length>0 then x=x*edge/length;y=y*edge/length end
        x=x*halfW;y=y*halfH
        if pin.x~=x or pin.y~=y then pin:ClearAllPoints();pin:SetPoint("CENTER",parent,"CENTER",x,y);pin.x=x;pin.y=y end
        local alpha=pin.outside and 0.65 or 1;if pin.alpha~=alpha then pin:SetAlpha(alpha);pin.alpha=alpha end
        setShown(pin,true)
    end
end
function ASXEventPinsReadout()
    local result={};for n,pin in ipairs(eventPins)do if pin.asxShown then table.insert(result,{event=pin.data.event,kind=pin.data.kind,outside=pin.outside,x=pin.x,y=pin.y})end end;return result
end
-- Retained display only: native simulation owns presence, progress and completion.
local ritualFrame
local function refreshRitual()
    local own=supplyOwner();local r=own and own.ritual
    if not r or not r.inside then if ritualFrame then ritualFrame:Hide();ritualFrame.rendered=r and r.held or 0 end;return end
    if not ritualFrame then
        local f=CreateFrame("Frame","ASXSaygeTrialProgress",UIParent)
        f:SetWidth(306);f:SetHeight(66);f:SetPoint("TOP",UIParent,"TOP",0,-75);f:EnableMouse(false);eventPanel(f,306,66,10)
        f.title=eventFont(f,"GameFontNormal","MORPHEUS",22);f.title:SetPoint("TOPLEFT",f,"TOPLEFT",15,-10);f.title:SetText("Sayge's Trial");f.title:SetTextColor(asGold[1],asGold[2],asGold[3])
        f.note=eventFont(f,"GameFontHighlightSmall","FRIZQT__",13);f.note:SetPoint("TOPRIGHT",f,"TOPRIGHT",-15,-15);f.note:SetTextColor(asMuted[1],asMuted[2],asMuted[3])
        local track=CreateFrame("Frame",nil,f);track:SetWidth(276);track:SetHeight(14);track:SetPoint("BOTTOM",f,"BOTTOM",0,12);eventPanel(track,276,14,4)
        f.bar=CreateFrame("StatusBar",nil,track);f.bar:SetWidth(270);f.bar:SetHeight(8);f.bar:SetPoint("CENTER",track,"CENTER",0,0)
        f.bar:SetStatusBarTexture("Interface\\Buttons\\WHITE8X8");f.bar:SetStatusBarColor(0.82,0.61,0.23);f.bar:SetMinMaxValues(0,2000);f.rendered=r.held
        ritualFrame=f
    end
    local identity=model.epoch..":"..model.own..":"..r.event
    if ritualFrame.identity~=identity then ritualFrame.identity=identity;ritualFrame.rendered=r.held end
    ritualFrame.target=r.held;ritualFrame.note:SetText(math.floor(r.held/100).." / 20 sec")
    ritualFrame:Show()
end
function ASXRitualReadout()
    local own=supplyOwner();local r=own and own.ritual
    return r and r.event,r and r.held,r and r.inside,ritualFrame and ritualFrame:IsShown() or false
end
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
            blessingToast=discoveryToast("ASXDarkmoonBlessingToast","Sayge's Dark Fortune",blessingIcon)
        end
        blessingToast:SetAlpha(1)
        local c=blessingColors[b.rarity+1]
        blessingToast.reward:SetText("+"..blessingPercent(b.amount).." "..blessingLabels[b.category+1])
        blessingToast.reward:SetTextColor(c[1],c[2],c[3])
        blessingToast.note:SetText("Fortune of "..blessingNames[b.category+1])
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
local function refresh()
    refreshModel()
    refreshBlessingToast()
    refreshSupplyBuffButtons()
    refreshSupplyToast()
    refreshRitual()
    refreshEventPins()
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
-- Model/buff work stays at 4 Hz. Only retained pin geometry follows each frame;
-- hover handlers still validate live owner/connection/buff identity immediately.
observer:SetScript("OnUpdate", function()
    local now = GetTime()
    if now-last >= interval then
        last=now
    refresh()
    end
    layoutEventPins()
    updateRitualBar(now)
end)
-- A native user/party ping changes the projection's reference world point.
-- Rebase the observer only; never create, move or suppress a native ping.
observer:RegisterEvent("MINIMAP_PING")
observer:SetScript("OnEvent",function()
    local own=supplyOwner();local map=own and own.minimap;local parent=getglobal("Minimap")
    if map and parent and viewCenterX and type(arg2)=="number"and type(arg3)=="number"then
        local zoom=parent.GetZoom and parent:GetZoom()or map.zoom;local radius=map.radii[zoom+1]
        if radius then localPingX=viewCenterX+2*radius*arg3;localPingY=viewCenterY-2*radius*arg2;pendingNativePing=false;return end
    end
    pendingNativePing=true
end)
-- Read-only validation surface; never used as a simulation or membership authority.
function AzCoopOfficialPartyReadout()
    local result = {}
    for index, row in pairs(rows) do
        result[index] = {owner=row.owner, buffs=row.count, parent=row.parent:GetName(), attached=row.buttons[1]:GetParent()==row.parent, member=row.member, spells=row.spells, items=row.items}
    end
    return result, built, model and model.own, asxNativeActive, blessingFor("player")
end
