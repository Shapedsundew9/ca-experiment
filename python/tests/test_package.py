import unittest

import tools


class ToolsPackageTests(unittest.TestCase):
    def test_package_is_importable(self) -> None:
        self.assertEqual(tools.__name__, "tools")


if __name__ == "__main__":
    unittest.main()
