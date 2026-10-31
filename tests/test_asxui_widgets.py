"""Native-value badges, resurrection channels and exact native reward receipts."""
import unittest
import test_asxui_frames as frames
import test_asxui_events as events

class MedallionTests(unittest.TestCase):
 def setUp(self):
  frames.FramesTests.setUp(self)
  self.lua.execute(r'''
view.data.crystals={red=1,blue=2,green=3,yellow=4,total=10,index=0}
view.texts.crystals={title="Crystal Resonance",icon="Interface\\Icons\\INV_Misc_Gem_Variety_01",note="Crystals collected: 10"}
view.data.fortune={boss=1,category=1,amount=1250,best=1800}
view.texts.fortune={title="Native fortune title",icon="Interface\\Icons\\INV_Misc_Orb_02",reward="ASX fortune",note="ASX note"}
function GetPlayerBuffTexture(index)if index==0 then return "Interface\\Icons\\INV_Misc_Gem_Variety_01"elseif index==1 then return "Interface\\Icons\\INV_Misc_Orb_02"else return "ordinary"end end
for i=0,1 do local f=getglobal("BuffButton"..i);function f:IsMouseEnabled()return self.mouse~=false end end
function GameTooltip:SetPlayerBuff(index)self.nativeIndex=index;self.title="Original native tooltip"end
ASXUI_DockAuras(view)
''')
 def test_badges_use_structured_counts_active_party_bonus_and_native_tooltip(self):
  self.lua.execute('assert(ASXUICrystalMedallion:IsShown() and ASXUICrystalMedallion.counts.blue:GetText()=="2");assert(ASXUIFortuneMedallion.value:GetText()=="+18%" and ASXUIFortuneMedallion.category:GetText()=="XP");assert(BuffButton0:GetAlpha()==0 and BuffButton1:GetAlpha()==0);hudCall("ASXUICrystalMedallion","OnEnter");assert(GameTooltip.nativeIndex==0 and GameTooltip.title=="Original native tooltip");hudCall("ASXUIFortuneMedallion","OnEnter");assert(GameTooltip.nativeIndex==1)')
 def test_missing_crystal_native_slot_cannot_bind_to_the_fortune_slot(self):
  self.lua.execute('view.data.crystals.index=32;ASXUI_DockAuras(view);assert(ASXUICrystalMedallion.native==nil and ASXUIFortuneMedallion.native==BuffButton1);assert(BuffButton0:GetAlpha()==1);hudCall("ASXUICrystalMedallion","OnEnter");assert(GameTooltip.title=="Crystal Resonance")')
 def test_native_tooltip_does_not_follow_a_reused_aura_index(self):
  self.lua.execute('function GetPlayerBuffTexture()return "ordinary"end;GameTooltip.nativeIndex=nil;hudCall("ASXUIFortuneMedallion","OnEnter");assert(GameTooltip.nativeIndex==nil and GameTooltip.title=="Native fortune title")')
 def test_stale_state_restores_native_visuals_and_does_not_keep_a_badge(self):
  self.lua.execute('view.available=false;ASXUI_DockAuras(view);assert(not ASXUICrystalMedallion:IsShown() and not ASXUIFortuneMedallion:IsShown());assert(BuffButton0:GetAlpha()==1 and BuffButton0:IsMouseEnabled());view.available=true;ASXUI_DockAuras(view);ASXUI_ConfigureHud("off");arg1=.1;ASXUIFramesController.scripts.OnUpdate();assert(BuffButton1:GetAlpha()==1 and BuffButton1:IsMouseEnabled())')
 def test_right_click_forwards_current_native_handler_but_not_a_reused_index(self):
  self.lua.execute('cancelled=0;BuffButton1.scripts.OnClick=function()assert(this==BuffButton1 and arg1=="RightButton");cancelled=cancelled+1 end;arg1="RightButton";hudCall("ASXUIFortuneMedallion","OnClick");assert(cancelled==1);function GetPlayerBuffTexture()return "ordinary"end;hudCall("ASXUIFortuneMedallion","OnClick");assert(cancelled==1)')
 def test_fortune_rounding_matches_provider(self):
  self.lua.execute('view.data.fortune.best=1000;ASXUI_DockAuras(view);assert(ASXUIFortuneMedallion.value:GetText()=="+13%");local b=ASXUIFortuneMedallion;assert(b.nature:IsShown() and string.find(b.naturePath,"INV_Misc_Book_09",1,true));view.data.fortune.category=0;ASXUI_DockAuras(view);assert(string.find(b.naturePath,"Tarot_Stack_01",1,true));view.data.fortune.category=2;ASXUI_DockAuras(view);assert(string.find(b.naturePath,"Coin_01",1,true));view.data.fortune.category=4;ASXUI_DockAuras(view);assert(not b.nature:IsShown() and not b.natureRim:IsShown())')

