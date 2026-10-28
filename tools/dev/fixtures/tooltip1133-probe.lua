-- Owner and rendered ally tooltip comparison. Read-only native UI callbacks.
local probe=CreateFrame("Frame","AzCoopTooltip1133Probe",UIParent)
local at=0
local function hex(s)return string.gsub(s or "",".",function(c)return string.format("%02x",string.byte(c))end)end
local function split(s)local out={};for p in string.gfind(s.."|","(.-)|") do table.insert(out,p) end;return out end
local function rgb(f)local r,g,b=f:GetTextColor();return math.floor((r or 1)*255+0.5)..","..math.floor((g or 1)*255+0.5)..","..math.floor((b or 1)*255+0.5)end
local function lines(name)
 local out={}
 for n=1,12 do local l=getglobal(name.."TextLeft"..n);local r=getglobal(name.."TextRight"..n)
  if l and l:IsShown() and l:GetText() then table.insert(out,hex(l:GetText())..":"..hex(r and r:IsShown() and r:GetText()or"")..":"..rgb(l)..":"..(r and rgb(r)or"255,255,255")) end
 end
 return table.concat(out,";")
end
local function emit(s)if string.len(s)<=2048 then SetCVar("azcoop_probe_report",s)end end
probe:SetScript("OnUpdate",function()
 if GetTime()-at<1 or not AzCoopPartyUIReadout or not AzCoopPartyUITooltipSourceReadout then return end
 at=GetTime();local roster,own,active=AzCoopPartyUIReadout();if not active then return end
 local function prefix(kind,category,owner,id,rank,name)return "AZCOOP1125 TIP own="..own.." kind="..kind.." category="..category.." owner="..owner.." id="..id.." rank="..rank.." name="..name end
 local captures={}
 for line in string.gfind(AzCoopPartyUITooltipSourceReadout().."\n","([^\n]+)\n")do
  local a=split(line)
  if table.getn(a)==8 then
   local key=prefix("source","S",own,a[1],a[2],a[3]);captures[key]=captures[key]or{};table.insert(captures[key],a[4]..":"..a[5]..":"..a[6]..":"..a[7])
  end
 end
 for key,rows in pairs(captures)do emit(key.." rows="..table.concat(rows,";"))end
 if roster[own]then for _,item in ipairs(roster[own].items)do
  local desc=item.tooltip and item.tooltip[1]and item.tooltip[1].left or"";if desc~=""then emit(prefix("itemsource","I",own,item.id,item.rank,hex(item.name)).." desc="..hex(desc))end
 end end
 for slot,member in pairs(roster)do if slot~=own and member.index>0 then
  local card=getglobal("AzCoopPartyCard"..member.index)
  for _,category in ipairs({"spells","items"})do for n,icon in ipairs(member[category])do
   if card and icon.tooltip and table.getn(icon.tooltip)>0 then
    local button=card[category][n];local old=this;this=button;button:GetScript("OnEnter")();local body=lines("GameTooltip");button:GetScript("OnLeave")();this=old
    emit(prefix("remote",category=="spells"and"S"or"I",slot,icon.id,icon.rank,hex(icon.name)).." rows="..body)
   end
  end end
 end end
end)
