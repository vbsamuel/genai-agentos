#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fmt, str::FromStr, time::Duration};
use thiserror::Error;
use ulid::Ulid;

macro_rules! strong_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Ulid);

        impl $name {
            pub fn new() -> Self { Self(Ulid::new()) }
            pub fn as_ulid(self) -> Ulid { self.0 }
        }

        impl Default for $name { fn default() -> Self { Self::new() } }
        impl fmt::Display for $name { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) } }
        impl FromStr for $name {
            type Err = ulid::DecodeError;
            fn from_str(value: &str) -> Result<Self, Self::Err> { Ok(Self(Ulid::from_string(value)?)) }
        }
    };
}

strong_id!(TenantId);
strong_id!(PrincipalId);
strong_id!(CapabilityId);
strong_id!(OperationId);
strong_id!(SessionId);
strong_id!(ConnectionId);
strong_id!(StreamId);
strong_id!(EventId);
strong_id!(AuditId);
strong_id!(QuotaLeaseId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProductPlane {
    BehaviorControl,
    EventOrchestration,
    EventFlow,
    DataFlow,
    TransitionState,
    Governance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Surface {
    HttpJson,
    GrpcUnary,
    GrpcClientStream,
    GrpcServerStream,
    GrpcBidirectional,
    WebSocket,
    Event,
    Embedded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationState {
    New,
    Validating,
    Authenticated,
    TenantResolved,
    Authorized,
    Admitted,
    Materialized,
    Runnable,
    Executing,
    WaitingDependency,
    WaitingApproval,
    Backpressured,
    Cancelling,
    Committing,
    Committed,
    Finalizing,
    Completed,
    Cancelled,
    Rejected,
    Expired,
    FailedRetryable,
    FailedTerminal,
    Compensating,
    Compensated,
    RecoveryRequired,
}

impl OperationState {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Rejected | Self::Expired | Self::FailedTerminal | Self::Compensated)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TerminalDisposition {
    SucceededCommitted,
    SucceededNoMutation,
    RejectedPrecondition,
    RejectedAuthentication,
    RejectedAuthorization,
    RejectedQuota,
    RejectedConflict,
    CancelledBeforeCommit,
    CommittedAfterCancellationRequest,
    ExpiredBeforeExecution,
    FailedRetryable,
    FailedTerminal,
    Compensated,
    Superseded,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub deadline_ms: u64,
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
    pub max_memory_bytes: u64,
    pub max_queue_wait_ms: u64,
    pub max_model_tokens: u64,
    pub max_tool_calls: u32,
    pub max_iterations: u32,
}

impl ResourceBudget {
    pub fn deadline(&self) -> Duration { Duration::from_millis(self.deadline_ms) }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntegrityDescriptor {
    pub algorithm: String,
    pub digest_hex: String,
}

impl IntegrityDescriptor {
    pub fn sha256(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        Self { algorithm: "SHA-256".into(), digest_hex: format!("{digest:x}") }
    }

    pub fn verify(&self, bytes: &[u8]) -> bool {
        self.algorithm == "SHA-256" && self.digest_hex == Self::sha256(bytes).digest_hex
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FabricEnvelope {
    pub schema_version: u16,
    pub tenant_id: TenantId,
    pub principal_id: PrincipalId,
    pub capability_id: CapabilityId,
    pub operation_id: OperationId,
    pub session_id: Option<SessionId>,
    pub connection_id: Option<ConnectionId>,
    pub stream_id: Option<StreamId>,
    pub surface: Surface,
    pub plane: ProductPlane,
    pub sequence_no: Option<u64>,
    pub owner_epoch: u64,
    pub policy_epoch: u64,
    pub idempotency_key: Option<String>,
    pub payload_length: u64,
    pub integrity: IntegrityDescriptor,
    pub budget: ResourceBudget,
}

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("unsupported schema version {0}")]
    UnsupportedSchema(u16),
    #[error("payload exceeds declared budget")]
    PayloadTooLarge,
    #[error("integrity verification failed")]
    IntegrityFailure,
    #[error("invalid idempotency key")]
    InvalidIdempotencyKey,
}

impl FabricEnvelope {
    pub fn validate(&self, payload: &[u8]) -> Result<(), ContractError> {
        if self.schema_version != 1 { return Err(ContractError::UnsupportedSchema(self.schema_version)); }
        if payload.len() as u64 != self.payload_length || self.payload_length > self.budget.max_input_bytes {
            return Err(ContractError::PayloadTooLarge);
        }
        if !self.integrity.verify(payload) { return Err(ContractError::IntegrityFailure); }
        if self.idempotency_key.as_deref().is_some_and(|v| v.is_empty() || v.len() > 128) {
            return Err(ContractError::InvalidIdempotencyKey);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_state_is_closed() {
        assert!(OperationState::Completed.is_terminal());
        assert!(!OperationState::Executing.is_terminal());
    }

    #[test]
    fn envelope_integrity_is_enforced() {
        let payload = b"hello";
        let envelope = FabricEnvelope {
            schema_version: 1,
            tenant_id: TenantId::new(),
            principal_id: PrincipalId::new(),
            capability_id: CapabilityId::new(),
            operation_id: OperationId::new(),
            session_id: None,
            connection_id: None,
            stream_id: None,
            surface: Surface::HttpJson,
            plane: ProductPlane::BehaviorControl,
            sequence_no: None,
            owner_epoch: 1,
            policy_epoch: 1,
            idempotency_key: Some("request-1".into()),
            payload_length: payload.len() as u64,
            integrity: IntegrityDescriptor::sha256(payload),
            budget: ResourceBudget { deadline_ms: 1000, max_input_bytes: 1024, max_output_bytes: 1024, max_memory_bytes: 4096, max_queue_wait_ms: 50, max_model_tokens: 0, max_tool_calls: 0, max_iterations: 0 },
        };
        assert!(envelope.validate(payload).is_ok());
        assert!(envelope.validate(b"tampered").is_err());
    }
}
