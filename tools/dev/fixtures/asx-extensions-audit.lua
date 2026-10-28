-- Sparse native UI readback, diagnostic only. No saved CVar or input.
local f=CreateFrame("Frame")
local last,seen=-100,nil
local text=f:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
text:SetPoint("TOPLEFT",UIParent,"TOPLEFT",10,-65);text:SetWidth(650);text:SetHeight(140);text:SetJustifyH("LEFT");text:SetJustifyV("TOP")
local history={"ASX EXTENSIONS - actual native UI readback"}
f:SetScript("OnUpdate",function()
 local now=GetTime();if now-last<2 then return end;last=now
 if not AzCoopOfficialPartyReadout then return end
 local _,_,_,_,b=AzCoopOfficialPartyReadout();if not b or b.boss==seen then return end
 local found,permanent=nil,nil
 for i=0,31 do
  local slot,flag=GetPlayerBuff(i,"HELPFUL")
  local tex=slot and slot>=0 and GetPlayerBuffTexture(slot)
  if tex and string.find(string.lower(tex),"inv_misc_orb_02",1,true)then found=slot;permanent=flag;break end
 end
 if not found then return end
 local toast=ASXDarkmoonBlessingToast and ASXDarkmoonBlessingToast:IsShown()
 if not toast then return end
 GameTooltip:SetOwner(UIParent,"ANCHOR_NONE");GameTooltip:SetPlayerBuff(found)
 local title=GameTooltipTextLeft1 and GameTooltipTextLeft1:GetText()or""
 local valid=string.find(title,"Sayge's Dark Fortune",1,true)and true or false
 GameTooltip:Hide()
 -- Reserved diagnostic channel is unavailable on 0.120.1; retain visible readback.
 table.insert(history,"Boss "..b.boss.." | category "..b.category.." | permanent "..tostring(permanent==1).." | tooltip "..tostring(valid).." | toast "..tostring(toast and true or false))
 text:SetText(table.concat(history,"\n"))
 seen=b.boss
end)
