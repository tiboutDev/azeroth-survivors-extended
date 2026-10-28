-- Read-only native-frame probe. No errors, protected actions or UI suppression.
RegisterCVar("azcoop_native_probe", "")
local observer=CreateFrame("Frame","AzCoopOfficialPartyProbe",UIParent)
local previous=0
local ownerTip=CreateFrame("GameTooltip","AzCoopOfficialOwnerTip",UIParent,"GameTooltipTemplate")
local function hex(s)return string.gsub(tostring(s or ""),".",function(c)return string.format("%02x",string.byte(c))end)end
local function tipLines(name)
 local lines={}
 for n=1,16 do
  local left=getglobal(name.."TextLeft"..n);local right=getglobal(name.."TextRight"..n)
  if left and left:IsShown() and left:GetText()then
   local r,g,b=left:GetTextColor();local rr,rg,rb=1,1,1
   if right then rr,rg,rb=right:GetTextColor()end
   local function rgb(r,g,b)return math.floor(r*255+0.5).."."..math.floor(g*255+0.5).."."..math.floor(b*255+0.5)end
   table.insert(lines,hex(left:GetText())..":"..hex(right and right:IsShown() and right:GetText() or "")..":"..rgb(r,g,b)..":"..rgb(rr,rg,rb))
  end
 end
 return table.concat(lines,",")
end
local function safely(f)
 local ok,value=pcall(f)
 if not ok then return "unsupported" end
 return tostring(value or "nil")
end
observer:SetScript("OnUpdate",function()
 local now=GetTime()
 if now-previous<1 then return end
 previous=now
 local count=GetNumPartyMembers()
 local view,built,own={},0,nil
 if AzCoopOfficialPartyReadout then view,built,own=AzCoopOfficialPartyReadout()end
 if own and count==2 then
  for slot=1,6 do
   if HasAction(slot)then
    ownerTip:SetOwner(UIParent,"ANCHOR_NONE");ownerTip:ClearLines();ownerTip:SetAction(slot)
    SetCVar("azcoop_native_probe","AZCOOP_PARTY115 own-tip="..own..","..slot..","..tipLines("AzCoopOfficialOwnerTip"))
    ownerTip:Hide()
   end
  end
 end
 local rows={"time="..now,"members="..count,"self="..hex(UnitName("player")),"built="..built,"run="..safely(function()return AZS_RUN.t end),"ownbuild="..safely(function()return GetCVar(AZS_HUD_CVARS.build)end)}
 for i=1,4 do
  local unit="party"..i
  local frame=getglobal("PartyMemberFrame"..i)
  local health=getglobal("PartyMemberFrame"..i.."HealthBar")
  local buffs=0
  for n=1,32 do if UnitBuff(unit,n) then buffs=buffs+1 end end
  local fields={i,hex(UnitName(unit)),safely(function()if UnitGUID then return UnitGUID(unit)end return "unavailable"end),safely(function()return UnitExists(unit)end),safely(function()return frame and frame:IsVisible() or false end),safely(function()return UnitHealth(unit)end),safely(function()return UnitHealthMax(unit)end),safely(function()return health and health:GetValue() or "missing"end),buffs,safely(function()return UnitIsConnected(unit)end)}
  for n=1,table.getn(fields) do fields[n]=tostring(fields[n])end
  table.insert(rows,"frame="..table.concat(fields,","))
  local data=view[i]
  if data then table.insert(rows,"extra="..i..","..hex(data.owner)..","..data.buffs..","..tostring(data.attached))end
 end
 for i,data in pairs(view)do
  if data.member then
   for _,kind in ipairs({"spells","items"})do
    for n,button in ipairs(data[kind])do
     if button:IsVisible() and button.data then
      local old=this;this=button
      button:GetScript("OnEnter")()
      local title=GameTooltipTextLeft1 and GameTooltipTextLeft1:GetText() or ""
      local detail=GameTooltipTextLeft2 and GameTooltipTextLeft2:IsShown() and GameTooltipTextLeft2:GetText() or ""
      local rendered=tipLines("GameTooltip")
      SetCVar("azcoop_native_probe","AZCOOP_PARTY115 remote-tip="..data.member.id..","..kind..","..button.data.id..","..button.data.rank..","..rendered)
      button:GetScript("OnLeave")();this=old
      local item=button.data
      table.insert(rows,"build="..i..","..data.member.id..","..kind..","..item.id..","..item.rank..","..hex(item.name)..","..hex(item.icon)..","..hex(item.text)..","..hex(title)..","..hex(detail))
     end
    end
   end
  end
 end
 SetCVar("azcoop_native_probe","AZCOOP_PARTY115 "..table.concat(rows," "))
end)
