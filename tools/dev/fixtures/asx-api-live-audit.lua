-- Opt-in real API audit; no provider replacement or private payload rewriting.
local f=CreateFrame("Frame","ASXAPILiveAudit",UIParent)
local last,count=-100,0
local good,bad,callbackMismatch=0,0,0
local subscribed,configured=false,false
local seen={}
if seterrorhandler then seterrorhandler(function()end)end
local function quote(s)
    return '"'..string.gsub(tostring(s),'[%z\001-\031\\"]',function(c)return string.format("\\u%04x",string.byte(c))end)..'"'
end
local function json(v)
    if type(v)=="string"then return quote(v)end
    if type(v)=="boolean"or type(v)=="number"then return tostring(v)end
    if type(v)~="table"then return "null"end
    local out={};for k,item in pairs(v)do table.insert(out,quote(k)..":"..json(item))end
    return "{"..table.concat(out,",").."}"
end
f:SetScript("OnUpdate",function()
    local now=GetTime()
    if now-last<1 then return end
    last=now;count=count+1
    local out={sample=count,time=now,api=type(ASX_GetState)=="function",callbacks=good,badCallbacks=bad,callbackMismatch=callbackMismatch,fixtureVersion=3,native={phase=AZS_GetPhase and AZS_GetPhase(),clock=AZS_GetRunTime and AZS_GetRunTime(),x=AZS_RUN and AZS_RUN.px,y=AZS_RUN and AZS_RUN.py,speed=AZS_STATS and AZS_STATS.spd},widgets={}}
    if not out.api then error("ASX_API_LIVE "..json(out),0)end
    out.major,out.minor=ASX_GetAPIVersion()
    if not subscribed then
        ASX_RegisterStateHandler(function()bad=bad+1;error("deliberate isolated consumer error")end)
        ASX_RegisterStateHandler(function(s)good=good+1;local again=ASX_GetState(1);if again.revision~=s.revision then callbackMismatch=callbackMismatch+1 end end)
        subscribed=true
    end
    if not configured and ASXUI_ConfigureWidget then
        ASXUI_ConfigureWidget("anchor","position","0","-85")
        ASXUI_ConfigureWidget("supplies","scale","0.85")
        ASXUI_ConfigureWidget("fortune","opacity","0.75")
        ASXUI_ConfigureWidget("sayge","scale","0.9")
        configured=true
    end
    local s=ASX_GetState(1)
    out.available=s.available;out.reason=s.reason;out.phase=s.phase;out.revision=s.revision
    out.recipient=s.session and s.session.recipientGUID;out.epoch=s.session and s.session.epoch
    out.structured=s.data;out.texts=s.texts
    out.globals=type(ASX_STATE)=="table"and type(ASX_STATE_UPDATED)=="number"
    s.texts.auditOnly={changed=true};local nextState=ASX_GetState(1)
    out.copyIsolated=nextState.texts.auditOnly==nil
    if s.data.sayge then s.data.sayge.held=-999;out.deepCopyIsolated=ASX_GetState(1).data.sayge.held==nextState.data.sayge.held end
    out.structured=nextState.data;out.texts=nextState.texts
    for _,entry in ipairs({{"supplies","ASXSupplyDiscoveryToast"},{"fortune","ASXDarkmoonBlessingToast"},{"sayge","ASXSaygeTrialProgress"}})do
        local key,name=entry[1],entry[2];local frame=getglobal(name)
        local item={exists=frame~=nil,shown=frame and frame:IsShown()or false}
        if frame then
            item.scale=frame:GetScale();item.alpha=frame:GetAlpha()
            item.title=frame.title:GetText();item.note=frame.note:GetText()
            if frame.reward then item.reward=frame.reward:GetText()end
            local text=nextState.texts[key]
            item.textMatches=text and item.title==text.title and item.note==text.note and(not frame.reward or item.reward==text.reward)or false
            if item.shown and not seen[key]then
                local before=nextState.data.sayge and nextState.data.sayge.held
                ASXUI_ConfigureWidget(key,"show","off");item.hideWorks=not frame:IsShown()
                ASXUI_ConfigureWidget(key,"show","on");item.showWorks=frame:IsShown()
                local after=ASX_GetState(1);item.displayOnly=before==(after.data.sayge and after.data.sayge.held)
                seen[key]=true
            end
        end
        out.widgets[key]=item
    end
    if nextState.data.crystals then
        local c=nextState.data.crystals;local index=c.index
        local v={visible=false,permanent=false,tooltip=false}
        if index and index<32 and GetPlayerBuffTexture then
            local texture=GetPlayerBuffTexture(index)
            v.visible=type(texture)=="string"and string.find(texture,"INV_Misc_Gem_Variety_01",1,true)~=nil
            local _,permanent=GetPlayerBuff(index,"HELPFUL");v.permanent=permanent==1 and GetPlayerBuffTimeLeft(index)==0
            if v.visible and GameTooltip then
                local button
                for n=0,31 do local candidate=getglobal("BuffButton"..n);if candidate and candidate.buffIndex==index and candidate:IsShown()then button=candidate;break end end
                GameTooltip:SetOwner(button or UIParent,button and "ANCHOR_BOTTOMLEFT"or "ANCHOR_CURSOR");GameTooltip:SetPlayerBuff(index)
                local title=getglobal("GameTooltipTextLeft1");v.tooltip=title and title:GetText()=="Crystal Resonance"or false
                if button then
                    local duration=getglobal(button:GetName().."Duration");v.durationShown=duration and duration:IsShown()or false;v.durationText=duration and duration:GetText();v.untilCancelled=button.untilCancelled
                end
                if not f.crystalTooltipAt and c.total>=5 then f.crystalTooltipAt=now end
                if f.crystalTooltipAt and now-f.crystalTooltipAt<15 then GameTooltip:Show()else GameTooltip:Hide()end
            end
        end
        out.crystalBuff=v
    end
    local popup=getglobal("ASXCrystalDiscoveryToast")
    if popup then
        local v={shown=popup:IsShown(),title=popup.title:GetText(),reward=popup.reward:GetText(),note=popup.note:GetText(),alpha=popup:GetAlpha(),scale=popup:GetScale(),verbatim=false}
        for _,text in pairs(nextState.texts.crystals and nextState.texts.crystals.pickups or{})do
            if v.title==text.title and v.reward==text.reward and v.note==text.note then v.verbatim=true end
        end
        out.crystalPopup=v
    end
    if ASX_WOW_VISUAL_AUDIT then
        local v={};local right,top=-1e9,-1e9;local fortune,slot
        for n=0,31 do local button=getglobal("BuffButton"..n)
            if button and button:IsShown()then
                local index=button.buffIndex or(GetPlayerBuff and GetPlayerBuff(button:GetID(),"HELPFUL"))
                local texture=type(index)=="number"and GetPlayerBuffTexture(index)
                if type(button.GetRight)=="function"and type(button.GetTop)=="function"then
                    right=math.max(right,button:GetRight()or -1e9);top=math.max(top,button:GetTop()or -1e9)
                end
                if type(texture)=="string"and string.find(string.lower(texture),"inv_misc_orb_02",1,true)then fortune=button;slot=index end
            end
        end
        v.fortuneShown=fortune~=nil
        if nextState.data.fortune and fortune and type(fortune.GetRight)=="function"and type(fortune.GetTop)=="function"then
            v.first=fortune:GetRight()>=right-0.5 and fortune:GetTop()>=top-0.5;v.right=fortune:GetRight();v.top=fortune:GetTop();v.button=fortune:GetName();v.slot=slot
            if not f.tooltipStart then f.tooltipStart=now end
            if now-f.tooltipStart<20 then
                GameTooltip:SetOwner(fortune,"ANCHOR_BOTTOMLEFT");GameTooltip:SetPlayerBuff(slot);GameTooltip:Show()
                local lines={};for n=1,12 do local text=getglobal("GameTooltipTextLeft"..n);if text and text:IsShown()then table.insert(lines,text:GetText()or "")end end;v.tooltip=table.concat(lines," | ")
            elseif not f.tooltipClosed then GameTooltip:Hide();f.tooltipClosed=true end
        end
        out.buffOrder=v
    end
    error("ASX_API_LIVE "..json(out),0)
end)
