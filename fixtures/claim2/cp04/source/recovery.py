from dataclasses import dataclass


@dataclass(frozen=True)
class RestoreReceipt:
    branch: str
    source_revision: str
    dirty: bool
    candidate_failure_id: str


def restored_origin_is_valid(receipt: RestoreReceipt, expected_revision: str) -> bool:
    return (receipt.branch == "stable" and receipt.source_revision == expected_revision
            and receipt.dirty is False and bool(receipt.candidate_failure_id))
