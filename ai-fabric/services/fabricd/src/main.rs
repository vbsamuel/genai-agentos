#![forbid(unsafe_code)]

use ai_fabric_governance::{LocalGovernanceKernel, PrincipalContext};
use ai_fabric_runtime::{consumer_laptop_budget, FabricKernel, OperationRequest};
use ai_fabric_state::OperationLedger;
use ai_fabric_types::{
    CapabilityId, FabricEnvelope, IntegrityDescriptor, OperationId, PrincipalId, ProductPlane,
    ResourceBudget, Surface, TenantId,
};
use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "ai-fabricd", version, about = "AI Fabric singular local control plane")]
struct Args {
    #[arg(long, env = "AI_FABRIC_BIND", default_value = "127.0.0.1:8787")]
    bind: SocketAddr,
    #[arg(long, env = "AI_FABRIC_STATE", default_value = "./var/ai-fabric.redb")]
    state: PathBuf,
    #[arg(long, env = "AI_FABRIC_EVENT_CAPACITY", default_value_t = 4096)]
    event_capacity: usize,
}

type Kernel = FabricKernel<LocalGovernanceKernel>;

#[derive(Clone)]
struct AppState { kernel: Arc<Kernel> }

#[derive(Debug, Deserialize)]
struct ExecuteRequest {
    tenant_id: TenantId,
    principal_id: PrincipalId,
    capability_id: CapabilityId,
    payload: serde_json::Value,
    idempotency_key: String,
    #[serde(default)]
    budget: Option<ResourceBudget>,
}

#[derive(Debug, Serialize)]
struct ExecuteResponse {
    operation_id: OperationId,
    state: String,
    disposition: String,
    closure_manifest_hash: String,
    output: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct HealthResponse { status: &'static str, product: &'static str, namespace: &'static str }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).json().init();
    let args = Args::parse();
    if let Some(parent) = args.state.parent() { std::fs::create_dir_all(parent).context("create state directory")?; }
    let ledger = OperationLedger::open(&args.state).context("open operation ledger")?;
    let kernel = FabricKernel::new(LocalGovernanceKernel { minimum_assurance: 1, laptop_ceiling: consumer_laptop_budget() }, ledger, args.event_capacity);
    let state = AppState { kernel: Arc::new(kernel) };

    let app = Router::new()
        .route("/health/live", get(health))
        .route("/health/ready", get(health))
        .route("/v1/capabilities/{capability_id}:execute", post(execute))
        .route("/v1/events:drain", post(drain_events))
        .layer(RequestBodyLimitLayer::new(16 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(args.bind).await.context("bind local listener")?;
    info!(bind = %args.bind, "AI_Fabric_Microservices control plane ready");
    axum::serve(listener, app).with_graceful_shutdown(shutdown()).await.context("serve")
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "READY", product: "AI_Fabric_Microservices", namespace: "ai_fabric" })
}

async fn execute(
    State(state): State<AppState>,
    Path(capability_id): Path<CapabilityId>,
    Json(input): Json<ExecuteRequest>,
) -> impl IntoResponse {
    if input.capability_id != capability_id {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error":"CAPABILITY_PATH_BODY_CONFLICT"}))).into_response();
    }
    let payload_vec = match serde_json::to_vec(&input.payload) {
        Ok(value) => value,
        Err(error) => return (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({"error":"INVALID_JSON","detail":error.to_string()}))).into_response(),
    };
    let operation_id = OperationId::new();
    let budget = input.budget.unwrap_or_else(consumer_laptop_budget);
    let envelope = FabricEnvelope {
        schema_version: 1,
        tenant_id: input.tenant_id,
        principal_id: input.principal_id,
        capability_id,
        operation_id,
        session_id: None,
        connection_id: None,
        stream_id: None,
        surface: Surface::HttpJson,
        plane: ProductPlane::BehaviorControl,
        sequence_no: None,
        owner_epoch: 1,
        policy_epoch: 1,
        idempotency_key: Some(input.idempotency_key),
        payload_length: payload_vec.len() as u64,
        integrity: IntegrityDescriptor::sha256(&payload_vec),
        budget,
    };
    let request = OperationRequest {
        envelope,
        principal: PrincipalContext { principal_id: input.principal_id, tenant_id: input.tenant_id, assurance_level: 1, policy_epoch: 1, revoked: false },
        payload: Bytes::from(payload_vec),
    };
    match state.kernel.execute(request).await {
        Ok(receipt) => {
            let output = serde_json::from_slice(&receipt.output).unwrap_or(serde_json::Value::Null);
            (StatusCode::OK, Json(serde_json::to_value(ExecuteResponse {
                operation_id: receipt.operation_id,
                state: format!("{:?}", receipt.final_state),
                disposition: format!("{:?}", receipt.disposition),
                closure_manifest_hash: receipt.closure_manifest_hash,
                output,
            }).unwrap())).into_response()
        }
        Err(error) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"FABRIC_EXECUTION_REJECTED","detail":error.to_string()}))).into_response(),
    }
}

async fn drain_events(State(state): State<AppState>) -> Json<serde_json::Value> {
    let events = state.kernel.drain_events(256);
    Json(serde_json::json!({"count":events.len(),"events":events.into_iter().map(|e| serde_json::json!({
        "event_id": e.event_id,
        "tenant_id": e.tenant_id,
        "operation_id": e.operation_id,
        "plane": format!("{:?}", e.plane),
        "kind": format!("{:?}", e.kind),
        "sequence": e.sequence,
        "payload_hash": e.payload_hash,
    })).collect::<Vec<_>>() }))
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}
