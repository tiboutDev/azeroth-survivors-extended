-- Read-only owner/remote tooltip comparison in disposable validation profiles.
local probe=CreateFrame("Frame","AzCoopOwnerTooltipProbe",UIParent)
local at=0
local function hex(value)return string.gsub(value or "", ".",function(c)return string.format("%02x",string.byte(c))end)end
local function rgb(font)
 local r,g,b=font:GetTextColor();return math.floor((r or 1)*255+0.5)..","..math.floor((g or 1)*255+0.5)..","..math.floor((b or 1)*255+0.5)
end
local function linesOf(name)
 local out={}
 for n=1,12 do
  local left=getglobal(name.."TextLeft"..n);local right=getglobal(name.."TextRight"..n)
  if left and left:IsShown() and left:GetText() then
   table.insert(out,hex(left:GetText())..":"..hex(right and right:IsShown() and right:GetText() or "")..":"..rgb(left)..":"..(right and rgb(right) or "255,255,255"))
  end
 end
 return table.concat(out,",")
end
probe:SetScript("OnUpdate",function()
 if GetTime()-at<1 or not AzCoopPartyUIReadout or not AzCoopPartyUITooltipSourceReadout then return end
 at=GetTime()
 local roster,own,active=AzCoopPartyUIReadout()
 if not active then return end
 local messages={"source="..hex(AzCoopPartyUITooltipSourceReadout())}
 if roster[own] then
  for _,item in ipairs(roster[own].items) do
   local text=item.tooltip and item.tooltip[1] and item.tooltip[1].left or ""
   table.insert(messages,"item_source="..item.rank.."/"..hex(item.name).."/"..hex(text))
  end
 end
 for slot,member in pairs(roster) do
  if slot~=own and member.index>0 then
   local card=getglobal("AzCoopPartyCard"..member.index)
   for _,kind in ipairs({"spells","items"}) do
    for n,icon in ipairs(member[kind]) do
     if card and icon.tooltip and table.getn(icon.tooltip)>0 then
      local button=card[kind][n];local old=this;this=button
      button:GetScript("OnEnter")()
      local rendered=linesOf("GameTooltip");button:GetScript("OnLeave")();this=old
      table.insert(messages,"remote="..slot.."/"..kind.."/"..icon.id.."/"..icon.rank.."/"..hex(icon.name).."/"..rendered)
     end
    end
   end
  end
 end
 error("AZCOOP_OWNER_TOOLTIP own="..own.." "..table.concat(messages," "),0)
end)
