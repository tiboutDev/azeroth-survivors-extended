-- Opt-in solo visual observations: native presence, retained panel visibility,
-- constant opacity observations and at most four extra native screenshots.
local frame=CreateFrame("Frame")
local label=frame:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
label:SetPoint("TOPLEFT",UIParent,"TOPLEFT",180,-40);label:SetWidth(440);label:SetHeight(65);label:SetJustifyH("LEFT")
local started,lastLabel,lastShot=GetTime(),-1,-100
local insideVisible,outsideHidden,wasOutside=false,false,false
local opacityOK,rewardSeen,removed=true,false,false
local zoomBase,zoomStep,zoomAt,zoomKept=nil,0,0,true
local shots={};local function shot(key,now)
 if not shots[key] and now-lastShot>10 and Screenshot then shots[key]=true;lastShot=now;Screenshot()end
end
frame:SetScript("OnUpdate",function()
 local now=GetTime();if now-started>260 then frame:SetScript("OnUpdate",nil);return end
 if not ASXRitualReadout or not ASXEventVisualReadout then return end
 local id,held,inside,shown=ASXRitualReadout()
 if id and inside and shown then insideVisible=true;if held>=700 then shot("progress",now)end end
 if id and not inside and not shown then outsideHidden=true;if id>=2 then wasOutside=true;shot("outside",now)end end
 if id and inside and held>=300 and Minimap and Minimap.GetZoom and Minimap.SetZoom then
  if not zoomBase then zoomBase=Minimap:GetZoom();zoomAt=now end
  if zoomStep<4 and now-zoomAt>.7 then
   if zoomStep>0 then zoomKept=zoomKept and table.getn(ASXEventPinsReadout())>0 end
   zoomStep=zoomStep+1;zoomAt=now
   Minimap:SetZoom(math.mod(zoomStep,2)==1 and math.mod(zoomBase+1,6) or zoomBase)
  end
 end
 local visible,alpha=ASXEventVisualReadout()
 if visible then rewardSeen=true;opacityOK=opacityOK and alpha==1;shot("reward",now)
 elseif rewardSeen then removed=true;shot("removed",now)end

 if now-lastLabel>=2 then lastLabel=now;label:SetText("ASX UI VALIDATION\nInside visible="..tostring(insideVisible).." / Outside hidden="..tostring(outsideHidden).." / Second outside="..tostring(wasOutside).."\nZoom pins="..tostring(zoomKept).." ("..zoomStep..") / Full opacity="..tostring(opacityOK).." / Removed="..tostring(removed).." / Ritual="..tostring(id).." presence="..tostring(inside).." shown="..tostring(shown))end
end)
