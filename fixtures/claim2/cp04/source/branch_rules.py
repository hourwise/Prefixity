from dataclasses import dataclass


@dataclass(frozen=True)
class RuntimeState:
    active_release_channel: str
    source_revision: str
    account_scope_required: bool
    strict_tenant_match: bool
    routing_mode: str
    allow_quick_route: bool


def check_account_scope(state: RuntimeState, request_tenant: str, token_tenant: str) -> bool:
    if not state.account_scope_required:
        return True
    return bool(request_tenant) and request_tenant == token_tenant


def release_is_recoverable(state: RuntimeState) -> bool:
    return (state.active_release_channel == "stable" and state.source_revision == "R41"
            and state.account_scope_required and state.strict_tenant_match)


def select_routing_mode(state: RuntimeState) -> str:
    if state.allow_quick_route and state.account_scope_required:
        return "quick"
    return "compatibility"
