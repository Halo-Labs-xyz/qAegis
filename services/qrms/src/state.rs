//! Application State
//! Shared state and simulation loop

use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use serde::{Deserialize, Serialize};
use rand::Rng;

use crate::qrm::{QuantumResistanceMonitor, RiskRecommendation, ThreatIndicator, RiskAssessment, ThreatCategory, QuantumEra};
use crate::apqc::AdaptivePqcLayer;
use crate::sequencer::{TeeSequencer, Transaction, Batch};
use crate::chain::{ChainState, Block};

/// Events broadcast to WebSocket clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    #[serde(rename = "qrm_update")]
    QrmUpdate {
        indicator: ThreatIndicator,
        risk: RiskAssessment,
    },
    #[serde(rename = "tx_submitted")]
    TxSubmitted(Transaction),
    #[serde(rename = "txs_ordered")]
    TxsOrdered {
        count: usize,
        txs: Vec<Transaction>,
    },
    #[serde(rename = "batch_created")]
    BatchCreated {
        batch: Batch,
        block: Block,
    },
    #[serde(rename = "rotation_scheduled")]
    RotationScheduled {
        effective_block: u64,
    },
    #[serde(rename = "rotation_executed")]
    RotationExecuted {
        rotation_type: String,
    },
    #[serde(rename = "simulation_started")]
    SimulationStarted,
    #[serde(rename = "simulation_stopped")]
    SimulationStopped,
}

/// Shared application state
pub struct AppState {
    pub qrm: Mutex<QuantumResistanceMonitor>,
    pub apqc: Mutex<AdaptivePqcLayer>,
    pub sequencer: Mutex<TeeSequencer>,
    pub chain: Mutex<ChainState>,
    pub simulation_running: Mutex<bool>,
    pub event_tx: broadcast::Sender<Event>,
}

impl AppState {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        
        Self {
            qrm: Mutex::new(QuantumResistanceMonitor::new()),
            apqc: Mutex::new(AdaptivePqcLayer::new()),
            sequencer: Mutex::new(TeeSequencer::new()),
            chain: Mutex::new(ChainState::new()),
            simulation_running: Mutex::new(false),
            event_tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.event_tx.subscribe()
    }

    pub fn broadcast(&self, event: Event) {
        let _ = self.event_tx.send(event);
    }
}

/// Status response structure
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub qrm: QrmStatus,
    pub apqc: ApqcStatus,
    pub sequencer: SequencerStatus,
    pub chain: ChainStatus,
}

#[derive(Debug, Serialize)]
pub struct QrmStatus {
    pub risk_score: u32,
    pub recommendation: RiskRecommendation,
    pub indicator_count: usize,
    pub thresholds: Thresholds,
}

#[derive(Debug, Serialize)]
pub struct Thresholds {
    pub scheduled: u32,
    pub emergency: u32,
}

