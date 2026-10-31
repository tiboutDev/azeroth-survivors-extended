"""Public ASX v1 schema, lifecycle, copy isolation and notification contracts."""
from pathlib import Path
import unittest
from test_official_party_lua import STUB,LuaRuntime
ROOT=Path(__file__).resolve().parents[1]
@unittest.skipUnless(LuaRuntime,"ASXUI API tests require Lua 5.1")
class APITests(unittest.TestCase):
    def setUp(self):
        self.lua=LuaRuntime(unpack_returned_tuples=True);self.lua.execute(STUB)
        self.lua.execute((ROOT/'native-ui/official-party.lua').read_text().replace('local asxNativeActive = 0 -- ASX_LOCAL_UI_GATE','local asxNativeActive = 1 -- ASX_LOCAL_UI_GATE',1))
    def publish(self,heartbeat=1,held=700,epoch=1,own='18446744073709551615'):
        h=lambda value:value.encode().hex()
        text=f"1|{epoch}|{own}|{heartbeat}\nM|{own}|0|{h('Player')}\nB|{own}|1|1|0|1200|1200\nF|{own}|2|45\nD|{own}|1|1|33|{h('Elixir')}|{h('INV_Potion_10')}|{h('Description')}\nR|{own}|1|{held}|1\nA|{own}|1000|2000|100000000|80000000|60000000|40000000|20000000|10000000|0|0\n"
        self.lua.globals().wire=text.encode().hex();self.lua.execute('AZS_RUN={asx=wire};AZS_RUN_UPDATED=now;now=now+.1')
    def test_version_and_unavailable_state_are_explicit(self):
        self.lua.execute('local value,reason=ASX_GetState(2);assert(value==nil and reason=="unsupported_version");local s=ASX_GetState(1);assert(ASX_GetAPIVersion()==1 and s.api.major==1 and not s.available and s.reason=="missing_or_invalid_payload" and next(s.data)==nil);local token,error=ASX_RegisterStateHandler("bad");assert(token==nil and error=="invalid_callback");assert(not ASX_UnregisterStateHandler(999))')
    def test_structured_values_and_texts_are_separate_and_guid_is_exact(self):
        self.publish();self.lua.execute('local s=ASX_GetState(1);assert(s.available and s.session.recipientGUID=="18446744073709551615");assert(s.data.fortune.amount==1200 and s.data.fatigue.seconds==45);assert(s.data.discovery.gold==33 and s.data.discovery.name==nil and s.data.discovery.text==nil);assert(s.texts.supplies.name=="Elixir" and s.texts.supplies.description=="Description" and s.texts.supplies.note=="Sold for 33 gold");assert(s.data.sayge.held==700 and s.data.sayge.maximum==2000 and s.texts.sayge.note=="7 / 20 sec");assert(s.data.minimap.zoom==nil and s.data.minimap.radii[1]==100)')
    def test_nested_mutation_never_reaches_asx_or_another_consumer(self):
        self.publish();self.lua.execute('local a=ASX_GetState(1);a.data.fortune.amount=999;a.data.minimap.radii[1]=999;a.texts.fortune.color[1]=999;a.session.recipientGUID="bad";local b=ASX_GetState(1);assert(b.data.fortune.amount==1200 and b.data.minimap.radii[1]==100 and b.texts.fortune.color[1]==.12 and b.session.recipientGUID=="18446744073709551615");local map=ASX_GetMinimapData();map.radii[1]=999;assert(ASX_GetState(1).data.minimap.radii[1]==100)')
    def test_heartbeat_and_timestamp_refresh_do_not_advance_revision(self):
        self.publish();self.lua.execute('before=ASX_GetState(1);notifications=0;token=ASX_RegisterStateHandler(function()notifications=notifications+1 end);update();notifications=0')
        self.publish(heartbeat=600);self.lua.execute('update();local s=ASX_GetState(1);assert(s.revision==before.revision and s.updatedAt>before.updatedAt and notifications==0)')
    def test_progress_session_phase_and_expiry_are_observable_changes(self):
        self.publish();self.lua.execute('update();last=ASX_GetState(1).revision;calls=0;ASX_RegisterStateHandler(function(s)calls=calls+1;observed=s.available;assert(ASX_GetState(1).revision==s.revision)end)')
        self.publish(held=800);self.lua.execute('update();assert(calls==1 and ASX_GetState(1).revision>last);phase="paused";update();assert(calls==2);now=now+5;update();local s=ASX_GetState(1);assert(calls==3 and observed==false and not s.available and s.reason=="stale" and s.session==nil and next(s.data)==nil)')
        self.publish(epoch=2,own='22');self.lua.execute('update();assert(calls==4 and observed==true and ASX_GetState(1).session.recipientGUID=="22");phase="ready";update();assert(calls==5 and ASX_GetState(1).reason=="outside_run")')
    def test_callback_errors_removals_and_additions_are_isolated(self):
        self.publish();self.lua.execute('update();calls=0;late=function()calls=calls+100 end;first=function()ASX_UnregisterStateHandler(second);ASX_RegisterStateHandler(late);error("consumer failure")end;second=function()calls=calls+10 end;third=function()calls=calls+1 end;ASX_RegisterStateHandler(first);ASX_RegisterStateHandler(second);ASX_RegisterStateHandler(third)')
        self.publish(held=800);self.lua.execute('update();assert(calls==1);assert(ASX_UnregisterStateHandler(first));assert(ASX_UnregisterStateHandler(third));assert(not ASX_UnregisterStateHandler(second))')
        self.publish(held=900);self.lua.execute('update();assert(calls==101)')
    def test_native_style_globals_and_handlers_are_detached(self):
        self.publish();self.lua.execute('seen=0;handler=function(s)s.data.fortune.amount=999;seen=seen+1 end;assert(ASX_RegisterStateHandler(handler));assert(ASX_RegisterStateHandler(handler));update();assert(seen==1 and ASX_STATE.available and type(ASX_STATE_UPDATED)=="number");ASX_STATE.data.fortune.amount=888;assert(ASX_GetState().data.fortune.amount==1200);assert(ASX_UnregisterStateHandler(handler));assert(not ASX_UnregisterStateHandler(handler))')
    def test_each_handler_gets_its_own_snapshot(self):
        self.publish();self.lua.execute('ASX_RegisterStateHandler(function(s)s.data.fortune.amount=999;s.texts.fortune.color[1]=999 end);ASX_RegisterStateHandler(function(s)assert(s.data.fortune.amount==1200 and s.texts.fortune.color[1]==.12);secondObserved=true end);update();assert(secondObserved)')
    def test_subscriptions_are_bounded(self):
        self.lua.execute('for i=1,64 do assert(ASX_RegisterStateHandler(function()end))end;local token,reason=ASX_RegisterStateHandler(function()end);assert(token==nil and reason=="subscriber_limit")')
    def equipment_wire(self, header='L|11|1', row=None):
        h=lambda value:value.encode().hex()
        if row is None:row='E|11|1|2|'+h('Native ring')+'|'+h('INV_Jewelry_Ring_01')+'|'+h('Exact owner text\n+10 damage')
        text=f"1|7|11|1\nM|11|0|{h('Player')}\n{header}\n{row}\n"
        self.lua.globals().wire=text.encode().hex()
        self.lua.execute('AZS_RUN={asx=wire};AZS_RUN_UPDATED=now;now=now+.1')
    def test_local_equipment_native_text_copy_isolation_and_zero_item_snapshot(self):
        self.equipment_wire()
        self.lua.execute('local major,minor=ASX_GetAPIVersion();assert(major==1 and minor==6);local s=ASX_GetState(1);assert(s.available and s.data.equipment[1].slot==1 and s.data.equipment[1].rank==2 and s.data.equipment[1].name==nil);assert(s.texts.equipment[1].name=="Native ring" and s.texts.equipment[1].text=="Exact owner text\\n+10 damage");s.texts.equipment[1].name="Mutated";assert(ASX_GetState(1).texts.equipment[1].name=="Native ring")')
        self.equipment_wire(header='L|11|0',row='')
        self.lua.execute('local s=ASX_GetState(1);assert(s.available and table.getn(s.data.equipment)==0 and table.getn(s.texts.equipment)==0)')
    def test_equipment_retains_native_category_and_rejects_unknown_category(self):
        h=lambda value:value.encode().hex()
        row='E|11|1|2|'+h('Native ring')+'|'+h('INV_Ring')+'|'+h('Owner text')+'|ring'
        self.equipment_wire(row=row)
        self.lua.execute('assert(ASX_GetState(1).data.equipment[1].category=="ring")')
        self.equipment_wire(row=row.replace('|ring','|unknown'))
        self.lua.execute('assert(not ASX_GetState(1).available)')
    def test_local_equipment_rejects_missing_duplicate_out_of_order_or_foreign_items(self):
        h=lambda value:value.encode().hex()
        row=f"E|11|1|2|{h('Ring')}|{h('INV_Test')}|{h('Native text')}"
        for header,body in [('L|11|6',row),('L|11|1',''),('L|11|1',row+'\n'+row),('L|11|1',row.replace('E|11|1|','E|11|2|')),('L|11|1',row.replace('E|11|','E|22|')),('L|11|0',row)]:
            with self.subTest(header=header,body=body):
                self.equipment_wire(header,body)
                self.lua.execute('assert(not ASX_GetState(1).available)')
if __name__=='__main__':unittest.main()
