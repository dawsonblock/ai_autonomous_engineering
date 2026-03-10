CONTROLLER_EVENT_TYPES = {
    "workflow.started",
    "task.ready",
    "task.dispatched",
    "task.retry_scheduled",
    "task.succeeded",
    "task.failed",
    "task.blocked",
    "memory.updated",
    "workflow.completed",
}

DOMAIN_EVENT_TYPES = {
    "research.completed",
    "security.vulnerability_detected",
    "security.audit_completed",
    "swe.patch_generated",
    "swe.test_failed",
    "swe.build_completed",
}

ALL_EVENT_TYPES = CONTROLLER_EVENT_TYPES | DOMAIN_EVENT_TYPES
