local frame=CreateFrame("Frame",nil,UIParent)
local due=GetTime()+3
frame:SetScript("OnUpdate",function()
 if GetTime()<due then return end
 due=GetTime()+2
 if Screenshot then Screenshot() end
end)
