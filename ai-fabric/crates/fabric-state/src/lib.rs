#![forbid(unsafe_code)]

use ai_fabric_types::{OperationId, OperationState, TenantId, TerminalDisposition};
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use thiserror::Error;

const OPERATIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("ai_fabric_operations_v1");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationSnapshot {
    pub operation_id: OperationId,
    pub tenant_id: TenantId,
    pub state: OperationState,
    pub transition_sequence: u64,
    pub owner_epoch: u64,
    pub policy_epoch: u64,
    pub updated_unix_ms: u128,
    pub terminal_disposition: Option<TerminalDisposition>,
    pub closure_manifest_hash: Option<String>,
}

impl OperationSnapshot {
    pub fn new(operation_id: OperationId, tenant_id: TenantId, owner_epoch: u64, policy_epoch: u64) -> Self {
        Self { operation_id, tenant_id, state: OperationState::New, transition_sequence: 0, owner_epoch, policy_epoch, updated_unix_ms: now_ms(), terminal_disposition: None, closure_manifest_hash: None }
    }
}

#[derive(Clone, Debug)]
pub struct TransitionCommand {
    pub operation_id: OperationId,
    pub tenant_id: TenantId,
    pub expected_state: OperationState,
    pub next_state: OperationState,
    pub expected_owner_epoch: u64,
    pub terminal_disposition: Option<TerminalDisposition>,
    pub closure_manifest_hash: Option<String>,
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("operation not found")]
    NotFound,
    #[error("tenant mismatch")]
    TenantMismatch,
    #[error("stale owner epoch")]
    StaleOwnerEpoch,
    #[error("state conflict: expected {expected:?}, actual {actual:?}")]
    StateConflict { expected: OperationState, actual: OperationState },
    #[error("illegal transition from {from:?} to {to:?}")]
    IllegalTransition { from: OperationState, to: OperationState },
    #[error("terminal transition requires terminal disposition and closure manifest")]
    MissingClosureEvidence,
    #[error("storage error: {0}")]
    Storage(String),
}

#[derive(Clone)]
pub struct OperationLedger { db: Arc<Database> }

impl OperationLedger {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StateError> {
        let db = Database::create(path).map_err(storage)?;
        let write = db.begin_write().map_err(storage)?;
        { write.open_table(OPERATIONS).map_err(storage)?; }
        write.commit().map_err(storage)?;
        Ok(Self { db: Arc::new(db) })
    }

    pub fn create(&self, snapshot: OperationSnapshot) -> Result<OperationSnapshot, StateError> {
        let key = snapshot.operation_id.to_string();
        let bytes = serde_json::to_vec(&snapshot).map_err(storage)?;
        let write = self.db.begin_write().map_err(storage)?;
        {
            let mut table = write.open_table(OPERATIONS).map_err(storage)?;
            if table.get(key.as_str()).map_err(storage)?.is_some() { return Err(StateError::StateConflict { expected: OperationState::New, actual: OperationState::New }); }
            table.insert(key.as_str(), bytes.as_slice()).map_err(storage)?;
        }
        write.commit().map_err(storage)?;
        Ok(snapshot)
    }

    pub fn load(&self, operation_id: OperationId) -> Result<OperationSnapshot, StateError> {
        let read = self.db.begin_read().map_err(storage)?;
        let table = read.open_table(OPERATIONS).map_err(storage)?;
        let key = operation_id.to_string();
        let value = table.get(key.as_str()).map_err(storage)?.ok_or(StateError::NotFound)?;
        serde_json::from_slice(value.value()).map_err(storage)
    }

