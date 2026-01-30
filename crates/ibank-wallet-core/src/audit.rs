//! Audit types and logging hooks.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Audit event emitted during signing and submission.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event name.
    pub name: String,
    /// Event metadata.
    pub metadata: Value,
}

/// Simple audit log container.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuditLog {
    /// Recorded audit events.
    pub events: Vec<AuditEvent>,
}

impl AuditLog {
    /// Records a new audit event.
    pub fn record(&mut self, event: AuditEvent) {
        self.events.push(event);
    }
}
