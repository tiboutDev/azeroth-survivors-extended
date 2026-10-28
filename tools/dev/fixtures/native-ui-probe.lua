-- Explicit disposable UI audit only. At most one topology report and 12 events.
-- The selected sandbox has no print/file output; tagged errors are audit records.
local previous=geterrorhandler and geterrorhandler()
if seterrorhandler then seterrorhandler(function(message)
 if type(message)=="string" and string.find(message,"ASX_UI_PROBE",1,true) then return end
 if previous then previous(message) end
end) end
if SetCVar then pcall(SetCVar,"nameplateShowEnemies","1") end
local frame=CreateFrame("Frame",nil,UIParent)
local done,last,count,enabled=false,-100,0,false
local function safe(call)
 local ok,value=pcall(call)
 return ok and tostring(value) or "unsupported"
end
local function report(text) error("ASX_UI_PROBE "..text,0) end
frame:RegisterEvent("UNIT_COMBAT")
frame:RegisterEvent("COMBAT_TEXT_UPDATE")
frame:SetScript("OnEvent",function()
 if count>=12 then return end
 count=count+1
 report("event="..tostring(event).." unit="..tostring(arg1).." kind="..tostring(arg2).." arg3="..tostring(arg3).." arg4="..tostring(arg4).." arg5="..tostring(arg5))
end)
frame:SetScript("OnUpdate",function()
 local now=GetTime();if now-last<2 then return end;last=now
 if not enabled and now>=40 then enabled=true; pcall(SetCVar,"nameplateShowEnemies","1"); if ShowNameplates then pcall(ShowNameplates) end end
 if done or now<45 then return end
 done=true
 local rows={"font_type="..type(DAMAGE_TEXT_FONT),"font="..tostring(DAMAGE_TEXT_FONT),"set_marker="..type(SetRaidTarget),"get_marker="..type(GetRaidTargetIndex),"combat_add="..type(CombatText_AddMessage),"combat_frame="..type(CombatText)}
 if WorldFrame then
  local ok,children=pcall(function()return {WorldFrame:GetChildren()}end)
  table.insert(rows,"world_children="..(ok and table.getn(children)or "unsupported"))
  if ok then
   for n=1,math.min(table.getn(children),32)do
    local child=children[n]
    table.insert(rows,"child="..safe(function()return child:GetName()end)..":"..safe(function()return child:GetObjectType()end)..":"..safe(function()return child:IsShown()end))
    local ok2,regions=pcall(function()return {child:GetRegions()}end)
    if ok2 then for _,region in ipairs(regions)do table.insert(rows,"region="..safe(function()return region:GetObjectType()end)..":"..safe(function()return region:GetTexture()end)..":"..safe(function()return region:GetText()end))end end
    local ok3,subs=pcall(function()return {child:GetChildren()}end)
    if ok3 then for _,sub in ipairs(subs)do table.insert(rows,"sub="..safe(function()return sub:GetObjectType()end)..":"..safe(function()local r,g,b=sub:GetStatusBarColor();return tostring(r)..","..tostring(g)..","..tostring(b)end))end end
   end
  end
 end
 table.insert(rows,"own_marker="..safe(function()return GetRaidTargetIndex("player")end))
 report(table.concat(rows,"|"))
end)
