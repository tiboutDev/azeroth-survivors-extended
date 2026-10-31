-- ASXUI: independent presentation only, using the native addon API.
-- Plate styling is standalone; optional minimap/event views consume ASX bridges.
local addon = CreateFrame("Frame", "ASXUIController", UIParent)
local plates, known, scanned = {}, {}, -1
local originalDamageFont = DAMAGE_TEXT_FONT
local numberFont = "Interface\\AddOns\\ASXUI\\Fonts\\ASXUI-Numbers.ttf"
local borderTexture = "Interface\\Tooltips\\Nameplate-Border"
local ready = false
local function settings()
    if type(ASXUIDB) ~= "table" then ASXUIDB = {} end
    if type(ASXUIDB.plates) ~= "boolean" then ASXUIDB.plates = true end
    if type(ASXUIDB.numbers) ~= "boolean" then ASXUIDB.numbers = false end
    if type(ASXUIDB.scale) ~= "number" or ASXUIDB.scale ~= ASXUIDB.scale
        or ASXUIDB.scale < 0.5 or ASXUIDB.scale > 1.5 then ASXUIDB.scale = 0.7 end
    return ASXUIDB
end
local function say(text)
    if DEFAULT_CHAT_FRAME and DEFAULT_CHAT_FRAME.AddMessage then DEFAULT_CHAT_FRAME:AddMessage("ASXUI: " .. text) end
end
local function discover(plate)
    if known[plate] or table.getn(plates) >= 256 then return end
    if not plate.GetRegions or not plate.GetChildren or not plate.GetScale or not plate.SetScale then return end
    local regions = {plate:GetRegions()}
    local first = regions[1]
    if not first or not first.GetTexture or first:GetTexture() ~= borderTexture then return end
    local bar = plate:GetChildren()
    if not bar or not bar.GetStatusBarColor then return end
    local row = {plate=plate,bar=bar,index=table.getn(plates)+1,border=regions[1],glow=regions[2],scale=plate:GetScale(),labels={},geometry={}}
    -- Resize retained native geometry, not the world anchor or parent scale.
    -- The accelerated native plate renderer does not consistently inherit SetScale.
    local function geometry(widget)
        if widget.GetWidth and widget.GetHeight and widget.SetWidth and widget.SetHeight then
            table.insert(row.geometry,{widget,widget:GetWidth(),widget:GetHeight()})
        end
    end
    geometry(plate);geometry(bar)
    for i=1,math.min(2,table.getn(regions))do geometry(regions[i])end
    for _,region in ipairs(regions) do
        if region.GetObjectType and region:GetObjectType() == "FontString" then
            if not row.name then row.name=region else table.insert(row.labels,{region,region:GetAlpha()}) end
        end
    end
    if not row.name then return end
    row.nameAlpha=row.name:GetAlpha()
    -- The reviewed native layout has level/skull art immediately before its
    -- final raid icon. Keep the border, glow and raid icon untouched.
    if table.getn(regions)==6 and regions[5].GetTexture then
        row.skull={regions[5],regions[5]:GetAlpha()}
    end
    known[plate]=row;table.insert(plates,row)
end
local function finite(v)return type(v)=="number"and v==v and v-v==0 end
local function pill(row,active,r,g,b)
    local bar=row.bar
    local texture=bar.GetStatusBarTexture and bar:GetStatusBarTexture()
    local low,high
    if bar.GetMinMaxValues then low,high=bar:GetMinMaxValues()end
    local value=bar:GetValue()
    active=active and ASXUI_CreatePill and texture and texture.SetAlpha
        and finite(low)and finite(high)and high>low and finite(value)
        and finite(r)and finite(g)and finite(b)
    if active then
        if not row.pill then
            local name="ASXUIEnemyHealth"..row.index
            local root=CreateFrame("Frame",name,row.plate);root:EnableMouse(false)
            root:SetPoint("CENTER",bar,"CENTER",0,0)
            local outline=ASXUI_CreatePill(root,name.."Outline",70,8,.36,.34,.29);outline:SetPoint("CENTER",root,"CENTER",0,0)
            local background=ASXUI_CreatePill(outline,name.."Background",68,6,.045,.045,.035);background:SetPoint("CENTER",outline,"CENTER",0,0)
            local clip=CreateFrame("ScrollFrame",name.."Clip",background);clip:SetPoint("LEFT",background,"LEFT",0,0);clip:SetHeight(6)
            local fill=ASXUI_CreatePill(clip,name.."Fill",68,6,r,g,b,"Interface\\TargetingFrame\\UI-StatusBar");clip:SetScrollChild(fill)
            row.pill={root=root,outline=outline,background=background,clip=clip,fill=fill}
        end
        local view=row.pill
        local width=math.max(10,bar:GetWidth());local inner=width-2
        if view.width~=width then
            view.root:SetWidth(width);view.root:SetHeight(8);view.outline:SetWidth(width)
            view.background:SetWidth(inner);view.fill:SetWidth(inner);view.width=width;view.ratio=nil
        end
        local ratio=math.max(0,math.min(1,(value-low)/(high-low)))
        if view.ratio~=ratio then view.clip:SetWidth(math.max(.001,inner*ratio));view.ratio=ratio end
        if ratio>0 then view.clip:Show()else view.clip:Hide()end
        if view.r~=r or view.g~=g or view.b~=b then view.fill:SetPillColor(r,g,b);view.r=r;view.g=g;view.b=b end
        if not row.hiddenArt then row.hiddenArt={}end
        -- Only the health texture and decorative border/glow are replaced.
        -- The retained bar, native name, raid marker and scripts stay authoritative.
        for _,art in ipairs({texture,row.border,row.glow})do
            if not row.hiddenArt[art]then row.hiddenArt[art]=art:GetAlpha()end
            art:SetAlpha(0)
        end
        view.root:Show()
    else
        if row.pill then row.pill.root:Hide()end
        for art,alpha in pairs(row.hiddenArt or{})do art:SetAlpha(alpha)end
        row.hiddenArt=nil
    end
