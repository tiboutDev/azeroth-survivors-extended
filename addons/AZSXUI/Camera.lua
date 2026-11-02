-- Explicit camera/inspection modes. Native WorldFrame handlers remain authoritative.
BINDING_HEADER_AZSXUI="AZSXUI"
BINDING_NAME_AZSXUI_CAMERA_TOGGLE="Toggle camera / inspection"
BINDING_NAME_AZSXUI_CAMERA_INSPECT="Release camera for tooltips"
local controller=CreateFrame("Frame","AZSXUICameraController",UIParent)
local mode,pending,delay,scan="inspect",nil,0,0
local hookDown,hookUp
local indicator=controller:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
indicator:SetPoint("BOTTOMRIGHT",UIParent,"BOTTOMRIGHT",-28,182)
indicator:SetText("CAMERA")
indicator:Hide()
local panels={
 "GameMenuFrame","WorldMapFrame","InterfaceOptionsFrame","CharacterFrame",
 "SpellBookFrame","QuestLogFrame","MerchantFrame","GossipFrame",
 "StaticPopup1","StaticPopup2","ChatFrameEditBox","FriendsFrame",
 "TalentFrame","PlayerTalentFrame","AZSTalentFrame","AZSTalentsFrame",
 "MailFrame","BankFrame","AZSOptionsFrame","AZSLevelUpFrame"
}
local function inspectingUI()
 local phase
 if type(AZS_GetPhase)=="function" then
  local ok,value=pcall(AZS_GetPhase)
  if ok then phase=value end
 end
 if not phase and type(AZS_RUN)=="table" then phase=AZS_RUN.ph end
 if phase and phase~="playing" then return true end
 for _,name in ipairs(panels) do
  local frame=getglobal and getglobal(name) or _G[name]
  if frame and frame.IsShown and frame:IsShown() then return true end
 end
 if type(GetCurrentKeyBoardFocus)=="function" then
  local ok,focus=pcall(GetCurrentKeyBoardFocus)
  if ok and focus then return true end
 end
 return false
end
local function available()
 return type(MouselookStart)=="function" and type(MouselookStop)=="function"
end
local function conflict()
 -- This addon also owns WorldFrame and reacquires mouselook continuously.
 return type(SCM_Toggle)=="function" or type(SCM_Lock)=="function"
end
local function stop()
 pending=nil;delay=0;mode="inspect"
 indicator:Hide()
 if available() then pcall(MouselookStop) end
end
local function start()
 if not available() or conflict() or inspectingUI() then return false end
 local ok=pcall(MouselookStart)
 if ok then mode="camera";indicator:Show() end
 return ok
end
local function toggle(afterClick)
 if mode=="camera" or pending then
  stop()
 elseif not inspectingUI() and not conflict() then
  if afterClick then pending="start";delay=.10 else start() end
 end
end
function AZSXUI_CameraToggle() toggle(false) end
function AZSXUI_CameraInspect() stop() end
function AZSXUI_CameraStatus()
 return {mode=mode,pending=pending,available=available(),conflict=conflict(),ui=inspectingUI()}
end
local function install()
 if not WorldFrame or not WorldFrame.GetScript or not WorldFrame.SetScript then return end
 local down=WorldFrame:GetScript("OnMouseDown")
 local up=WorldFrame:GetScript("OnMouseUp")
 if not hookDown or down~=hookDown then
  local previous=down
  hookDown=function()
   local button=arg1
   if previous then previous() end
   if button=="RightButton" then
    local release=mode=="camera" or pending=="start"
    pending=release and "stop-on-up" or "start-on-up"
   end
  end
  WorldFrame:SetScript("OnMouseDown",hookDown)
 end
 if not hookUp or up~=hookUp then
  local previous=up
  hookUp=function()
   local button=arg1
   if previous then previous() end
   if button=="RightButton" and (pending=="start-on-up" or pending=="stop-on-up") then
    local action=pending
    pending=nil
    if action=="stop-on-up" then stop() else toggle(true) end
   end
  end
  WorldFrame:SetScript("OnMouseUp",hookUp)
 end
end
controller:RegisterEvent("PLAYER_ENTERING_WORLD")
controller:RegisterEvent("PLAYER_LEAVING_WORLD")
controller:SetScript("OnEvent",function()
 if event=="PLAYER_LEAVING_WORLD" then stop()
 elseif event=="PLAYER_ENTERING_WORLD" then stop();install() end
end)
controller:SetScript("OnUpdate",function()
 local dt=arg1 or 0
 scan=scan+dt
 if inspectingUI() or conflict() then
  if mode=="camera" or pending then stop() end
  return
 end
 if scan>=1 then scan=0;install() end
 if pending=="start" then
  delay=delay-dt
  if delay<=0 then pending=nil;start() end
 elseif mode=="camera" and type(IsMouselooking)=="function" then
  local ok,looking=pcall(IsMouselooking)
  if ok and not looking then stop() end
 end
end)
install()
