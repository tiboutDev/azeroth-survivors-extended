#!/usr/bin/env python3
"""Calculate common native item bonuses from the saved reference, offline."""
import argparse
import json
from pathlib import Path
from native_balance_math import RARITIES, amulet_effects, jewelry_bonuses, jewelry_history


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--catalog',type=Path,default=(Path(__file__).with_name('native-balance.json') if Path(__file__).with_name('native-balance.json').exists() else Path(__file__).resolve().parents[2]/'docs/native-balance/native-balance.json'))
    parser.add_argument('--item',required=True,help='Native item key')
    group=parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--history',help='Comma-separated acquisition rarities, e.g. Common,Rare,Epic')
    group.add_argument('--rank',type=int,help='Amulet rank')
    args=parser.parse_args()
    data=json.loads(args.catalog.read_text())
    try:
        if args.history:
            row=next(r for r in data['jewelry'] if r['key']==args.item)
            names={name.lower():i for i,name in enumerate(RARITIES)}
            names['pvp']=5
            history=[names[name.strip().lower()] for name in args.history.split(',')]
            if not 1<=len(history)<=10000:raise ValueError('History length must be 1–10000')
            value=jewelry_history(row,history)
            result={'item':row['name'],'rank':len(history),'displayed_rarity':(RARITIES+('PvP',))[max(history)],
                    'stored_value':value,'contributions':jewelry_bonuses(row,value),
                    'scope':'Arithmetic trajectory; offer eligibility and consumed lives are separate state'}
        else:
            row=next(r for r in data['amulets'] if r['key']==args.item)
            result={'item':row['name'],'rank':args.rank,'rarity':RARITIES[row['rarity']],
                    'contributions':[{'effect':label,'value':value,'unit':unit} for label,value,unit in amulet_effects(row,args.rank)]}
    except (ValueError,KeyError,StopIteration) as exc:
        parser.error(f'Invalid item, rarity, rank or history: {exc}')
    print(json.dumps(result,indent=2))


if __name__=='__main__':main()
