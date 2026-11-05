-- AZSXUI minimap renderer. Native gameplay, cache purchases and events stay native.
-- Requires the optional AZSX projection bridge; never guesses a panned map center.
local controller=CreateFrame("Frame","AZSXUIMinimapController",UIParent)
local blessingIcon="Interface\\Icons\\INV_Misc_Orb_02"
local function setShown(widget,wanted)
    if widget.azsxShown==wanted then return end
    widget.azsxShown=wanted
    if wanted then widget:Show()else widget:Hide()end
end
local function liveZoom(parent)
    if not parent or type(parent.GetZoom)~="function" then return nil end
    local zoom=parent:GetZoom()
    if type(zoom)~="number" or zoom~=zoom or zoom<0 or zoom>5 or zoom~=math.floor(zoom)then return nil end
    return zoom
end
local function finite(value)
    return type(value)=="number" and value==value and math.abs(value)<1e8
end
-- Read a single consistent cache page. The HUD supports at most 128 cache pins.
local function cacheRows()
    if type(AZS_GetWorldAnchors)~="function" then return AZS_CHESTS or {} end
    local page=AZS_GetWorldAnchors({kinds={"cache"},states={"present"}},nil,128)
    local player=AZS_GetPlayerPosition and AZS_GetPlayerPosition()
    if not page or not player or not player.runId or not player.spaceId or player.mapId==nil
        or page.runId~=player.runId or page.mapId~=player.mapId then return {} end
    local caches={}
    for _,anchor in ipairs(page.entries or {})do
        if anchor.kind=="cache" and anchor.positionAvailable and anchor.spaceId==player.spaceId then
            table.insert(caches,{id=anchor.cacheId,x=anchor.x,y=anchor.y})
        end
    end
    return caches
end

local function source(includeMarkers)
    local phase=AZS_GetPhase and AZS_GetPhase()
    if phase~="playing" and phase~="choosing" and phase~="paused" then return nil end
    if type(AZSXUIDB)=="table" and AZSXUIDB.pins==false then return nil end
    if type(AZSX_GetAPIVersion)~="function"or AZSX_GetAPIVersion()~=1 or type(AZSX_GetState)~="function"then return nil end
    local snapshot=AZSX_GetState(1)
    if not snapshot or not snapshot.available then return nil end
    local map,custom=snapshot.data.minimap,snapshot.data.markers
    if not map then return nil end
    if not includeMarkers then return map end
    local markers={}
    for i=1,math.min(table.getn(custom or {}),9)do table.insert(markers,custom[i])end
    local escort=snapshot.data.soloEvent
    if type(escort)=="table" and (escort.kind=="chicken_escort" or escort.kind=="banished_demon") and finite(escort.event) and escort.event>0
        and finite(escort.x) and finite(escort.y) and finite(escort.stage) and escort.stage>=0 and escort.stage<=2 then
        local text=snapshot.texts and snapshot.texts.soloEvent
        table.insert(markers,{event=escort.event,kind=escort.kind=="banished_demon" and 5 or 4,x=escort.x,y=escort.y,left=0,
            name=text and text.title or "Mechanical Chicken",note=text and text.note or "Escort to extraction"})
    end
    local caches=cacheRows()
    for i=1,math.min(table.getn(caches),128)do
        local cache=caches[i]
        if type(cache)=="table" and finite(cache.x) and finite(cache.y) and type(cache.id)=="number" and cache.id>0 and cache.id<=9007199254740991 and cache.id==math.floor(cache.id) then
            table.insert(markers,{event=cache.id,kind=2,x=cache.x,y=cache.y,left=0})
        end
    end
    local lairs=type(AZS_GetLairs)=="function"and AZS_GetLairs()
    if type(lairs)~="table"then
        local flagged=type(AZS_GetFlaggedLair)=="function"and AZS_GetFlaggedLair()
        lairs=flagged and{flagged}or{}
    end
    local seen={}
    for i=1,math.min(table.getn(lairs),128)do local lair=lairs[i]
        local available=type(lair)=="table"and(lair.active~=false or finite(lair.left)and lair.left>=0 and lair.left<=60)
        if available and finite(lair.x)and finite(lair.y)and finite(lair.index)and lair.index>=1 and lair.index==math.floor(lair.index)and not seen[lair.index]then
            seen[lair.index]=true
            table.insert(markers,{event=lair.index,kind=3,x=lair.x,y=lair.y,left=0,name=type(lair.name)=="string"and lair.name or"Guardian",active=lair.active})
        end
    end
    return map,markers
