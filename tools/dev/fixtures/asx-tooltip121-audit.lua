-- Private two-player one-shot readback; no native CVar channel or OS input.
local f=CreateFrame("Frame","ASXTooltip121Audit",UIParent)
local last=-100
local function text(prefix)
 local lines={}
 for n=1,24 do
  local l=getglobal(prefix.."TextLeft"..n);local r=getglobal(prefix.."TextRight"..n)
  if l and l:IsShown() and l:GetText() then table.insert(lines,l:GetText().." "..tostring(r and r:IsShown() and r:GetText() or "")) end
 end
 return table.concat(lines,"\n")
end
f:SetScript("OnUpdate",function()
 local now=GetTime();if now-last<2 then return end;last=now
 if now>180 then f:SetScript("OnUpdate",nil);return end
 if not AZS_RUN or (tonumber(AZS_RUN.t)or 0)<12 or not AzCoopOfficialPartyReadout then return end
 local rows,built,own,active=AzCoopOfficialPartyReadout();if active~=1 then return end
 local row,button
 for _,candidate in pairs(rows)do
  if candidate.member then
   for _,b in pairs(candidate.spells)do if b:IsVisible()and b.data and b.data.tip and b.data.text~="" then row=candidate;button=b;break end end
  end
  if button then break end
 end
 if not button then return end
 local panels={}
 for n=1,2 do
  local p=f:CreateFontString(nil,"OVERLAY","GameFontNormalSmall");p:SetPoint("TOPLEFT",UIParent,"TOPLEFT",15+(n-1)*390,-185);p:SetWidth(375);p:SetHeight(380);p:SetJustifyH("LEFT");p:SetJustifyV("TOP");p:SetFont("Fonts\\FRIZQT__.TTF",12,"OUTLINE");panels[n]=p
 end
 local ownTip=CreateFrame("GameTooltip","ASXTooltip121Owner",UIParent,"GameTooltipTemplate");ownTip:SetOwner(UIParent,"ANCHOR_NONE")
 for slot=1,6 do if HasAction(slot)then ownTip:SetAction(slot);break end end
 panels[1]:SetText("NATIVE OWNER "..tostring(UnitName("player")).."\n"..text("ASXTooltip121Owner"));ownTip:Hide()
 local previous=this;this=button;button:GetScript("OnEnter")();panels[2]:SetText("REMOTE OWNER "..tostring(row.member.name).."\n"..text("GameTooltip"));button:GetScript("OnLeave")();this=previous
 f:SetScript("OnUpdate",nil)
end)
