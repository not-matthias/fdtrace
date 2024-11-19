# exit afterwards

```
Syscall { ts: 13504613183083, pid: 57923, tid: 57923, raw: OpenAt { dirfd: -1059204248, path: "", flags: -1059204248 } }
Syscall { ts: 13504613189439, pid: 57923, tid: 57923, raw: OpenAtExit { fd: 4 } }
Syscall { ts: 13504613189648, pid: 57923, tid: 57923, raw: OpenAtExit { fd: -1059224344 } }
Syscall { ts: 13504613190631, pid: 57923, tid: 57923, raw: Read { fd: 4, count: 832 } }
Syscall { ts: 13504613191103, pid: 57923, tid: 57923, raw: Read { fd: -1059200480, count: 18446631905284448800 } }
Syscall { ts: 13504613194409, pid: 57923, tid: 57923, raw: ReadExit { read: 832 } }
Syscall { ts: 13504613195002, pid: 57923, tid: 57923, raw: ReadExit { read: -1059197968 } }
Syscall { ts: 13504613244545, pid: 57923, tid: 57923, raw: Close { fd: 4 } }
Syscall { ts: 13504613245007, pid: 57923, tid: 57923, raw: Close { fd: -1059218064 } }
Syscall { ts: 13504613246541, pid: 57923, tid: 57923, raw: CloseExit { ret: 0 } }
Syscall { ts: 13504613247104, pid: 57923, tid: 57923, raw: CloseExit { ret: -1059196712 } }

```



[17:11] not-matthias:fdtrace (main)> sudo -E cargo rr '/nix/store/5jbs3aj3m3zsl6fc4w7sfsna57zjqf2y-user-environment/bin/rg telegram /home/not-matthias/Documents' --debug
