"""Opt-in private startup measurements. No input, affinity or priority changes."""
import ctypes,json,threading,time
from ctypes import wintypes as W

class Sampler:
    def __init__(self, children):
        self.children=children; self.rows=[]; self.stop=threading.Event(); self.started=time.monotonic()
        self.kernel=ctypes.WinDLL("kernel32",use_last_error=True)
        self.kernel.GetProcessTimes.argtypes=[W.HANDLE,*([ctypes.POINTER(W.FILETIME)]*4)]
        self.thread=threading.Thread(target=self.run,daemon=True);self.thread.start()
    def run(self):
        while not self.stop.is_set():
            for slot,p in enumerate(list(self.children),1):
                a,b,k,u=(W.FILETIME()for _ in range(4))
                if self.kernel.GetProcessTimes(W.HANDLE(int(p._handle)),ctypes.byref(a),ctypes.byref(b),ctypes.byref(k),ctypes.byref(u)):
                    seconds=sum((x.dwHighDateTime<<32)|x.dwLowDateTime for x in(k,u))/1e7
                    self.rows.append(dict(t=time.monotonic()-self.started,slot=slot,pid=p.pid,cpu_seconds=seconds))
            self.stop.wait(.5)
    def finish(self,path):
        self.stop.set();self.thread.join(timeout=3)
        path.write_text(json.dumps(self.rows,indent=2))

def instrument_lua(source):
    # Applied only to a disposable profile. Prefix remains compatible with activation.
    source=source.replace('local rows, built, hovered = {}, 0, nil', r'''local rows, built, hovered = {}, 0, nil
local perf={updates=0,events=0,refreshes=0,ms=0,max=0,first=0,decode_ms=0,decodes=0,pin_ms=0,pin_frames=0,animation_ms=0}
function ASXStartupPerformance()return perf end
local perfClock=debugprofilestop
assert(type(perfClock)=="function", "Native profiling clock unavailable")''',1)
    source=source.replace('local nextModel=decode(wire)', 'local stamp=perfClock();local nextModel=decode(wire);perf.decode_ms=perf.decode_ms+perfClock()-stamp;perf.decodes=perf.decodes+1',1)
    source=source.replace('function() dirty = true end','function() perf.events=perf.events+1;dirty = true end',1)
    source=source.replace('observer:SetScript("OnUpdate", function()', 'observer:SetScript("OnUpdate", function()\n    perf.updates=perf.updates+1',1)
    source=source.replace('    refresh()','    local stamp=perfClock();refresh();local ms=perfClock()-stamp;perf.refreshes=perf.refreshes+1;perf.ms=perf.ms+ms;perf.max=math.max(perf.max,ms);if perf.refreshes==1 then perf.first=ms end',1)
    source=source.replace('    layoutEventPins()', '    local pinStamp=perfClock();layoutEventPins();perf.pin_ms=perf.pin_ms+perfClock()-pinStamp;perf.pin_frames=perf.pin_frames+1',1)
    source=source.replace('    updateRitualBar(now)', '    local animationStamp=perfClock();updateRitualBar(now);perf.animation_ms=perf.animation_ms+perfClock()-animationStamp',1)
    return source

OVERLAY = r'''local f=CreateFrame("Frame","ASXStartupPerformancePanel",UIParent)
local label=f:CreateFontString(nil,"OVERLAY","GameFontNormalSmall")
label:SetPoint("TOPRIGHT",UIParent,"TOPRIGHT",-12,-15)
label:SetWidth(350);label:SetHeight(120);label:SetJustifyH("LEFT")
local last=-100
f:SetScript("OnUpdate",function()
 local now=GetTime();if now-last<2 then return end;last=now
 local p=ASXStartupPerformance and ASXStartupPerformance()
 if not p then label:SetText("STARTUP PROFILE MISSING");return end
 label:SetText(string.format("ASX PROFILE t=%.1f fps=%.1f\nupdates=%d events=%d refresh=%d\nrefresh ms=%.3f max=%.3f first=%.3f\ndecode ms=%.3f count=%d",now,GetFramerate(),p.updates,p.events,p.refreshes,p.ms,p.max,p.first,p.decode_ms,p.decodes))
end)
'''