#[derive(Debug, Serialize)]
pub struct ApqcStatus {
    pub signatures: Vec<String>,
    pub kems: Vec<String>,
    pub rotation_pending: bool,
    pub rotation_block: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SequencerStatus {
    pub mempool_size: usize,
    pub ordered_queue: usize,
    pub batch_count: usize,
    pub tee_platform: String,
    pub mrenclave: String,
}

#[derive(Debug, Serialize)]
pub struct ChainStatus {
    pub height: u64,
    pub algorithm_set: crate::chain::AlgorithmSet,
    pub risk_score: u32,
}

/// Run the simulation loop
pub async fn run_simulation(state: Arc<AppState>) {
    let mut _tx_counter: u64 = 0;
    
    loop {
        // Check if simulation should run
        {
            let running = state.simulation_running.lock().await;
            if !*running {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }
        }

        // 1. Simulate QRM threat feed
        let (indicator, risk) = {
            let mut qrm = state.qrm.lock().await;
            let indicator = qrm.simulate_threat_feed();
            let risk = qrm.calculate_risk();
            (indicator, risk)
        };

        state.broadcast(Event::QrmUpdate {
            indicator,
            risk: risk.clone(),
        });

        // 2. Generate random transactions
        let tx_count = {
            let mut rng = rand::thread_rng();
            rng.gen_range(1..=3)
        };
        
        for _ in 0..tx_count {
            let (sender, data, fee) = {
                let mut rng = rand::thread_rng();
                (
                    format!("0x{:016x}", rng.gen::<u64>()),
                    format!("transfer({})", rng.gen_range(1..1000)),
                    rng.gen_range(1..100),
                )
            };
            
            let tx = Transaction::new(sender, data, fee);
            
            {
                let mut sequencer = state.sequencer.lock().await;
                let submitted = sequencer.submit_transaction(tx);
                state.broadcast(Event::TxSubmitted(submitted));
            }
            
            _tx_counter += 1;
        }

        // 3. Process transactions through sequencer
        let ordered = {
            let mut sequencer = state.sequencer.lock().await;
            sequencer.decrypt_and_order()
        };
        
        if !ordered.is_empty() {
            state.broadcast(Event::TxsOrdered {
                count: ordered.len(),
                txs: ordered,
            });
        }

        // 4. Create batch if enough transactions
        let should_create_batch = {
            let sequencer = state.sequencer.lock().await;
            sequencer.ordered_queue_size() >= sequencer.batch_size
        };

        if should_create_batch {
            let batch_result = {
                let mut sequencer = state.sequencer.lock().await;
                let mut apqc = state.apqc.lock().await;
                sequencer.create_batch(&mut apqc).await
            };

            if let Some(batch) = batch_result {
                let block = {
                    let mut chain = state.chain.lock().await;
                    chain.commit_batch(&batch, &risk)
                };

                state.broadcast(Event::BatchCreated { batch, block });
            }
        }

        // 5. Check for rotation
        let current_block = {
            let chain = state.chain.lock().await;
            chain.current_height
        };

        if risk.recommendation == RiskRecommendation::EmergencyRotation {
            let mut apqc = state.apqc.lock().await;
            apqc.execute_rotation().await;
            state.broadcast(Event::RotationExecuted {
                rotation_type: "emergency".to_string(),
            });
        } else if risk.recommendation == RiskRecommendation::ScheduleRotation {
            let mut apqc = state.apqc.lock().await;
            if !apqc.rotation_pending {
                let effective_block = current_block + 10;
                apqc.schedule_rotation(effective_block);
                state.broadcast(Event::RotationScheduled { effective_block });
            }
        }

        // Sleep between iterations
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

/// Inject high severity threats across multiple categories
pub async fn inject_high_threats(state: &AppState) {
    let mut qrm = state.qrm.lock().await;
    
    // Inject critical threats across key categories
    let critical_threats = [
        (ThreatCategory::DigitalSignatures, "ECDSA/secp256k1", "Shor's algorithm breakthrough - practical ECDLP attack demonstrated"),
        (ThreatCategory::DecryptionHndl, "HNDL Active Collection", "Nation-state HNDL campaign confirmed - urgent migration needed"),
        (ThreatCategory::ConsensusAttacks, "PoS Validator Keys", "Validator key forgery technique published"),
        (ThreatCategory::KeyManagement, "MPC/TSS Shares", "Threshold secret reconstruction vulnerability"),
        (ThreatCategory::SmartContracts, "ecrecover Bypass", "On-chain signature verification attack demonstrated"),
    ];
    
    for (category, sub_cat, desc) in critical_threats {
        let indicator = ThreatIndicator {
            category,
            sub_category: sub_cat.to_string(),
            severity: 0.95,
            confidence: 0.95,
            source: "Emergency Alert".to_string(),
            timestamp: chrono::Utc::now(),
            description: desc.to_string(),
            era_relevance: QuantumEra::Nisq,  // Imminent threat
            references: vec!["EMERGENCY-2026-001".to_string()],
        };
        qrm.add_indicator(indicator);
    }
    
    let risk = qrm.calculate_risk();
    
    // Get last indicator for event
    if let Some(indicator) = qrm.get_indicators().last().cloned() {
        state.broadcast(Event::QrmUpdate { indicator, risk });
    }
}
