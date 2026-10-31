"""Create an owned trial child on an actual Windows private desktop.

CPython's STARTUPINFO wrapper omits lpDesktop. Use the Windows structure and
inherit only the explicit log/stdin handles, with a gate before native clients.
"""
import ctypes,os,subprocess
from ctypes import wintypes as W

def current_private_desktop(expected=None):
 if os.name!="nt":raise RuntimeError("Windows only")
 user=ctypes.WinDLL("user32",use_last_error=True);kernel=ctypes.WinDLL("kernel32",use_last_error=True)
 user.GetThreadDesktop.argtypes=[W.DWORD];user.GetThreadDesktop.restype=W.HANDLE
 user.GetUserObjectInformationW.argtypes=[W.HANDLE,ctypes.c_int,ctypes.c_void_p,W.DWORD,ctypes.POINTER(W.DWORD)]
 desktop=user.GetThreadDesktop(kernel.GetCurrentThreadId());name=ctypes.create_unicode_buffer(256);needed=W.DWORD()
 if not user.GetUserObjectInformationW(desktop,2,name,ctypes.sizeof(name),ctypes.byref(needed)):raise ctypes.WinError(ctypes.get_last_error())
 if not name.value.lower().startswith("azcoopdev_")or(expected is not None and name.value!=expected):raise RuntimeError("Trial child is not on its owned private desktop")
 return name.value

def spawn(command,cwd,desktop,stream):
 if os.name!="nt"or not desktop.lower().startswith("azcoopdev_"):raise ValueError("An explicit owned private desktop is required")
 import msvcrt
 kernel=ctypes.WinDLL("kernel32",use_last_error=True)
 class STARTUPINFO(ctypes.Structure):
  _fields_=[("cb",W.DWORD),("lpReserved",W.LPWSTR),("lpDesktop",W.LPWSTR),("lpTitle",W.LPWSTR),("dwX",W.DWORD),("dwY",W.DWORD),("dwXSize",W.DWORD),("dwYSize",W.DWORD),("dwXCountChars",W.DWORD),("dwYCountChars",W.DWORD),("dwFillAttribute",W.DWORD),("dwFlags",W.DWORD),("wShowWindow",W.WORD),("cbReserved2",W.WORD),("lpReserved2",ctypes.POINTER(ctypes.c_byte)),("hStdInput",W.HANDLE),("hStdOutput",W.HANDLE),("hStdError",W.HANDLE)]
 class STARTUPINFOEX(ctypes.Structure):_fields_=[("StartupInfo",STARTUPINFO),("lpAttributeList",ctypes.c_void_p)]
 class PROCESS_INFORMATION(ctypes.Structure):_fields_=[("hProcess",W.HANDLE),("hThread",W.HANDLE),("dwProcessId",W.DWORD),("dwThreadId",W.DWORD)]
 kernel.InitializeProcThreadAttributeList.argtypes=[ctypes.c_void_p,W.DWORD,W.DWORD,ctypes.POINTER(ctypes.c_size_t)]
 kernel.UpdateProcThreadAttribute.argtypes=[ctypes.c_void_p,W.DWORD,ctypes.c_size_t,ctypes.c_void_p,ctypes.c_size_t,ctypes.c_void_p,ctypes.c_void_p]
 kernel.DeleteProcThreadAttributeList.argtypes=[ctypes.c_void_p]
 kernel.CreateProcessW.argtypes=[W.LPCWSTR,W.LPWSTR,ctypes.c_void_p,ctypes.c_void_p,W.BOOL,W.DWORD,ctypes.c_void_p,W.LPCWSTR,ctypes.POINTER(STARTUPINFOEX),ctypes.POINTER(PROCESS_INFORMATION)]
 kernel.CloseHandle.argtypes=[W.HANDLE]
 null=open(os.devnull,"rb");stdin=msvcrt.get_osfhandle(null.fileno());stdout=msvcrt.get_osfhandle(stream.fileno());handles=(W.HANDLE*2)(stdin,stdout)
 originals={h:os.get_handle_inheritable(h)for h in [stdin,stdout]};size=ctypes.c_size_t();kernel.InitializeProcThreadAttributeList(None,1,0,ctypes.byref(size));attributes=ctypes.create_string_buffer(size.value)
 initialized=False
 try:
  for h in originals:os.set_handle_inheritable(h,True)
  if not kernel.InitializeProcThreadAttributeList(attributes,1,0,ctypes.byref(size)):raise ctypes.WinError(ctypes.get_last_error())
  initialized=True
  if not kernel.UpdateProcThreadAttribute(attributes,0,0x20002,handles,ctypes.sizeof(handles),None,None):raise ctypes.WinError(ctypes.get_last_error())
  info=STARTUPINFOEX();info.StartupInfo.cb=ctypes.sizeof(info);info.StartupInfo.lpDesktop=desktop;info.StartupInfo.dwFlags=0x100;info.StartupInfo.hStdInput=stdin;info.StartupInfo.hStdOutput=stdout;info.StartupInfo.hStdError=stdout;info.lpAttributeList=ctypes.cast(attributes,ctypes.c_void_p)
  process=PROCESS_INFORMATION();line=ctypes.create_unicode_buffer(subprocess.list2cmdline([str(v)for v in command]))
  if not kernel.CreateProcessW(str(command[0]),line,None,None,True,0x08000000|0x00080000|0x20,None,str(cwd),ctypes.byref(info),ctypes.byref(process)):raise ctypes.WinError(ctypes.get_last_error())
  kernel.CloseHandle(process.hThread)
 finally:
  if initialized:kernel.DeleteProcThreadAttributeList(attributes)
  for h,old in originals.items():os.set_handle_inheritable(h,old)
  null.close()
 class Child:
  def __init__(self):self.pid=process.dwProcessId;self._handle=process.hProcess;self.returncode=None
  def poll(self):
   code=W.DWORD();kernel.GetExitCodeProcess.argtypes=[W.HANDLE,ctypes.POINTER(W.DWORD)]
   if not kernel.GetExitCodeProcess(self._handle,ctypes.byref(code)):raise ctypes.WinError(ctypes.get_last_error())
   self.returncode=None if code.value==259 else code.value;return self.returncode
  def wait(self,timeout=None):
   kernel.WaitForSingleObject.argtypes=[W.HANDLE,W.DWORD]
   result=kernel.WaitForSingleObject(self._handle,0xffffffff if timeout is None else min(0xfffffffe,int(timeout*1000)))
   if result==258:raise subprocess.TimeoutExpired(command,timeout)
   if result!=0:raise ctypes.WinError(ctypes.get_last_error())
   return self.poll()
  def terminate(self):
   kernel.TerminateProcess.argtypes=[W.HANDLE,W.UINT]
   if not kernel.TerminateProcess(self._handle,1)and self.poll()is None:raise ctypes.WinError(ctypes.get_last_error())
  def close(self):
   if self._handle:kernel.CloseHandle(self._handle);self._handle=None
 return Child()
