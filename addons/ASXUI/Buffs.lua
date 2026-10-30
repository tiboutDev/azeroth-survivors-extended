-- Visual order only. Native buff IDs, click handlers and durations stay untouched.
local controller=CreateFrame("Frame","ASXUIBuffOrderController",UIParent)
local owned={}
local cachedState,lastState=nil,-100
local lastFrames,lastFortune,lastCrystal={},nil,nil
local refresh
local subscribed=false
local function points(frame)
    if type(frame.GetPoint)~="function"or type(frame.GetNumPoints)~="function"then return nil end
    local result={}
    for index=1,frame:GetNumPoints()do result[index]={frame:GetPoint(index)}end
    return result
end
local function equal(a,b)
    if not a or not b or table.getn(a)~=table.getn(b)then return false end
    for i=1,table.getn(a)do for j=1,5 do if a[i][j]~=b[i][j]then return false end end end
    return true
end
local function apply(frame,anchors)
    frame:ClearAllPoints()
    for _,p in ipairs(anchors)do frame:SetPoint(p[1],p[2],p[3],p[4],p[5])end
end
local function restore()
    local reset={}
    for frame,v in pairs(owned)do
        if equal(points(frame),v.applied)then reset[frame]=v.original end
    end
    -- Break the remapped anchor graph before restoring it, avoiding cycles.
    for frame,_ in pairs(reset)do frame:ClearAllPoints()end
    for frame,original in pairs(reset)do apply(frame,original)end
    owned={}
end
local function buffKind(frame,state)
    local index=frame.buffIndex
    if type(index)~="number" and GetPlayerBuff then index=GetPlayerBuff(frame:GetID(),"HELPFUL")end
    if type(index)~="number"or index<0 or not GetPlayerBuffTexture then return nil end
    local texture=GetPlayerBuffTexture(index)
    if type(texture)~="string"then return nil end
    texture=string.gsub(string.gsub(string.lower(texture),"/","\\"),"%.blp$","")
    if state.data.fortune and texture=="interface\\icons\\inv_misc_orb_02"then return "fortune"end
    local crystal=state.data.crystals
    if crystal and crystal.index==index and texture=="interface\\icons\\inv_misc_gem_variety_01"then return "crystal"end
end
refresh=function(force)
    local now=GetTime()
    if force or now-lastState>=0.2 then
        lastState=now;cachedState=nil
        if ASX_GetAPIVersion and ASX_GetAPIVersion()==1 and ASX_GetState then cachedState=ASX_GetState(1)end
    end
    local state=cachedState
    if not state or not state.available or not state.data or not(state.data.fortune or state.data.crystals)then restore();lastFrames={};lastFortune=nil;lastCrystal=nil;return end
    local frames,anchors,indexByFrame={}, {}, {}
    local fortune,crystal
    for n=0,31 do
        local frame=getglobal("BuffButton"..n)
        if frame and frame:IsShown()then
            local p=points(frame)
            if not p or table.getn(p)==0 then restore();return end
            table.insert(frames,frame);anchors[frame]=p;indexByFrame[frame]=table.getn(frames)
            local kind=buffKind(frame,state)
            if kind=="fortune"then if fortune then restore();return end;fortune=frame
            elseif kind=="crystal"then if crystal then restore();return end;crystal=frame end
        end
    end
    local same=fortune==lastFortune and crystal==lastCrystal and table.getn(frames)==table.getn(lastFrames) and next(owned)~=nil
    if same then
        for index,frame in ipairs(frames)do
            local v=owned[frame]
            if frame~=lastFrames[index]or not v or not equal(anchors[frame],v.applied)then same=false;break end
        end
    end
    if same then return end
    restore();lastFrames={};lastFortune=nil;lastCrystal=nil
    if not fortune and not crystal then return end
    for _,frame in ipairs(frames)do anchors[frame]=points(frame)end
    lastFrames=frames;lastFortune=fortune;lastCrystal=crystal
    local order={}
    if fortune then table.insert(order,fortune)end
    if crystal then table.insert(order,crystal)end
    for _,frame in ipairs(frames)do if frame~=fortune and frame~=crystal then table.insert(order,frame)end end
    for index,source in ipairs(frames)do
        local target=order[index];local moved={}
        for _,p in ipairs(anchors[source])do
            local relative=p[2]
            if type(relative)=="string"then relative=getglobal(relative)or relative end
            local slot=indexByFrame[relative]
            table.insert(moved,{p[1],slot and order[slot]or relative,p[3],p[4],p[5]})
        end
        apply(target,moved);owned[target]={original=anchors[target],applied=points(target)}
    end
end
controller:SetScript("OnUpdate",function()
    if not subscribed and type(ASX_RegisterStateHandler)=="function"then
        ASX_RegisterStateHandler(function(state)cachedState=state;lastState=GetTime();refresh(false)end);subscribed=true
    end
    refresh(false)
end)
controller:RegisterEvent("PLAYER_AURAS_CHANGED")
controller:RegisterEvent("PLAYER_ENTERING_WORLD")
controller:SetScript("OnEvent",function()refresh(true)end)
