"""Explicit focus notifications for owned windows on an existing private desktop.

No desktop switching, keyboard/mouse input, process discovery or game-state writes.
"""
import ctypes,os
from ctypes import wintypes as W

def notify_owned_windows(process_ids):
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
   if user.PostMessageW(hwnd,6,1,0)and user.PostMessageW(hwnd,7,0,0):seen.add(pid.value)
  return True
 user.EnumWindows.argtypes=[callback_type,W.LPARAM]
 if not user.EnumWindows(visit,0):raise ctypes.WinError(ctypes.get_last_error())
 return len(seen)
