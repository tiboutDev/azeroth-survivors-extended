-- Disposable UI-only inspection; no Windows input or gameplay commands.
local probe=CreateFrame("Frame","AzCoopPartyUIProbe",UIParent)
local previous,first,stage=0,nil,0
probe:SetScript("OnUpdate",function()
    local now=GetTime()
    if now-previous<0.5 or not AzCoopPartyUIReadout then return end
    previous=now
    local roster,own,active,settings,refreshes=AzCoopPartyUIReadout()
    if not active then return end
    first=first or now
    local total,build,buffs,damage,down,revive=0,0,0,0,0,0
    for slot,row in pairs(roster) do
        total=total+1;damage=damage+row.damage
        if row.down then down=down+1 end
        if row.revive>revive then revive=row.revive end
        if slot~=own then build=build+table.getn(row.spells)+table.getn(row.items);buffs=buffs+table.getn(row.buffs) end
    end
    local collapsed=settings.collapsed
    if now-first>5 and stage==0 then
        local f=AzCoopGroupMeterCollapse:GetScript("OnClick");f();stage=1
    elseif now-first>7 and stage==1 then
        local f=AzCoopGroupMeterCollapse:GetScript("OnClick");f();stage=2
    elseif now-first>9 and stage==2 then
        local f=AzCoopGroupMeterMode:GetScript("OnClick");f();stage=3
    elseif now-first>11 and stage==3 then
        local f=AzCoopGroupMeterMode:GetScript("OnClick");f();stage=4
    end
    local card=AzCoopPartyCard1
    local aligned=card and PartyMemberFrame1 and card:GetLeft()>=PartyMemberFrame1:GetRight()
    local compact=not card or not card.items[1]:IsVisible() or math.abs(card.spells[1]:GetTop()-card.items[1]:GetTop()-18)<0.1
    local nativeBuff=card and card.buffs[1].image==getglobal(card.buffs[1]:GetName().."Icon")
    local plainIcons=card and not card.spells[1].rank and not card.items[1].rank
    local plainBar=not AzCoopDownedNotice.progress.text:GetText() or AzCoopDownedNotice.progress.text:GetText()==""
    local nativeCast=AzCoopDownedNotice.progress.border and AzCoopDownedNotice.progress.spark
    error("AZCOOP_PARTY_UI_PROBE members="..total.." build="..build.." buffs="..buffs.." damage="..damage.." down="..down.." revive="..revive.." visible="..tostring(AzCoopGroupMeter:IsVisible()).." card="..tostring(card and card:IsVisible()).." aligned="..tostring(aligned).." collapsed="..tostring(collapsed).." mode="..tostring(settings.mode).." stage="..stage.." compact="..tostring(compact).." center_revive="..tostring(AzCoopDownedNotice:IsVisible()).." party_cast="..tostring(card.revive~=nil).." native_buff="..tostring(nativeBuff).." native_cast="..tostring(nativeCast~=nil).." plain_icons="..tostring(plainIcons).." plain_bar="..tostring(plainBar).." refreshes="..tostring(refreshes).." hidden_build="..tostring(GetCVar("hudBuild")),0)
end)