end
local function apply(row,db,targetName,targetCount)
    local r,g,b=row.bar:GetStatusBarColor()
    local enemy=type(r)=="number" and type(b)=="number" and r>0.75 and b<0.25
    local active=db.plates and enemy
    local share=active and db.scale or 1
    for _,entry in ipairs(row.geometry)do
        local widget,width,height=entry[1],entry[2]*share,entry[3]*share
        if math.abs(widget:GetWidth()-width)>0.001 then widget:SetWidth(width)end
        if math.abs(widget:GetHeight()-height)>0.001 then widget:SetHeight(height)end
    end
    pill(row,active,r,g,b)
    local hideName=active
    -- A plate has no GUID/unit token. Never classify through names in AZS_UNITS.
    -- Only a unique highlighted target can positively identify a special enemy.
    if active and targetCount==1 and row.plate:GetAlpha()==1 and row.name:GetText()==targetName then
        local kind
        if type(AZS_UnitKind)=="function" then kind=AZS_UnitKind("target")
        elseif type(UnitClassification)=="function" then kind=UnitClassification("target") end
        hideName=not (kind=="elite" or kind=="guardian" or kind=="boss" or kind=="final" or kind=="worldboss" or kind=="rareelite")
    end
    row.name:SetAlpha(hideName and 0 or row.nameAlpha)
    for _,label in ipairs(row.labels)do label[1]:SetAlpha(active and 0 or label[2])end
    if row.skull then row.skull[1]:SetAlpha(active and 0 or row.skull[2])end
end
local function refresh()
    if not ready or not WorldFrame or not WorldFrame.GetNumChildren or not WorldFrame.GetChildren then return end
    local count=WorldFrame:GetNumChildren()
    if count~=scanned then
        local children={WorldFrame:GetChildren()}
        for i=1,math.min(table.getn(children),4096)do discover(children[i])end
        scanned=count
    end
    local targetName=UnitExists and UnitExists("target") and UnitName("target") or nil
    local matches=0
    for _,row in ipairs(plates)do
        if row.plate:IsVisible() and row.plate:GetAlpha()==1 and row.name:GetText()==targetName then matches=matches+1 end
    end
    local db=settings()
    for _,row in ipairs(plates)do
        if row.plate:IsVisible() or not db.plates then apply(row,db,targetName,matches)end
    end
end
addon:RegisterEvent("ADDON_LOADED")
addon:SetScript("OnEvent",function()
    if arg1~="ASXUI"then return end
    ready=true
    local db=settings()
    DAMAGE_TEXT_FONT=db.numbers and numberFont or originalDamageFont
end)
addon:SetScript("OnUpdate",refresh)
SLASH_ASXUI1="/asxui"
SlashCmdList["ASXUI"]=function(message)
    local _,_,command,argument=string.find(message or "","^(%S*)%s*(.*)$")
    local db=settings()
    if command=="widget" and ASXUI_ConfigureWidget then
        local _,_,kind,property,a,b=string.find(argument,"^(%S+)%s+(%S+)%s*(%S*)%s*(%S*)$")
        if not ASXUI_ConfigureWidget(kind,property,a,b)then say("widget anchor|supplies|fortune|crystals|sayge|revive|loot|marks: position X Y; scale 0.5..2; opacity 0..1; show on|off; reset")else say(kind.." widget updated.")end
    elseif command=="hud" and ASXUI_ConfigureHud and ASXUI_ConfigureHud(argument)then
        say("Native HUD extension "..argument..".")
    elseif command=="pins" and (argument=="on"or argument=="off")then
        db.pins=argument=="on";say("Minimap markers "..argument..".")
    elseif command=="plates" and (argument=="on"or argument=="off")then
        db.plates=argument=="on";refresh();say("Nameplate styling "..argument..". Native visibility keys are unchanged.")
    elseif command=="scale"then
        local value=tonumber(argument)
        if not value or value~=value or value<0.5 or value>1.5 then say("Scale must be between 0.5 and 1.5.");return end
        db.scale=value;refresh();say("Nameplate scale: "..value)
    elseif command=="numbers"and(argument=="on"or argument=="off")then
        db.numbers=argument=="on";say("World number font "..argument..". Applies after restarting the client.")
    else
        say("/asxui hud on|off; /asxui widget supplies|fortune|crystals|sayge|revive|loot|marks ...; /asxui pins on|off; /asxui plates on|off; /asxui scale 0.5..1.5; /asxui numbers on|off")
    end
end
function ASXUI_Readout()
    local result={ready=ready,plates=table.getn(plates),numberFont=DAMAGE_TEXT_FONT,rows={}}
    for _,row in ipairs(plates)do
        table.insert(result.rows,{name=row.name:GetText(),visible=row.plate:IsVisible(),scale=(row.geometry[1] and row.geometry[1][2]~=0 and row.plate:GetWidth()/row.geometry[1][2] or 1),alpha=row.name:GetAlpha(),pill=row.pill and row.pill.root:IsVisible()or false,pillWidth=row.pill and row.pill.width,pillHeight=row.pill and 8,health=row.bar:GetValue()})
    end
    return result
end
