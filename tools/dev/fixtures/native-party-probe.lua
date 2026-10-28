-- Read-only probe for disposable test profiles. No mouse or keyboard input.
if seterrorhandler then seterrorhandler(function() end) end
local probe = CreateFrame("Frame", "AzCoopNativePartyProbe", UIParent)
local previous = 0
local captures = 0
local first_ready = nil
probe:SetScript("OnUpdate", function()
    local now = GetTime()
    if now - previous < 1 then return end
    previous = now
    local frame = PartyMemberFrame1
    local health = PartyMemberFrame1HealthBar
    if frame and frame:IsVisible() and UnitHealthMax("party1")>1 and Screenshot then
        first_ready = first_ready or now
        if captures < 2 and now-first_ready >= 8+captures*10 then
            Screenshot();captures=captures+1
        end
    end
    -- The 0.110 native Bevy port has no Lua menu compositor/CVar. Absence is
    -- expected there; querying an unregistered CVar raises in the native API.
    local draw_ok, draw = pcall(GetCVar, "azcoop_menu_draw")
    if not draw_ok or not draw then draw = "0" end
    local row = "members=" .. tostring(GetNumPartyMembers())
        .. " exists=" .. tostring(UnitExists("party1"))
        .. " shown=" .. tostring(frame and frame:IsShown())
        .. " visible=" .. tostring(frame and frame:IsVisible())
        .. " hp=" .. tostring(UnitHealth("party1"))
        .. " max=" .. tostring(UnitHealthMax("party1"))
        .. " value=" .. tostring(health and health:GetValue())
        .. " menu=" .. tostring(AzCoopMenuButton and AzCoopMenuButton:IsVisible() or 0)
        .. " draw=" .. tostring(draw)
    if GetRaidTargetIndex then
        local own_ok, own = pcall(GetRaidTargetIndex, "player")
        local ally_ok, ally = pcall(GetRaidTargetIndex, "party1")
        row = row .. " raid_self=" .. tostring(own_ok and own) .. " raid_party=" .. tostring(ally_ok and ally)
    end
    -- The sandbox exposes no print/file output. A deliberately tagged diagnostic
    -- reaches the native script log; this fixture is never loaded by the mod.
    error("AZCOOP_FRAME_PROBE " .. row, 0)
end)
