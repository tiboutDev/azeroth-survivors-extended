"""Owned-window close and native class-XP evidence for disposable 0.120 profiles."""
from pathlib import Path

def class_xp(executable,save,class_id):
 import sys
 sys.path.insert(0,str(Path(__file__).resolve().parents[2]/"tools/dev"))
 import native_profile_codec as azs_rse1
 keys=azs_rse1.key_candidates_from_bytes(Path(executable).read_bytes())
 if len(keys)!=1:raise ValueError("Ambiguous reviewed save codec")
 raw=Path(save).read_bytes()
 if len(raw)>4*1024*1024:raise ValueError("Disposable save extent")
 text=azs_rse1.open_save(raw,keys[0])[1]
 rows=[line.split("|") for line in text.splitlines()if line.startswith("talent|")]
 values=[int(row[2])for row in rows if len(row)>=3 and row[1]==str(class_id)]
 if len(values)!=1:raise ValueError("Missing unique native class-XP row")
 return values[0]

def close_owned_window(pid,executable):
 import ctypes as c,ctypes.wintypes as w
 k=c.WinDLL("kernel32",use_last_error=True);u=c.WinDLL("user32",use_last_error=True)
 k.OpenProcess.argtypes=[w.DWORD,w.BOOL,w.DWORD];k.OpenProcess.restype=w.HANDLE;k.CloseHandle.argtypes=[w.HANDLE]
 k.QueryFullProcessImageNameW.argtypes=[w.HANDLE,w.DWORD,w.LPWSTR,c.POINTER(w.DWORD)]
 h=k.OpenProcess(0x1000,False,pid)
 if not h:raise c.WinError(c.get_last_error())
 try:
  name=c.create_unicode_buffer(32768);size=w.DWORD(len(name))
  if not k.QueryFullProcessImageNameW(h,0,name,c.byref(size)):raise c.WinError(c.get_last_error())
  if Path(name.value).resolve()!=Path(executable).resolve():raise ValueError("Process is not the owned isolated client")
 finally:k.CloseHandle(h)
 cb=c.WINFUNCTYPE(w.BOOL,w.HWND,w.LPARAM);windows=[]
 u.GetWindowThreadProcessId.argtypes=[w.HWND,c.POINTER(w.DWORD)]
 @cb
 def each(hwnd,param):
  owner=w.DWORD();u.GetWindowThreadProcessId(hwnd,c.byref(owner))
  if owner.value==pid:windows.append(hwnd)
  return True
 u.EnumWindows.argtypes=[cb,w.LPARAM];u.EnumWindows(each,0)
 if not windows:raise ValueError("Owned native window not found")
 u.PostMessageW.argtypes=[w.HWND,w.UINT,w.WPARAM,w.LPARAM]
 for hwnd in windows:
  if not u.PostMessageW(hwnd,0x0010,0,0):raise c.WinError(c.get_last_error())
 return len(windows)

def boss_progress(executable,save):
 """Read native Boss Hunter counter/achievement IDs from a disposable profile."""
 import sys
 sys.path.insert(0,str(Path(__file__).resolve().parents[2]/"tools/dev"))
 import native_profile_codec as azs_rse1
 keys=azs_rse1.key_candidates_from_bytes(Path(executable).read_bytes())
 if len(keys)!=1:raise ValueError("Ambiguous reviewed save codec")
 raw=Path(save).read_bytes()
 if len(raw)>4*1024*1024:raise ValueError("Disposable save extent")
 text=azs_rse1.open_save(raw,keys[0])[1]
 stats=[line[5:]for line in text.splitlines()if line.startswith("stat|")]
 if len(stats)>1:raise ValueError("Duplicate native statistics row")
 counters=dict(value.split("=",1)for value in stats[0].split(",")) if stats else {}
 bosses=int(counters.get("bosses",0))
 if bosses<0:raise ValueError("Invalid boss counter")
 achievements=[line[4:]for line in text.splitlines()if line.startswith("ach|")]
 return {"bosses":bosses,"achievements":achievements[0].split(",") if achievements else []}