end
-- Retained minimap children use the exact native minimap center and view radius.
local eventPins,hoveredEventPin={},nil
local nativeFlagPreference
local function nativeFlag(replacing)
    if type(AZS_SetHudShown)~="function"or type(AZS_IsHudShown)~="function"then return end
    if replacing then
        if nativeFlagPreference==nil then nativeFlagPreference=AZS_IsHudShown("flag") end
        if AZS_IsHudShown("flag") then AZS_SetHudShown("flag",false)end
    elseif nativeFlagPreference~=nil then
        AZS_SetHudShown("flag",nativeFlagPreference);nativeFlagPreference=nil
    end
end
local sessionIdentity
local eventPinCount,mapSample,viewCenterX,viewCenterY,pendingNativePing=0,nil,nil,nil,false
local localPingX,localPingY,referencePingX,referencePingY
local regroup=true
local function refreshEventPins()
    local map,markers=source(true);local parent=getglobal("Minimap");local used=0
    if parent and parent:IsVisible() and map then
        for _,data in ipairs(markers or {})do
            if data.kind==1 or data.kind==2 or data.kind==3 or data.kind==4 or data.kind==5 or data.left>0 then
                if used>=265 then break end
                used=used+1;local pin=eventPins[used]
                if not pin then
                    pin=CreateFrame("Frame","AZSXEventPin"..used,parent);pin:SetWidth(20);pin:SetHeight(20);pin:EnableMouse(true)
                    pin.icon=pin:CreateTexture("AZSXEventPin"..used.."Icon","ARTWORK");pin.icon:SetAllPoints(pin)
                    local rim=pin:CreateTexture(nil,"OVERLAY");rim:SetTexture("Interface\\Minimap\\MiniMap-TrackingBorder");rim:SetTexCoord(0,0.625,0,0.625);rim:SetWidth(26);rim:SetHeight(26);rim:SetPoint("CENTER",pin,"CENTER",0,0)
                    pin.badge=pin:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
                    pin.badge:SetWidth(28);pin.badge:SetHeight(16);pin.badge:SetJustifyH("RIGHT")
                    pin.badge:SetPoint("BOTTOMRIGHT",pin,"BOTTOMRIGHT",4,-3)
                    pin.badge:SetFont("Fonts"..string.char(92).."FRIZQT__.TTF",12,"OUTLINE")
                    pin.badge:SetTextColor(1,1,1);pin.badge:Hide()
                    pin.showTip=function()
                        local v=pin.data;if not v then return end;hoveredEventPin=pin;GameTooltip:SetOwner(pin,"ANCHOR_LEFT");GameTooltip:ClearLines()
                        GameTooltip:AddLine((v.kind==3 or v.kind==4 or v.kind==5) and v.name or(v.kind==0 and "Lost Supplies"or(v.kind==2 and "Cache"or"Sayge's Trial")),1,0.82,0)
                        GameTooltip:AddLine((v.kind==4 or v.kind==5) and v.note or(v.kind==3 and (v.active==false and "Guardian returning"or "Guardian lair")or(v.kind==1 and "Available until completed"or(v.kind==2 and ("Next cache: "..tostring(AZS_RUN and AZS_RUN.chest or "?").." gold")or(v.left.."s remaining")))),1,1,1)
                        if (pin.quantity or 1)>1 then GameTooltip:AddLine("Markers here: "..pin.quantity,1,1,1)end
                        if pin.outside then GameTooltip:AddLine("Beyond minimap range",0.8,0.8,0.8)end;GameTooltip:Show()
                    end
                    pin:SetScript("OnEnter",pin.showTip)
                    pin:SetScript("OnLeave",function()if hoveredEventPin==this then hoveredEventPin=nil;GameTooltip:Hide()end end);eventPins[used]=pin
                end
                if hoveredEventPin==pin and pin.data and (pin.data.event~=data.event or pin.data.kind~=data.kind)then hoveredEventPin=nil;GameTooltip:Hide()end
                pin.data=data
                -- Stable semantic layers survive retained-slot reuse. Returning
                -- guardians stay behind actionable pins; native pins are untouched.
                local priority=(data.kind==1 or data.kind==4 or data.kind==5) and 4 or(data.kind==3 and(data.active==false and 0 or 3)or(data.kind==0 and 2 or 1))
                local level=(parent.GetFrameLevel and parent:GetFrameLevel()or 0)+2+priority*2
                if pin.priority~=priority or pin.azsxLevel~=level then regroup=true;pin:SetFrameLevel(level);pin.priority=priority;pin.azsxLevel=level end
                if pin.kind~=data.kind then
                    regroup=true
                    SetPortraitToTexture(pin.icon:GetName(),data.kind==5 and "Interface\\Icons\\Spell_Shadow_Cripple"or(data.kind==4 and "Interface\\Icons\\INV_Misc_EngGizmos_06"or(data.kind==3 and "Interface\\Icons\\Ability_Warrior_Challange"or(data.kind==1 and blessingIcon or(data.kind==2 and "Interface\\Icons\\INV_Box_02"or "Interface\\Icons\\INV_Crate_01")))));pin.kind=data.kind
                end
            end
        end
    end
    nativeFlag(parent and parent:IsVisible() and map and liveZoom(parent)~=nil and (type(AZS_GetLairs)=="function"or type(AZS_GetFlaggedLair)=="function"))
    if eventPinCount~=used then regroup=true end
    eventPinCount=used
    for n=used+1,table.getn(eventPins)do local pin=eventPins[n];if hoveredEventPin==pin then hoveredEventPin=nil;GameTooltip:Hide()end;pin.data=nil;setShown(pin,false)end
