-- Opt-in diagnostic observer. Read-only widgets; no input, tooltip activation or screenshots.
local cfg=ASXDiagnosticConfig
if not cfg or cfg.enabled~=true then return end
local transport=cfg.sink and 'configured'or'native-reserved-sink'
local sink=cfg.sink
if not sink and type(SetCVar)=='function'and type(RegisterCVar)=='function'then
 RegisterCVar('asx_diagnostic_mailbox','')
 sink=function(row)SetCVar('asx_diagnostic_mailbox',row)end
end
if type(sink)~='function'then error('ASX diagnostic transport unavailable')end
local started,last,sequence=GetTime(),-100,0
local maximum=math.min(tonumber(cfg.max_records)or 256,512)
local duration=math.min(tonumber(cfg.duration)or 180,300)
local previous={}
local function quote(s)
 s=string.sub(tostring(s),1,512)
 s=string.gsub(s,'\\','\\\\');s=string.gsub(s,'"','\\"')
 s=string.gsub(s,'[%z\1-\31]',function(c)return string.format('\\u%04x',string.byte(c))end)
 return '"'..s..'"'
end
local function encode(value,depth)
 depth=depth or 0
 if depth>7 then return 'null' end
 local kind=type(value)
 if kind=='nil' then return 'null' end
 if kind=='boolean' then return value and 'true'or'false' end
 if kind=='number' then if value~=value or value-value~=0 then return 'null'end;return tostring(value)end
 if kind=='string'then return quote(value)end
 if kind~='table'then return quote('unsupported:'..kind)end
 local keys={};for k in pairs(value)do if type(k)=='string'then table.insert(keys,k)end end;table.sort(keys)
 local out={};for _,k in ipairs(keys)do table.insert(out,quote(k)..':'..encode(value[k],depth+1))end
 return '{'..table.concat(out,',')..'}'
end
local function call(object,method)
 if not object or type(object[method])~='function'then return nil end
 local ok,value=pcall(object[method],object)
 if ok then return value end
 return nil
end
local function flag(object,method)
 if not object or type(object[method])~='function'then return nil end
 local ok,value=pcall(object[method],object)
 if ok then return value~=nil and value~=false and value~=0 end
 return nil
end
local function frame(name)
 local f=getglobal(name)
 if not f then return {exists=false}end
 return {exists=true,shown=flag(f,'IsShown'),visible=flag(f,'IsVisible'),width=call(f,'GetWidth'),height=call(f,'GetHeight'),alpha=call(f,'GetAlpha'),left=call(f,'GetLeft'),top=call(f,'GetTop'),level=call(f,'GetFrameLevel'),strata=call(f,'GetFrameStrata'),title=call(f.title,'GetText'),reward=call(f.reward,'GetText'),note=call(f.note,'GetText'),icon=call(f.icon,'GetTexture'),value=call(f.bar,'GetValue')}
end
local function emit(kind,data)
 if sequence>=maximum then return end
 sequence=sequence+1
 local record={schema=1,session=cfg.session,slot=cfg.slot,sequence=sequence,clock=GetTime(),run_clock=AZS_RUN and AZS_RUN.t,kind=kind,data=data}
 if type(sink)=='function'then sink('ASX_DIAG '..encode(record))end
end
local function sample(kind,data)
 local encoded=encode(data)
 if previous[kind]~=encoded then previous[kind]=encoded;emit(kind,data)end
end
local observer=CreateFrame('Frame')
emit('observer.started',{transport=transport,interval=0.5,record_limit=maximum,duration=duration})
observer:SetScript('OnUpdate',function()
 local now=GetTime()
 if now-started>=duration or sequence>=maximum-1 then emit('observer.stopped',{reason=sequence>=maximum-1 and'budget'or'duration'});observer:SetScript('OnUpdate',nil);return end
 if now-last<0.5 then return end;last=now
 sample('ui.supply_toast',frame('ASXSupplyDiscoveryToast'))
 sample('ui.sayge_progress',frame('ASXSaygeTrialProgress'))
 local buffs={available=type(GetPlayerBuff)=='function'and type(GetPlayerBuffTexture)=='function',slots={}}
 if buffs.available then for i=0,31 do
  local slot,permanent=GetPlayerBuff(i,'HELPFUL')
  if slot and slot>=0 then buffs.slots['slot_'..i]={slot=slot,permanent=permanent,texture=GetPlayerBuffTexture(slot),remaining=GetPlayerBuffTimeLeft and GetPlayerBuffTimeLeft(slot),widget=frame('BuffButton'..i)}end
 end end
 sample('ui.buffs',buffs)
 local tip=frame('GameTooltip')
 if tip.visible then tip.lines={};for i=1,24 do local label=getglobal('GameTooltipTextLeft'..i);if label and call(label,'IsShown')then tip.lines['line_'..i]=call(label,'GetText')end end end
 sample('ui.tooltip',tip)
 if type(ASXRitualReadout)=='function'then local id,held,inside,shown=ASXRitualReadout();sample('asx.ritual_model',{event=id,held=held,inside=inside,shown=shown})end
end)
