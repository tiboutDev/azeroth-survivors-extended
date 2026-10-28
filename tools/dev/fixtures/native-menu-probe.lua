-- Development-only native widget exercise. No OS input or desktop interaction.
if seterrorhandler then seterrorhandler(function() end) end
RegisterCVar("azcoop_probe_role", "")
local probe=CreateFrame("Frame","AzCoopNativeMenuProbe",nil)
probe:Show()
local shown,created,joined,started,confirmed=false,false,false,false,false
local captured=false
local last=0
local ticks=0
local hidden_stock=false
local stock_roots={}
local stock_models={}
probe:SetScript("OnUpdate",function()
    ticks=ticks+1
    local ui=AzCoopNativeUI
    local role=GetCVar("azcoop_probe_role")
    if role~="host" and role~="guest" then return end
    if math.mod(ticks,120)==1 then
        error("AZCOOP_MENU_TICK ui="..tostring(ui~=nil).." role="..tostring(role).." enabled="..tostring(GetCVar("azcoop_enabled")).." player="..tostring(UnitExists and UnitExists("player")).." visible="..tostring(ui and ui.button:IsVisible()).." time="..tostring(GetTime()),0)
    end
    if not ui or (role~="host" and role~="guest") or not AzCoopMenuRoot:IsVisible() then return end
    if not shown then
        ui.button:Click()
        ui.player:SetText(role=="host" and "NativeHost" or "NativeGuest")
        ui.room:SetText("Native UI isolated check")
        shown=true
    end
    local state=ui.state or {}
    if (state.test_hide_stock=="1" or state.test_hide_stock=="2") and not hidden_stock then
        local frame=EnumerateFrames()
        while frame do
            local name=frame:GetName() or ""
            if string.sub(name,1,6)~="AzCoop" then
                if frame.ClearModel then table.insert(stock_models,frame);frame:ClearModel() end
                if not frame:GetParent() then
                    table.insert(stock_roots,frame)
                    if state.test_hide_stock=="1" then frame:Hide() else frame:SetAlpha(0) end
                end
            end
            frame=EnumerateFrames(frame)
        end
        hidden_stock=true
    end
    if state.test_hide_stock=="2" then
        for _,frame in ipairs(stock_roots) do frame:SetAlpha(0) end
        for _,frame in ipairs(stock_models) do frame:ClearModel() end
    end
    if role=="host" and state.can_create=="1" and not created then ui.create:Click();ui.create:Click();created=ui.sequence>0 end
    if role=="guest" and state.can_join=="1" and state.test_invite and not joined then
        ui.code:SetText(state.test_invite);ui.join:Click();ui.join:Click();joined=true
    end
    if role=="guest" and state.can_ready=="1" and state.self_ready~="1" and not confirmed then
        ui.readyButton:Click();ui.readyButton:Click();confirmed=true
    end
    if role=="host" and state.can_start=="1" and not started then
        if not captured and Screenshot then Screenshot();captured=true;return end
        ui.start:Click();ui.start:Click();started=true
    end
    local now=GetTime()
    if now-last>1 then
        last=now
        local parent=AzCoopMenuRoot:GetParent()
        local alpha=1;local f=ui.button;local depth=0
        while f and depth<20 do alpha=alpha*f:GetAlpha();f=f:GetParent();depth=depth+1 end
        error("AZCOOP_MENU_PROBE role="..role.." button="..tostring(ui.button:IsVisible()).." panel="..tostring(AzCoopLobbyPanel:IsVisible()).." width="..tostring(ui.button:GetWidth()).." ack="..tostring(state.ack).." ready="..tostring(state.core_ready).." seq="..ui.sequence.." left="..tostring(ui.button:GetLeft()).." top="..tostring(ui.button:GetTop()).." screen="..tostring(GetScreenHeight()).." alpha="..tostring(alpha).." parent="..tostring(parent and parent:GetName()),0)
    end
end)
