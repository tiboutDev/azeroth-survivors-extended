-- Native HUD toggles are persistent CVars. Keep the player's original values
-- in SavedVariables as well as restoring them on a normal logout.
local nativeParts={"build","tally","clock","loot","revive"}
local closing=false
local function originals()
 if type(ASXUIDB)~="table"then ASXUIDB={}end
 if type(ASXUIDB.nativeHudVisibility)~="table"then ASXUIDB.nativeHudVisibility={}end
 return ASXUIDB.nativeHudVisibility
end
local function allowed(part)for _,name in ipairs(nativeParts)do if name==part then return true end end end
function ASXUI_ClaimNativeHud(part)
 if not allowed(part)or not AZS_IsHudShown or not AZS_SetHudShown then return nil end
 local saved=originals();local value=saved[part]
 if type(value)~="boolean"then value=AZS_IsHudShown(part)and true or false;saved[part]=value end
 if not closing and AZS_IsHudShown(part)then AZS_SetHudShown(part,false)end
 return value
end
function ASXUI_RestoreNativeHud(part,retain)
 if not AZS_IsHudShown or not AZS_SetHudShown then return end
 if part==nil then for _,name in ipairs(nativeParts)do ASXUI_RestoreNativeHud(name,retain)end;return end
 if not allowed(part)then return end
 local saved=originals();local value=saved[part]
 if closing then retain=true end
 if not retain then saved[part]=nil end
 -- Preserve an externally restored shown=true value. Owned writes only hide.
 if value==true and not AZS_IsHudShown(part)then AZS_SetHudShown(part,true)end
end
local visibility=CreateFrame("Frame","ASXUINativeVisibilityController",UIParent)
visibility:RegisterEvent("ADDON_LOADED");visibility:RegisterEvent("PLAYER_LOGOUT");visibility:RegisterEvent("PLAYER_ENTERING_WORLD")
visibility:SetScript("OnEvent",function()
 if event=="ADDON_LOADED"and arg1=="ASXUI"then closing=false;ASXUI_RestoreNativeHud()
 elseif event=="PLAYER_LOGOUT"then closing=true;ASXUI_RestoreNativeHud(nil,true)
 elseif event=="PLAYER_ENTERING_WORLD"then closing=false end
end)

-- Shared native steel dialog frame and opaque dark stone. No bundled images.
function ASXUI_StylePanel(frame,warm)
 frame:SetBackdrop({bgFile="Interface\\DialogFrame\\UI-DialogBox-Background",edgeFile="Interface\\DialogFrame\\UI-DialogBox-Border",tile=true,tileSize=16,edgeSize=warm and 12 or 16,insets={left=6,right=6,top=6,bottom=6}})
 frame:SetBackdropColor(.065,.06,.05,1);frame:SetBackdropBorderColor(1,1,1,1)
 if not frame.asxPanelFill then
  local fill=frame:CreateTexture(nil,"BACKGROUND");fill:SetTexture(.035,.031,.025,1);fill:SetPoint("TOPLEFT",frame,"TOPLEFT",6,-6);fill:SetPoint("BOTTOMRIGHT",frame,"BOTTOMRIGHT",-6,6);frame.asxPanelFill=fill
 end
 if warm then
  -- Unmarked paper inside the native quest-log atlas, darkened for combat.
  -- Continue the paper beneath the foreground border, without a black inset.
  frame.asxPanelFill:ClearAllPoints();frame.asxPanelFill:SetPoint("TOPLEFT",frame,"TOPLEFT",2,-2);frame.asxPanelFill:SetPoint("BOTTOMRIGHT",frame,"BOTTOMRIGHT",-2,2)
  frame.asxPanelFill:SetTexture("Interface\\QuestFrame\\UI-QuestLog-TopLeft")
  frame.asxPanelFill:SetTexCoord(32/256,240/256,190/256,250/256)
  frame.asxPanelFill:SetVertexColor(.24,.24,.24,1)
 end
