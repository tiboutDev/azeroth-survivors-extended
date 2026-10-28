"""Conservative pointer-origin analysis over x64 basic blocks (static porting evidence).

Only copies, fixed LEAs, stack spills and fixed pointer increments preserve an origin.
Loads from heap fields and volatile registers across calls never inherit Game ownership.
At a control-flow join, an origin survives only when every reachable predecessor agrees.
These observations identify candidates; they do not approve calls or native patches.
"""
from collections import deque
import capstone as cs

GPRS=['rax','rcx','rdx','rbx','rsp','rbp','rsi','rdi']+[f'r{i}' for i in range(8,16)]
ALIASES={}
for index,name in enumerate(GPRS):
    names=[name]
    if index<8:
        names += [('e'+name[1:]),name[1:],['al','cl','dl','bl','spl','bpl','sil','dil'][index]]
    else: names += [name+'d',name+'w',name+'b']
    for alias in names: ALIASES[alias]=index
for index,alias in enumerate(('ah','ch','dh','bh')): ALIASES[alias]=index
VOLATILE=(0,1,2,8,9,10,11)

def reg(md,operand):
    return ALIASES.get(md.reg_name(operand))

def memory(md,operand,state):
    if operand.type!=cs.x86.X86_OP_MEM or operand.mem.index: return None
    index=reg(md,operand.mem.base)
    if index is None: return None
    base=state.get(('r',index))
    return (base[0],base[1]+operand.mem.disp) if base else None

def transfer(md,ins,state):
    state=dict(state); ops=ins.operands
    before=dict(state)
    source=None; destination=None
    if len(ops)>=2 and ins.mnemonic in ('mov','lea'):
        dst,src=ops[:2]
        if ins.mnemonic=='lea':
            source=memory(md,src,before)
            if source==('server',16): source=('game',0)
        elif src.type==cs.x86.X86_OP_REG and src.size==8:
            source=before.get(('r',reg(md,src.reg)))
        elif src.type==cs.x86.X86_OP_MEM and src.size==8:
            key=memory(md,src,before)
            if key and key[0]=='stack': source=before.get(key)
        if dst.type==cs.x86.X86_OP_REG and dst.size==8 and reg(md,dst.reg) is not None: destination=('r',reg(md,dst.reg))
        elif dst.type==cs.x86.X86_OP_MEM:
            key=memory(md,dst,before)
            if key and key[0]=='stack':
                # Kill overlapping spills, including partial writes to a pointer slot.
                for stored in list(state):
                    if stored[0]=='stack' and stored[1]<key[1]+dst.size and key[1]<stored[1]+8: state.pop(stored)
                if dst.size==8: destination=key
    if ins.mnemonic in ('add','sub') and len(ops)==2 and ops[0].type==cs.x86.X86_OP_REG and ops[0].size==8 and ops[1].type==cs.x86.X86_OP_IMM:
        destination=('r',reg(md,ops[0].reg)); value=before.get(destination)
        if value: source=(value[0],value[1]+ops[1].imm*(1 if ins.mnemonic=='add' else -1))
    _,writes=ins.regs_access()
    for register in writes:
        index=reg(md,register)
        if index is not None: state.pop(('r',index),None)
    if ins.mnemonic=='push':
        sp=before.get(('r',4))
        if sp and sp[0]=='stack':
            state[('r',4)]=('stack',sp[1]-8)
            if ops[0].type==cs.x86.X86_OP_REG:
                value=before.get(('r',reg(md,ops[0].reg)))
                if value: state[('stack',sp[1]-8)]=value
    elif ins.mnemonic=='pop':
        sp=before.get(('r',4))
        if sp and sp[0]=='stack':
            state[('r',4)]=('stack',sp[1]+8)
            if ops[0].type==cs.x86.X86_OP_REG:
                value=before.get(('stack',sp[1]))
                if value: state[('r',reg(md,ops[0].reg))]=value
    if destination and source: state[destination]=source
    if ins.group(cs.CS_GRP_CALL):
        for index in VOLATILE: state.pop(('r',index),None)
        # A passed pointer to one of our spill slots lets an unknown callee overwrite it.
        for index in (1,2,8,9):
            value=before.get(('r',index))
            if value and value[0]=='stack': state.pop(value,None)
    return state

def analyze(md,instructions,root='game',extra_edges=None,initial=None,edge_overrides=None):
    if not instructions: return {},{}
    by_address={ins.address:i for i,ins in enumerate(instructions)}
    successors=[]
    for index,ins in enumerate(instructions):
        following=[index+1] if index+1<len(instructions) else []
        if ins.group(cs.CS_GRP_RET) or ins.mnemonic in ('ud2','int3'): following=[]
        elif ins.group(cs.CS_GRP_JUMP):
            target=by_address.get(ins.operands[0].imm) if ins.operands and ins.operands[0].type==cs.x86.X86_OP_IMM else None
            targets=[target] if target is not None else [by_address[a] for a in (extra_edges or {}).get(ins.address,[]) if a in by_address]
            following=targets + ([] if ins.mnemonic=='jmp' else following)
        if ins.address in (edge_overrides or {}):
            destinations=edge_overrides[ins.address]
            if not ins.group(cs.CS_GRP_JUMP) or ins.mnemonic=='jmp' or any(a not in by_address for a in destinations):
                raise ValueError('Invalid conditional-branch analysis override')
            if any(by_address[a] not in following for a in destinations):
                raise ValueError('Analysis override must only remove existing successors')
            following=[by_address[a] for a in destinations]
        successors.append(following)
    incoming={0:{('r',1):(root,0),('r',4):('stack',0),**(initial or {})}}
    work=deque([0]); queued={0}
    while work:
        index=work.popleft();queued.discard(index)
        outgoing=transfer(md,instructions[index],incoming[index])
        for target in successors[index]:
            old=incoming.get(target)
            merged=outgoing if old is None else {k:v for k,v in old.items() if outgoing.get(k)==v}
            if old!=merged:
                incoming[target]=dict(merged)
                if target not in queued: work.append(target);queued.add(target)
    references={}
    for index,state in incoming.items():
        ins=instructions[index]
        for op_index,operand in enumerate(ins.operands):
            origin=memory(md,operand,state)
            if origin and origin[0] in ('game','server',root):
                references[ins.address,op_index]=dict(domain=origin[0],offset=origin[1],size=operand.size)
    return references,{instructions[i].address:s for i,s in incoming.items()}
