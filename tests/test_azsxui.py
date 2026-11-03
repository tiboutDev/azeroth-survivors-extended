"""Independent AZSXUI behavior under the client's Lua 5.1 contract."""
from pathlib import Path
import unittest
try:
    from lupa.lua51 import LuaRuntime
except ImportError:
    LuaRuntime=None
ROOT=Path(__file__).resolve().parents[1]
STUB=r"""
math.huge=nil;objects={}; SlashCmdList={}; AZSXUIDB=nil; DAMAGE_TEXT_FONT="native.ttf"
function getglobal(n)return _G[n]end
function UnitExists(unit)return unit=="target"end
function UnitName(unit)return "Boar"end
rank="normal";function UnitClassification(unit)return rank end
local function object(kind,name)
 local o={kind=kind,name=name,alpha=1,scale=1,visible=true,children={},regions={},scripts={}}
 function o:GetName()return self.name end
 function o:SetPoint(...)self.point={...}end
 function o:EnableMouse(v)self.mouse=v end
 function o:SetTexture(v)self.texture=v end
 function o:SetTexCoord(...)self.coords={...}end
 function o:SetVertexColor(...)self.tint={...}end
 function o:Show()self.visible=true end
 function o:Hide()self.visible=false end
 function o:SetScrollChild(v)self.scrollChild=v end
 function o:CreateTexture(name,layer)local t=object("Texture",name);table.insert(self.regions,t);return t end
 function o:GetStatusBarTexture()return self.healthTexture end
 function o:GetMinMaxValues()return 0,self.maximum end
 function o:GetObjectType()return self.kind end
 function o:GetTexture()return self.texture end
 function o:GetRegions()return unpack(self.regions)end
 function o:GetChildren()return unpack(self.children)end
 function o:GetNumChildren()return table.getn(self.children)end
 function o:GetWidth()return self.width or 100 end
 function o:GetHeight()return self.height or 14 end
 function o:SetWidth(v)self.width=v end
 function o:SetHeight(v)self.height=v end
 function o:GetScale()return self.scale end
 function o:SetScale(v)self.scale=v end
 function o:GetAlpha()return self.alpha end
 function o:SetAlpha(v)self.alpha=v end
 function o:GetText()return self.text end
 function o:GetFont()return "native.ttf",self.fontSize or 12,"OUTLINE" end
 function o:SetFont(path,size,flags)self.fontSize=size end
 function o:GetValue()return self.health end
 function o:GetStatusBarColor()return unpack(self.color)end
 function o:IsVisible()return self.visible end
 function o:RegisterEvent(e)end
 function o:SetScript(e,f)self.scripts[e]=f end
 if name then _G[name]=o end
 return o
end
function CreateFrame(kind,name,parent)return object(kind,name)end
function SetPortraitToTexture(name,path)_G[name].portrait=path end
UIParent=object("Frame","UIParent");WorldFrame=object("Frame","WorldFrame")
function makePlate(name,r,g,b)
 local p=object("Frame");p.originalUpdate=function()end;p.scripts.OnUpdate=p.originalUpdate
 local bar=object("StatusBar");bar.health=37;bar.maximum=100;bar.healthTexture=object("Texture");bar.color={r,g,b};p.children={bar}
 local border=object("Texture");border.texture="Interface\\Tooltips\\Nameplate-Border"
 local glow=object("Texture");local text=object("FontString");text.text=name
 local level=object("FontString");level.text="5"
 p.regions={border,glow,text,level,object("Texture"),object("Texture")}
 table.insert(WorldFrame.children,p);return p
end
plateShows=0;layout="stack"
function ShowNameplates()plateShows=plateShows+1 end
function SetCVar(key,value)assert(key=="nameplateLayout" and value=="overlap");layout=value end
function loadAddon()arg1="AZSXUI";AZSXUIController.scripts.OnEvent();arg1=nil end
function tick()AZSXUIController.scripts.OnUpdate()end
"""

