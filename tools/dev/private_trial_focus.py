"""Window messages for owned clients on an existing private trial desktop.

Target windows are selected by the supplied child process IDs.
"""
import ctypes,os
from ctypes import wintypes as W

def _post_owned_windows(process_ids, messages):
 if os.name!="nt":raise RuntimeError("Private window focus is Windows-only")
 targets=set(process_ids)
 if not targets or len(targets)>2 or any(type(pid)is not int or pid<=0 for pid in targets):raise ValueError("One or two owned process IDs required")
 user=ctypes.WinDLL("user32",use_last_error=True);kernel=ctypes.WinDLL("kernel32",use_last_error=True)
 user.GetThreadDesktop.argtypes=[W.DWORD];user.GetThreadDesktop.restype=W.HANDLE
 user.GetUserObjectInformationW.argtypes=[W.HANDLE,ctypes.c_int,ctypes.c_void_p,W.DWORD,ctypes.POINTER(W.DWORD)]
 desktop=user.GetThreadDesktop(kernel.GetCurrentThreadId());name=ctypes.create_unicode_buffer(256);needed=W.DWORD()
 ok=user.GetUserObjectInformationW(desktop,2,name,ctypes.sizeof(name),ctypes.byref(needed))
 if not ok or not name.value.lower().startswith("azcoopdev_"):raise RuntimeError(f"Focus notifications refused: desktop query={bool(ok)} name={name.value!r} error={ctypes.get_last_error()}")
 user.GetWindowThreadProcessId.argtypes=[W.HWND,ctypes.POINTER(W.DWORD)]
 user.IsWindowVisible.argtypes=[W.HWND];user.PostMessageW.argtypes=[W.HWND,W.UINT,W.WPARAM,W.LPARAM]
 callback_type=ctypes.WINFUNCTYPE(W.BOOL,W.HWND,W.LPARAM);seen=set()
 @callback_type
 def visit(hwnd,_):
  pid=W.DWORD();user.GetWindowThreadProcessId(hwnd,ctypes.byref(pid))
  if pid.value in targets and user.IsWindowVisible(hwnd):
   # Each participant represents a separate real player with a focused client.
   # Notify only the two already-owned game windows; never choose a global target.
   if all(user.PostMessageW(hwnd,msg,key,info) for msg,key,info in messages):seen.add(pid.value)
  return True
 user.EnumWindows.argtypes=[callback_type,W.LPARAM]
 if not user.EnumWindows(visit,0):raise ctypes.WinError(ctypes.get_last_error())
 return len(seen)

def notify_owned_windows(process_ids):
 return _post_owned_windows(process_ids, [(6,1,0),(7,0,0)])

def step_owned_windows(process_ids, tick):
 # Hold W or S for three samples and alternate directions. Only these HWNDs
 # receive the key. The clock/pose assessor independently verifies the outcome.
 messages=[(6,1,0),(7,0,0)]
 phase=tick%4
 if phase in (0,3):
  key,scan=(0x57,0x11) if tick%8<4 else (0x53,0x1f)
  release=phase==3
  messages.append((0x101 if release else 0x100,key,
                   (scan<<16)|1|(0xc0000000 if release else 0)))
 return _post_owned_windows(process_ids,messages)
