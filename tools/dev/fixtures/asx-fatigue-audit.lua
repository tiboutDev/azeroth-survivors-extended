-- Private one-shot readback. Poll at most once per second, retire within 120s.
local frame=CreateFrame("Frame","ASXFatigueAudit",UIParent)
local started,last=GetTime(),-1
local panel
local function show(lines)
    if not panel then
        panel=frame:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
        panel:SetPoint("TOPLEFT",UIParent,"TOPLEFT",180,-170)
        panel:SetWidth(600);panel:SetHeight(220);panel:SetJustifyH("LEFT");panel:SetJustifyV("TOP")
        panel:SetFont("Fonts\\FRIZQT__.TTF",14,"OUTLINE")
    end
    panel:SetText(table.concat(lines,"\n"))
end
frame:SetScript("OnUpdate",function()
    local now=GetTime()
    if now-started>120 then frame:SetScript("OnUpdate",nil);return end
    if now-last<1 then return end
    last=now
    if now-started<25 then return end
    if not AZS_RUN or (tonumber(AZS_RUN.t)or 0)<35 then return end
    local selected,texture,count
    for index=0,16 do
        local icon,stacks=UnitDebuff("player",index)
        if icon and string.find(string.lower(icon),"spell_shadow_deathscream",1,true)then selected=index;texture=icon;count=stacks;break end
    end
    show({"NATIVE AURA PROBE", "Index: "..tostring(selected).." texture: "..tostring(texture).." count: "..tostring(count)})
    if selected==nil or (count or 0)<2 then return end
    GameTooltip:SetOwner(UIParent,"ANCHOR_NONE")
    GameTooltip:SetUnitDebuff("player",selected)
    local lines={"NATIVE DEBUFF READBACK",tostring(texture),"Stacks: "..tostring(count)}
    for n=1,6 do
        local l=getglobal("GameTooltipTextLeft"..n)
        local r=getglobal("GameTooltipTextRight"..n)
        if l and l:IsShown() and l:GetText() then table.insert(lines,l:GetText().." "..tostring(r and r:GetText()or""))end
    end
    show(lines)
    GameTooltip:Hide()
    frame:SetScript("OnUpdate",nil)
end)
