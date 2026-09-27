import unittest
from recovery import RestoreReceipt, restored_origin_is_valid


class RecoveryTests(unittest.TestCase):
    def test_restore_returns_exact_origin_and_keeps_failure_record(self) -> None:
        receipt = RestoreReceipt("stable", "R41", False, "BRANCH-CP04-PROBE-42")
        self.assertTrue(restored_origin_is_valid(receipt, "R41"))
        self.assertFalse(restored_origin_is_valid(receipt, "R42"))
