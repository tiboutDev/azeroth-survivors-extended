-- Disposable one-shot tooltip readback. Readiness polls at most once per 2 s.
-- No startup widget construction, per-frame readback, or repeated hidden tooltips.
local frame=CreateFrame("Frame","ASXTooltipAudit",UIParent)
local previous=-100
local function lines(prefix)
 local result={}
 for n=1,24 do
  local l=getglobal(prefix.."TextLeft"..n);local r=getglobal(prefix.."TextRight"..n)
  if l and l:IsShown()and l:GetText()then
   local right=r and r:IsShown()and r:GetText()or""
   table.insert(result,l:GetText()..(right~=""and("    "..right)or""))
  end
 end
 return table.concat(result,"\n")
end
frame:SetScript("OnUpdate",function()
 local now=GetTime();if now-previous<2 then return end;previous=now
 local clock=tonumber(AZS_RUN and AZS_RUN.t)
 if not clock or clock<35 or not ASXOfficialPartyReadout then return end
 local rows,built,own,active=ASXOfficialPartyReadout()
 if active~=1 or not own then return end
 local ownerSlot=nil
 for slot=1,6 do if HasAction(slot)then ownerSlot=slot;break end end
 if not ownerSlot then return end
 local selected={}
 for i=1,1 do
  local row=rows[i]
  if not row or not row.member then return end
  for n=1,16 do
   local button=row.spells[n]
   if button and button:IsVisible()and button.data and button.data.text and string.find(button.data.text,"Glyph:",1,true) then selected[i]=button;break end
  end
  if not selected[i]then return end
 end
 -- Build panels at capture time, after the viewport has settled.
 local panels={}
 for n=1,2 do
  local p=frame:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
  local width=UIParent:GetWidth()/2
  p:SetPoint("TOPLEFT",UIParent,"TOPLEFT",10+(n-1)*width,-185)
  p:SetWidth(width-20);p:SetHeight(UIParent:GetHeight()-200);p:SetJustifyH("LEFT");p:SetJustifyV("TOP")
  p:SetFont("Fonts\\FRIZQT__.TTF",13,"OUTLINE");panels[n]=p
 end
 local ownTip=CreateFrame("GameTooltip","ASXTooltipOwnerTip",UIParent,"GameTooltipTemplate")
 ownTip:SetOwner(UIParent,"ANCHOR_NONE");ownTip:ClearLines();ownTip:SetAction(ownerSlot)
 panels[1]:SetText("NATIVE OWNER "..tostring(UnitName("player")).."\n"..lines("ASXTooltipOwnerTip"))
 ownTip:Hide()
 for i=1,1 do
  local button=selected[i];local old=this;this=button;button:GetScript("OnEnter")()
  panels[i+1]:SetText("REMOTE "..rows[i].member.name.."\n"..lines("GameTooltip"))
  button:GetScript("OnLeave")();this=old
 end
 -- Keep the captured text visible, but retire the diagnostic callback entirely.
 frame:SetScript("OnUpdate",nil)
end)
