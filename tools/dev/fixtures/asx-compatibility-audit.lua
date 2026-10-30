-- Disposable read-only visual audit; never distributed with ASX.
local f=CreateFrame("Frame","ASXCompatibilityAudit",UIParent)
f:SetPoint("BOTTOM",UIParent,"BOTTOM",0,75);f:SetWidth(780);f:SetHeight(90)
local label=f:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
label:SetPoint("CENTER",f,"CENTER",0,0)
label:SetFont("Fonts\\FRIZQT__.TTF",18,"OUTLINE")
local last=-10
f:SetScript("OnUpdate",function()
 local now=GetTime();if now-last<2 then return end;last=now
 local rows,count,own,active={},0,nil,0
 if ASXOfficialPartyReadout then rows,count,own,active=ASXOfficialPartyReadout()end
 local raw=AZS_RUN and AZS_RUN.asx
 local lines={"ASX compatibility audit: active="..tostring(active).." rows="..tostring(count).." own="..tostring(own),"wire="..type(raw).."/"..tostring(type(raw)=="string"and string.len(raw)or 0).." fps="..tostring(GetFramerate and math.floor(GetFramerate())or "unknown")}
 local hovered=false -- no automatic tooltip opening in the lightweight audit
 for i=1,2 do
  local row=rows[i];local n=0;local member=row and row.member
  if row then for _,button in ipairs(row.spells)do if button:IsVisible()then n=n+1 end end end
  table.insert(lines,"party"..i.."="..tostring(UnitName("party"..i)).." row="..tostring(row and row.owner).." member="..tostring(member and member.name).." spells="..n)

 end
 label:SetText(table.concat(lines,"\n"))
end)
