//! HTTP and WebSocket Handlers

use std::sync::Arc;
use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message}},
    response::IntoResponse,
    Json,
};
use futures::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};

use crate::state::{AppState, StatusResponse, QrmStatus, ApqcStatus, SequencerStatus, ChainStatus, Thresholds, Event, inject_high_threats};
use crate::qrm::{ThreatCategory, ThreatIndicator, QuantumEra};

/// GET /api/status
pub async fn get_status(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    // Acquire locks one at a time and release before next to avoid deadlocks
    let (risk, indicator_count, threshold_scheduled, threshold_emergency) = {
        let mut qrm = state.qrm.lock().await;
        let risk = qrm.calculate_risk();
        (risk, qrm.indicator_count(), qrm.threshold_scheduled, qrm.threshold_emergency)
    };
    
    let apqc_status = {
        let apqc = state.apqc.lock().await;
        ApqcStatus {
            signatures: apqc.active_signatures.iter().map(|s| s.name().to_string()).collect(),
            kems: apqc.active_kems.iter().map(|k| k.name().to_string()).collect(),
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
        source: payload.source.unwrap_or_else(|| "Manual Injection".to_string()),
        timestamp: chrono::Utc::now(),
        description: payload.description.unwrap_or_else(|| "Manually injected threat".to_string()),
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
    
    Json(SimulationResponse { status: "running".to_string() })
}

/// POST /api/simulation/stop
pub async fn stop_simulation(State(state): State<Arc<AppState>>) -> Json<SimulationResponse> {
    let mut running = state.simulation_running.lock().await;
    *running = false;
    state.broadcast(Event::SimulationStopped);
    
    Json(SimulationResponse { status: "stopped".to_string() })
}

#[derive(Serialize)]
pub struct SimulationResponse {
    status: String,
}

/// POST /api/inject_high_threat
pub async fn inject_high_threat(State(state): State<Arc<AppState>>) -> Json<SimulationResponse> {
    inject_high_threats(&state).await;
    Json(SimulationResponse { status: "injected".to_string() })
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
        let (risk, indicator_count, threshold_scheduled, threshold_emergency) = {
            let mut qrm = state.qrm.lock().await;
            let risk = qrm.calculate_risk();
            (risk, qrm.indicator_count(), qrm.threshold_scheduled, qrm.threshold_emergency)
        };
        
        let apqc_status = {
            let apqc = state.apqc.lock().await;
            ApqcStatus {
                signatures: apqc.active_signatures.iter().map(|s| s.name().to_string()).collect(),
                kems: apqc.active_kems.iter().map(|k| k.name().to_string()).collect(),
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
            sequencer: sequencer_status,
            chain: chain_status,
        }
    };
    
    // Send initial status
    if let Ok(status_json) = serde_json::to_string(&initial_status) {
        let _ = sender.send(Message::Text(format!(r#"{{"type":"status","data":{}}}"#, status_json))).await;
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
