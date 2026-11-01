-- Explicit diagnostic UI actions, separate from the passive observer.
if not ASXDiagnosticConfig or not ASXDiagnosticConfig.coop_ui then return end
local f=CreateFrame("Frame",nil,UIParent)
local started,stage=nil,0
ASXCoopUIActions={stage=0}
f:SetScript("OnUpdate",function()
 if not ASXUI_HudReadout or not ASXUI_HudReadout().enabled then return end
 if AZS_GetPhase and AZS_GetPhase()~="playing"then return end
 local now=GetTime();if not started then started=now end
 local age=now-started
 local nextStage=age>=50 and 4 or age>=42 and 3 or age>=38 and 2 or age>=30 and 1 or 0
 if nextStage<=stage then return end;stage=nextStage;ASXCoopUIActions.stage=stage
 if stage<=2 then ToggleCharacter("PaperDollFrame")else
  local b=ASXUIHudStats;local click=b and b:GetScript("OnClick")
  if click then local old=this;this=b;click();this=old end
 end
end)
