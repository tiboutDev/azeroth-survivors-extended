local recoveryProbe=CreateFrame("Frame",nil,UIParent)
local recoveryLast="";local recoveryDue=0;local recoveryWaitTested=false
recoveryProbe:SetScript("OnUpdate",function()
 if GetTime()<recoveryDue then return end;recoveryDue=GetTime()+0.25
 local frame=AzCoopRecovery
 if not frame or not frame.epoch then return end
 local value="shown="..tostring(frame:IsShown() and true or false).." epoch="..frame.epoch.." phase="..tostring(frame.phase).." timeout="..tostring(frame.timedOut)
 if frame.own==1 and frame.timedOut and not recoveryWaitTested then
  recoveryWaitTested=true;frame.waitButton:GetScript("OnClick")();error("AZCOOP_RECOVERY_WAIT_CLICK")
 end
 if value~=recoveryLast then recoveryLast=value;error("AZCOOP_RECOVERY_UI "..value) end
end)