end
-- Native projection at the current zoom. A supplies the native world ping
-- and all six radii; GetPingPosition supplies its live normalized projection.
-- Their inverse gives the live center without waiting for a metadata refresh.
-- Use the rendered hero position only to fade pins near the native arrow.
local function renderedPlayer()
    if type(AZS_GetPlayerPosition)=="function"then
        local ok,p=pcall(AZS_GetPlayerPosition)
        if not ok or type(p)~="table"or p.apiVersion~=1 or not finite(p.x)or not finite(p.y)or not finite(p.facing)then return nil end
        if AZS_ZONE and(AZS_ZONE.map~=p.mapId or(type(AZS_ZONE.run)=="string"and p.runId~=AZS_ZONE.run))then return nil end
        return p.x,p.y,p.facing
    end
    -- Older standalone addon hosts retain their native snapshot fallback.
    local run=AZS_RUN
    if run and finite(run.px)and finite(run.py)and finite(run.pf)then return run.px,run.py,run.pf end
end
-- Group by projected icon centers, including the clamped minimap rim.
-- Membership uses the original projected centers, never the moving centroid.
-- This preserves the merge distance and prevents a chain from growing by drift.
local function groupPins()
    for n=1,eventPinCount do
        local pin=eventPins[n];pin.quantity=1;pin.grouped=false;pin.sumX=pin.projectedX;pin.sumY=pin.projectedY
        for i=1,n-1 do
            local other=eventPins[i]
            if not other.grouped and other.kind==pin.kind and other.priority==pin.priority and other.outside==pin.outside then
                local dx,dy=pin.projectedX-other.projectedX,pin.projectedY-other.projectedY
                -- Require the centers of 20-unit portraits to be less than one radius apart.
                if dx*dx+dy*dy<10*10 then
                    other.quantity=other.quantity+1;other.sumX=other.sumX+pin.projectedX;other.sumY=other.sumY+pin.projectedY;pin.grouped=true;break
                end
            end
        end
    end
    for n=1,eventPinCount do
        local pin=eventPins[n]
        pin.x=pin.sumX/pin.quantity;pin.y=pin.sumY/pin.quantity
        if hoveredEventPin==pin then
            if pin.grouped then hoveredEventPin=nil;GameTooltip:Hide()
            elseif pin.badgeCount~=pin.quantity then pin.showTip()end
        end
        if pin.badgeCount~=pin.quantity then
            pin.badgeCount=pin.quantity
            if pin.quantity>1 then pin.badge:SetText(tostring(pin.quantity));pin.badge:Show()else pin.badge:Hide()end
        end
        setShown(pin,not pin.grouped)
    end
    regroup=false
