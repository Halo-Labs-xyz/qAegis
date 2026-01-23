//! Phala Network TEE Sequencer
//! Quantum-resistant transaction ordering with asset protection and state migration
//!
//! Architecture:
//! - Runs as Phat Contract on Phala Cloud TEE workers
//! - Encrypted mempool with threshold encryption
//! - Quantum-resistant batch signing (ML-DSA-87 + SLH-DSA-256s)
//! - Asset protection for on-chain and off-chain data
//! - State migration for seamless upgrades

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;
use chrono::{DateTime, Utc};
use std::collections::{VecDeque, HashMap};

use crate::apqc::AdaptivePqcLayer;
use crate::qrm::{QuantumResistanceMonitor, RiskAssessment};

/// Phala TEE attestation (TDX/SEV)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhalaAttestation {
    pub worker_id: String,
    pub enclave_id: String,
    pub quote: Vec<u8>,              // TEE quote bytes
    pub quote_type: String,           // "TDX" or "SEV"
    pub mr_enclave: String,           // Measurement of enclave code
    pub mr_signer: String,            // Measurement of signer
    pub report_data: Vec<u8>,         // Custom report data (batch hash)
    pub timestamp: DateTime<Utc>,
    pub phala_verification: bool,     // Verified by Phala network
}

/// Asset protection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetProtection {
    pub asset_id: String,
    pub asset_type: AssetType,
    pub chain_id: Option<u64>,        // None for off-chain
    pub contract_address: Option<String>,
    pub encryption_key: Vec<u8>,      // Encrypted with TEE key
    pub access_policy: AccessPolicy,
    pub migration_state: MigrationState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetType {
    OnChainToken,
    OnChainNFT,
    OnChainData,
    OffChainDatabase,
    OffChainFile,
    OffChainStream,
    CrossChainBridge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub allowed_operations: Vec<String>,
    pub requires_pqc: bool,
    pub requires_tee: bool,
    pub risk_threshold: u32,           // Minimum risk score to trigger protection
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MigrationState {
    Active,                            // Currently active
    Preparing,                         // Preparing for migration
    Migrating,                         // Migration in progress
    Migrated,                          // Successfully migrated
    Rollback,                          // Rolled back to previous state
}

/// Encrypted transaction with asset context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedTransaction {
    pub tx_id: String,
    pub encrypted_data: Vec<u8>,       // Encrypted with TEE key
    pub asset_refs: Vec<String>,       // Referenced asset IDs
    pub priority_fee: u64,
    pub timestamp: DateTime<Utc>,
    pub risk_level: u32,               // Current QRM risk score
    pub requires_migration: bool,      // Flag for migration-aware ordering
}

/// Migration checkpoint for state preservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationCheckpoint {
    pub checkpoint_id: String,
    pub block_number: u64,
    pub state_hash: String,
    pub asset_snapshots: Vec<AssetSnapshot>,
    pub timestamp: DateTime<Utc>,
    pub pqc_signature: String,         // ML-DSA signature
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSnapshot {
    pub asset_id: String,
    pub state: Vec<u8>,                // Encrypted state
    pub metadata: HashMap<String, String>,
}

/// Quantum-resistant batch with intelligence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumResistantBatch {
    pub batch_id: String,
    pub block_number: u64,
    pub transactions: Vec<DecryptedTransaction>,
    pub ml_dsa_sig: String,
    pub slh_dsa_sig: String,
    pub attestation: PhalaAttestation,
    pub risk_assessment: RiskAssessment,
    pub asset_protections: Vec<AssetProtection>,
    pub migration_checkpoint: Option<MigrationCheckpoint>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedTransaction {
    pub tx_id: String,
    pub sender: String,
    pub data: String,
    pub asset_refs: Vec<String>,
    pub priority_fee: u64,
    pub timestamp: DateTime<Utc>,
}

/// Intelligence-based ordering strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntelligenceOrdering {
    RiskAware,                         // Order by risk level (high risk first)
    AssetProtection,                   // Prioritize protected assets
    MigrationAware,                    // Group migration-related txs
    Hybrid,                            // Combine multiple strategies
}

/// Phala TEE Sequencer
pub struct PhalaTeeSequencer {
    // Encrypted mempool (only decrypted inside TEE)
    encrypted_mempool: VecDeque<EncryptedTransaction>,
    
