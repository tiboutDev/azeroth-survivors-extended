-- Read-only native party widgets and cosmetic build readout. No input/errors.
RegisterCVar("azcoop_probe_report", "")
local probe=CreateFrame("Frame","AzCoopLobbyParty1133Probe",UIParent)
local last=0
probe:SetScript("OnUpdate",function()
 local now=GetTime()
 if now-last<1 or not UnitExists("player") then return end
 last=now
 local members,frames=GetNumPartyMembers(),0
 for i=1,members do local f=getglobal("PartyMemberFrame"..i);if UnitExists("party"..i) and f and f:IsVisible() then frames=frames+1 end end
 local total,build=0,0
 if AzCoopPartyUIReadout then
  local roster,own,active=AzCoopPartyUIReadout()
  if active then for slot,row in pairs(roster) do total=total+1;if slot~=own then build=build+table.getn(row.spells)+table.getn(row.items) end end end
 end
 SetCVar("azcoop_probe_report","AZCOOP1125 LOBBYPARTY1133 members="..members.." frames="..frames.." roster="..total.." build="..build)
end)