end
local function layoutEventPins()
    local map=source();local parent=getglobal("Minimap")
    if not map or not parent or not parent:IsVisible()then
        regroup=true;for n=1,eventPinCount do setShown(eventPins[n],false)end;mapSample=nil;sessionIdentity=nil;viewCenterX=nil;viewCenterY=nil;localPingX=nil;localPingY=nil;pendingNativePing=false;return
    end
    if sessionIdentity~=map.identity then localPingX=nil;localPingY=nil;viewCenterX=nil;viewCenterY=nil;sessionIdentity=map.identity end
    local sampleKey=table.concat({map.identity,map.x,map.y,map.pingX,map.pingY,unpack(map.radii)},"|")
    if mapSample~=sampleKey then
        if referencePingX~=map.pingX or referencePingY~=map.pingY then localPingX=nil;localPingY=nil end
        referencePingX,referencePingY=map.pingX,map.pingY;mapSample=sampleKey;pendingNativePing=false
    end
    local zoom=liveZoom(parent)
    if not zoom then
        regroup=true;for n=1,eventPinCount do setShown(eventPins[n],false)end
        return
    end
    local radius=map.radii[zoom+1];local cx,cy=map.x,map.y
    if pendingNativePing and viewCenterX then cx,cy=viewCenterX,viewCenterY
    elseif parent.GetPingPosition then
        local px,py=parent:GetPingPosition()
        if type(px)=="number"and type(py)=="number"and px==px and py==py and math.abs(px)<1e8 and math.abs(py)<1e8 then cx=(localPingX or map.pingX)-2*radius*py;cy=(localPingY or map.pingY)+2*radius*px end
    end
    viewCenterX,viewCenterY=cx,cy
    local halfW,halfH=parent:GetWidth()/2,parent:GetHeight()/2
    local px,py=renderedPlayer()
    local heroX,heroY
    if px then
        local x,y=-(py-cy)/radius,(px-cx)/radius
        if x*x+y*y<=1 then heroX,heroY=x*halfW,y*halfH end
    end
    local edge=math.max(0,1-11/math.max(1,math.min(halfW,halfH)))
    for n=1,eventPinCount do
        local pin=eventPins[n];local data=pin.data
        local x,y=-(data.y-cy)/radius,(data.x-cx)/radius
        local length=math.sqrt(x*x+y*y);local outside=length>1
        if pin.outside~=outside then pin.outside=outside;regroup=true end
        if length>edge and length>0 then x=x*edge/length;y=y*edge/length end
        x=x*halfW;y=y*halfH
        if pin.projectedX~=x or pin.projectedY~=y then regroup=true;pin.projectedX=x;pin.projectedY=y end
    end
    if regroup then groupPins()end
    for n=1,eventPinCount do
        local pin=eventPins[n]
        if pin.drawX~=pin.x or pin.drawY~=pin.y then
            pin:ClearAllPoints();pin:SetPoint("CENTER",parent,"CENTER",pin.x,pin.y);pin.drawX=pin.x;pin.drawY=pin.y
        end
        local alpha=pin.outside and 0.65 or 1
        if heroX then
            local dx,dy=pin.x-heroX,pin.y-heroY
            local distance=math.sqrt(dx*dx+dy*dy)
            alpha=alpha*math.max(0.15,math.min(1,(distance-5)/15))
        end
        if pin.alpha~=alpha then pin:SetAlpha(alpha);pin.alpha=alpha end
    end
end
function AZSXUI_MinimapReadout()
    local result={};for n,pin in ipairs(eventPins)do if pin.azsxShown then table.insert(result,{event=pin.data.event,kind=pin.data.kind,outside=pin.outside,x=pin.x,y=pin.y,quantity=pin.quantity,priority=pin.priority,level=pin.azsxLevel})end end;return result
