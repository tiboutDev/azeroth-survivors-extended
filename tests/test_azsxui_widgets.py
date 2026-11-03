"""Native-value badges, resurrection channels and exact native reward receipts."""
import unittest
import test_azsxui_frames as frames
import test_azsxui_events as events

class MedallionTests(unittest.TestCase):
 def setUp(self):
  frames.FramesTests.setUp(self)
  self.lua.execute(r'''
view.data.crystals={red=1,blue=2,green=3,yellow=4,total=10,index=0}
view.texts.crystals={title="Crystal Resonance",icon="Interface\\Icons\\INV_Misc_Gem_Variety_01",note="Crystals collected: 10"}
view.data.fortune={boss=1,category=1,amount=1250,best=1800}
view.texts.fortune={title="Native fortune title",icon="Interface\\Icons\\INV_Misc_Orb_02",reward="AZSX fortune",note="AZSX note"}
function GetPlayerBuffTexture(index)if index==0 then return "Interface\\Icons\\INV_Misc_Gem_Variety_01"elseif index==1 then return "Interface\\Icons\\INV_Misc_Orb_02"else return "ordinary"end end
for i=0,1 do local f=getglobal("BuffButton"..i);function f:IsMouseEnabled()return self.mouse~=false end end
function GameTooltip:SetPlayerBuff(index)self.nativeIndex=index;self.title="Original native tooltip"end
AZSXUI_DockAuras(view)
''')
 def test_badges_use_structured_counts_active_party_bonus_and_native_tooltip(self):
  self.lua.execute('assert(AZSXUICrystalMedallion:IsShown() and AZSXUICrystalMedallion.total:GetText()=="10" and AZSXUICrystalMedallion.counts==nil and AZSXUICrystalMedallion:GetWidth()==36);assert(not AZSXUIFortuneMedallion.value and not AZSXUIFortuneMedallion.category);assert(BuffButton0:GetAlpha()==0 and BuffButton1:GetAlpha()==0);hudCall("AZSXUICrystalMedallion","OnEnter");assert(GameTooltip.nativeIndex==0 and GameTooltip.title=="Original native tooltip");hudCall("AZSXUIFortuneMedallion","OnEnter");assert(GameTooltip.nativeIndex==1)')
 def test_charged_potion_joins_row_then_releases_reused_slot(self):
  self.lua.execute("""
local gadget=CreateFrame("Button","AZSXUIEngineeringBadge",UIParent);gadget:SetWidth(28);gadget:Show()
local native=GetPlayerBuffTexture
function GetPlayerBuffTexture(i)if i==2 then return "potion"end;return native(i)end
view.data.consumables={[7]={index=2}};view.texts.consumables={[7]={icon="potion"}}
AZSXUI_DockAuras(view)
local left=AZSXUICrystalMedallion.anchor[4]
assert(BuffButton2:GetAlpha()==0 and AZSXUIPotionBadge.icon.portraitMasked)
assert(gadget.anchor[4]<AZSXUIPotionBadge.anchor[4] and left==-(AZSXUIPotionBadge.anchor[4]+AZSXUIPotionBadge:GetWidth()))
view.data.consumables={};AZSXUI_DockAuras(view)
assert(BuffButton2.anchor[2]==AZSXUIHudAuras and AZSXUICrystalMedallion.anchor[4]>left)
""")
 def test_potion_without_native_button_uses_owner_icon_until_spent(self):
  self.lua.execute("""
view.data.consumables={[7]={index=32}}
view.texts.consumables={[7]={icon="potion",name="Healing Potion",description="One charge."}}
AZSXUI_DockAuras(view)
assert(AZSXUIPotionBadge:IsShown() and AZSXUIPotionBadge.anchor[2]==AZSXUIHudVitals)
hudCall("AZSXUIPotionBadge","OnEnter");assert(GameTooltip.title=="Healing Potion")
view.data.consumables={};AZSXUI_DockAuras(view);assert(not AZSXUIPotionBadge:IsShown())
""")
 def test_zero_crystals_keep_medallion_and_native_tooltip_fallback(self):
  self.lua.execute('view.data.crystals={red=0,blue=0,green=0,yellow=0,total=0,index=32};view.texts.crystals.note="Crystals collected: 0";view.texts.crystals.details={"Find crystals across the map to gain permanent bonuses."};AZSXUI_DockAuras(view);assert(AZSXUICrystalMedallion:IsShown() and AZSXUICrystalMedallion.total:GetText()=="");hudCall("AZSXUICrystalMedallion","OnEnter");assert(GameTooltip.title=="Crystal Resonance")')
 def test_missing_crystal_native_slot_cannot_bind_to_the_fortune_slot(self):
  self.lua.execute('view.data.crystals.index=32;AZSXUI_DockAuras(view);assert(AZSXUICrystalMedallion.native==nil and AZSXUIFortuneMedallion.native==BuffButton1);assert(BuffButton0:GetAlpha()==1);hudCall("AZSXUICrystalMedallion","OnEnter");assert(GameTooltip.title=="Crystal Resonance")')
 def test_native_tooltip_does_not_follow_a_reused_aura_index(self):
  self.lua.execute('function GetPlayerBuffTexture()return "ordinary"end;GameTooltip.nativeIndex=nil;hudCall("AZSXUIFortuneMedallion","OnEnter");assert(GameTooltip.nativeIndex==nil and GameTooltip.title=="Native fortune title")')
 def test_stale_state_restores_native_visuals_and_does_not_keep_a_badge(self):
  self.lua.execute('view.available=false;AZSXUI_DockAuras(view);assert(not AZSXUICrystalMedallion:IsShown() and not AZSXUIFortuneMedallion:IsShown());assert(BuffButton0:GetAlpha()==1 and BuffButton0:IsMouseEnabled());view.available=true;AZSXUI_DockAuras(view);AZSXUI_ConfigureHud("off");arg1=.1;AZSXUIFramesController.scripts.OnUpdate();assert(BuffButton1:GetAlpha()==1 and BuffButton1:IsMouseEnabled())')
 def test_right_click_forwards_current_native_handler_but_not_a_reused_index(self):
  self.lua.execute('cancelled=0;BuffButton1.scripts.OnClick=function()assert(this==BuffButton1 and arg1=="RightButton");cancelled=cancelled+1 end;arg1="RightButton";hudCall("AZSXUIFortuneMedallion","OnClick");assert(cancelled==1);function GetPlayerBuffTexture()return "ordinary"end;hudCall("AZSXUIFortuneMedallion","OnClick");assert(cancelled==1)')
 def test_fortune_rounding_matches_provider(self):
  self.lua.execute('view.data.fortune.best=1000;AZSXUI_DockAuras(view);local b=AZSXUIFortuneMedallion;assert(b.nature:IsShown() and string.find(b.naturePath,"INV_Misc_Book_09",1,true));view.data.fortune.category=0;AZSXUI_DockAuras(view);assert(string.find(b.naturePath,"Tarot_Stack_01",1,true));view.data.fortune.category=2;AZSXUI_DockAuras(view);assert(string.find(b.naturePath,"Coin_01",1,true));view.data.fortune.category=4;AZSXUI_DockAuras(view);assert(not b.nature:IsShown() and not b.natureRim:IsShown())')

