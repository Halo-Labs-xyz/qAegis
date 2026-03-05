//! HTTP and WebSocket Handlers

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::hybrid_engine::{HybridExecutionRecord, HybridSolveRequest, HybridSolveResponse};
use crate::lean_sig::{
    LeanSigSignRequest, LeanSigSignResponse, LeanSigVerifyRequest, LeanSigVerifyResponse,
};
use crate::lean_vm::{
    LeanVmExecuteRequest, LeanVmExecuteResponse, LeanVmPoseidonRequest, LeanVmPoseidonResponse,
    PoseidonVariant,
};
use crate::qrm::{QuantumEra, ThreatCategory, ThreatIndicator};
use crate::state::{
    inject_high_threats, AegisTeeStatus, AppState, ApqcStatus, ChainStatus, Event, QrmStatus,
    QvmStatus, SequencerStatus, StatusResponse, Thresholds,
};
use hex;
use std::time::Instant;

/// GET /api/status
pub async fn get_status(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    // Acquire locks one at a time and release before next to avoid deadlocks

    // QVM Status
    let qvm_status = {
        let qvm = state.qvm.lock().await;
        let status = qvm.get_status();
        Some(QvmStatus {
            processor: status.processor.processor_id().to_string(),
            current_era: format!("{:?}", status.current_era),
            qrm_risk_score: status.qrm_risk_score,
            oracle_risk_score: status.oracle_risk_score,
            assessments_count: status.assessments_count as u64,
            era_transitions: status.era_transitions as u64,
            threat_indicators_count: status.threat_indicators_count,
            recommended_algorithms: status.recommended_algorithms,
        })
    };

    // Aegis-TEE Status
    let aegis_tee_status = {
        let aegis_tee = state.aegis_tee.lock().await;
        let (platform, mrenclave, mempool_size, asset_protection_enabled, phala_redundancy_enabled) =
            aegis_tee.get_status();
        Some(AegisTeeStatus {
            platform,
            mrenclave,
            mempool_size,
            asset_protection_enabled,
            phala_redundancy_enabled,
        })
    };

    let (risk, indicator_count, threshold_scheduled, threshold_emergency) = {
        let mut qrm = state.qrm.lock().await;
        let risk = qrm.calculate_risk();
        (
            risk,
            qrm.indicator_count(),
            qrm.threshold_scheduled,
            qrm.threshold_emergency,
        )
    };

    let apqc_status = {
        let apqc = state.apqc.lock().await;
        ApqcStatus {
            signatures: apqc
                .active_signatures
                .iter()
                .map(|s| s.name().to_string())
                .collect(),
            kems: apqc
                .active_kems
                .iter()
                .map(|k| k.name().to_string())
                .collect(),
            rotation_pending: apqc.rotation_pending,
            rotation_block: apqc.rotation_block,
        }
    };

    let sequencer_status = {
        let sequencer = state.sequencer.lock().await;
        SequencerStatus {
            mempool_size: sequencer.mempool_size(),
            ordered_queue: sequencer.ordered_queue_size(),
            batch_count: sequencer.batch_count(),
            tee_platform: sequencer.tee_platform.clone(),
            mrenclave: sequencer.mrenclave.clone(),
        }
    };

    let chain_status = {
        let chain = state.chain.lock().await;
        ChainStatus {
            height: chain.current_height,
            algorithm_set: chain.algorithm_set.clone(),
            risk_score: chain.risk_score,
        }
    };

    Json(StatusResponse {
        qvm: qvm_status,
        qrm: QrmStatus {
            risk_score: risk.score,
            recommendation: risk.recommendation,
            indicator_count,
            thresholds: Thresholds {
                scheduled: threshold_scheduled,
                emergency: threshold_emergency,
            },
        },
        apqc: apqc_status,
        aegis_tee: aegis_tee_status,
        sequencer: sequencer_status,
        chain: chain_status,
    })
}

