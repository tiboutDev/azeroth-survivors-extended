-- Native HUD toggles are persistent CVars. Keep the player's original values
-- in SavedVariables as well as restoring them on a normal logout.
local nativeParts={"build","tally","clock","loot","revive","bosses"}
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

-- Native dialog cartouche with complete rails and ornamented corners. Only
-- the straight body stretches vertically; top/bottom corners keep their size.
function ASXUI_StyleHeader(frame)
 frame:SetBackdrop(nil);if frame.asxHeaderOrnaments then return end
 frame.asxHeaderOrnaments={}
 local fill=frame:CreateTexture(nil,"BACKGROUND");fill:SetTexture(.035,.031,.025,1);fill:SetPoint("TOPLEFT",frame,"TOPLEFT",7,-7);fill:SetPoint("BOTTOMRIGHT",frame,"BOTTOMRIGHT",-7,7)
 local atlas="Interface\\DialogFrame\\UI-DialogBox-Header"
 for _,part in ipairs({{62,82,"LEFT"},{82,174},{174,194,"RIGHT"}})do
  for _,strip in ipairs({{2,12,10,"TOP",0},{12,28,frame:GetHeight()-20,"TOP",-10},{28,38,10,"BOTTOM",0}})do
   local t=frame:CreateTexture(nil,"BORDER");t:SetTexture(atlas);t:SetTexCoord(part[1]/256,part[2]/256,strip[1]/64,strip[2]/64);t:SetHeight(strip[3])
   if part[3]then t:SetWidth(20);local point=strip[4]..part[3];t:SetPoint(point,frame,point,0,strip[5])
   else t:SetPoint(strip[4].."LEFT",frame,strip[4].."LEFT",20,strip[5]);t:SetPoint(strip[4].."RIGHT",frame,strip[4].."RIGHT",-20,strip[5])end
   table.insert(frame.asxHeaderOrnaments,t)
  end
 end
end
function ASXUI_StyleTopCounters(frame)ASXUI_StyleHeader(frame)end

-- Native 128px portrait alpha mask preserves smooth caps at HUD/plate sizes.
-- Body and caps share the native status-bar bitmap's eight-row grayscale
-- ramp at pixel-sized bands. Equal shading avoids a cap/body modulation seam.
function ASXUI_CreatePill(parent,name,width,height,r,g,b,texture)
 local mask="Interface\\CharacterFrame\\TempPortraitAlphaMask"
 local gradient=texture=="Interface\\TargetingFrame\\UI-StatusBar"
 local frame=CreateFrame("Frame",name,parent);frame:SetWidth(width);frame:SetHeight(height)
 local colors={};local bodyShades={};local shades={136,189,216,189,144,101,122,101}
 local bands=gradient and math.ceil(height)or 1
 for _,side in ipairs({"Left","Right"})do
  local point=side=="Left"and"TOPLEFT"or"TOPRIGHT"
  for row=1,bands do
   local t=frame:CreateTexture(name..side..row,"ARTWORK");t:SetWidth(height);t:SetHeight(height/bands)
   t:SetPoint(point,frame,point,0,-(row-1)*height/bands)
   t:SetTexture(mask);t:SetTexCoord(0,1,(row-1)/bands,row/bands)
   local v=1
   if gradient then
    local sample=math.max(0,math.min(7,((row-.5)/bands)*8-.5));local low=math.floor(sample)
    v=(shades[low+1]+(shades[math.min(8,low+2)]-shades[low+1])*(sample-low))/255
   end
   bodyShades[row]=v
   t:SetVertexColor(r*v,g*v,b*v,1);table.insert(colors,{t,v})
  end
 end
 local center
 for row=1,bands do
  local t=frame:CreateTexture(nil,"ARTWORK");t:SetTexture("Interface\\ChatFrame\\ChatFrameBackground")
  t:SetPoint("TOPLEFT",frame,"TOPLEFT",height/2,-(row-1)*height/bands)
  t:SetPoint("TOPRIGHT",frame,"TOPRIGHT",-height/2,-(row-1)*height/bands);t:SetHeight(height/bands)
  local v=bodyShades[row];t:SetVertexColor(r*v,g*v,b*v,1);table.insert(colors,{t,v});center=center or t
 end
 function frame:SetPillColor(red,green,blue)
  for _,entry in ipairs(colors)do local v=entry[2];entry[1]:SetVertexColor(red*v,green*v,blue*v,1)end
 end
 frame.centerTexture=center;return frame
end

-- ASX owns the wording; only numeric bonus tokens receive native green markup.
-- Native loot receipts keep their original item rarity and message formatting.
function ASXUI_FormatBonus(text)
 return (string.gsub(text or"","([%+%-]?%d+%.?%d*%%?)","|cff1eff00%1|r"))
end

-- Thin native frame for statistics; its bottom edge is clipped by Hud.lua.
-- The objectives ribbon uses the native XP/casting trim at its natural height,
-- with no right end cap against the screen. No stretched dialog-title atlas.
function ASXUI_StyleOpenPanel(frame,rightOpen)
 if frame.asxOpenFill then return end
 local fill=frame:CreateTexture(nil,"BACKGROUND");fill:SetTexture(.035,.031,.025,1);frame.asxOpenFill=fill
 if not rightOpen then
  frame:SetBackdrop({bgFile="Interface\\DialogFrame\\UI-DialogBox-Background",edgeFile="Interface\\Tooltips\\UI-Tooltip-Border",tile=true,tileSize=16,edgeSize=8,insets={left=3,right=3,top=3,bottom=3}})
  frame:SetBackdropColor(.08,.07,.055,1);frame:SetBackdropBorderColor(.8,.77,.68,1)
  fill:SetPoint("TOPLEFT",frame,"TOPLEFT",3,-3);fill:SetPoint("BOTTOMRIGHT",frame,"BOTTOMRIGHT",-3,3)
 else
  frame:SetBackdrop(nil);fill:SetPoint("TOPLEFT",frame,"TOPLEFT",4,-7);fill:SetPoint("BOTTOMRIGHT",frame,"BOTTOMRIGHT",0,6)
  for _,part in ipairs({{22,40},{40,216}})do
   local t=frame:CreateTexture(nil,"BORDER");t:SetTexture("Interface\\CastingBar\\UI-CastingBar-Border");t:SetTexCoord(part[1]/256,part[2]/256,.25,.75);t:SetHeight(32)
   if part[1]==22 then t:SetWidth(18);t:SetPoint("LEFT",frame,"LEFT",-1,0)
   else t:SetPoint("LEFT",frame,"LEFT",17,0);t:SetPoint("RIGHT",frame,"RIGHT",0,0)end
  end
 end
end
