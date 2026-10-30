-- Opt-in read-only event UI audit. No input or gameplay decisions.
local f=CreateFrame("Frame")
local label=f:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
label:SetPoint("TOPLEFT",UIParent,"TOPLEFT",180,-190);label:SetWidth(650);label:SetHeight(90);label:SetJustifyH("LEFT")
local last,lastShot=-1,-100
local shots={}
local ritualAt,pingAt,pingX,pingY,pingStable=nil,nil,nil,nil,nil
f:SetScript("OnUpdate",function()
 local now=GetTime();if now-last<1 then return end;last=now
 if not ASXSupplyReadout then return end
 local buffs,toast,discovery=ASXSupplyReadout();local pins=ASXEventPinsReadout();local id,held,inside,shown=ASXRitualReadout()
 if id and not ritualAt then ritualAt=now end
 if id and ritualAt and now-ritualAt>4 and not pingAt and table.getn(pins)>0 and Minimap and Minimap.PingLocation then
  pingAt=now;pingX=pins[1].x;pingY=pins[1].y;Minimap:PingLocation(.15,.1)
 elseif pingAt and pingStable==nil and now-pingAt>1 and table.getn(pins)>0 then
  pingStable=math.abs(pins[1].x-pingX)<2 and math.abs(pins[1].y-pingY)<2
 end
 local title="COOP EVENTS / native UI\nBuffs="..table.getn(buffs).." discovery="..tostring(discovery).." pins="..table.getn(pins).."\nRitual="..tostring(id).." held="..tostring(held).." inside="..tostring(inside).." panel="..tostring(shown).." / Ping stable="..tostring(pingStable)
 label:SetText(title)
 local key
 if id then key=inside and "ritual-inside"or"ritual-outside" elseif table.getn(pins)>0 then key="supplies" elseif discovery then key="claimed"end
 if key and not shots[key]and now-lastShot>=10 and Screenshot then shots[key]=true;lastShot=now;Screenshot()end
end)
