-- Native owner/party tooltip inspection; no errors, input or gameplay writes.
local f=CreateFrame("Frame","AzCoopTankingProbe",UIParent)
local at=0
local function contains(s)return s and string.find(s,"Coop: Taunts",1,true)end
f:SetScript("OnUpdate",function()
 if GetTime()-at<2 or not AzCoopPartyUIReadout then return end
 at=GetTime()
 local roster,own,active=AzCoopPartyUIReadout()
 if not active then return end
 local source=AzCoopPartyUITooltipSourceReadout and AzCoopPartyUITooltipSourceReadout() or ""
 local ownTip=string.find(source,"436f6f703a205461756e7473",1,true) and 1 or 0
 local remote=0
 for slot,member in pairs(roster) do
  if slot~=own then
   for _,spell in ipairs(member.spells) do
    for _,line in ipairs(spell.tooltip or {}) do if contains(line.left) then remote=remote+1 end end
   end
  end
 end
 SetCVar("azcoop_probe_report","AZCOOP1125 tanking own="..own.." owner_tip="..ownTip.." remote_tips="..remote)
end)
