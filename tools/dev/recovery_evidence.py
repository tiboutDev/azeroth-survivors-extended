"""Fault injection only in the bounded private two-player harness."""
import copy,re,threading
from collections import Counter
class Faults:
    def __init__(self,mode,timeout_end=52):
        if mode not in ('multi','timeout','reconnect','all'):raise ValueError('Unknown recovery trial')
        if not 52<=timeout_end<=70:raise ValueError("Bounded timeout cut required")
        self.timeout_end=timeout_end
        self.mode=mode;self.lock=threading.Lock();self.counts=Counter();self.old=None;self.replayed=False
    def send(self,number,message,elapsed):
        with self.lock:
            if message.get('kind')!='state':return [message]
            if self.mode=='reconnect':return [message]
            if self.mode=='all':cut=(55<=elapsed<63 and number==1) or 95<=elapsed<132
            elif self.mode=='timeout':cut=15<=elapsed<self.timeout_end
            else:cut=15<=elapsed<23 or (35<=elapsed<43 and number==1)
            if cut:
                self.counts['dropped_'+str(number)]+=1
                if self.old is None and number==1:self.old=copy.deepcopy(message)
                return []
            out=copy.deepcopy(message)
            if ((self.mode=='multi' and 55<=elapsed<63) or (self.mode=='all' and 75<=elapsed<83)) and number==1 and isinstance(out.get('enemy_update'),dict):
                out['enemy_update']['base']=900000000;self.counts['invalid_baseline']+=1
            rows=[out]
            if self.old is not None and not self.replayed and number==1 and elapsed>=24:
                rows.insert(0,self.old);self.replayed=True;self.counts['old_generation_replayed']+=1
            return rows

def assess(logs,mode):
    result={'mode':mode,'players':{},'passed':True}
    for number,text in logs.items():
        starts=len(re.findall(r'event native_start player=',text));begins=re.findall(r'event recovery_started epoch=(\d+)',text)
        completed=re.findall(r'event recovery_complete epoch=(\d+)',text)
        applied=re.findall(r'event delivery_event_applied peer=(\d+) sequence=(\d+)',text)
        faults=[line for line in text.splitlines() if any(k in line for k in ('[FAILED]','ACCESS_VIOLATION','peer_state_rejected'))]
        row=dict(starts=starts,recoveries_started=len(begins),completed_generations=sorted(set(map(int,completed))),
                 timeout='event recovery_timeout' in text,duplicate_events=len(applied)-len(set(applied)),faults=faults)
        row['passed']=starts==1 and len(set(completed))>=(5 if mode=='all' else 3 if mode=='multi' else 2 if mode=='reconnect' else 1) and not row['duplicate_events'] and not faults and (mode not in ('timeout','all') or row['timeout'])
        result['players'][number]=row;result['passed']&=row['passed']
    return result