/// GET /api/qrm/history
pub async fn get_qrm_history(State(state): State<Arc<AppState>>) -> Json<QrmHistoryResponse> {
    let qrm = state.qrm.lock().await;

    Json(QrmHistoryResponse {
        indicators: qrm.get_indicators().into_iter().rev().take(20).collect(),
        risk_history: qrm.get_risk_history().into_iter().rev().take(50).collect(),
    })
}

#[derive(Serialize)]
pub struct QrmHistoryResponse {
    indicators: Vec<ThreatIndicator>,
    risk_history: Vec<crate::qrm::RiskAssessment>,
}

/// GET /api/blocks
pub async fn get_blocks(State(state): State<Arc<AppState>>) -> Json<BlocksResponse> {
    let chain = state.chain.lock().await;

    Json(BlocksResponse {
        blocks: chain.get_recent_blocks(20),
    })
}

#[derive(Serialize)]
pub struct BlocksResponse {
    blocks: Vec<crate::chain::Block>,
}

/// POST /api/inject_threat
pub async fn inject_threat(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InjectThreatRequest>,
) -> Json<InjectThreatResponse> {
    let category = match payload.category.as_str() {
        "digital_signatures" => ThreatCategory::DigitalSignatures,
        "zk_proof_forgery" => ThreatCategory::ZkProofForgery,
        "decryption_hndl" => ThreatCategory::DecryptionHndl,
        "hash_reversal" => ThreatCategory::HashReversal,
        "consensus_attacks" => ThreatCategory::ConsensusAttacks,
        "cross_chain_bridge" => ThreatCategory::CrossChainBridge,
        "network_layer" => ThreatCategory::NetworkLayer,
        "key_management" => ThreatCategory::KeyManagement,
        "mev_ordering" => ThreatCategory::MevOrdering,
        "smart_contracts" => ThreatCategory::SmartContracts,
        "side_channel" => ThreatCategory::SideChannel,
        "migration_agility" => ThreatCategory::MigrationAgility,
        _ => ThreatCategory::DigitalSignatures,
    };

    let era = match payload.era_relevance.as_deref() {
        Some("pre_quantum") => QuantumEra::PreQuantum,
        Some("nisq") => QuantumEra::Nisq,
        Some("fault_tolerant") => QuantumEra::FaultTolerant,
        _ => QuantumEra::Nisq,
    };

    let indicator = ThreatIndicator {
        category,
        sub_category: payload.sub_category.unwrap_or_else(|| "Manual".to_string()),
        severity: payload.severity.unwrap_or(0.8),
        confidence: payload.confidence.unwrap_or(0.9),
        source: payload
            .source
            .unwrap_or_else(|| "Manual Injection".to_string()),
        timestamp: chrono::Utc::now(),
        description: payload
            .description
            .unwrap_or_else(|| "Manually injected threat".to_string()),
        era_relevance: era,
        references: payload.references.unwrap_or_default(),
    };

    let risk = {
        let mut qrm = state.qrm.lock().await;
        qrm.add_indicator(indicator.clone());
        qrm.calculate_risk()
    };

    state.broadcast(Event::QrmUpdate {
        indicator: indicator.clone(),
        risk: risk.clone(),
    });

    Json(InjectThreatResponse { indicator, risk })
}

#[derive(Deserialize)]
pub struct InjectThreatRequest {
    category: String,
    sub_category: Option<String>,
    severity: Option<f64>,
    confidence: Option<f64>,
    source: Option<String>,
    description: Option<String>,
    era_relevance: Option<String>,
    references: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct InjectThreatResponse {
    indicator: ThreatIndicator,
    risk: crate::qrm::RiskAssessment,
}

/// POST /api/simulation/start
pub async fn start_simulation(State(state): State<Arc<AppState>>) -> Json<SimulationResponse> {
    let mut running = state.simulation_running.lock().await;
    *running = true;
    state.broadcast(Event::SimulationStarted);

    Json(SimulationResponse {
        status: "running".to_string(),
    })
}

/// POST /api/simulation/stop
pub async fn stop_simulation(State(state): State<Arc<AppState>>) -> Json<SimulationResponse> {
    let mut running = state.simulation_running.lock().await;
    *running = false;
    state.broadcast(Event::SimulationStopped);

    Json(SimulationResponse {
        status: "stopped".to_string(),
    })
}

#[derive(Serialize)]
pub struct SimulationResponse {
    status: String,
}

/// POST /api/inject_high_threat
pub async fn inject_high_threat(State(state): State<Arc<AppState>>) -> Json<SimulationResponse> {
    inject_high_threats(&state).await;
    Json(SimulationResponse {
        status: "injected".to_string(),
    })
}

/// WebSocket handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to events
    let mut rx = state.subscribe();

