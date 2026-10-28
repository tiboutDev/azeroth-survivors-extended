-- Read-only native UI evidence; never raises an error or changes gameplay.
RegisterCVar("azcoop_probe_report", "")
local f=CreateFrame("Frame","AzCoop1133Probe",UIParent)
local last=0
f:SetScript("OnUpdate",function()
 local now=GetTime()
 if now-last<1 or not UnitExists("player") then return end
 last=now
 local count,shown=GetNumPartyMembers(),0
 for i=1,count do
  local frame=getglobal("PartyMemberFrame"..i)
  if UnitExists("party"..i) and frame and frame:IsVisible() then shown=shown+1 end
 end
 local total,build,buffs,tips=0,0,0,0
 if AzCoopPartyUIReadout then
  local roster,own,active,settings,refreshes=AzCoopPartyUIReadout()
  if active then
   for slot,row in pairs(roster) do
    total=total+1
    if slot~=own then
     build=build+table.getn(row.spells)+table.getn(row.items)
     buffs=buffs+table.getn(row.buffs)
     for _,icon in ipairs(row.spells) do if icon.tooltip and table.getn(icon.tooltip)>0 then tips=tips+1 end end
    end
   end
  end
 end
 local card=AzCoopPartyCard1
 local source=AzCoopPartyUITooltipSourceReadout and AzCoopPartyUITooltipSourceReadout() or ""
 local title=AzCoopOwnerSpellTooltipTextLeft1
 local diagnostic=title and title:GetText() or ""
 diagnostic=string.gsub(diagnostic or "","[%c ]","_")

 SetCVar("azcoop_probe_report","AZCOOP1125 class="..tostring(UnitClass("player")).." members="..count.." frames="..shown.." roster="..total.." build="..build.." buffs="..buffs.." tips="..tips.." card="..tostring(card and card:IsVisible()).." meter="..tostring(AzCoopGroupMeter and AzCoopGroupMeter:IsVisible()).." source_bytes="..string.len(source).." title="..diagnostic)
end)
