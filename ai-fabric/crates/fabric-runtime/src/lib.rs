#![forbid(unsafe_code)]

use ai_fabric_governance::{AdmissionRequest, GovernanceKernel, PrincipalContext};
use ai_fabric_state::{OperationLedger, OperationSnapshot, StateError, TransitionCommand};
use ai_fabric_types::{
    CapabilityId, FabricEnvelope, OperationId, OperationState, ProductPlane, ResourceBudget,
    TenantId, TerminalDisposition,
};
use bytes::Bytes;
use parking_lot::Mutex;
use sha2::{Digest, Sha256};
use std::{collections::{HashMap, VecDeque}, sync::Arc, time::Instant};
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum FabricEventKind {
    OperationCreated,
    OperationAdmitted,
    InputCommitted,
    ExecutionStarted,
    OutputCommitted,
    OperationClosed,
    OperationRejected,
}

#[derive(Clone, Debug)]
pub struct FabricEvent {
    pub event_id: ai_fabric_types::EventId,
    pub tenant_id: TenantId,
    pub operation_id: OperationId,
    pub plane: ProductPlane,
    pub kind: FabricEventKind,
    pub sequence: u64,
    pub payload_hash: String,
}

#[derive(Clone, Debug)]
pub struct OperationRequest {
    pub envelope: FabricEnvelope,
    pub principal: PrincipalContext,
    pub payload: Bytes,
}

#[derive(Clone, Debug)]
pub struct OperationReceipt {
    pub operation_id: OperationId,
    pub final_state: OperationState,
    pub disposition: TerminalDisposition,
    pub closure_manifest_hash: String,
    pub output: Bytes,
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("contract validation failed: {0}")]
    Contract(String),
    #[error("governance denied: {0}")]
    Governance(String),
    #[error("state error: {0}")]
    State(#[from] StateError),
    #[error("duplicate idempotency key with conflicting payload")]
    IdempotencyConflict,
    #[error("operation deadline exceeded")]
    DeadlineExceeded,
    #[error("event queue capacity exhausted")]
    EventQueueFull,
}

#[derive(Clone)]
pub struct FabricKernel<G: GovernanceKernel> {
    governance: Arc<G>,
    ledger: OperationLedger,
    event_queue: Arc<Mutex<VecDeque<FabricEvent>>>,
    idempotency: Arc<Mutex<HashMap<(TenantId, String), (String, OperationReceipt)>>>,
    event_capacity: usize,
}

impl<G: GovernanceKernel> FabricKernel<G> {
    pub fn new(governance: G, ledger: OperationLedger, event_capacity: usize) -> Self {
        Self {
            governance: Arc::new(governance),
            ledger,
            event_queue: Arc::new(Mutex::new(VecDeque::with_capacity(event_capacity))),
            idempotency: Arc::new(Mutex::new(HashMap::new())),
            event_capacity,
        }
    }

