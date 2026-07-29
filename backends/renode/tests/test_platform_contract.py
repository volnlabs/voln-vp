from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
BOOT_SCRIPT = ROOT / "backends/renode/scripts/boot-virt-pi5.resc"
PLATFORM = ROOT / "boards/virt-pi5/renode/virt-pi5.repl"


class PlatformContractTests(unittest.TestCase):
    def test_direct_kernel_enters_at_el1_before_image_load(self):
        script = BOOT_SCRIPT.read_text()
        el1 = script.index("cpu SetAvailableExceptionLevels false false")
        load_dtb = script.index("sysbus LoadFdt")
        load_elf = script.index("sysbus LoadELF")

        self.assertLess(el1, load_dtb)
        self.assertLess(el1, load_elf)
        self.assertIn("gic DisabledSecurity true", script)

    def test_canonical_platform_has_no_high_address_alias(self):
        platform = PLATFORM.read_text().lower()

        self.assertNotIn("ffff8000", platform)
        self.assertIn("ram: memory.mappedmemory @ sysbus 0x0", platform)
        self.assertIn("size: 0x200000000", platform)


if __name__ == "__main__":
    unittest.main()