class NativeWidgetTests(unittest.TestCase):
 def setUp(self):
  events.EventTests.setUp(self)
  self.lua.execute((events.ROOT/"addons/AZSXUI/Theme.lua").read_text())
  self.lua.execute('function tick()now=now+.3;objects.AZSXUIEventsController.scripts.OnUpdate()end;hudActive=true;nativePrefs={revive=true,loot=true};function AZSXUI_GetHudAnchors()if hudActive then return {}end end;function AZS_IsHudShown(key)return nativePrefs[key]end;function AZS_SetHudShown(key,value)nativePrefs[key]=value end;function GetLocale()return locale or "enUS"end;revive={};function AZS_GetRevive()return revive end;function receipt(channel,text)event=channel;arg1=text;objects.AZSXUIEventsController.scripts.OnEvent()end;function reward()return objects.AZSXUINativeRewardToast end;function res()return objects.AZSXUIResurrectionProgress end')
 def test_marks_burst_adds_decimal_gains_and_native_total_in_one_popup(self):
  self.lua.execute('receipt("CHAT_MSG_SYSTEM","You earn 0.5 marks (2.5 in all).");receipt("CHAT_MSG_SYSTEM","You earn 1.25 marks (3.75 in all).");tick();assert(reward().reward.text=="|cff1eff00+1.75|r marks" and reward().note.text=="Total: |cff1eff003.75|r");receipt("CHAT_MSG_SYSTEM","You earn 0.5 marks (4.25 in all).");tick();assert(reward().reward.text=="|cff1eff00+2.25|r marks");receipt("CHAT_MSG_SYSTEM","You earn 0.5 marks (4.25 in all).");tick();assert(reward().reward.text=="|cff1eff00+2.25|r marks");now=now+4;tick();assert(not reward().shown)')
 def test_contiguous_duplicate_marks_extend_one_popup_past_first_receipt_window(self):
  self.lua.execute('receipt("CHAT_MSG_SYSTEM","You earn 1 marks (1 in all).");tick();now=now+2.3;receipt("CHAT_MSG_SYSTEM","You earn 2 marks (3 in all).");tick();now=now+2.3;receipt("CHAT_MSG_SYSTEM","You earn 3 marks (6 in all).");tick();assert(reward().reward.text=="|cff1eff00+6|r marks" and reward().note.text=="Total: |cff1eff006|r");assert(reward().shown);now=now+3.2;tick();assert(not reward().shown)')
 def test_large_mark_totals_keep_decimals_and_do_not_use_scientific_notation(self):
  self.lua.execute('receipt("CHAT_MSG_SYSTEM","You earn 0.5 marks (1234567.5 in all).");receipt("CHAT_MSG_SYSTEM","You earn 0.25 marks (1234567.75 in all).");tick();assert(reward().reward.text=="|cff1eff00+0.75|r marks" and reward().note.text=="Total: |cff1eff001234567.75|r")')
 def test_marks_do_not_merge_across_an_item_or_an_unobserved_total_gap(self):
  self.lua.execute('receipt("CHAT_MSG_SYSTEM","You earn 1 marks (1 in all).");receipt("CHAT_MSG_LOOT","You receive loot: [New item]");receipt("CHAT_MSG_SYSTEM","You earn 2 marks (3 in all).");tick();assert(reward().reward.text=="|cff1eff00+1|r marks");now=now+4;tick();assert(reward().title.text=="Loot received");now=now+4;tick();assert(reward().reward.text=="|cff1eff00+2|r marks");receipt("CHAT_MSG_SYSTEM","You earn 1 marks (20 in all).");now=now+4;tick();assert(reward().reward.text=="|cff1eff00+1|r marks" and reward().note.text=="Total: |cff1eff0020|r")')
 def test_native_revive_progress_both_directions_and_interruption(self):
  self.lua.execute('revive={reviving="Alice",rshare=.35};tick();assert(res().shown and res().bar.value==.35 and res().title.text=="Reviving Alice" and not nativePrefs.revive);assert(not objects.AZSXSaygeTrialProgress.shown);revive={down=1,share=.6,by="Bob"};tick();assert(res().bar.value==.6 and res().note.text=="Being revived by Bob");revive={};tick();assert(not res().shown);hudActive=false;tick();assert(nativePrefs.revive)')
 def test_hidden_native_revive_and_invalid_data_fail_open(self):
  self.lua.execute('nativePrefs.revive=false;revive={down=1,share=.5};tick();assert(not res());hudActive=false;tick();nativePrefs.revive=true;hudActive=true;revive={down=1,share=2};tick();assert(nativePrefs.revive and not res())')
 def test_marks_preserve_native_decimal_receipt_and_loot_is_queued(self):
  self.lua.execute('local marks="|cffffd100You earn 0.5 marks (0.5 in all).|r";receipt("CHAT_MSG_SYSTEM",marks);tick();assert(reward().shown and reward().reward.text=="|cff1eff00+0.5|r marks" and reward().note.text=="Total: |cff1eff000.5|r" and reward().title.text=="Marks earned" and nativePrefs.loot);receipt("CHAT_MSG_LOOT","You receive loot: [Native item]");assert(not nativePrefs.loot);now=now+4;tick();assert(reward().title.text=="Loot received" and reward().reward.text=="Native item" and reward().note.text=="Quantity: |cff1eff001|r");hudActive=false;tick();assert(nativePrefs.loot and not reward().shown)')
 def test_logout_does_not_allow_event_widgets_to_hide_native_parts_again(self):
  self.lua.execute('revive={reviving="Alice",rshare=.3};tick();receipt("CHAT_MSG_LOOT","You receive loot: [Native item]");tick();assert(not nativePrefs.loot and not nativePrefs.revive);event="PLAYER_LOGOUT";objects.AZSXUINativeVisibilityController.scripts.OnEvent();assert(nativePrefs.loot and nativePrefs.revive);tick();receipt("CHAT_MSG_LOOT","You receive loot: [Another item]");assert(nativePrefs.loot and nativePrefs.revive)')
 def test_native_rewards_present_item_quantity_rarity_and_marks_without_chat_phrase(self):
  self.lua.execute('receipt("CHAT_MSG_LOOT","You receive loot: |cff0070dd|Hitem:123:0:0|h[Native item]|h|rx2.");tick();assert(reward().reward.text=="|cff0070ddNative item|r" and reward().note.text=="Quantity: |cff1eff002|r");now=now+4;receipt("CHAT_MSG_SYSTEM","You earn 1.25 marks (8.75 in all).");tick();assert(reward().reward.text=="|cff1eff00+1.25|r marks" and reward().note.text=="Total: |cff1eff008.75|r")')
 def test_recorded_native_collection_receipt_becomes_item_name_and_quantity(self):
  # Actual English receipt captured in the isolated 0.131.0 solo trial.
  self.lua.execute("receipt(\"CHAT_MSG_LOOT\",\"You receive loot: |cffffffff[Recruit's Helm]|r |cff40ff40New in your collection!|r\");tick();assert(reward().reward.text==\"|cffffffffRecruit's Helm|r\" and reward().note.text==\"Quantity: |cff1eff001|r\")")
 def test_incomplete_native_loot_receipt_does_not_hide_native_notice(self):
  self.lua.execute('receipt("CHAT_MSG_LOOT","You receive loot: unknown payload");tick();assert(nativePrefs.loot and not reward())')
 def test_native_item_icon_accepts_classic_and_extended_item_info(self):
  for count in (9,10):
   with self.subTest(fields=count):
    self.setUp()
    values=['"Native item"','"link"','1','10']+(['1']if count==10 else[])+['"Armor"','"Plate"','1','"INVTYPE_HEAD"','"Interface\\\\Icons\\\\INV_Helmet_01"']
    self.lua.execute('function GetItemInfo()return '+','.join(values)+' end;receipt("CHAT_MSG_LOOT","You receive loot: [Native item]");tick();assert(reward().icon.texture=="Interface\\\\Icons\\\\INV_Helmet_01")')
 def test_structured_native_loot_uses_actual_icon_and_does_not_repeat_on_chat_or_heartbeat(self):
  self.lua.execute('local old=AZSX_GetState;function AZSX_GetState()local s=old();s.api={major=1,minor=6};s.data.loot={{event=1,rarity=0,isNew=true}};s.texts.loot={[1]={name="Recruit Helm",icon="INV_Helmet_02"}};return s end;receipt("CHAT_MSG_SYSTEM","You receive loot: [Recruit Helm]");assert(nativePrefs.loot);tick();tick();assert(not nativePrefs.loot and reward().icon.texture=="Interface\\\\Icons\\\\INV_Helmet_02" and reward().note.text=="New in your collection!");now=now+4;tick();assert(not reward().shown);receipt("CHAT_MSG_SYSTEM","You receive loot: [Recruit Helm]");tick();assert(not reward().shown)')
 def test_pending_loot_rebinds_recipient_without_showing_old_session_receipt(self):
  self.lua.execute('local old=AZSX_GetState;function AZSX_GetState()local s=old();s.api={major=1,minor=6};s.data.loot={{event=1,rarity=0,isNew=true}};s.texts.loot={[1]={name=s.session.recipientGUID=="11"and"Old owner"or"New owner",icon="INV_Helmet_02"}};return s end;view.owner.ritual=nil;tick();view.own="22";view.epoch="2";tick();tick();assert(reward().reward.text=="|cffffffffNew owner|r")')
 def test_current_provider_waits_for_real_metadata_instead_of_guessing_during_unavailability(self):
  self.lua.execute('function AZSX_GetAPIVersion()return 1,6 end;function AZSX_GetState()return {available=false,api={major=1,minor=6}}end;receipt("CHAT_MSG_SYSTEM","You receive loot: [Unresolved item]");tick();assert(nativePrefs.loot and not reward())')
 def test_already_collected_native_items_hide_both_toasts_but_preserve_marks(self):
  self.lua.execute('local old=AZSX_GetState;function AZSX_GetAPIVersion()return 1,6 end;function AZSX_GetState()local s=old();s.api={major=1,minor=6};s.data.loot={{event=1,rarity=0,isNew=false}};s.texts.loot={[1]={name="Already collected",icon="INV_Helmet_02"}};return s end;tick();assert(not nativePrefs.loot and not reward());receipt("CHAT_MSG_SYSTEM","You receive loot: [Already collected]");tick();assert(not reward());receipt("CHAT_MSG_SYSTEM","You earn 0.5 marks (2.5 in all).");receipt("CHAT_MSG_SYSTEM","You earn 1.25 marks (3.75 in all).");tick();tick();assert(reward().title.text=="Marks earned" and reward().reward.text=="|cff1eff00+1.75|r marks");now=now+4;tick();assert(not reward().shown);function AZSX_GetState()return {available=false}end;tick();assert(nativePrefs.loot)')
 def test_new_ritual_banner_defers_receipts_without_losing_their_lifetime(self):
  self.lua.execute('tick();view.owner.ritual.event=3;receipt("CHAT_MSG_SYSTEM","You earn 0.5 marks (0.5 in all).");tick();assert(not reward());for i=1,21 do tick()end;assert(reward().shown and reward().title.text=="Marks earned")')
 def test_overlapping_choice_and_ritual_wait_do_not_extend_receipt_twice(self):
  self.lua.execute('tick();receipt("CHAT_MSG_SYSTEM","You earn 0.5 marks (0.5 in all).");tick();assert(reward().shown);view.owner.ritual.event=3;phase="choosing";for i=1,21 do tick()end;phase="playing";tick();assert(reward().shown);for i=1,11 do tick()end;assert(not reward().shown)')
 def test_unsupported_locale_and_player_messages_keep_native_loot(self):
  self.lua.execute('locale="frFR";receipt("CHAT_MSG_SYSTEM","You receive loot: [Native item]");tick();assert(nativePrefs.loot and not reward());locale="enUS";receipt("CHAT_MSG_SAY","You earn 50 marks (50 in all).");tick();assert(not reward())')
 def test_receipts_defer_during_choice_and_do_not_replay_after_disable(self):
  self.lua.execute('phase="choosing";receipt("CHAT_MSG_SYSTEM","You earn 0.5 marks (0.5 in all).");tick();assert(not reward());now=now+10;phase="playing";tick();assert(reward().shown);hudActive=false;tick();hudActive=true;tick();assert(not reward().shown)')

if __name__=='__main__':unittest.main()
