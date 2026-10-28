-- Diagnostic-only sparse readback of real native auras and tooltip.
local f=CreateFrame("Frame","ASXBlessingsAudit",UIParent)
local last,started,seen=-1,GetTime(),nil
local panel=f:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
panel:SetPoint("TOPLEFT",UIParent,"TOPLEFT",190,-265)
panel:SetWidth(560);panel:SetHeight(190);panel:SetJustifyH("LEFT");panel:SetJustifyV("TOP")
f:SetScript("OnUpdate",function()
 local now=GetTime();if now-started>130 then f:SetScript("OnUpdate",nil);return end
 if now-last<2 then return end;last=now
 if not AzCoopOfficialPartyReadout then return end
 local _,_,_,_,b=AzCoopOfficialPartyReadout();if not b or b.boss==seen then return end
 local found
 for i=0,32 do local tex=UnitBuff("player",i);if tex and string.find(string.lower(tex),"inv_misc_orb_02",1,true)then found=i;break end end
 if found==nil then panel:SetText("ASX BLESSING: waiting for native buff");return end
 GameTooltip:SetOwner(UIParent,"ANCHOR_NONE");GameTooltip:SetUnitBuff("player",found)
 local permanent
 for i=0,31 do
  local slot,flag=GetPlayerBuff(i,"HELPFUL")
  local tex=slot and slot>=0 and GetPlayerBuffTexture(slot)
  if tex and string.find(string.lower(tex),"inv_misc_orb_02",1,true)then permanent=flag;break end
 end
 local lines={"NATIVE BLESSING READBACK - boss "..b.boss.." / slot "..found,"Native permanent flag: "..tostring(permanent)}
 for i=1,6 do
  local l=getglobal("GameTooltipTextLeft"..i);local rr=getglobal("GameTooltipTextRight"..i)
  if l and l:IsShown()and l:GetText()then table.insert(lines,l:GetText().." "..tostring(rr and rr:IsShown()and rr:GetText()or""))end
 end
 panel:SetText(table.concat(lines,"\n"));GameTooltip:Hide();seen=b.boss
end)
