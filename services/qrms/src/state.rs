//! Application State
//! Shared state and simulation loop

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use crate::aegis_tee::AegisTeeSequencer;
use crate::apqc::AdaptivePqcLayer;
use crate::chain::{Block, ChainState};
use crate::hybrid_engine::HybridExecutionEngine;
use crate::lean_sig::LeanSigIntegration;
use crate::lean_vm::LeanVmIntegration;
use crate::qrm::{
    QuantumEra, QuantumResistanceMonitor, RiskAssessment, RiskRecommendation, ThreatCategory,
    ThreatIndicator,
};
use crate::qvm::{QuantumProcessor, QvmConfig, QvmProtocolStack};
use crate::sequencer::{Batch, TeeSequencer, Transaction};

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
    TxsOrdered { count: usize, txs: Vec<Transaction> },
    #[serde(rename = "batch_created")]
    BatchCreated { batch: Batch, block: Block },
    #[serde(rename = "rotation_scheduled")]
    RotationScheduled { effective_block: u64 },
    #[serde(rename = "rotation_executed")]
    RotationExecuted { rotation_type: String },
    #[serde(rename = "simulation_started")]
    SimulationStarted,
    #[serde(rename = "simulation_stopped")]
    SimulationStopped,
    #[serde(rename = "qvm_assessment")]
    QvmAssessment {
        grover_threats: Vec<crate::qvm::GroverThreatAssessment>,
        shor_threats: Vec<crate::qvm::ShorThreatAssessment>,
        composite_risk: u32,
    },
    #[serde(rename = "qvm_circuit_update")]
    QvmCircuitUpdate {
        circuit: crate::qvm::QuantumCircuit,
        result: Option<crate::qvm::CircuitResult>,
    },
}

/// Shared application state
pub struct AppState {
    pub qvm: Mutex<QvmProtocolStack>,
    pub qrm: Mutex<QuantumResistanceMonitor>,
    pub apqc: Mutex<AdaptivePqcLayer>,
    pub aegis_tee: Mutex<AegisTeeSequencer>,
    pub sequencer: Mutex<TeeSequencer>,
    pub chain: Mutex<ChainState>,
    pub simulation_running: Mutex<bool>,
    pub event_tx: broadcast::Sender<Event>,
    pub lean_sig: Arc<LeanSigIntegration>,
    pub lean_vm: Arc<LeanVmIntegration>,
    pub hybrid_engine: Mutex<HybridExecutionEngine>,
}

impl AppState {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        // Initialize QVM Protocol Stack
        // Check for Azure Quantum credentials (prefers Quantinuum)
        let azure_resource_id = std::env::var("AZURE_QUANTUM_RESOURCE_ID").ok();
        let azure_location = std::env::var("AZURE_QUANTUM_LOCATION").ok();
        let use_azure = azure_resource_id.is_some() && azure_location.is_some();

        // Legacy IonQ direct API (fallback)
        let ionq_api_key = std::env::var("IONQ_API_KEY").ok();
        let use_ionq = ionq_api_key.is_some() && !use_azure;

        let processor = if use_azure {
            QuantumProcessor::QuantinuumAPIV // Default to API validator when Azure Quantum is available
        } else if use_ionq {
            QuantumProcessor::IonQAria // Fallback to IonQ if available
        } else {
            QuantumProcessor::WillowPink // Default to simulator
        };

        let qvm_config = QvmConfig {
            processor,
            auto_era_transition: true,
            azure_quantum_resource_id: azure_resource_id.clone(),
            azure_quantum_location: azure_location.clone(),
            ionq_api_key: ionq_api_key.clone(),
            use_ionq_hardware: use_azure || use_ionq,
            ..Default::default()
        };
        let qvm = QvmProtocolStack::new(qvm_config);

        // Initialize Aegis-TEE Sequencer
        let aegis_tee = AegisTeeSequencer::default();

        // Initialize APQC
        let apqc = Mutex::new(AdaptivePqcLayer::new());

        // Initialize LeanSig integration (needs Arc for sharing)
        let apqc_for_lean_sig = Arc::new(Mutex::new(AdaptivePqcLayer::new()));
        let lean_sig = Arc::new(LeanSigIntegration::new(apqc_for_lean_sig.clone()));

        // Initialize LeanVM integration
        let lean_vm = Arc::new(LeanVmIntegration::new(
            apqc_for_lean_sig.clone(),
            lean_sig.clone(),
        ));

        Self {
            qvm: Mutex::new(qvm),
            qrm: Mutex::new(QuantumResistanceMonitor::new()),
            apqc,
            aegis_tee: Mutex::new(aegis_tee),
            sequencer: Mutex::new(TeeSequencer::new()),
            chain: Mutex::new(ChainState::new()),
            simulation_running: Mutex::new(false),
            event_tx,
            lean_sig,
            lean_vm,
            hybrid_engine: Mutex::new(HybridExecutionEngine::new(
                std::env::var("HYBRID_AUDIT_LOG_PATH")
                    .unwrap_or_else(|_| "storage/hybrid_transparency.log".to_string()),
            )),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.event_tx.subscribe()
    }

    pub fn broadcast(&self, event: Event) {
        let _ = self.event_tx.send(event);
    }
}

