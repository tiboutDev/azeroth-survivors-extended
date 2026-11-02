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
    for i=1,math.min(table.getn(AZS_CHESTS or {}),128)do
        local cache=AZS_CHESTS[i]
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
local playerPin
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
                    pin:SetScript("OnEnter",function()
                        local v=this.data;if not v then return end;hoveredEventPin=this;GameTooltip:SetOwner(this,"ANCHOR_LEFT");GameTooltip:ClearLines()
                        GameTooltip:AddLine((v.kind==3 or v.kind==4 or v.kind==5) and v.name or(v.kind==0 and "Lost Supplies"or(v.kind==2 and "Cache"or"Sayge's Trial")),1,0.82,0)
                        GameTooltip:AddLine((v.kind==4 or v.kind==5) and v.note or(v.kind==3 and (v.active==false and "Guardian returning"or "Guardian lair")or(v.kind==1 and "Available until completed"or(v.kind==2 and ("Next cache: "..tostring(AZS_RUN and AZS_RUN.chest or "?").." gold")or(v.left.."s remaining")))),1,1,1)
                        if this.outside then GameTooltip:AddLine("Beyond minimap range",0.8,0.8,0.8)end;GameTooltip:Show()
                    end)
                    pin:SetScript("OnLeave",function()if hoveredEventPin==this then hoveredEventPin=nil;GameTooltip:Hide()end end);eventPins[used]=pin
                end
                if hoveredEventPin==pin and pin.data and (pin.data.event~=data.event or pin.data.kind~=data.kind)then hoveredEventPin=nil;GameTooltip:Hide()end
                pin.data=data
                -- Stable semantic layers survive retained-slot reuse. Returning
                -- guardians stay behind actionable pins; native pins are untouched.
                local priority=(data.kind==1 or data.kind==4 or data.kind==5) and 4 or(data.kind==3 and(data.active==false and 0 or 3)or(data.kind==0 and 2 or 1))
                local level=(parent.GetFrameLevel and parent:GetFrameLevel()or 0)+2+priority*2
                if pin.priority~=priority or pin.azsxLevel~=level then pin:SetFrameLevel(level);pin.priority=priority;pin.azsxLevel=level end
                if pin.kind~=data.kind then
                    SetPortraitToTexture(pin.icon:GetName(),data.kind==5 and "Interface\\Icons\\Spell_Shadow_Cripple"or(data.kind==4 and "Interface\\Icons\\INV_Misc_EngGizmos_06"or(data.kind==3 and "Interface\\Icons\\Ability_Warrior_Challange"or(data.kind==1 and blessingIcon or(data.kind==2 and "Interface\\Icons\\INV_Box_02"or "Interface\\Icons\\INV_Crate_01")))));pin.kind=data.kind
                end
                setShown(pin,true)
            end
        end
    end
    nativeFlag(parent and parent:IsVisible() and map and liveZoom(parent)~=nil and (type(AZS_GetLairs)=="function"or type(AZS_GetFlaggedLair)=="function"))
    eventPinCount=used
    for n=used+1,table.getn(eventPins)do local pin=eventPins[n];if hoveredEventPin==pin then hoveredEventPin=nil;GameTooltip:Hide()end;pin.data=nil;setShown(pin,false)end
end
-- Native projection at the current zoom. A supplies the native world ping
-- and all six radii; GetPingPosition supplies its live normalized projection.
-- Their inverse gives the live center without waiting for a metadata refresh.
-- Native hero position/facing and native arrow texture; no second world marker
-- source or memory access. The original native arrow remains the fallback.
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
local function playerArrow(parent,cx,cy,radius,halfW,halfH)
    local px,py,facing=renderedPlayer()
    if not px then
        if playerPin then setShown(playerPin,false)end;return
    end
    local x,y=-(py-cy)/radius,(px-cx)/radius
    if x*x+y*y>1 then if playerPin then setShown(playerPin,false)end;return end
    if not playerPin then
        playerPin=CreateFrame("Frame","AZSXUIPlayerMinimapArrow",parent);playerPin:SetWidth(24);playerPin:SetHeight(24);playerPin:EnableMouse(false)
        playerPin.icon=playerPin:CreateTexture(nil,"OVERLAY");playerPin.icon:SetAllPoints(playerPin);playerPin.icon:SetTexture("Interface\\Minimap\\MinimapArrow")
    end
    playerPin:SetFrameLevel((parent.GetFrameLevel and parent:GetFrameLevel()or 0)+12)
    playerPin:ClearAllPoints();playerPin:SetPoint("CENTER",parent,"CENTER",x*halfW,y*halfH)
    local c,s=math.cos(facing),math.sin(facing)
    local function uv(u,v)return .5+c*u-s*v,.5+s*u+c*v end
    local ul,vl=uv(-.5,-.5);local ll,lv=uv(-.5,.5);local ur,vr=uv(.5,-.5);local lr,rv=uv(.5,.5)
    playerPin.icon:SetTexCoord(ul,vl,ll,lv,ur,vr,lr,rv);setShown(playerPin,true)
end
local function layoutEventPins()
    local map=source();local parent=getglobal("Minimap")
    if not map or not parent or not parent:IsVisible()then
        if playerPin then setShown(playerPin,false)end
        for n=1,eventPinCount do setShown(eventPins[n],false)end;mapSample=nil;sessionIdentity=nil;viewCenterX=nil;viewCenterY=nil;localPingX=nil;localPingY=nil;pendingNativePing=false;return
    end
    if sessionIdentity~=map.identity then localPingX=nil;localPingY=nil;viewCenterX=nil;viewCenterY=nil;sessionIdentity=map.identity end
    local sampleKey=table.concat({map.identity,map.x,map.y,map.pingX,map.pingY,unpack(map.radii)},"|")
    if mapSample~=sampleKey then
        if referencePingX~=map.pingX or referencePingY~=map.pingY then localPingX=nil;localPingY=nil end
        referencePingX,referencePingY=map.pingX,map.pingY;mapSample=sampleKey;pendingNativePing=false
    end
    local zoom=liveZoom(parent)
    if not zoom then
        if playerPin then setShown(playerPin,false)end
        for n=1,eventPinCount do setShown(eventPins[n],false)end
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
    playerArrow(parent,cx,cy,radius,halfW,halfH)
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
function AZSXUI_MinimapReadout()
    local result={};for n,pin in ipairs(eventPins)do if pin.azsxShown then table.insert(result,{event=pin.data.event,kind=pin.data.kind,outside=pin.outside,x=pin.x,y=pin.y,priority=pin.priority,level=pin.azsxLevel})end end;return result
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

local last=-1
controller:SetScript("OnUpdate",function()
    local now=GetTime()
    if now-last>=0.1 then last=now;refreshEventPins()end
    layoutEventPins()
end)
-- Retained diagnostic name used by existing explicit UI probes.
function AZSXEventPinsReadout()return AZSXUI_MinimapReadout()end