    // Asset registry
    asset_registry: HashMap<String, AssetProtection>,
    
    // Migration state
    migration_state: Option<MigrationCheckpoint>,
    migration_in_progress: bool,
    
    // Intelligence components
    qrm: QuantumResistanceMonitor,
    intelligence_mode: IntelligenceOrdering,
    
    // Batch management
    batches: Vec<QuantumResistantBatch>,
    current_block: u64,
    batch_size: usize,
    
    // Phala-specific
    worker_id: String,
    enclave_id: String,
    tee_platform: String,              // "TDX" or "SEV"
}

impl PhalaTeeSequencer {
    /// Initialize Phala TEE sequencer
    pub fn new(worker_id: String, enclave_id: String, tee_platform: String) -> Self {
        Self {
            encrypted_mempool: VecDeque::with_capacity(10000),
            asset_registry: HashMap::new(),
            migration_state: None,
            migration_in_progress: false,
            qrm: QuantumResistanceMonitor::new(),
            intelligence_mode: IntelligenceOrdering::Hybrid,
            batches: Vec::with_capacity(1000),
            current_block: 0,
            batch_size: 10,
            worker_id,
            enclave_id,
            tee_platform,
        }
    }

    /// Register asset for protection
    pub fn register_asset(&mut self, asset: AssetProtection) {
        self.asset_registry.insert(asset.asset_id.clone(), asset);
    }

    /// Submit encrypted transaction (from outside TEE)
    pub fn submit_encrypted(&mut self, encrypted_tx: EncryptedTransaction) {
        self.encrypted_mempool.push_back(encrypted_tx);
    }

    /// Decrypt and order transactions (inside TEE only)
    /// This function simulates TEE operation - in production, runs inside Phala enclave
    pub fn decrypt_and_order_intelligent(
        &mut self,
        tee_key: &[u8],  // TEE-protected decryption key
    ) -> Vec<DecryptedTransaction> {
        if self.encrypted_mempool.is_empty() {
            return vec![];
        }

        // Decrypt transactions (simulated - real implementation uses TEE key)
        let mut decrypted: Vec<(DecryptedTransaction, u32, Vec<String>)> = Vec::new();
        
        for enc_tx in self.encrypted_mempool.iter() {
            // In real TEE: decrypt with tee_key
            // For now, simulate decryption
            let decrypted_tx = DecryptedTransaction {
                tx_id: enc_tx.tx_id.clone(),
                sender: "0x".to_string() + &hex::encode(&enc_tx.encrypted_data[..8]),
                data: String::from_utf8_lossy(&enc_tx.encrypted_data).to_string(),
                asset_refs: enc_tx.asset_refs.clone(),
                priority_fee: enc_tx.priority_fee,
                timestamp: enc_tx.timestamp,
            };
            
            decrypted.push((
                decrypted_tx,
                enc_tx.risk_level,
                enc_tx.asset_refs.clone(),
            ));
        }

        // Clear processed transactions
        self.encrypted_mempool.clear();

        // Intelligence-based ordering
        let ordered = match self.intelligence_mode {
            IntelligenceOrdering::RiskAware => {
                self.order_by_risk(decrypted)
            }
            IntelligenceOrdering::AssetProtection => {
                self.order_by_asset_protection(decrypted)
            }
            IntelligenceOrdering::MigrationAware => {
                self.order_by_migration(decrypted)
            }
            IntelligenceOrdering::Hybrid => {
                self.order_hybrid(decrypted)
            }
        };

        ordered.into_iter().take(self.batch_size).collect()
    }

    /// Order by risk level (high risk first for faster protection)
    fn order_by_risk(
        &self,
        mut txs: Vec<(DecryptedTransaction, u32, Vec<String>)>,
    ) -> Vec<DecryptedTransaction> {
        txs.sort_by(|a, b| b.1.cmp(&a.1)); // Descending risk
        txs.into_iter().map(|(tx, _, _)| tx).collect()
    }