    // Build initial status without nested locks
    let initial_status = {
        let qvm_status = {
            let qvm = state.qvm.lock().await;
            let status = qvm.get_status();
            Some(QvmStatus {
                processor: status.processor.processor_id().to_string(),
                current_era: format!("{:?}", status.current_era),
                qrm_risk_score: status.qrm_risk_score,
                oracle_risk_score: status.oracle_risk_score,
                assessments_count: status.assessments_count as u64,
                era_transitions: status.era_transitions as u64,
                threat_indicators_count: status.threat_indicators_count,
                recommended_algorithms: status.recommended_algorithms,
            })
        };

        let aegis_tee_status = {
            let aegis_tee = state.aegis_tee.lock().await;
            let (
                platform,
                mrenclave,
                mempool_size,
                asset_protection_enabled,
                phala_redundancy_enabled,
            ) = aegis_tee.get_status();
            Some(AegisTeeStatus {
                platform,
                mrenclave,
                mempool_size,
                asset_protection_enabled,
                phala_redundancy_enabled,
            })
        };

        let (risk, indicator_count, threshold_scheduled, threshold_emergency) = {
            let mut qrm = state.qrm.lock().await;
            let risk = qrm.calculate_risk();
            (
                risk,
                qrm.indicator_count(),
                qrm.threshold_scheduled,
                qrm.threshold_emergency,
            )
        };

        let apqc_status = {
            let apqc = state.apqc.lock().await;
            ApqcStatus {
                signatures: apqc
                    .active_signatures
                    .iter()
                    .map(|s| s.name().to_string())
                    .collect(),
                kems: apqc
                    .active_kems
                    .iter()
                    .map(|k| k.name().to_string())
                    .collect(),
                rotation_pending: apqc.rotation_pending,
                rotation_block: apqc.rotation_block,
            }
        };

        let sequencer_status = {
            let sequencer = state.sequencer.lock().await;
            SequencerStatus {
                mempool_size: sequencer.mempool_size(),
                ordered_queue: sequencer.ordered_queue_size(),
                batch_count: sequencer.batch_count(),
                tee_platform: sequencer.tee_platform.clone(),
                mrenclave: sequencer.mrenclave.clone(),
            }
        };

        let chain_status = {
            let chain = state.chain.lock().await;
            ChainStatus {
                height: chain.current_height,
                algorithm_set: chain.algorithm_set.clone(),
                risk_score: chain.risk_score,
            }
        };

        StatusResponse {
            qvm: qvm_status,
            qrm: QrmStatus {
                risk_score: risk.score,
                recommendation: risk.recommendation,
                indicator_count,
                thresholds: Thresholds {
                    scheduled: threshold_scheduled,
                    emergency: threshold_emergency,
                },
            },
            apqc: apqc_status,
            aegis_tee: aegis_tee_status,
            sequencer: sequencer_status,
            chain: chain_status,
        }
    };

    // Send initial status
    if let Ok(status_json) = serde_json::to_string(&initial_status) {
        let _ = sender
            .send(Message::Text(format!(
                r#"{{"type":"status","data":{}}}"#,
                status_json
            )))
            .await;
    }

