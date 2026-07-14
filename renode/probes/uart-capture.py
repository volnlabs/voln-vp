# Renode Python peripheral — UART0 capture shim.
#
# Captures every byte written to UART0 (PL011) into a file at the path
# stored in env UART_CAPTURE_PATH. The probe driver greps this file for
# canonical kernel markers.
#
# Renode version notes:
#   - Loaded via machine LoadAttenuator / python: blocks in .resc scripts.
#   - API differences across versions: this file uses the common surface;
#     if a particular Renode rejects a hook, the failure surfaces in Task 1.8.

import os
import sys

try:
    from Antmicro.Renode.Core import Machine
    from Antmicro.Renode.Logging import LogLevel
    IN_RENODE = True
except ImportError:
    IN_RENODE = False

MARKER = b"axiomOS booted"
OUT_PATH = os.environ.get("UART_CAPTURE_PATH", "/tmp/uart-capture.log")


class UartCapture:
    def __init__(self):
        self._buf = bytearray()
        try:
            self._log = open(OUT_PATH, "wb")
        except OSError as e:
            print(f"[uart-capture] cannot open {OUT_PATH}: {e}", file=sys.stderr)
            self._log = None

    def write(self, value):
        b = bytes([value & 0xff])
        self._buf.extend(b)
        if self._log is not None:
            self._log.write(b)
            self._log.flush()
        # Marker detection is in the shell driver (greps the file).
        # Here we just keep the buffer bounded.
        if len(self._buf) > 8192:
            self._buf = self._buf[-4096:]

    def reset(self):
        self._buf.clear()


# Provide a no-op fallback for unit tests that import this module outside
# Renode (e.g. AST parsing or pytest).
if not IN_RENODE:
    pass
else:
    # The actual wiring to sysbus PL011 happens via machine LoadAttenuator
    # in the .resc script. We expose the class so the script can attach it.
    pass