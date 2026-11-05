"""Fail closed on unenumerated selected-field uses, including indirect Rust paths.

This checks source/descriptor coverage, not native ABI or gameplay behavior.
Pinned-image execution and expected-value integration tests remain separate gates.
"""
import json
import re
from pathlib import Path
try:
 from .binding_registry import ROOT, load_binding, parse_literal
 from .compatibility import inventory
except ImportError:
 from binding_registry import ROOT, load_binding, parse_literal
 from compatibility import inventory

TABLES = dict(game_field='GAME_RESOLVED_FIELDS', actor_field='ACTOR_RESOLVED_FIELDS',
              field117='FIELD117_RESOLVED_FIELDS', field1201='FIELD1201_RESOLVED_FIELDS')

def literal_keys(expression, values):
 expression=re.sub(r'\s+as\s+u(?:64|size)$','',expression.strip())
 expression=re.sub(r'(u64|usize)$','',expression)
 if re.fullmatch(r'0x[0-9a-fA-F]+|[0-9]+',expression):return [int(expression,0)]
 match=re.fullmatch(r'(?:v|crate::current|layout)::([A-Z_0-9]+)(?:\[(\d+)\])?',expression)
 if not match:return None
 value=values.get(match[1])
 if match[2] is not None:
  if not isinstance(value,(list,tuple)) or int(match[2])>=len(value):return None
  value=value[int(match[2])]
 return [value] if type(value)is int else None

def signature(edge):return edge['target'],tuple(edge['arguments'])

def audit(contracts, values, reviews):
 errors=[];lookups=[];used=set();review_by_owner={r['owner']:r for r in reviews}
 if len(review_by_owner)!=len(reviews):raise ValueError('Duplicate field-review owner')
 for contract in contracts:
  owner=contract['id'];review=review_by_owner.get(owner)
  if not contract.get('access_inventory_complete'):
   errors.append(dict(owner=owner,reason='Missing AST access inventory'));continue
  for edge in contract['native_accesses']:
   if edge['kind']!='field':continue
   cases=None;status='resolved';checked=None
   if review:
    for candidate in review['uses']:
     if signature(candidate)==signature(edge):checked=candidate;break
   if checked:
    used.add((owner,signature(checked)))
    if review['source_sha256']!=contract['adapter_source_sha256']:
     errors.append(dict(owner=owner,line=edge['line'],reason='Indirect field review is stale'));continue
    status=checked.get('status','resolved')
    if status=='retired':
     guard=checked['guard']
     if values.get(guard['constant'])!=guard['value']:
      errors.append(dict(owner=owner,line=edge['line'],reason='Retired field guard changed'));continue
    if status in ('retired','forwarder'):
     lookups.append(dict(owner=owner,line=edge['line'],status=status));continue
    if status!='resolved':raise ValueError('Unknown field-review status')
    cases=checked['cases']
   else:
    keys=literal_keys(edge['arguments'][-1],values)
    if keys is not None:
     if edge['target']=='row_field':
      host=edge['arguments'][-2]
      tables=['GAME_RESOLVED_FIELDS']if host=='true'else['ACTOR_RESOLVED_FIELDS']if host=='false'else['GAME_RESOLVED_FIELDS','ACTOR_RESOLVED_FIELDS']
     else:tables=[TABLES[edge['target']]]
     cases=[dict(namespace=table,key=key)for table in tables for key in keys]
   if not cases:
    errors.append(dict(owner=owner,line=edge['line'],target=edge['target'],arguments=edge['arguments'],reason='Unresolved field expression'));continue
   for case in cases:
    table,key=case['namespace'],case['key']
    if table not in ('GAME_RESOLVED_FIELDS','ACTOR_RESOLVED_FIELDS','FIELD117_RESOLVED_FIELDS','FIELD1201_RESOLVED_FIELDS'):raise ValueError('Unknown field namespace')
    hits=[native+key-start for start,end,native in values[table]if start<=key<=end]
    if len(hits)!=1:errors.append(dict(owner=owner,line=edge['line'],namespace=table,key=key,reason='Missing or ambiguous selected field'))
    lookups.append(dict(owner=owner,line=edge['line'],namespace=table,key=key,native=hits,status=status))
 for review in reviews:
  for use in review['uses']:
   if (review['owner'],signature(use))not in used:errors.append(dict(owner=review['owner'],reason='Unused indirect field review',target=use['target'],arguments=use['arguments']))
 return dict(schema=1,passed=not errors,errors=errors,lookups=lookups,
             scope='AST field-use coverage only; native schema, extents, ownership, ABI and expected outputs require their independent checks.')

def current_report():
 version,client,binding=load_binding();data=json.loads((ROOT/'compat/field-uses.json').read_text())
 if data.get('schema')!=1 or data.get('version')!=version or data.get('native_sha256')!=client['sha256']:raise ValueError('Field-use review does not match the selected client')
 values={c['name']:parse_literal(c['value'])for c in binding['constants']}
 report=audit(inventory()['contracts'],values,data['reviews'])
 return dict(version=version,native_sha256=client['sha256'],**report)

if __name__=='__main__':
 report=current_report();print(json.dumps(report,indent=2));raise SystemExit(0 if report['passed']else 1)
