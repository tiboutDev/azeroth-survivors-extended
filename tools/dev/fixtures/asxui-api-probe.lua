-- Explicit standalone API probe; never loaded by ASX or ASXUI distributions.
local frame=CreateFrame("Frame", "ASXUIApiProbe", UIParent)
local last,count=-100,0
if not ASX_GetMinimapData then
    function ASX_GetMinimapData()
        if not AZS_RUN or type(AZS_RUN.px)~="number"or type(AZS_RUN.py)~="number"then return nil end
        local radii={233.333324,200,166.66666,133.333328,100,66.666664}
        return {identity="probe-only",x=AZS_RUN.px,y=AZS_RUN.py,radii=radii,pingX=0,pingY=0},
            {{event=1,kind=0,x=AZS_RUN.px+20,y=AZS_RUN.py,left=60},{event=2,kind=1,x=AZS_RUN.px,y=AZS_RUN.py+20,left=0}}
    end
end
if type(seterrorhandler)=="function"then seterrorhandler(function()end)end
local eventProbeConfigured=false
if not ASX_GetEventPresentation then
    function ASX_GetEventPresentation()
        local now=GetTime()
        return {epoch="probe",own="11",owner={discovery={event=math.floor(now/10),title="Lost Supplies",icon="Interface\\Icons\\INV_Crate_01",reward="Probe: Elixir of Intellect",note="Probe display only",color={.12,1,0}},ritual={event=1,held=700,inside=true,maximum=2000,title="Sayge's Trial",note="7 / 20 sec"}},blessing={boss=math.floor(now/10),title="Sayge's Dark Fortune",icon="Interface\\Icons\\INV_Misc_Orb_02",reward="Probe: +12% Experience",note="Fortune of Wisdom",color={1,1,1}}}
    end
end
if not ASX_GetState then
    function ASX_GetAPIVersion()return 1,0 end
    function ASX_GetState(major)
        if major~=1 then return nil,"unsupported_version"end
        local view=ASX_GetEventPresentation();local map,pins=ASX_GetMinimapData()
        return {available=true,api={major=1,minor=0},revision=1,session={epoch=view.epoch,recipientGUID=view.own},data={discovery=view.owner.discovery,sayge=view.owner.ritual,fortune=view.blessing,minimap=map,markers=pins},texts={supplies=view.owner.discovery,sayge=view.owner.ritual,fortune=view.blessing}}
    end
end
local function attempt(object,method)
    if not object or type(object[method])~="function" then return "unavailable" end
    local ok,value=pcall(object[method],object)
    return ok and tostring(value) or "error"
end
local function describe(object)
    return attempt(object,"GetName")..":"..attempt(object,"GetObjectType")..":"..attempt(object,"IsShown")
end
frame:SetScript("OnUpdate",function()
    local now=GetTime()
    if now<5 or now-last<8 or count>=4 then return end
    last=now;count=count+1
    if not eventProbeConfigured and ASXUI_ConfigureWidget then
        ASXUI_ConfigureWidget("supplies","position","-350","-140");ASXUI_ConfigureWidget("supplies","scale","0.85");ASXUI_ConfigureWidget("fortune","position","350","-140");ASXUI_ConfigureWidget("fortune","opacity","0.7");eventProbeConfigured=true
    end
    if ShowNameplates then ShowNameplates();NAMEPLATES_ON=1 end
    if ASXUI_Readout then
        local state=ASXUI_Readout()
        local rows={}
        for _,row in ipairs(state.rows)do table.insert(rows,tostring(row.name)..":"..tostring(row.scale)..":"..tostring(row.health))end
        local details={}
        if ASXEventVisualReadout then local b,ba,d,da=ASXEventVisualReadout();table.insert(details,"event_widgets="..tostring(b)..":"..tostring(ba)..":"..tostring(d)..":"..tostring(da))end
        for _,key in ipairs({"WorldTextScale","damageFontSize","floatingCombatTextCombatDamageDirectionalScale","CombatDamage"})do
            local ok,value=pcall(GetCVar,key);table.insert(details,key.."="..(ok and tostring(value)or"error"))
        end
        for _,key in ipairs({"CombatText","CombatTextFont","COMBAT_TEXT_HEIGHT","WorldToScreen","GetUnitScreenPosition"})do table.insert(details,key.."="..type(getglobal(key)))end
        for _,row in ipairs(state.rows)do table.insert(details,"name="..tostring(row.name)..":"..tostring(row.visible)..":"..tostring(row.alpha))end
        for i=1,math.min(4,table.getn(AZS_CHESTS or{}))do local c=AZS_CHESTS[i];table.insert(details,"cache="..tostring(c.id)..":"..tostring(c.x)..":"..tostring(c.y))end
        local pins=ASXUI_MinimapReadout and ASXUI_MinimapReadout()or{}
        local message="ASXUI_ADDON_PROBE sample="..count.." plates="..state.plates.." font="..tostring(state.numberFont).." rows="..table.concat(rows,",").." native_caches="..table.getn(AZS_CHESTS or{}).." pins="..table.getn(pins).." details="..table.concat(details,";")
        error(message,0)
    end
    local out={"ASXUI_API_PROBE sample="..count.." phase="..tostring(AZS_RUN and AZS_RUN.ph)}
    for _,name in ipairs({"EnumerateFrames","GetCVar","SetCVar","WorldFrame","CombatText_UpdateDisplayedMessages","CombatText_AddMessage","CombatTextSetActiveUnit","SetRaidTarget"}) do
        table.insert(out,name.."="..type(getglobal(name)))
    end
    if WorldFrame and type(WorldFrame.GetChildren)=="function" then
        local children={WorldFrame:GetChildren()};table.insert(out,"world_children="..table.getn(children))
        for i=1,math.min(table.getn(children),40) do
            local child=children[i];table.insert(out,"child="..describe(child))
            if type(child.GetRegions)=="function" then
                local regions={child:GetRegions()}
                for j=1,math.min(table.getn(regions),6)do
                    table.insert(out,"region="..describe(regions[j])..":"..attempt(regions[j],"GetText"))
                end
            end
        end
    end
    local names={}
    for key,value in pairs(getfenv(0))do
        if type(key)=="string" then
            local low=string.lower(key)
            if string.find(low,"plate",1,true) or string.find(low,"damage",1,true) or string.find(low,"combattext",1,true) then
                table.insert(names,key.."="..type(value))
            end
        end
    end
    table.sort(names)
    for i=1,math.min(table.getn(names),100)do table.insert(out,names[i])end
    if GetCVar then
        for _,key in ipairs({"nameplateShowEnemies","nameplateShowFriends","nameplateScale","nameplateWidth","nameplateHeight","floatingCombatTextCombatDamage","damageFontSize","WorldTextScale"})do
            local ok,value=pcall(GetCVar,key);table.insert(out,"cvar:"..key.."="..(ok and tostring(value)or"error"))
        end
    end
    error(table.concat(out," | "),0)
end)