/// Periodic purge loop for transient hybrid execution artifacts.
pub async fn run_hybrid_purge(state: Arc<AppState>) {
    loop {
        {
            let mut engine = state.hybrid_engine.lock().await;
            let purged = engine.purge_transient();
            if purged > 0 {
                tracing::info!("Purged {} transient hybrid execution records", purged);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

/// Status response structure
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub qvm: Option<QvmStatus>,
    pub qrm: QrmStatus,
    pub apqc: ApqcStatus,
    pub aegis_tee: Option<AegisTeeStatus>,
    pub sequencer: SequencerStatus,
    pub chain: ChainStatus,
}

#[derive(Debug, Serialize)]
pub struct QvmStatus {
    pub processor: String,
    pub current_era: String,
    pub qrm_risk_score: u32,
    pub oracle_risk_score: u32,
    pub assessments_count: u64,
    pub era_transitions: u64,
    pub threat_indicators_count: usize,
    pub recommended_algorithms: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AegisTeeStatus {
    pub platform: String,
    pub mrenclave: String,
    pub mempool_size: usize,
    pub asset_protection_enabled: bool,
    pub phala_redundancy_enabled: bool,
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
    let mut qvm_assessment_counter = 0u64;

    loop {
        // Check if simulation should run
        {
            let running = state.simulation_running.lock().await;
            if !*running {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }
        }

        // 0. Run QVM Oracle assessment periodically (every 10 iterations = ~20 seconds)
        if qvm_assessment_counter % 10 == 0 {
            // Run QVM assessment with circuit execution
            let (qvm_assessment, grover_circuit_update, bell_circuit_update) = {
                let mut qvm = state.qvm.lock().await;

                // Run a Grover circuit for symmetric threat assessment
                let grover_circuit = crate::qvm::build_grover_circuit(4, 2); // 4 qubits, 2 iterations
                let grover_result = if let Some(ref ionq) = qvm.ionq_backend {
                    // Use IonQ hardware if available
                    ionq.run_circuit(&grover_circuit, 1000, "qpu")
                        .unwrap_or_else(|e| {
                            tracing::warn!(
                                "IonQ execution failed: {}, falling back to simulator",
                                e
                            );
                            qvm.oracle.simulator_mut().run(&grover_circuit, 1000)
                        })
                } else {
                    // Use local simulator
                    qvm.oracle.simulator_mut().run(&grover_circuit, 1000)
                };

                // Run a Bell state circuit for demonstration
                let bell_circuit = crate::qvm::build_bell_state_circuit();
                let bell_result = if let Some(ref ionq) = qvm.ionq_backend {
                    // Use IonQ hardware if available
                    ionq.run_circuit(&bell_circuit, 1000, "qpu")
                        .unwrap_or_else(|e| {
                            tracing::warn!(
                                "IonQ execution failed: {}, falling back to simulator",
                                e
                            );
                            qvm.oracle.simulator_mut().run(&bell_circuit, 1000)
                        })
                } else {
                    // Use local simulator
                    qvm.oracle.simulator_mut().run(&bell_circuit, 1000)
                };

                // Run full assessment
                let _risk_assessment = qvm.assess_and_update();

                // Bridge QVM threat indicators to Aegis-TEE
                let mut aegis_tee = state.aegis_tee.lock().await;
                qvm.bridge_to_tee(&mut aegis_tee);

                // Extract threat data from last_assessment for broadcasting
                let assessment_data = if let Some(ref assessment) = qvm.last_assessment {
                    Some((
                        assessment.grover_assessments.clone(),
                        assessment.shor_assessments.clone(),
                        assessment.composite_risk,
                    ))
                } else {
                    None
                };

                (
                    assessment_data,
                    (grover_circuit, grover_result),
                    (bell_circuit, bell_result),
                )
            };

            // Broadcast circuit updates (outside lock)
            state.broadcast(Event::QvmCircuitUpdate {
                circuit: grover_circuit_update.0,
                result: Some(grover_circuit_update.1),
            });

            state.broadcast(Event::QvmCircuitUpdate {
                circuit: bell_circuit_update.0,
                result: Some(bell_circuit_update.1),
            });

            // Broadcast QVM assessment results if available
            if let Some((grover_threats, shor_threats, composite_risk)) = qvm_assessment {
                state.broadcast(Event::QvmAssessment {
                    grover_threats,
                    shor_threats,
                    composite_risk,
                });
            }
        }
        qvm_assessment_counter += 1;

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
        (
            ThreatCategory::DigitalSignatures,
            "ECDSA/secp256k1",
            "Shor's algorithm breakthrough - practical ECDLP attack demonstrated",
        ),
        (
            ThreatCategory::DecryptionHndl,
            "HNDL Active Collection",
            "Nation-state HNDL campaign confirmed - urgent migration needed",
        ),
        (
            ThreatCategory::ConsensusAttacks,
            "PoS Validator Keys",
            "Validator key forgery technique published",
        ),
        (
            ThreatCategory::KeyManagement,
            "MPC/TSS Shares",
            "Threshold secret reconstruction vulnerability",
        ),
        (
            ThreatCategory::SmartContracts,
            "ecrecover Bypass",
            "On-chain signature verification attack demonstrated",
        ),
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
            era_relevance: QuantumEra::Nisq, // Imminent threat
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
