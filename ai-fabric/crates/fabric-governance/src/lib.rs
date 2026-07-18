#![forbid(unsafe_code)]

use ai_fabric_types::{CapabilityId, OperationId, PrincipalId, ProductPlane, QuotaLeaseId, ResourceBudget, TenantId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrincipalContext {
    pub principal_id: PrincipalId,
    pub tenant_id: TenantId,
    pub assurance_level: u8,
    pub policy_epoch: u64,
    pub revoked: bool,
}

#[derive(Clone, Debug)]
pub struct AdmissionRequest {
    pub operation_id: OperationId,
    pub capability_id: CapabilityId,
    pub principal: PrincipalContext,
    pub plane: ProductPlane,
    pub requested: ResourceBudget,
    pub payload_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceDecision {
    pub permitted: bool,
    pub reason_code: String,
    pub policy_epoch: u64,
    pub quota_lease_id: Option<QuotaLeaseId>,
    pub effective_budget: Option<ResourceBudget>,
}

#[derive(Debug, Error)]
pub enum GovernanceError {
    #[error("principal is revoked")]
    Revoked,
    #[error("assurance level is insufficient")]
    InsufficientAssurance,
    #[error("payload exceeds admitted input budget")]
    PayloadBudgetExceeded,
    #[error("capability denied")]
    CapabilityDenied,
}

#[async_trait]
pub trait GovernanceKernel: Send + Sync {
    async fn admit(&self, request: AdmissionRequest) -> Result<GovernanceDecision, GovernanceError>;
}

#[derive(Clone)]
pub struct LocalGovernanceKernel {
    pub minimum_assurance: u8,
    pub laptop_ceiling: ResourceBudget,
}

#[async_trait]
impl GovernanceKernel for LocalGovernanceKernel {
    async fn admit(&self, request: AdmissionRequest) -> Result<GovernanceDecision, GovernanceError> {
        if request.principal.revoked { return Err(GovernanceError::Revoked); }
        if request.principal.assurance_level < self.minimum_assurance { return Err(GovernanceError::InsufficientAssurance); }
        if request.payload_bytes > request.requested.max_input_bytes || request.payload_bytes > self.laptop_ceiling.max_input_bytes {
            return Err(GovernanceError::PayloadBudgetExceeded);
        }
        let effective = ResourceBudget {
            deadline_ms: request.requested.deadline_ms.min(self.laptop_ceiling.deadline_ms),
            max_input_bytes: request.requested.max_input_bytes.min(self.laptop_ceiling.max_input_bytes),
            max_output_bytes: request.requested.max_output_bytes.min(self.laptop_ceiling.max_output_bytes),
            max_memory_bytes: request.requested.max_memory_bytes.min(self.laptop_ceiling.max_memory_bytes),
            max_queue_wait_ms: request.requested.max_queue_wait_ms.min(self.laptop_ceiling.max_queue_wait_ms),
            max_model_tokens: request.requested.max_model_tokens.min(self.laptop_ceiling.max_model_tokens),
            max_tool_calls: request.requested.max_tool_calls.min(self.laptop_ceiling.max_tool_calls),
            max_iterations: request.requested.max_iterations.min(self.laptop_ceiling.max_iterations),
        };
        Ok(GovernanceDecision {
            permitted: true,
            reason_code: "PERMIT_LOCAL_POLICY".into(),
            policy_epoch: request.principal.policy_epoch,
            quota_lease_id: Some(QuotaLeaseId::new()),
            effective_budget: Some(effective),
        })
    }
}