    pub fn compare_and_transition(&self, command: TransitionCommand) -> Result<OperationSnapshot, StateError> {
        let key = command.operation_id.to_string();
        let write = self.db.begin_write().map_err(storage)?;
        let updated;
        {
            let mut table = write.open_table(OPERATIONS).map_err(storage)?;
            let current_bytes = table.get(key.as_str()).map_err(storage)?.ok_or(StateError::NotFound)?;
            let mut current: OperationSnapshot = serde_json::from_slice(current_bytes.value()).map_err(storage)?;
            drop(current_bytes);
            if current.tenant_id != command.tenant_id { return Err(StateError::TenantMismatch); }
            if current.owner_epoch != command.expected_owner_epoch { return Err(StateError::StaleOwnerEpoch); }
            if current.state != command.expected_state { return Err(StateError::StateConflict { expected: command.expected_state, actual: current.state }); }
            if !legal_transition(current.state, command.next_state) { return Err(StateError::IllegalTransition { from: current.state, to: command.next_state }); }
            if command.next_state.is_terminal() && (command.terminal_disposition.is_none() || command.closure_manifest_hash.as_deref().is_none_or(str::is_empty)) {
                return Err(StateError::MissingClosureEvidence);
            }
            current.state = command.next_state;
            current.transition_sequence += 1;
            current.updated_unix_ms = now_ms();
            current.terminal_disposition = command.terminal_disposition;
            current.closure_manifest_hash = command.closure_manifest_hash;
            let encoded = serde_json::to_vec(&current).map_err(storage)?;
            table.insert(key.as_str(), encoded.as_slice()).map_err(storage)?;
            updated = current;
        }
        write.commit().map_err(storage)?;
        Ok(updated)
    }
}

pub fn legal_transition(from: OperationState, to: OperationState) -> bool {
    use OperationState::*;
    matches!((from, to),
        (New, Validating) | (Validating, Authenticated) | (Authenticated, TenantResolved) |
        (TenantResolved, Authorized) | (Authorized, Admitted) | (Admitted, Materialized) |
        (Materialized, Runnable) | (Runnable, Executing) | (Executing, WaitingDependency) |
        (WaitingDependency, Executing) | (Executing, WaitingApproval) | (WaitingApproval, Executing) |
        (Executing, Backpressured) | (Backpressured, Executing) | (Executing, Committing) |
        (Committing, Committed) | (Committed, Finalizing) | (Finalizing, Completed) |
        (Executing, Cancelling) | (WaitingDependency, Cancelling) | (WaitingApproval, Cancelling) |
        (Backpressured, Cancelling) | (Cancelling, Cancelled) |
        (Executing, FailedRetryable) | (FailedRetryable, RecoveryRequired) |
        (RecoveryRequired, Executing) | (RecoveryRequired, Compensating) |
        (Compensating, Compensated) | (_, Rejected) | (_, Expired) | (_, FailedTerminal)
    ) && !from.is_terminal()
}

fn now_ms() -> u128 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() }
fn storage(error: impl std::fmt::Display) -> StateError { StateError::Storage(error.to_string()) }

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn stale_owner_and_illegal_transition_are_rejected() {
        let path = std::env::temp_dir().join(format!("ai-fabric-state-{}.redb", OperationId::new()));
        let ledger = OperationLedger::open(&path).unwrap();
        let op = OperationId::new();
        let tenant = TenantId::new();
        ledger.create(OperationSnapshot::new(op, tenant, 7, 1)).unwrap();
        assert!(matches!(ledger.compare_and_transition(TransitionCommand { operation_id: op, tenant_id: tenant, expected_state: OperationState::New, next_state: OperationState::Validating, expected_owner_epoch: 6, terminal_disposition: None, closure_manifest_hash: None }), Err(StateError::StaleOwnerEpoch)));
        ledger.compare_and_transition(TransitionCommand { operation_id: op, tenant_id: tenant, expected_state: OperationState::New, next_state: OperationState::Validating, expected_owner_epoch: 7, terminal_disposition: None, closure_manifest_hash: None }).unwrap();
        assert!(matches!(ledger.compare_and_transition(TransitionCommand { operation_id: op, tenant_id: tenant, expected_state: OperationState::Validating, next_state: OperationState::Executing, expected_owner_epoch: 7, terminal_disposition: None, closure_manifest_hash: None }), Err(StateError::IllegalTransition { .. })));
        drop(ledger);
        let _ = fs::remove_file(path);
    }
}