    /// Order by asset protection priority
    fn order_by_asset_protection(
        &self,
        mut txs: Vec<(DecryptedTransaction, u32, Vec<String>)>,
    ) -> Vec<DecryptedTransaction> {
        txs.sort_by(|a, b| {
            let a_protected = a.2.iter()
                .any(|asset_id| self.asset_registry.contains_key(asset_id));
            let b_protected = b.2.iter()
                .any(|asset_id| self.asset_registry.contains_key(asset_id));
            
            match (a_protected, b_protected) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.1.cmp(&b.1), // Fallback to risk
            }
        });
        txs.into_iter().map(|(tx, _, _)| tx).collect()
    }

    /// Order by migration requirements
    fn order_by_migration(
        &self,
        mut txs: Vec<(DecryptedTransaction, u32, Vec<String>)>,
    ) -> Vec<DecryptedTransaction> {
        if self.migration_in_progress {
            // Group migration-related transactions
            txs.sort_by(|a, b| {
                let a_migration = a.0.tx_id.contains("migration");
                let b_migration = b.0.tx_id.contains("migration");
                
                match (a_migration, b_migration) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.1.cmp(&b.1),
                }
            });
        } else {
            txs.sort_by(|a, b| b.1.cmp(&a.1));
        }
        txs.into_iter().map(|(tx, _, _)| tx).collect()
    }

    /// Hybrid ordering: combines risk, asset protection, and migration
    fn order_hybrid(
        &self,
        mut txs: Vec<(DecryptedTransaction, u32, Vec<String>)>,
    ) -> Vec<DecryptedTransaction> {
        // Score each transaction
        txs.sort_by(|a, b| {
            let a_score = self.calculate_priority_score(&a.0, a.1, &a.2);
            let b_score = self.calculate_priority_score(&b.0, b.1, &b.2);
            b_score.cmp(&a_score)
        });
        txs.into_iter().map(|(tx, _, _)| tx).collect()
    }

    /// Calculate priority score for hybrid ordering
    fn calculate_priority_score(
        &self,
        tx: &DecryptedTransaction,
        risk: u32,
        asset_refs: &[String],
    ) -> u64 {
        let mut score = risk as u64 * 100;
        
        // Asset protection bonus
        for asset_id in asset_refs {
            if let Some(asset) = self.asset_registry.get(asset_id) {
                if asset.access_policy.requires_pqc {
                    score += 1000;
                }
                if asset.access_policy.requires_tee {
                    score += 2000;
                }
            }
        }
        
        // Migration bonus
        if self.migration_in_progress && tx.tx_id.contains("migration") {
            score += 5000;
        }
        
        // Priority fee bonus
        score += tx.priority_fee as u64;
        
        score
    }

    /// Create quantum-resistant batch with intelligence
    pub async fn create_quantum_batch(
        &mut self,
        apqc: &mut AdaptivePqcLayer,
        tee_key: &[u8],
    ) -> Option<QuantumResistantBatch> {
        // Get current risk assessment
        let risk = self.qrm.calculate_risk();
        
        // Decrypt and order transactions
        let ordered_txs = self.decrypt_and_order_intelligent(tee_key);
        
        if ordered_txs.is_empty() {
            return None;
        }

        // Collect asset protections for this batch
        let mut batch_assets = Vec::new();
        for tx in &ordered_txs {
            for asset_id in &tx.asset_refs {
                if let Some(asset) = self.asset_registry.get(asset_id) {
                    if !batch_assets.iter().any(|a: &AssetProtection| a.asset_id == asset.asset_id) {
                        batch_assets.push(asset.clone());
                    }
                }
            }
        }

        // Create batch data
        let batch_data = serde_json::to_vec(&ordered_txs).unwrap_or_default();
        
        let mut hasher = Sha256::new();
        hasher.update(&batch_data);
        hasher.update(&self.current_block.to_be_bytes());
        let batch_id = hex::encode(&hasher.finalize());

        // Sign with dual PQC
        let signatures = apqc.sign_dual(&batch_data).await;

        // Generate Phala attestation
        let attestation = self.generate_phala_attestation(&batch_id);

        // Create migration checkpoint if needed
        let checkpoint = if self.migration_in_progress {
            Some(self.create_migration_checkpoint(&ordered_txs, apqc).await)
        } else {
            None
        };

        let batch = QuantumResistantBatch {
            batch_id,
            block_number: self.current_block,
            transactions: ordered_txs,
            ml_dsa_sig: signatures.ml_dsa.signature,
            slh_dsa_sig: signatures.slh_dsa.signature,
            attestation,
            risk_assessment: risk,
            asset_protections: batch_assets,
            migration_checkpoint: checkpoint,
            timestamp: Utc::now(),
        };

        self.batches.push(batch.clone());
        self.current_block += 1;

        Some(batch)
    }

    /// Generate Phala TEE attestation
    fn generate_phala_attestation(&self, batch_id: &str) -> PhalaAttestation {
        let mut hasher = Sha256::new();
        hasher.update(batch_id.as_bytes());
        hasher.update(&self.current_block.to_be_bytes());
        hasher.update(self.enclave_id.as_bytes());
        let report_data = hasher.finalize().to_vec();

        let mut mrenclave_hasher = Sha256::new();
        mrenclave_hasher.update(b"QuantumAegis-Phala-Enclave");
        mrenclave_hasher.update(self.enclave_id.as_bytes());
        let mr_enclave = hex::encode(&mrenclave_hasher.finalize()[..16]);

        let mut mrsigner_hasher = Sha256::new();
        mrsigner_hasher.update(b"QuantumAegis-Signer");
        let mr_signer = hex::encode(&mrsigner_hasher.finalize()[..16]);

        // Simulated quote (in production, get from Phala TEE)
        let quote = report_data.clone();

        PhalaAttestation {
            worker_id: self.worker_id.clone(),
            enclave_id: self.enclave_id.clone(),
            quote,
            quote_type: self.tee_platform.clone(),
            mr_enclave,
            mr_signer,
            report_data,
            timestamp: Utc::now(),
            phala_verification: true,
        }
    }

    /// Create migration checkpoint
    async fn create_migration_checkpoint(
        &self,
        txs: &[DecryptedTransaction],
        apqc: &mut AdaptivePqcLayer,
    ) -> MigrationCheckpoint {
        // Snapshot asset states
        let mut snapshots = Vec::new();
        for tx in txs {
            for asset_id in &tx.asset_refs {
                if let Some(asset) = self.asset_registry.get(asset_id) {
                    let mut metadata = HashMap::new();
                    metadata.insert("asset_type".to_string(), format!("{:?}", asset.asset_type));
                    metadata.insert("chain_id".to_string(), asset.chain_id.map(|c| c.to_string()).unwrap_or_default());
                    
                    snapshots.push(AssetSnapshot {
                        asset_id: asset_id.clone(),
                        state: asset.encryption_key.clone(), // Encrypted state
                        metadata,
                    });
                }
            }
        }

        let checkpoint_data = serde_json::to_vec(&snapshots).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&checkpoint_data);
        hasher.update(&self.current_block.to_be_bytes());
        let state_hash = hex::encode(&hasher.finalize());

        // Sign checkpoint with PQC
        let sig = apqc.sign_dual(&checkpoint_data).await;

        MigrationCheckpoint {
            checkpoint_id: format!("checkpoint_{}", self.current_block),
            block_number: self.current_block,
            state_hash,
            asset_snapshots: snapshots,
            timestamp: Utc::now(),
            pqc_signature: sig.ml_dsa.signature,
        }
    }

    /// Start migration process
    pub fn start_migration(&mut self) {
        self.migration_in_progress = true;
    }

    /// Complete migration
    pub fn complete_migration(&mut self, checkpoint: MigrationCheckpoint) {
        self.migration_state = Some(checkpoint);
        self.migration_in_progress = false;
    }

    /// Get asset protection status
    pub fn get_asset_protection(&self, asset_id: &str) -> Option<&AssetProtection> {
        self.asset_registry.get(asset_id)
    }

    /// Update QRM with new threat indicator
    pub fn update_threat(&mut self, indicator: crate::qrm::ThreatIndicator) {
        self.qrm.add_indicator(indicator);
    }

    /// Get recent batches
    pub fn get_recent_batches(&self, count: usize) -> Vec<QuantumResistantBatch> {
        self.batches.iter().rev().take(count).cloned().collect()
    }
}

impl Default for PhalaTeeSequencer {
    fn default() -> Self {
        Self::new(
            "worker_0".to_string(),
            "enclave_0".to_string(),
            "TDX".to_string(),
        )
    }
}