class NativeWidgetTests(unittest.TestCase):
 def setUp(self):
  events.EventTests.setUp(self)
  self.lua.execute((events.ROOT/"addons/ASXUI/Theme.lua").read_text())
  self.lua.execute('function tick()now=now+.3;objects.ASXUIEventsController.scripts.OnUpdate()end;hudActive=true;nativePrefs={revive=true,loot=true};function ASXUI_GetHudAnchors()if hudActive then return {}end end;function AZS_IsHudShown(key)return nativePrefs[key]end;function AZS_SetHudShown(key,value)nativePrefs[key]=value end;function GetLocale()return locale or "enUS"end;revive={};function AZS_GetRevive()return revive end;function receipt(channel,text)event=channel;arg1=text;objects.ASXUIEventsController.scripts.OnEvent()end;function reward()return objects.ASXUINativeRewardToast end;function res()return objects.ASXUIResurrectionProgress end')
 def test_native_revive_progress_both_directions_and_interruption(self):
  self.lua.execute('revive={reviving="Alice",rshare=.35};tick();assert(res().shown and res().bar.value==.35 and res().title.text=="Reviving Alice" and not nativePrefs.revive);assert(not objects.ASXSaygeTrialProgress.shown);revive={down=1,share=.6,by="Bob"};tick();assert(res().bar.value==.6 and res().note.text=="Being revived by Bob");revive={};tick();assert(not res().shown);hudActive=false;tick();assert(nativePrefs.revive)')
 def test_hidden_native_revive_and_invalid_data_fail_open(self):
  self.lua.execute('nativePrefs.revive=false;revive={down=1,share=.5};tick();assert(not res());hudActive=false;tick();nativePrefs.revive=true;hudActive=true;revive={down=1,share=2};tick();assert(nativePrefs.revive and not res())')
 def test_marks_preserve_native_decimal_receipt_and_loot_is_queued(self):
  self.lua.execute('local marks="|cffffd100You earn 0.5 marks (0.5 in all).|r";receipt("CHAT_MSG_SYSTEM",marks);tick();assert(reward().shown and reward().reward.text==marks and reward().title.text=="Marks earned" and nativePrefs.loot);receipt("CHAT_MSG_LOOT","You receive loot: [Native item]");assert(not nativePrefs.loot);now=now+4;tick();assert(reward().title.text=="Loot received" and reward().reward.text=="You receive loot: [Native item]");hudActive=false;tick();assert(nativePrefs.loot and not reward().shown)')
 def test_logout_does_not_allow_event_widgets_to_hide_native_parts_again(self):
  self.lua.execute('revive={reviving="Alice",rshare=.3};tick();receipt("CHAT_MSG_LOOT","You receive loot: [Native item]");tick();assert(not nativePrefs.loot and not nativePrefs.revive);event="PLAYER_LOGOUT";objects.ASXUINativeVisibilityController.scripts.OnEvent();assert(nativePrefs.loot and nativePrefs.revive);tick();receipt("CHAT_MSG_LOOT","You receive loot: [Another item]");assert(nativePrefs.loot and nativePrefs.revive)')
 def test_native_item_icon_accepts_classic_and_extended_item_info(self):
  for count in (9,10):
   with self.subTest(fields=count):
    self.setUp()
    values=['"Native item"','"link"','1','10']+(['1']if count==10 else[])+['"Armor"','"Plate"','1','"INVTYPE_HEAD"','"Interface\\\\Icons\\\\INV_Helmet_01"']
    self.lua.execute('function GetItemInfo()return '+','.join(values)+' end;receipt("CHAT_MSG_LOOT","You receive loot: [Native item]");tick();assert(reward().icon.texture=="Interface\\\\Icons\\\\INV_Helmet_01")')
 def test_new_ritual_banner_defers_receipts_without_losing_their_lifetime(self):
  self.lua.execute('tick();view.owner.ritual.event=3;receipt("CHAT_MSG_SYSTEM","You earn 0.5 marks (0.5 in all).");tick();assert(not reward());for i=1,21 do tick()end;assert(reward().shown and reward().title.text=="Marks earned")')
 def test_overlapping_choice_and_ritual_wait_do_not_extend_receipt_twice(self):
  self.lua.execute('tick();receipt("CHAT_MSG_SYSTEM","You earn 0.5 marks (0.5 in all).");tick();assert(reward().shown);view.owner.ritual.event=3;phase="choosing";for i=1,21 do tick()end;phase="playing";tick();assert(reward().shown);for i=1,11 do tick()end;assert(not reward().shown)')
 def test_unsupported_locale_and_player_messages_keep_native_loot(self):
  self.lua.execute('locale="frFR";receipt("CHAT_MSG_SYSTEM","You receive loot: [Native item]");tick();assert(nativePrefs.loot and not reward());locale="enUS";receipt("CHAT_MSG_SAY","You earn 50 marks (50 in all).");tick();assert(not reward())')
 def test_receipts_defer_during_choice_and_do_not_replay_after_disable(self):
  self.lua.execute('phase="choosing";receipt("CHAT_MSG_SYSTEM","You earn 0.5 marks (0.5 in all).");tick();assert(not reward());now=now+10;phase="playing";tick();assert(reward().shown);hudActive=false;tick();hudActive=true;tick();assert(not reward().shown)')

if __name__=='__main__':unittest.main()
