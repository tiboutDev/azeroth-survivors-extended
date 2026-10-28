local frame=CreateFrame("Frame",nil,UIParent)
local due=GetTime()+8
frame:SetScript("OnUpdate",function()
 if GetTime()<due then return end
 due=GetTime()+4
 if Screenshot then Screenshot() end
end)
