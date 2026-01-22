//! Chain State
//! Manages blockchain state and block production

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

use crate::qrm::RiskAssessment;
use crate::sequencer::Batch;

/// Algorithm set configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmSet {
    pub signatures: Vec<String>,
    pub kems: Vec<String>,
}

impl Default for AlgorithmSet {
    fn default() -> Self {
        Self {
            signatures: vec!["ML-DSA-87".to_string(), "SLH-DSA-256s".to_string()],
            kems: vec!["ML-KEM-1024".to_string(), "HQC-256".to_string()],
        }
    }
}

/// A block in the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: u64,
    pub batch_id: String,
    pub tx_count: usize,
    pub timestamp: DateTime<Utc>,
    pub attestation_valid: bool,
    pub risk_score: u32,
    pub algorithms: AlgorithmSet,
}

/// Pending rotation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingRotation {
    pub new_set: AlgorithmSet,
    pub effective_block: u64,
}

/// Chain state manager
pub struct ChainState {
    blocks: VecDeque<Block>,
    pub current_height: u64,
    pub algorithm_set: AlgorithmSet,
    pub risk_score: u32,
    pub pending_rotation: Option<PendingRotation>,
    max_blocks: usize,
}

impl ChainState {
    pub fn new() -> Self {
        Self {
            blocks: VecDeque::with_capacity(1000),
            current_height: 0,
            algorithm_set: AlgorithmSet::default(),
            risk_score: 0,
            pending_rotation: None,
            max_blocks: 1000,
        }
    }

    /// Commit a batch as a new block
    pub fn commit_batch(&mut self, batch: &Batch, risk_assessment: &RiskAssessment) -> Block {
        let block = Block {
            height: self.current_height,
            batch_id: batch.batch_id.clone(),
            tx_count: batch.transactions.len(),
            timestamp: batch.timestamp,
            attestation_valid: true,
            risk_score: risk_assessment.score,
            algorithms: self.algorithm_set.clone(),
        };

        self.blocks.push_back(block.clone());
        while self.blocks.len() > self.max_blocks {
            self.blocks.pop_front();
        }

        self.current_height += 1;
        self.risk_score = risk_assessment.score;

        block
    }

    /// Get recent blocks
    pub fn get_recent_blocks(&self, count: usize) -> Vec<Block> {
        self.blocks.iter().rev().take(count).cloned().collect()
    }

    /// Schedule algorithm rotation
    pub fn schedule_rotation(&mut self, effective_block: u64) {
        self.pending_rotation = Some(PendingRotation {
            new_set: AlgorithmSet::default(), // In real impl, would be new algorithms
            effective_block,
        });
    }

    /// Check and execute pending rotation
    pub fn check_rotation(&mut self) -> bool {
        if let Some(ref rotation) = self.pending_rotation {
            if self.current_height >= rotation.effective_block {
                self.algorithm_set = rotation.new_set.clone();
                self.pending_rotation = None;
                return true;
            }
        }
        false
    }
}

impl Default for ChainState {
    fn default() -> Self {
        Self::new()
    }
}