end

-- Open-top ribbon: the native header's vertical sides/body and lower trim.
-- Its upper rail/corners are deliberately omitted. Body and bottom trim have
-- separate vertical sizing so the icon padding cannot stretch the lower edge.
function ASXUI_StyleTopCounters(frame)
 if frame.asxCounterOrnaments then return end
 frame:SetBackdrop(nil)
 frame.asxCounterOrnaments={}
 local atlas="Interface\\DialogFrame\\UI-DialogBox-Header"
 for _,part in ipairs({{62,82,"LEFT"},{82,174},{174,194,"RIGHT"}})do
  for _,strip in ipairs({{12,28,36,"TOP"},{28,38,10,"BOTTOM"}})do
   local t=frame:CreateTexture(nil,"BORDER");t:SetTexture(atlas)
   t:SetTexCoord(part[1]/256,part[2]/256,strip[1]/64,strip[2]/64);t:SetHeight(strip[3])
   if part[3]then t:SetWidth(20);local point=strip[4]..part[3];t:SetPoint(point,frame,point,0,0)
   else t:SetPoint(strip[4].."LEFT",frame,strip[4].."LEFT",20,0);t:SetPoint(strip[4].."RIGHT",frame,strip[4].."RIGHT",-20,0)end
   table.insert(frame.asxCounterOrnaments,t)
  end
 end
end

-- Capsule made only from the native white texture and native circular portrait
-- mask. The center overlaps half of each cap, avoiding rectangular corners.
function ASXUI_CreatePill(parent,name,width,height,r,g,b,texture)
 texture=texture or"Interface\\Buttons\\WHITE8X8"
 local frame=CreateFrame("Frame",name,parent);frame:SetWidth(width);frame:SetHeight(height)
 local colors={}
 for _,side in ipairs({"Left","Right"})do
  local point=side=="Left"and"TOPLEFT"or"TOPRIGHT"
  if texture=="Interface\\TargetingFrame\\UI-StatusBar"then
   -- The native status-bar bitmap is 64x8; portrait-masking that non-square
   -- image cuts off its ends. Use the native square white mask for the caps,
   -- shaded with the reviewed bitmap's eight grayscale rows. The body keeps
   -- the original status-bar texture and no replacement image is bundled.
   local shades={136,189,216,189,144,101,122,101}
   for row,shade in ipairs(shades)do
    local t=frame:CreateTexture(name..side..row,"ARTWORK");t:SetWidth(height);t:SetHeight(height/8)
    t:SetPoint(point,frame,point,0,-(row-1)*height/8)
    SetPortraitToTexture(t:GetName(),"Interface\\Buttons\\WHITE8X8")
    t:SetTexCoord(0,1,(row-1)/8,row/8);local v=shade/255;t:SetVertexColor(r*v,g*v,b*v,1);table.insert(colors,{t,v})
   end
  else
   local t=frame:CreateTexture(name..side,"ARTWORK");t:SetWidth(height);t:SetHeight(height)
   t:SetPoint(string.upper(side),frame,string.upper(side),0,0)
   SetPortraitToTexture(t:GetName(),texture);t:SetVertexColor(r,g,b,1);table.insert(colors,{t,1})
  end
 end
 local center=frame:CreateTexture(nil,"ARTWORK");center:SetTexture(texture);center:SetVertexColor(r,g,b,1)
 center:SetPoint("TOPLEFT",frame,"TOPLEFT",height/2,0);center:SetPoint("BOTTOMRIGHT",frame,"BOTTOMRIGHT",-height/2,0)
 table.insert(colors,{center,1})
 function frame:SetPillColor(red,green,blue)
  for _,entry in ipairs(colors)do local v=entry[2];entry[1]:SetVertexColor(red*v,green*v,blue*v,1)end
 end
 frame.centerTexture=center;return frame
end
