"""Portable-check CPU budget and lightweight live-test capture defaults."""
# Live game clients have no imposed CPU/priority limit and no FPS override.
def affinity_mask(available,percent=20):
    if not isinstance(available,int)or available<=0 or not 5<=percent<=50:raise ValueError("Invalid test CPU budget")
    bits=[1<<n for n in range(available.bit_length())if available&(1<<n)]
    count=max(1,len(bits)*percent//100)
    return sum(bits[-count:])
def environment(visual=False):
    result={"WOW_ROGUE_SHOTS_EVERY":"15"}
    return result

def portable_limits(percent=20):
    """Limit only this check runner and descendants, never the user's shell."""
    import os
    if not hasattr(os,"sched_getaffinity"):return {"supported":False}
    available=os.sched_getaffinity(0)
    mask=affinity_mask(sum(1<<n for n in available),percent)
    selected={n for n in available if mask&(1<<n)}
    os.sched_setaffinity(0,selected)
    os.setpriority(os.PRIO_PROCESS,0,max(10,os.getpriority(os.PRIO_PROCESS,0)))
    return {"supported":True,"available_cpus":len(available),"allowed_cpus":sorted(selected),"nice":os.getpriority(os.PRIO_PROCESS,0)}
