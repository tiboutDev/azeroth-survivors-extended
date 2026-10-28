-- Disposable diagnostic only: select a native enemy and show its native tooltip.
local visualProbe=CreateFrame("Frame","AzCoopVisualProbe",UIParent)
local last=0
local first=nil
local isolation=false
local isolationDone=false
visualProbe:SetScript("OnUpdate",function()
    local now=GetTime()
    if now-last<0.1 or not UnitExists("player") then return end
    last=now
    if not UnitExists("target") or UnitIsDead("target") then pcall(TargetNearestEnemy) end
    if UnitExists("target") then
        first=first or now
        if now-first>=2 and now-first<3.5 then
            if not isolation then
                GameTooltip:SetOwner(UIParent,"ANCHOR_NONE")
                GameTooltip:SetText("Coop tooltip isolation check")
                GameTooltip:Show();isolation=true
            end
            error("AZCOOP_HEALTH_TOOLTIP_ISOLATION text="..tostring(GameTooltipTextLeft1 and GameTooltipTextLeft1:GetText()),0)
            return
        elseif isolation and not isolationDone then
            GameTooltip:Hide();isolationDone=true
        end
        if GameTooltip and not GameTooltip:IsShown() then
            GameTooltip:SetOwner(UIParent,"ANCHOR_NONE")
            GameTooltip:SetPoint("BOTTOMRIGHT",UIParent,"BOTTOMRIGHT",-20,120)
            GameTooltip:SetUnit("target")
            GameTooltip:Show()
        end
        local bar=TargetFrameHealthBar
        local tip=GameTooltipStatusBar
        error("AZCOOP_HEALTH_PROBE name="..tostring(UnitName("target")).." hp="..tostring(UnitHealth("target")).." max="..tostring(UnitHealthMax("target")).." bar="..tostring(bar and bar:GetValue()).." tip="..tostring(tip and tip:GetValue()),0)
    end
end)
