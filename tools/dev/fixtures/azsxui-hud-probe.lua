-- Isolated native HUD inspection only; not part of the shipped addon.
local frame=CreateFrame('Frame','AZSXUIHudProbe',UIParent)
local last,count=-100,0
local function emit(rows)error('AZSXUI_HUD_PROBE '..table.concat(rows,' | '),0)end
frame:SetScript('OnUpdate',function()
 local now=GetTime();if now<6 or now-last<8 or count>=4 then return end
 last=now;count=count+1
 local rows={'sample='..count,'phase='..tostring(AZS_RUN and AZS_RUN.ph)}
 local hud=AZSXUI_HudReadout and AZSXUI_HudReadout()
 local frames=AZSXUI_FramesReadout and AZSXUI_FramesReadout()
 table.insert(rows,"standaloneHUD="..tostring(hud and hud.enabled and frames and frames.vitalsShown and frames.health~="—" and frames.level~="" and not AZSXUI_HasModAPI() and not AZSXCooperativeRulesBuff and not AZSXUIEngineeringBadge))
 for _,name in ipairs({'MainMenuBar','MainMenuBarArtFrame','MainMenuBarLeftEndCap','MainMenuBarRightEndCap','MainMenuExpBar','AZS_MainMenuBarTexture4','CharacterFinger0Slot','CharacterFinger1Slot','CharacterNeckSlot','CharacterTrinket0Slot','CharacterTrinket1Slot'})do
  local v=getglobal(name);table.insert(rows,name..'='..tostring(v and v:GetObjectType())..':'..tostring(v and v:GetWidth())..':'..tostring(v and v:IsShown()))
 end
 for _,slot in ipairs({2,11,12,13,14})do
  local ok,texture=pcall(GetInventoryItemTexture,'player',slot)
  local ok2,link=pcall(GetInventoryItemLink,'player',slot)
  table.insert(rows,'inventory'..slot..'='..(ok and tostring(texture)or'error')..':'..(ok2 and tostring(link)or'error'))
  if GameTooltip and GameTooltip.SetInventoryItem then
   local ok3=pcall(function()GameTooltip:SetOwner(frame,'ANCHOR_NONE');GameTooltip:SetInventoryItem('player',slot)end)
   local title=getglobal('GameTooltipTextLeft1');table.insert(rows,'tooltip'..slot..'='..tostring(ok3)..':'..tostring(title and title:GetText()));GameTooltip:Hide()
  end
 end
 local names={};for key,value in pairs(getfenv(0))do if type(key)=='string'and(string.find(key,'AZS',1,true)or string.find(string.lower(key),'jewel',1,true))then table.insert(names,key..':'..type(value))end end
 table.sort(names);for i=1,math.min(160,table.getn(names))do table.insert(rows,names[i])end
 if AZS_STATS then local keys={};for key,value in pairs(AZS_STATS)do table.insert(keys,key..'='..tostring(value))end;table.sort(keys);table.insert(rows,'stats='..table.concat(keys,','))end
 emit(rows)
end)
