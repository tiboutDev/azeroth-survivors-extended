"""Strict guardian offer and world-UI evidence; rendering review stays separate."""
import json,re,struct,math

def guardian(logs):
 kills=re.findall(r'([\d.]+)s coverage native guardian killed: Ok\(Some\((\d+)\)\)',logs[0])
 offers=[]
 for text in logs:
  rows=[]
  for line in text.splitlines():
   if 'coverage native choices: 'not in line:continue
   try:row=json.loads(line.split('coverage native choices: ',1)[1])
   except ValueError:continue
   if row.get('why')=='guardian':rows.append(row)
  offers.append(rows)
 ids=[[(row.get('offer'),len(row.get('choices',[])))for row in rows]for rows in offers]
 checks=dict(one_controlled_native_kill=len(kills)==1,two_actor_streams=len(logs)==2,guardian_offers_both=all(rows for rows in offers),same_offers=len(ids)==2 and ids[0]==ids[1],three_options=all(n==3 for rows in ids for _,n in rows),no_duplicate_offer=all(len(rows)==len({i for i,_ in rows})for rows in ids))
 # A positive kill alone must not pass: require a subsequent actual guardian offer.
 after=False
 if len(kills)==1:
  for line in logs[0].splitlines():
   time=re.search(r'([\d.]+)s coverage native choices: (.*)',line)
   if time and float(kills[0][0])<=float(time[1])<=float(kills[0][0])+10:
    try:after=after or json.loads(time[2]).get('why')=='guardian'
    except ValueError:pass
 checks['post_kill_offer_observed']=after
 return dict(checks=checks,passed=all(checks.values()),offers=ids,kills=kills,limit='Verifies unique three-option native item offers; does not infer a separate fixed spell grant from XP level-ups.')

def world(logs):
 checks={}
 for slot,text in enumerate(logs):
  layouts=re.findall(r'Native plate layout: index=\d+ original=\[([^]]+)\] scaled=\[([^]]+)\]',text)
  sizes=[]
  for original,scaled in layouts:
   a=list(struct.unpack('<13f',bytes(int(v)for v in original.split(','))));b=[float(v)for v in scaled.split(',')]
   sizes.append(dict(original=a,scaled=b))
  checks[f'plate_layout_{slot}']=bool(sizes)
  checks[f'plate_dimensions_{slot}']=bool(sizes)and all(len(row['scaled'])==13 and all(math.isclose(value,(row['original'][4]+2*row['original'][2])*.7 if i==0 else row['original'][i] if i==10 else row['original'][i]*.7,rel_tol=1e-5,abs_tol=1e-5)for i,value in enumerate(row['scaled']))for row in sizes)
  for periodic in('false','true'):
   checks[f'heal_draw_{slot}_{periodic}']=bool(re.search(r'native heal text: periodic='+periodic+r' event=HealText \{[^\n]+\} result=Ok\(true\)',text))
 return dict(checks=checks,passed=all(checks.values()),limit='Native layout and message-queue observations; pixel alignment and animations require a separate visual review.')
