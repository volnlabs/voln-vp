# BCM2712 mailbox register stub for Renode's Python.PythonPeripheral.
#
# Renode executes this file once per transaction with `request` and `self`
# injected into a persistent IronPython scope. This models the mailbox register
# handshake only. Firmware property-buffer interpretation is deliberately not
# claimed because current axiomOS does not issue those requests.

READ = 0x00
READ_STATUS = 0x18
WRITE = 0x20
WRITE_STATUS = 0x38
EMPTY = 1 << 30

if request.IsInit:
    last_value = 0
    response_pending = False
elif request.IsWrite:
    if request.Offset == WRITE:
        last_value = request.Value
        response_pending = True
elif request.IsRead:
    if request.Offset == READ:
        request.Value = last_value if response_pending else 0
        response_pending = False
    elif request.Offset == READ_STATUS:
        request.Value = 0 if response_pending else EMPTY
    elif request.Offset == WRITE_STATUS:
        request.Value = 0
    else:
        request.Value = 0