    pub async fn execute(&self, request: OperationRequest) -> Result<OperationReceipt, RuntimeError> {
        let started = Instant::now();
        request.envelope.validate(&request.payload).map_err(|e| RuntimeError::Contract(e.to_string()))?;
        let payload_hash = hex_sha256(&request.payload);

        if let Some(key) = request.envelope.idempotency_key.as_ref() {
            if let Some((existing_hash, receipt)) = self.idempotency.lock().get(&(request.envelope.tenant_id, key.clone())).cloned() {
                return if existing_hash == payload_hash { Ok(receipt) } else { Err(RuntimeError::IdempotencyConflict) };
            }
        }

        let op = request.envelope.operation_id;
        let tenant = request.envelope.tenant_id;
        self.ledger.create(OperationSnapshot::new(op, tenant, request.envelope.owner_epoch, request.envelope.policy_epoch))?;
        self.emit(tenant, op, ProductPlane::TransitionState, FabricEventKind::OperationCreated, 1, &payload_hash)?;
        self.step(op, tenant, OperationState::New, OperationState::Validating, request.envelope.owner_epoch)?;
        self.step(op, tenant, OperationState::Validating, OperationState::Authenticated, request.envelope.owner_epoch)?;
        self.step(op, tenant, OperationState::Authenticated, OperationState::TenantResolved, request.envelope.owner_epoch)?;

        let decision = self.governance.admit(AdmissionRequest {
            operation_id: op,
            capability_id: request.envelope.capability_id,
            principal: request.principal,
            plane: request.envelope.plane,
            requested: request.envelope.budget.clone(),
            payload_bytes: request.payload.len() as u64,
        }).await.map_err(|e| RuntimeError::Governance(e.to_string()))?;

        if !decision.permitted {
            return Err(RuntimeError::Governance(decision.reason_code));
        }
        self.step(op, tenant, OperationState::TenantResolved, OperationState::Authorized, request.envelope.owner_epoch)?;
        self.step(op, tenant, OperationState::Authorized, OperationState::Admitted, request.envelope.owner_epoch)?;
        self.emit(tenant, op, ProductPlane::Governance, FabricEventKind::OperationAdmitted, 2, &payload_hash)?;
        self.step(op, tenant, OperationState::Admitted, OperationState::Materialized, request.envelope.owner_epoch)?;
        self.step(op, tenant, OperationState::Materialized, OperationState::Runnable, request.envelope.owner_epoch)?;
        self.step(op, tenant, OperationState::Runnable, OperationState::Executing, request.envelope.owner_epoch)?;
        self.emit(tenant, op, ProductPlane::EventOrchestration, FabricEventKind::ExecutionStarted, 3, &payload_hash)?;

        if started.elapsed() > request.envelope.budget.deadline() { return Err(RuntimeError::DeadlineExceeded); }

        // V1 deterministic behavior: persist and echo the canonical payload. AI/model execution adapters
        // are postcursor modules; they cannot bypass this state, budget, event, and closure kernel.
        let output = request.payload.clone();
        self.emit(tenant, op, ProductPlane::DataFlow, FabricEventKind::InputCommitted, 4, &payload_hash)?;
        self.step(op, tenant, OperationState::Executing, OperationState::Committing, request.envelope.owner_epoch)?;
        self.step(op, tenant, OperationState::Committing, OperationState::Committed, request.envelope.owner_epoch)?;
        self.emit(tenant, op, ProductPlane::EventFlow, FabricEventKind::OutputCommitted, 5, &hex_sha256(&output))?;
        self.step(op, tenant, OperationState::Committed, OperationState::Finalizing, request.envelope.owner_epoch)?;

        let closure_manifest_hash = hex_sha256(format!("{op}:{tenant}:{}:{}", request.payload.len(), output.len()).as_bytes());
        self.ledger.compare_and_transition(TransitionCommand {
            operation_id: op,
            tenant_id: tenant,
            expected_state: OperationState::Finalizing,
            next_state: OperationState::Completed,
            expected_owner_epoch: request.envelope.owner_epoch,
            terminal_disposition: Some(TerminalDisposition::SucceededCommitted),
            closure_manifest_hash: Some(closure_manifest_hash.clone()),
        })?;
        self.emit(tenant, op, ProductPlane::TransitionState, FabricEventKind::OperationClosed, 6, &closure_manifest_hash)?;

        let receipt = OperationReceipt { operation_id: op, final_state: OperationState::Completed, disposition: TerminalDisposition::SucceededCommitted, closure_manifest_hash, output };
        if let Some(key) = request.envelope.idempotency_key {
            self.idempotency.lock().insert((tenant, key), (payload_hash, receipt.clone()));
        }
        Ok(receipt)
    }

    pub fn drain_events(&self, max: usize) -> Vec<FabricEvent> {
        let mut queue = self.event_queue.lock();
        (0..max.min(queue.len())).filter_map(|_| queue.pop_front()).collect()
    }

    fn step(&self, op: OperationId, tenant: TenantId, from: OperationState, to: OperationState, epoch: u64) -> Result<(), RuntimeError> {
        self.ledger.compare_and_transition(TransitionCommand { operation_id: op, tenant_id: tenant, expected_state: from, next_state: to, expected_owner_epoch: epoch, terminal_disposition: None, closure_manifest_hash: None })?;
        Ok(())
    }

    fn emit(&self, tenant: TenantId, op: OperationId, plane: ProductPlane, kind: FabricEventKind, sequence: u64, payload_hash: &str) -> Result<(), RuntimeError> {
        let mut queue = self.event_queue.lock();
        if queue.len() >= self.event_capacity { return Err(RuntimeError::EventQueueFull); }
        queue.push_back(FabricEvent { event_id: ai_fabric_types::EventId::new(), tenant_id: tenant, operation_id: op, plane, kind, sequence, payload_hash: payload_hash.to_owned() });
        Ok(())
    }
}

fn hex_sha256(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

pub fn consumer_laptop_budget() -> ResourceBudget {
    ResourceBudget { deadline_ms: 30_000, max_input_bytes: 16 * 1024 * 1024, max_output_bytes: 16 * 1024 * 1024, max_memory_bytes: 256 * 1024 * 1024, max_queue_wait_ms: 500, max_model_tokens: 32_768, max_tool_calls: 16, max_iterations: 32 }
}
