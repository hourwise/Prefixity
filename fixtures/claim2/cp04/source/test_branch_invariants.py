import unittest
from branch_rules import RuntimeState, check_account_scope, release_is_recoverable


class BranchInvariantTests(unittest.TestCase):
    def test_recovered_release_keeps_account_scope(self) -> None:
        state = RuntimeState("stable", "R41", True, True, "compatibility", False)
        self.assertTrue(release_is_recoverable(state))
        self.assertTrue(check_account_scope(state, "tenant-a", "tenant-a"))
        self.assertFalse(check_account_scope(state, "tenant-a", "tenant-b"))
