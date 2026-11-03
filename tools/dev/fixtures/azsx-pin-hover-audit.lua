-- Disposable visual fixture. Only the UI receives the sample boss and two neighboring lair markers.
local frame=CreateFrame("Frame",nil,UIParent)
local label=frame:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
label:SetPoint("TOPLEFT",UIParent,"TOPLEFT",12,-25)
label:SetWidth(350);label:SetHeight(24)
local started=false
frame:SetScript("OnUpdate",function()
    if not AZS_RUN or (tonumber(AZS_RUN.t) or 0)<3 then return end
    if not started then
        local original=AZS_GetBosses
        AZS_GetBosses=function()
            local list={{id=-31415,name="Hover fixture",hp=80,hpm=100,
                debuffs={{name="Fixture poison",icon="Ability_Poisons",stacks=2,remaining=9}}}}
            for _,entry in ipairs(original() or {})do table.insert(list,entry)end
            return list
        end
        local lairs=AZS_GetLairs
        local position=AZS_GetPlayerPosition()
        if not position then position={x=AZS_RUN.px,y=AZS_RUN.py}end
        if position and position.x and position.y then
            local x,y=position.x+40,position.y+55
            AZS_GetLairs=function()
                local list={{index=20001,name="Marker A",x=x,y=y,active=true},
                            {index=20002,name="Marker B",x=x+1,y=y,active=true}}
                for _,entry in ipairs(lairs and lairs() or {})do table.insert(list,entry)end
                return list
            end
        end
        started=true
    end
    local button=getglobal("AZSXUIEncounter1Debuff1")
    if button and button:IsVisible() and button.effect and button.effect.name=="Fixture poison" then
        local enter=button:GetScript("OnEnter")
        if enter then
            enter()
            for _,pin in ipairs(AZSXUI_MinimapReadout())do
                if pin.event==20001 and pin.kind==3 and pin.quantity>=2 then
                    label:SetText("Fixture: grouped lairs = "..pin.quantity)
                    frame:SetScript("OnUpdate",nil);break
                end
            end
        end
    end
end)
