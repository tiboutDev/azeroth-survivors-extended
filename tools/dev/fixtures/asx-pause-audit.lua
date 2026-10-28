-- Diagnostic only: native client readback. No OS input, CVar or position writes.
local f=CreateFrame("Frame",nil,UIParent)
local text=f:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
text:SetPoint("TOPLEFT",UIParent,"TOPLEFT",10,-65)
text:SetWidth(900);text:SetHeight(200);text:SetJustifyH("LEFT");text:SetJustifyV("TOP")
local last=-100
local history={"ASX PAUSE - native client position readback"}
f:SetScript("OnUpdate",function()
 local now=GetTime();if now-last<2 then return end;last=now
 local x,y
 if GetPlayerMapPosition then x,y=GetPlayerMapPosition("player") end
 local phase=AZS_RUN and AZS_RUN.ph or "unknown"
 local clock=AZS_RUN and AZS_RUN.t or -1
 table.insert(history,string.format("UI %.1f | phase %s | run %s | map %s / %s",now,tostring(phase),tostring(clock),tostring(x),tostring(y)))
 while table.getn(history)>10 do table.remove(history,2)end
 text:SetText(table.concat(history,"\n"))
end)