@unittest.skipUnless(LuaRuntime,"Explicit AZSXUI profile requires lupa Lua 5.1")
class AZSXUITests(unittest.TestCase):
    def setUp(self):
        self.lua=LuaRuntime(unpack_returned_tuples=True)
        self.lua.execute(STUB)
        self.lua.execute((ROOT/"addons/AZSXUI/AZSXUI.lua").read_text())
        self.lua.execute((ROOT/"addons/AZSXUI/Theme.lua").read_text())
        self.lua.execute("loadAddon()")
    def run_lua(self,source):self.lua.execute(source)
    def test_enemy_scale_is_stable_and_preserves_native_health_and_scripts(self):
        self.run_lua('p=makePlate("Boar",1,0,0);for i=1,20 do tick()end;assert(p:GetWidth()==50);assert(p.children[1].health==37);assert(p.regions[3]:GetFont()=="native.ttf");assert(p.scripts.OnUpdate==p.originalUpdate)')
    def test_friendly_reuse_restores_original_scale_and_labels(self):
        self.run_lua('p=makePlate("Boar",1,0,0);tick();p.children[1].color={0,1,0};tick();assert(p:GetWidth()==100);assert(p.regions[3].alpha==1);assert(p.regions[4].alpha==1)')
    def test_unknown_or_ambiguous_classification_hides_names(self):
        self.run_lua('p=makePlate("Boar",1,0,0);q=makePlate("Boar",1,0,0);tick();assert(p.regions[3].alpha==0);assert(q.regions[3].alpha==0);q.visible=false;rank=nil;tick();assert(p.regions[3].alpha==0)')
    def test_confirmed_normal_target_hides_name_special_rank_keeps_it(self):
        self.run_lua('p=makePlate("Boar",1,0,0);tick();assert(p.regions[3].alpha==0);rank="elite";tick();assert(p.regions[3].alpha==1)')
    def test_native_kind_wins_over_generic_classification_and_does_not_leak_on_reuse(self):
        self.run_lua('p=makePlate("Boar",1,0,0);rank="elite";kind="guardian";function AZS_UnitKind()return kind end;tick();assert(p.regions[3].alpha==1);kind="normal";tick();assert(p.regions[3].alpha==0);for _,v in ipairs({"elite","guardian","boss","final"})do kind=v;tick();assert(p.regions[3].alpha==1)end;kind=nil;tick();assert(p.regions[3].alpha==0);p.regions[3].text="Another";kind="boss";tick();assert(p.regions[3].alpha==0)')
    def test_pill_tracks_native_health_color_and_reuses_its_frames(self):
        self.run_lua('''p=makePlate("Boar",1,0,0);tick();local root=AZSXUIEnemyHealth1
        assert(root:GetWidth()==50 and root:GetHeight()==6 and root.visible)
        assert(AZSXUIEnemyHealth1Clip:GetWidth()==48*.37)
        assert(p.children[1].healthTexture.alpha==0 and p.regions[1].alpha==0 and p.regions[2].alpha==0)
        assert(p.regions[6].alpha==1)
        p.children[1].health=1;tick();assert(AZSXUIEnemyHealth1Clip:GetWidth()==.48)
        p.children[1].health=0;tick();assert(not AZSXUIEnemyHealth1Clip.visible)
        p.children[1].health=999;p.children[1].color={1,.5,0};tick()
        assert(AZSXUIEnemyHealth1Clip:GetWidth()==48 and AZSXUIEnemyHealth1Fill.centerTexture.tint[2]==AZSXUIEnemyHealth1Fill.centerTexture.tint[1]*.5 and AZSXUIEnemyHealth1FillLeft1.tint[2]==AZSXUIEnemyHealth1Fill.centerTexture.tint[2])
        assert(root==AZSXUIEnemyHealth1 and p.children[1].health==999)
        SlashCmdList.AZSXUI("scale 1.5");assert(root:GetWidth()==150 and root:GetHeight()==6)''')
    def test_pill_restores_native_art_on_invalid_health_friendly_reuse_and_disable(self):
        self.run_lua('''p=makePlate("Boar",1,0,0);p.regions[1].alpha=.6;tick()
        p.children[1].maximum=0;tick();assert(not AZSXUIEnemyHealth1.visible and p.regions[1].alpha==.6 and p.children[1].healthTexture.alpha==1)
        p.children[1].maximum=100;tick();p.children[1].color={0,1,0};tick()
        assert(not AZSXUIEnemyHealth1.visible and p.regions[1].alpha==.6)
        p.children[1].color={1,0,0};tick();p.visible=false
        SlashCmdList.AZSXUI("plates off");assert(not AZSXUIEnemyHealth1.visible and p.regions[1].alpha==.6 and p.children[1].healthTexture.alpha==1)''')
    def test_native_defaults_apply_once_and_later_choices_are_preserved(self):
        self.run_lua('assert(plateShows==1 and NAMEPLATES_ON==1 and layout=="overlap");layout="stack";NAMEPLATES_ON=0;loadAddon();tick();assert(plateShows==1 and layout=="stack" and NAMEPLATES_ON==0);AZSXUIDB.scale=.9;AZSXUIDB.compactPlates=nil;loadAddon();assert(AZSXUIDB.scale==.9);AZSXUIDB.scale=.7;AZSXUIDB.compactPlates=nil;loadAddon();assert(AZSXUIDB.scale==.5)')
    def test_native_damage_face_is_preserved_by_default(self):
        self.run_lua('AZSXUIDB.numbers=true;DAMAGE_TEXT_FONT="other.ttf";loadAddon();assert(DAMAGE_TEXT_FONT=="other.ttf" and AZSXUIDB.numbers==nil)')
    def test_toggle_restores_native_widgets_and_does_not_toggle_visibility(self):
        self.run_lua('function ShowNameplates()error("visibility changed")end;p=makePlate("Boar",1,0,0);tick();SlashCmdList.AZSXUI("plates off");assert(p:GetWidth()==100);assert(p.regions[4].alpha==1);assert(p.regions[5].alpha==1);assert(p.regions[6].alpha==1)')
    def test_invalid_settings_are_bounded(self):
        self.run_lua('AZSXUIDB.scale=0/0;p=makePlate("Boar",1,0,0);tick();assert(p:GetWidth()==50)')
    def test_unrelated_world_children_are_untouched(self):
        self.run_lua('p=makePlate("Other",1,0,0);p.regions[1].texture="other";tick();assert(p:GetWidth()==100);assert(p.regions[4].alpha==1)')
    def test_unit_pool_discovery_handles_new_children_and_recycled_names(self):
        self.run_lua('p=makePlate("Boar",1,0,0);tick();p.regions[3].text="Rare";tick();assert(p.regions[3].alpha==0);q=makePlate("Other",1,0,0);tick();assert(AZSXUI_Readout().plates==2)')

if __name__=="__main__":unittest.main()
