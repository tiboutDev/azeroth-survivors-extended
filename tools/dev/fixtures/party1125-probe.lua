-- Read-only evidence in a bounded, opt-in native diagnostic mailbox.
-- No protected movement calls, error-handler replacement or deliberate Lua errors.
RegisterCVar("azcoop_probe_report", "")
local probe=CreateFrame("Frame","AzCoop1125PartyProbe",UIParent)
local last=0
probe:SetScript("OnUpdate",function()
 local now=GetTime()
 if now-last<1 or not UnitExists("player") then return end
 last=now
 local count=GetNumPartyMembers()
 local shown=0
 for i=1,count do
  local frame=getglobal("PartyMemberFrame"..i)
  if UnitExists("party"..i) and frame and frame:IsVisible() then shown=shown+1 end
 end
 local own,ally=nil,nil
 if GetRaidTargetIndex then own=GetRaidTargetIndex("player");if UnitExists("party1") then ally=GetRaidTargetIndex("party1") end end
 SetCVar("azcoop_probe_report","AZCOOP1125 class="..tostring(UnitClass("player")).." members="..tostring(count).." frames="..tostring(shown).." raid_self="..tostring(own).." raid_party="..tostring(ally))
end)
