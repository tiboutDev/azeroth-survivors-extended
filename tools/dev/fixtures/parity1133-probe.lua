-- Read-only UI observations; never selects a target or raises script errors.
local f=CreateFrame("Frame",nil,UIParent)
local last=0
f:SetScript("OnUpdate",function()
 local now=GetTime()
 if now-last<1 or not UnitExists("player") then return end
 last=now
 local hp,bar,tip=-1,-1,-1
 if UnitExists("target") and UnitCanAttack("player","target") then
  hp=UnitHealth("target")
  if GameTooltip then
   GameTooltip:SetOwner(UIParent,"ANCHOR_NONE")
   GameTooltip:SetUnit("target")
   GameTooltip:Show()
  end
  bar=TargetFrameHealthBar and TargetFrameHealthBar:GetValue() or -1
  tip=GameTooltipStatusBar and GameTooltipStatusBar:GetValue() or -1
 end
 local buildKey=AZS_HUD_CVARS and AZS_HUD_CVARS.build or "missing"
 local ok,buildShown=pcall(GetCVar,buildKey)
 local roster,own,active=AzCoopPartyUIReadout()
 local down,revive,reviver=0,0,0
 for slot,row in pairs(roster) do
  if row.down then down=down+1 end
  if slot==own then revive=row.revive;reviver=row.reviver end
 end
 SetCVar("azcoop_probe_report","AZCOOP1125 probe=parity hp="..hp.." bar="..bar.." tip="..tip.." down="..down.." revive="..revive.." reviver="..reviver.." center="..tostring(AzCoopDownedNotice:IsVisible()).." build="..tostring(ok and buildShown or "unregistered").." build_key="..tostring(buildKey))
end)
