import pathlib
import unittest


class Request:
    def __init__(self, operation, offset=0, value=0):
        self.IsInit = operation == "init"
        self.IsRead = operation == "read"
        self.IsWrite = operation == "write"
        self.Offset = offset
        self.Value = value
        self.Length = 4


class MailboxScriptTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path = (
            pathlib.Path(__file__).resolve().parents[1]
            / "peripherals"
            / "mailbox.py"
        )
        cls.code = compile(path.read_text(encoding="utf-8"), str(path), "exec")

    def setUp(self):
        self.scope = {"request": Request("init")}
        exec(self.code, self.scope)

    def transact(self, operation, offset=0, value=0):
        request = Request(operation, offset, value)
        self.scope["request"] = request
        exec(self.code, self.scope)
        return request.Value

    def test_empty_status_before_write(self):
        self.assertEqual(self.transact("read", 0x18), 1 << 30)

    def test_write_is_returned_once(self):
        self.transact("write", 0x20, 0x1234_5678)

        self.assertEqual(self.transact("read", 0x18), 0)
        self.assertEqual(self.transact("read", 0x00), 0x1234_5678)
        self.assertEqual(self.transact("read", 0x18), 1 << 30)
        self.assertEqual(self.transact("read", 0x00), 0)

    def test_write_side_is_never_full(self):
        self.assertEqual(self.transact("read", 0x38), 0)


if __name__ == "__main__":
    unittest.main()