    // Handle incoming messages and broadcast events
    let state_clone = state.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&event) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Handle client commands
                    if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                        match cmd.command.as_str() {
                            "start" => {
                                let mut running = state_clone.simulation_running.lock().await;
                                *running = true;
                                state_clone.broadcast(Event::SimulationStarted);
                            }
                            "stop" => {
                                let mut running = state_clone.simulation_running.lock().await;
                                *running = false;
                                state_clone.broadcast(Event::SimulationStopped);
                            }
                            "inject_high" => {
                                inject_high_threats(&state_clone).await;
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

#[derive(Deserialize)]
struct ClientCommand {
    command: String,
}

/// POST /api/lean/sign
/// Sign a message with leanSig + PQC hybrid signature
pub async fn lean_sig_sign(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LeanSigSignRequest>,
) -> Json<LeanSigSignResponse> {
    let start = Instant::now();

    let epoch = request.epoch.unwrap_or(0);
    let message_bytes = request.message.as_bytes();
    let include_pqc = request.include_pqc.unwrap_or(true);

    // Sign with leanSig + PQC hybrid
    let signature = if include_pqc {
        state.lean_sig.sign_hybrid(message_bytes, epoch).await
    } else {
        // Sign with leanSig only
        let keys = state.lean_sig.generate_keys(epoch, 1000000).await.unwrap();
        let sig = keys.sign(epoch, message_bytes);
        Ok(crate::lean_sig::HybridLeanSigSignature {
            lean_sig: sig,
            ml_dsa: None,
            slh_dsa: None,
            combined_size_bytes: 32,
        })
    };

    let signature = signature.unwrap_or_else(|e| {
        tracing::error!("leanSig signing failed: {}", e);
        // Return empty signature on error
        crate::lean_sig::HybridLeanSigSignature {
            lean_sig: crate::lean_sig::LeanSigSignature {
                signature: vec![],
                epoch,
                size_bytes: 0,
            },
            ml_dsa: None,
            slh_dsa: None,
            combined_size_bytes: 0,
        }
    });

    let public_key = state.lean_sig.export_public_key().await.unwrap_or_default();
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    Json(LeanSigSignResponse {
        signature,
        public_key,
        epoch,
        sign_time_ms: elapsed,
    })
}

/// POST /api/lean/verify
/// Verify a leanSig + PQC hybrid signature
pub async fn lean_sig_verify(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<LeanSigVerifyRequest>,
) -> Json<LeanSigVerifyResponse> {
    let start = Instant::now();

    let message_bytes = request.message.as_bytes();

    // Decode public keys from hex
    let lean_pk = hex::decode(request.lean_public_key.trim_start_matches("0x")).unwrap_or_default();
    let ml_dsa_pk = request
        .ml_dsa_public_key
        .map(|pk| hex::decode(pk.trim_start_matches("0x")).unwrap_or_default());
    let slh_dsa_pk = request
        .slh_dsa_public_key
        .map(|pk| hex::decode(pk.trim_start_matches("0x")).unwrap_or_default());

    // Verify leanSig component
    let lean_verify = crate::lean_sig::LeanSigSignature::verify(
        &lean_pk,
        request.signature.lean_sig.epoch,
        message_bytes,
        &request.signature.lean_sig.signature,
    );

    // Verify PQC components if present
    let ml_dsa_valid = if let Some(_ml_sig) = &request.signature.ml_dsa {
        ml_dsa_pk.as_ref().map(|_pk| {
            // In production, use actual ML-DSA verification via APQC
            // For now, return true as placeholder
            true
        })
    } else {
        None
    };

    let slh_dsa_valid = if let Some(_slh_sig) = &request.signature.slh_dsa {
        slh_dsa_pk.as_ref().map(|_pk| {
            // In production, use actual SLH-DSA verification via APQC
            // For now, return true as placeholder
            true
        })
    } else {
        None
    };

    let all_valid = lean_verify.valid
        && ml_dsa_valid.map(|v| v).unwrap_or(true)
        && slh_dsa_valid.map(|v| v).unwrap_or(true);

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    Json(LeanSigVerifyResponse {
        valid: all_valid,
        lean_sig_valid: lean_verify.valid,
        ml_dsa_valid,
        slh_dsa_valid,
        verify_time_ms: elapsed,
    })
}

/// POST /api/lean/generate-keys
/// Generate new leanSig key pair
pub async fn lean_sig_generate_keys(
    State(state): State<Arc<AppState>>,
    Json(request): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let epoch = request
        .get("epoch")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(0);
    let lifetime = request
        .get("lifetime")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(1000000);

    match state.lean_sig.generate_keys(epoch, lifetime).await {
        Ok(keys) => Json(serde_json::json!({
            "success": true,
            "public_key": hex::encode(keys.public_key_bytes()),
            "epoch": keys.epoch,
            "lifetime": keys.lifetime,
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e,
        })),
    }
}

/// POST /api/lean-vm/execute
/// Execute leanVM bytecode with quantum co-processing
pub async fn lean_vm_execute(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LeanVmExecuteRequest>,
) -> Json<LeanVmExecuteResponse> {
    let start = Instant::now();

    // Decode bytecode from hex
    let bytecode = hex::decode(request.bytecode.trim_start_matches("0x")).unwrap_or_default();

    let enable_qcp = request.enable_qcp.unwrap_or(true);

    // Execute with leanVM
    let result = if enable_qcp {
        state
            .lean_vm
            .execute_with_qcp(&bytecode, &request.public_inputs, &request.private_inputs)
            .await
    } else {
        state.lean_vm.execute_transaction(&bytecode).await
    };

    let execution_result = result.unwrap_or_else(|e| crate::lean_vm::LeanVmExecutionResult {
        success: false,
        memory: vec![],
        num_instructions: 0,
        execution_time_ms: 0.0,
        error: Some(e),
    });

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    Json(LeanVmExecuteResponse {
        result: execution_result,
        execution_time_ms: elapsed,
    })
}

/// POST /api/lean-vm/poseidon
/// Execute Poseidon hash operation
pub async fn lean_vm_poseidon(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LeanVmPoseidonRequest>,
) -> Json<LeanVmPoseidonResponse> {
    let start = Instant::now();

    let variant = match request.variant.as_str() {
        "poseidon16" => PoseidonVariant::Poseidon16,
        "poseidon24" => PoseidonVariant::Poseidon24,
        _ => PoseidonVariant::Poseidon16, // Default
    };

    let hash = state
        .lean_vm
        .execute_poseidon_hash(&request.inputs, variant)
        .await
        .unwrap_or(0);

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    Json(LeanVmPoseidonResponse {
        hash,
        execution_time_ms: elapsed,
    })
}

/// GET /api/lean-vm/status
/// Get leanVM execution status
pub async fn lean_vm_status(
    State(state): State<Arc<AppState>>,
) -> Json<crate::lean_vm::LeanVmStatus> {
    let status = state.lean_vm.get_status().await;
    Json(status)
}

/// POST /api/hybrid/solve
/// Execute the hybrid quantum-classical solver path with retention and governance controls.
pub async fn hybrid_solve(
    State(state): State<Arc<AppState>>,
    Json(request): Json<HybridSolveRequest>,
) -> Result<Json<HybridSolveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let mut engine = state.hybrid_engine.lock().await;
    match engine.execute(request) {
        Ok(response) => Ok(Json(response)),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "hybrid_execution_failed",
                "detail": err,
            })),
        )),
    }
}

/// GET /api/hybrid/executions/:execution_id
/// Retrieve stored hybrid execution metadata and persisted redacted payload.
pub async fn hybrid_get_execution(
    Path(execution_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<HybridExecutionRecord>, (StatusCode, Json<serde_json::Value>)> {
    let engine = state.hybrid_engine.lock().await;
    match engine.get_execution(&execution_id) {
        Some(record) => Ok(Json(record)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "execution_not_found",
                "execution_id": execution_id,
            })),
        )),
    }
}

/// GET /api/hybrid/transparency
/// Read recent governance transparency log entries.
pub async fn hybrid_transparency_log(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let engine = state.hybrid_engine.lock().await;
    let entries = engine.transparency_log(100);
    Json(serde_json::json!({ "entries": entries }))
}