end
-- A native user/party ping changes the projection's reference world point.
-- Rebase the observer only; never create, move or suppress a native ping.
controller:RegisterEvent("MINIMAP_PING")
controller:SetScript("OnEvent",function()
    local map=source();local parent=getglobal("Minimap")
    if map and parent and viewCenterX and type(arg2)=="number"and type(arg3)=="number"then
        local zoom=liveZoom(parent);local radius=zoom and map.radii[zoom+1]
        if radius then localPingX=viewCenterX+2*radius*arg3;localPingY=viewCenterY-2*radius*arg2;pendingNativePing=false;return end
    end
    pendingNativePing=true
end)

-- Downed party members have native, run-scoped world anchors even in instances.
-- Refresh membership at minimap cadence; project only while someone needs help.
local reviveRequests,reviveNames,revivePins={},{},{}
local projecting=false
local function hideRevivePins()
    for _,pin in ipairs(revivePins)do setShown(pin,false)end
end
local function projectRevives()
    local page=AZS_ProjectWorldAnchors(reviveRequests)
    local visible=0
    if page and page.frameCurrent then
        for _,entry in ipairs(page.entries or {})do
            local name=reviveNames[entry.anchorId]
            if name and entry.state=="onScreen" and finite(entry.x) and finite(entry.y) then
                visible=visible+1
                local pin=revivePins[visible]
                if not pin then
                    pin=CreateFrame("Frame","AZSXUIReviveAnchor"..visible,WorldFrame);pin:SetWidth(24);pin:SetHeight(24);pin:EnableMouse(false)
                    local icon=pin:CreateTexture(nil,"ARTWORK");icon:SetAllPoints(pin)
                    icon:SetTexture("Interface\\Icons\\Spell_Holy_Resurrection")
                    pin.label=pin:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
                    pin.label:SetPoint("BOTTOM",pin,"TOP",0,1);revivePins[visible]=pin
                end
                pin:ClearAllPoints();pin:SetPoint("CENTER",WorldFrame,"BOTTOMLEFT",entry.x,entry.y)
                local distance=entry.playerDistance
                pin.label:SetText(finite(distance) and distance>=0 and string.format("%s · %.0f yd",name,distance) or name)
                setShown(pin,true)
            end
        end
    end
    for i=visible+1,table.getn(revivePins)do setShown(revivePins[i],false)end
end
local function refreshRevives()
    if not WorldFrame or type(AZS_GetDownedPartyMembers)~="function" or type(AZS_ProjectWorldAnchors)~="function"
        or type(AZS_RegisterWorldAnchorFrameHandler)~="function" or type(AZS_UnregisterWorldAnchorFrameHandler)~="function" then return end
    local phase=AZS_GetPhase and AZS_GetPhase()
    local friends=phase=="playing" and (not AZSXUIDB or AZSXUIDB.pins~=false) and AZS_GetDownedPartyMembers()
    if not projecting and (not friends or table.getn(friends)==0)then return end
    reviveRequests,reviveNames={},{}
    for _,friend in ipairs(friends or {})do
        if table.getn(reviveRequests)>=4 then break end
        if friend.positionAvailable and friend.anchorStatus=="ready" and type(friend.anchorId)=="string" and not reviveNames[friend.anchorId] then
            table.insert(reviveRequests,{anchorId=friend.anchorId,attachment="position",offset={x=0,y=0,z=2}})
            reviveNames[friend.anchorId]=friend.name
        end
    end
    if table.getn(reviveRequests)>0 then
        if not projecting then projecting=AZS_RegisterWorldAnchorFrameHandler(projectRevives)end
    elseif projecting then
        AZS_UnregisterWorldAnchorFrameHandler(projectRevives);projecting=false;hideRevivePins()
    end
end

local last=-1
controller:SetScript("OnUpdate",function()
    local now=GetTime()
    if now-last>=0.1 then last=now;refreshEventPins();refreshRevives()end
    layoutEventPins()
end)
-- Retained diagnostic name used by existing explicit UI probes.
function AZSXEventPinsReadout()return AZSXUI_MinimapReadout()end
