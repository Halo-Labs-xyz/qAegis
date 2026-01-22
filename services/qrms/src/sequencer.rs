//! TEE Sequencer
//! Handles transaction ordering within a simulated TEE enclave

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use uuid::Uuid;

use crate::apqc::AdaptivePqcLayer;

/// Transaction status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TxStatus {
    Pending,
    Ordered,
    Signed,
    Committed,
}

/// A transaction in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub tx_id: String,
    pub sender: String,
    pub data: String,
    pub timestamp: DateTime<Utc>,
    pub priority_fee: u64,
    pub status: TxStatus,
}

impl Transaction {
    pub fn new(sender: String, data: String, priority_fee: u64) -> Self {
        Self {
            tx_id: format!("tx_{}", Uuid::new_v4().simple()),
            sender,
            data,
            timestamp: Utc::now(),
            priority_fee,
            status: TxStatus::Pending,
        }
    }
}

/// TEE attestation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeeAttestation {
    pub platform: String,
    pub mrenclave: String,
    pub mrsigner: String,
    pub report_data: String,
    pub nonce: String,
    pub timestamp: DateTime<Utc>,
    pub pqc_signed: bool,
}

/// A batch of transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub batch_id: String,
    pub transactions: Vec<Transaction>,
    pub ml_dsa_sig: String,
    pub slh_dsa_sig: String,
    pub attestation: TeeAttestation,
    pub timestamp: DateTime<Utc>,
}

/// Ordering mode for transactions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderingMode {
    Fcfs,           // First-come-first-served
    BatchAuction,   // Periodic batch with uniform price
}

/// TEE Sequencer
pub struct TeeSequencer {
    encrypted_mempool: VecDeque<Transaction>,
    ordered_queue: VecDeque<Transaction>,
    batches: Vec<Batch>,
    pub current_block: u64,
    pub batch_size: usize,
    pub ordering_mode: OrderingMode,
    pub tee_platform: String,
    pub mrenclave: String,
}

impl TeeSequencer {
    pub fn new() -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"QRMS-Enclave-v1");
        let mrenclave = hex::encode(&hasher.finalize()[..8]);

        Self {
            encrypted_mempool: VecDeque::with_capacity(1000),
            ordered_queue: VecDeque::with_capacity(1000),
            batches: Vec::with_capacity(1000),
            current_block: 0,
            batch_size: 5,
            ordering_mode: OrderingMode::Fcfs,
            tee_platform: "SGX".to_string(),
            mrenclave,
        }
    }

    /// Submit transaction to encrypted mempool
    pub fn submit_transaction(&mut self, mut tx: Transaction) -> Transaction {
        tx.status = TxStatus::Pending;
        self.encrypted_mempool.push_back(tx.clone());
        tx
    }

    /// Get mempool size
    pub fn mempool_size(&self) -> usize {
        self.encrypted_mempool.len()
    }

    /// Get ordered queue size
    pub fn ordered_queue_size(&self) -> usize {
        self.ordered_queue.len()
    }

    /// Get batch count
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    /// Get recent batches
    pub fn get_recent_batches(&self, count: usize) -> Vec<Batch> {
        self.batches.iter().rev().take(count).cloned().collect()
    }

    /// Decrypt and order transactions (simulate TEE operation)
    pub fn decrypt_and_order(&mut self) -> Vec<Transaction> {
        if self.encrypted_mempool.is_empty() {
            return vec![];
        }

        // Take up to batch_size transactions
        let mut to_order = Vec::with_capacity(self.batch_size);
        for _ in 0..self.batch_size {
            if let Some(tx) = self.encrypted_mempool.pop_front() {
                to_order.push(tx);
            } else {
                break;
            }
        }

        // Sort by timestamp (FCFS) or by priority fee (auction)
        match self.ordering_mode {
            OrderingMode::Fcfs => {
                to_order.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            }
            OrderingMode::BatchAuction => {
                to_order.sort_by(|a, b| b.priority_fee.cmp(&a.priority_fee));
            }
        }

        // Mark as ordered and add to queue
        for tx in &mut to_order {
            tx.status = TxStatus::Ordered;
            self.ordered_queue.push_back(tx.clone());
        }

        to_order
    }

    /// Create and sign a batch
    pub async fn create_batch(&mut self, apqc: &mut AdaptivePqcLayer) -> Option<Batch> {
        if self.ordered_queue.is_empty() {
            return None;
        }

        // Take transactions for batch
        let mut txs = Vec::with_capacity(self.batch_size);
        for _ in 0..self.batch_size {
            if let Some(mut tx) = self.ordered_queue.pop_front() {
                tx.status = TxStatus::Committed;
                txs.push(tx);
            } else {
                break;
            }
        }

        if txs.is_empty() {
            return None;
        }

        // Create batch data
        let batch_data = serde_json::to_vec(&txs).unwrap_or_default();
        
        let mut hasher = Sha256::new();
        hasher.update(&batch_data);
        let batch_id = hex::encode(&hasher.finalize()[..8]);

        // Sign with dual PQC (real implementation)
        let signatures = apqc.sign_dual(&batch_data).await;

        // Generate TEE attestation
        let attestation = self.generate_attestation(&batch_id);

        let batch = Batch {
            batch_id,
            transactions: txs,
            ml_dsa_sig: signatures.ml_dsa.signature,
            slh_dsa_sig: signatures.slh_dsa.signature,
            attestation,
            timestamp: Utc::now(),
        };

        self.batches.push(batch.clone());
        self.current_block += 1;

        Some(batch)
    }

    /// Generate mock TEE attestation
    fn generate_attestation(&self, batch_id: &str) -> TeeAttestation {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)).as_bytes());
        let nonce = hex::encode(&hasher.finalize()[..8]);

        let mut report_hasher = Sha256::new();
        report_hasher.update(batch_id.as_bytes());
        report_hasher.update(nonce.as_bytes());
        report_hasher.update(self.mrenclave.as_bytes());
        let report_data = hex::encode(&report_hasher.finalize()[..16]);

        let mut signer_hasher = Sha256::new();
        signer_hasher.update(b"QRMS-Signer");
        let mrsigner = hex::encode(&signer_hasher.finalize()[..8]);

        TeeAttestation {
            platform: self.tee_platform.clone(),
            mrenclave: self.mrenclave.clone(),
            mrsigner,
            report_data,
            nonce,
            timestamp: Utc::now(),
            pqc_signed: true,
        }
    }
}

impl Default for TeeSequencer {
    fn default() -> Self {
        Self::new()
    }
}
