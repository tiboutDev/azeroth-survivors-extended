-- Sparse read-only visual evidence. No input or gameplay mutation.
local frame=CreateFrame("Frame")
local started,last,peak=GetTime(),-1,0
local label=frame:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
label:SetPoint("TOPLEFT",UIParent,"TOPLEFT",180,-70);label:SetWidth(680);label:SetHeight(200);label:SetJustifyH("LEFT");label:SetJustifyV("TOP")
frame:SetScript("OnUpdate",function()
 local now=GetTime();if now-started>250 then frame:SetScript("OnUpdate",nil);return end;if now-last<2 then return end;last=now
 if not AZSXSupplyReadout then return end;local buffs,shown,event=AZSXSupplyReadout();peak=math.max(peak,table.getn(buffs));local lines={"SUPPLIES / MAP - READ ONLY", "Active="..table.getn(buffs).." peak="..peak.." discovery="..tostring(event).." toast="..tostring(shown)}
 for _,b in ipairs(buffs)do local slot,permanent=GetPlayerBuff(b.index,"HELPFUL");table.insert(lines,"family="..b.family.." tier="..b.tier.." slot="..b.index.." left="..b.remaining.." temporary="..tostring(permanent==0).." owned="..tostring(b.visible));end
 local pins=AZSXEventPinsReadout and AZSXEventPinsReadout()or{};table.insert(lines,"Event pins="..table.getn(pins));label:SetText(table.concat(lines,"\n"))
end)
