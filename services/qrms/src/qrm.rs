//! Quantum Resistance Monitor (QRM)
//! Monitors threat landscape and calculates risk scores
//! 
//! Covers 12 threat categories per threat_taxonomy.md:
//! 1. Digital Signatures
//! 2. ZK Proof Forgery
//! 3. Decryption/HNDL
//! 4. Hash Reversal
//! 5. Consensus Attacks
//! 6. Cross-Chain/Bridge
//! 7. Key Management
//! 8. Network Layer
//! 9. MEV/Ordering
//! 10. Smart Contracts
//! 11. Side-Channel
//! 12. Migration/Agility

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use rand::Rng;

/// Expanded threat indicator categories (12 total)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ThreatCategory {
    // Cryptographic Threats
    DigitalSignatures,      // ECDSA, EdDSA, BLS, Multi-sig, HD Wallets
    ZkProofForgery,         // SNARKs, Plonk, Rollup state, Private tx
    DecryptionHndl,         // Encrypted mempools, P2P, HNDL active threat
    HashReversal,           // SHA-256, Keccak, Poseidon (low risk)
    
    // Infrastructure Threats
    ConsensusAttacks,       // PoS keys, VRF, Finality sigs
    CrossChainBridge,       // Light clients, Relayers, IBC, Rollup sequencer
    NetworkLayer,           // Node discovery, Gossip, TLS, Libp2p
    
    // Operational Threats
    KeyManagement,          // HD wallets, MPC/TSS, Rotation, Custody
    MevOrdering,            // Encrypted mempool bypass, PBS, Auctions
    SmartContracts,         // ecrecover, CREATE2, Access control, Governance
    
    // Implementation Threats
    SideChannel,            // Timing, Power, Cache, Fault injection, TEE
    MigrationAgility,       // Downgrade, Hybrid bypass, Incomplete migration
}

impl ThreatCategory {
    /// Weight for risk calculation (must sum to 1.0)
    pub fn weight(&self) -> f64 {
        match self {
            // Cryptographic - 38%
            Self::DigitalSignatures => 0.12,
            Self::ZkProofForgery => 0.10,
            Self::DecryptionHndl => 0.12,  // Elevated due to HNDL
            Self::HashReversal => 0.04,
            
            // Infrastructure - 24%
            Self::ConsensusAttacks => 0.10,
            Self::CrossChainBridge => 0.08,
            Self::NetworkLayer => 0.06,
            
            // Operational - 26%
            Self::KeyManagement => 0.10,
            Self::MevOrdering => 0.08,
            Self::SmartContracts => 0.08,
            
            // Implementation - 12%
            Self::SideChannel => 0.06,
            Self::MigrationAgility => 0.06,
        }
    }

    /// Severity multiplier based on quantum era
    pub fn era_multiplier(&self, era: QuantumEra) -> f64 {
        match (self, era) {
            // Digital signatures become critical with fault-tolerant QC
            (Self::DigitalSignatures, QuantumEra::PreQuantum) => 0.2,
            (Self::DigitalSignatures, QuantumEra::Nisq) => 0.6,
            (Self::DigitalSignatures, QuantumEra::FaultTolerant) => 1.0,
            
            // ZK proof forgery
            (Self::ZkProofForgery, QuantumEra::PreQuantum) => 0.2,
            (Self::ZkProofForgery, QuantumEra::Nisq) => 0.6,
            (Self::ZkProofForgery, QuantumEra::FaultTolerant) => 1.0,
            
            // HNDL is ALREADY ACTIVE
            (Self::DecryptionHndl, QuantumEra::PreQuantum) => 0.8, // Active threat!
            (Self::DecryptionHndl, QuantumEra::Nisq) => 0.9,
            (Self::DecryptionHndl, QuantumEra::FaultTolerant) => 1.0,
            
            // Hash reversal stays low
            (Self::HashReversal, _) => 0.3,
            
            // Consensus attacks
            (Self::ConsensusAttacks, QuantumEra::PreQuantum) => 0.2,
            (Self::ConsensusAttacks, QuantumEra::Nisq) => 0.6,
            (Self::ConsensusAttacks, QuantumEra::FaultTolerant) => 1.0,
            
            // Bridge attacks
            (Self::CrossChainBridge, QuantumEra::PreQuantum) => 0.3,
            (Self::CrossChainBridge, QuantumEra::Nisq) => 0.6,
            (Self::CrossChainBridge, QuantumEra::FaultTolerant) => 0.85,
            
            // Network layer
            (Self::NetworkLayer, QuantumEra::PreQuantum) => 0.3,
            (Self::NetworkLayer, QuantumEra::Nisq) => 0.6,
            (Self::NetworkLayer, QuantumEra::FaultTolerant) => 0.85,
            
            // Key management critical
            (Self::KeyManagement, QuantumEra::PreQuantum) => 0.3,
            (Self::KeyManagement, QuantumEra::Nisq) => 0.6,
            (Self::KeyManagement, QuantumEra::FaultTolerant) => 1.0,
            
            // MEV ordering
            (Self::MevOrdering, QuantumEra::PreQuantum) => 0.3,
            (Self::MevOrdering, QuantumEra::Nisq) => 0.6,
            (Self::MevOrdering, QuantumEra::FaultTolerant) => 0.85,
            
            // Smart contracts critical
            (Self::SmartContracts, QuantumEra::PreQuantum) => 0.2,
            (Self::SmartContracts, QuantumEra::Nisq) => 0.6,
            (Self::SmartContracts, QuantumEra::FaultTolerant) => 1.0,
            
            // Side-channel already medium risk
            (Self::SideChannel, QuantumEra::PreQuantum) => 0.5,
            (Self::SideChannel, QuantumEra::Nisq) => 0.75,
            (Self::SideChannel, QuantumEra::FaultTolerant) => 0.85,
            
            // Migration attacks grow with urgency
            (Self::MigrationAgility, QuantumEra::PreQuantum) => 0.3,
            (Self::MigrationAgility, QuantumEra::Nisq) => 0.75,
            (Self::MigrationAgility, QuantumEra::FaultTolerant) => 0.85,
        }
    }

    /// All categories for iteration
    pub fn all() -> &'static [ThreatCategory] {
        &[
            Self::DigitalSignatures,
            Self::ZkProofForgery,
            Self::DecryptionHndl,
            Self::HashReversal,
            Self::ConsensusAttacks,
            Self::CrossChainBridge,
            Self::NetworkLayer,
            Self::KeyManagement,
            Self::MevOrdering,
            Self::SmartContracts,
            Self::SideChannel,
            Self::MigrationAgility,
        ]
    }

    /// Random category for simulation (weighted by importance)
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let roll: f64 = rng.gen();
        
        // Weighted random selection
        let mut cumulative = 0.0;
        for cat in Self::all() {
            cumulative += cat.weight();
            if roll < cumulative {
                return *cat;
            }
        }
        Self::DigitalSignatures
    }
    
    /// Display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::DigitalSignatures => "Digital Signatures",
            Self::ZkProofForgery => "ZK Proof Forgery",
            Self::DecryptionHndl => "Decryption/HNDL",
            Self::HashReversal => "Hash Reversal",
            Self::ConsensusAttacks => "Consensus Attacks",
            Self::CrossChainBridge => "Cross-Chain Bridge",
            Self::NetworkLayer => "Network Layer",
            Self::KeyManagement => "Key Management",
            Self::MevOrdering => "MEV/Ordering",
            Self::SmartContracts => "Smart Contracts",
            Self::SideChannel => "Side-Channel",
            Self::MigrationAgility => "Migration/Agility",
        }
    }
}

/// Quantum computing era for severity scaling
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuantumEra {
    PreQuantum,     // Current: no fault-tolerant QC
    Nisq,           // Near-term: noisy intermediate-scale quantum
    FaultTolerant,  // Future: cryptographically-relevant QC
}

/// A single threat indicator from monitoring sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    pub category: ThreatCategory,
    pub sub_category: String,       // Specific threat type
    pub severity: f64,              // 0.0 - 1.0
    pub confidence: f64,            // 0.0 - 1.0
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub era_relevance: QuantumEra,  // When this threat becomes critical
    pub references: Vec<String>,    // arXiv, CVE, etc.
}

/// Risk recommendation based on score
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskRecommendation {
    Continue,
    MonitorClosely,
    ScheduleRotation,
    EmergencyRotation,
}

/// Category-specific risk breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRisk {
    pub category: ThreatCategory,
    pub score: u32,
    pub indicator_count: usize,
    pub top_threats: Vec<String>,
}

/// Risk assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub score: u32,  // 0-10000 basis points
    pub recommendation: RiskRecommendation,
    pub category_breakdown: Vec<CategoryRisk>,
    pub indicators: Vec<ThreatIndicator>,
    pub current_era: QuantumEra,
    pub timestamp: DateTime<Utc>,
}

/// Quantum Resistance Monitor
pub struct QuantumResistanceMonitor {
    indicators: VecDeque<ThreatIndicator>,
    risk_history: VecDeque<RiskAssessment>,
    pub threshold_scheduled: u32,
    pub threshold_emergency: u32,
    pub current_era: QuantumEra,
    max_indicators: usize,
    max_history: usize,
}

impl QuantumResistanceMonitor {
    pub fn new() -> Self {
        Self {
            indicators: VecDeque::with_capacity(200),
            risk_history: VecDeque::with_capacity(500),
            threshold_scheduled: 6000,
            threshold_emergency: 9000,
            current_era: QuantumEra::PreQuantum,
            max_indicators: 200,
            max_history: 500,
        }
    }

    /// Add a new threat indicator
    pub fn add_indicator(&mut self, indicator: ThreatIndicator) {
        self.indicators.push_back(indicator);
        while self.indicators.len() > self.max_indicators {
            self.indicators.pop_front();
        }
    }

    /// Get recent indicators
    pub fn get_indicators(&self) -> Vec<ThreatIndicator> {
        self.indicators.iter().cloned().collect()
    }

    /// Get risk history
    pub fn get_risk_history(&self) -> Vec<RiskAssessment> {
        self.risk_history.iter().cloned().collect()
    }

    /// Get indicator count
    pub fn indicator_count(&self) -> usize {
        self.indicators.len()
    }

    /// Calculate category-specific risk
    fn calculate_category_risk(&self, category: ThreatCategory, recent: &[ThreatIndicator]) -> CategoryRisk {
        let cat_indicators: Vec<_> = recent.iter()
            .filter(|i| i.category == category)
            .collect();
        
        if cat_indicators.is_empty() {
            return CategoryRisk {
                category,
                score: 0,
                indicator_count: 0,
                top_threats: vec![],
            };
        }

        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;
        let mut threats: Vec<String> = vec![];

        for ind in &cat_indicators {
            let era_mult = category.era_multiplier(self.current_era);
            let w = ind.confidence * era_mult;
            weighted_sum += ind.severity * w;
            weight_total += w;
            threats.push(ind.sub_category.clone());
        }

        let score = if weight_total > 0.0 {
            ((weighted_sum / weight_total) * 10000.0) as u32
        } else {
            0
        };

        CategoryRisk {
            category,
            score,
            indicator_count: cat_indicators.len(),
            top_threats: threats.into_iter().take(3).collect(),
        }
    }

    /// Calculate current risk score
    pub fn calculate_risk(&mut self) -> RiskAssessment {
        if self.indicators.is_empty() {
            return RiskAssessment {
                score: 0,
                recommendation: RiskRecommendation::Continue,
                category_breakdown: vec![],
                indicators: vec![],
                current_era: self.current_era,
                timestamp: Utc::now(),
            };
        }

        // Use recent indicators (last 50)
        let recent: Vec<_> = self.indicators.iter().rev().take(50).cloned().collect();

        // Calculate per-category risk
        let category_risks: Vec<CategoryRisk> = ThreatCategory::all()
            .iter()
            .map(|cat| self.calculate_category_risk(*cat, &recent))
            .collect();

        // Weighted aggregate score
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for cat_risk in &category_risks {
            let w = cat_risk.category.weight();
            weighted_sum += (cat_risk.score as f64) * w;
            weight_total += w;
        }

        let score = if weight_total > 0.0 {
            (weighted_sum / weight_total) as u32
        } else {
            0
        };

        let recommendation = if score >= self.threshold_emergency {
            RiskRecommendation::EmergencyRotation
        } else if score >= self.threshold_scheduled {
            RiskRecommendation::ScheduleRotation
        } else if score >= self.threshold_scheduled / 2 {
            RiskRecommendation::MonitorClosely
        } else {
            RiskRecommendation::Continue
        };

        let assessment = RiskAssessment {
            score,
            recommendation,
            category_breakdown: category_risks,
            indicators: recent.into_iter().take(10).collect(),
            current_era: self.current_era,
            timestamp: Utc::now(),
        };

        self.risk_history.push_back(assessment.clone());
        while self.risk_history.len() > self.max_history {
            self.risk_history.pop_front();
        }

        assessment
    }

    /// Simulate a threat feed update
    pub fn simulate_threat_feed(&mut self) -> ThreatIndicator {
        let mut rng = rand::thread_rng();
        
        let sources = [
            "arXiv", "NIST", "IACR", "IBM Quantum", "Google AI", 
            "CVE Database", "GitHub Security", "Industry Report"
        ];
        
        let category = ThreatCategory::random();

        let (sub_category, descriptions) = match category {
            ThreatCategory::DigitalSignatures => {
                let subs = [
                    ("ECDSA/secp256k1", vec![
                        "Shor's algorithm optimization for ECDLP",
                        "Transaction signature forgery technique",
                        "New quantum circuit for secp256k1"
                    ]),
                    ("BLS Signatures", vec![
                        "Pairing-based crypto quantum attack",
                        "Aggregate signature vulnerability",
                        "Consensus signature attack vector"
                    ]),
                    ("Multi-sig/Threshold", vec![
                        "Partial key recovery enabling quorum bypass",
                        "Combined Shor attacks on TSS shares",
                        "Threshold signature reconstruction"
                    ]),
                    ("HD Wallet Derivation", vec![
                        "BIP-32 EC derivation exploitation",
                        "Master seed recovery technique",
                        "Child key derivation attack"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
            ThreatCategory::ZkProofForgery => {
                let subs = [
                    ("zk-SNARKs/Groth16", vec![
                        "Groth16 trusted setup vulnerability",
                        "Pairing assumption quantum break",
                        "False state transition proof"
                    ]),
                    ("Plonk/Kate", vec![
                        "Polynomial commitment forgery",
                        "EC-based commitment attack",
                        "Kate commitment quantum vulnerability"
                    ]),
                    ("zk-Rollup State", vec![
                        "Fraudulent L2 state claim technique",
                        "Rollup validity proof forgery",
                        "State transition proof manipulation"
                    ]),
                    ("Recursive Proofs", vec![
                        "Recursive proof chain collapse",
                        "Compound vulnerability in proof recursion",
                        "Nested proof integrity attack"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
            ThreatCategory::DecryptionHndl => {
                let subs = [
                    ("Encrypted Mempool", vec![
                        "Threshold encryption key recovery",
                        "Pre-execution transaction visibility",
                        "Mempool decryption via Shor"
                    ]),
                    ("P2P Communication", vec![
                        "TLS/ECDH quantum vulnerability",
                        "Peer discovery protocol exposure",
                        "Gossip protocol interception"
                    ]),
                    ("HNDL Active Collection", vec![
                        "Encrypted traffic harvesting detected",
                        "Historical ciphertext collection ongoing",
                        "Nation-state HNDL campaign reported"
                    ]),
                    ("TEE Attestation", vec![
                        "Remote attestation key exposure",
                        "Enclave authentication bypass",
                        "SGX/SEV key vulnerability"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
            ThreatCategory::HashReversal => {
                let subs = [
                    ("SHA-256", vec![
                        "Grover speedup analysis update",
                        "Mining shortcut theoretical study",
                        "Hash collision quantum bounds"
                    ]),
                    ("Keccak/SHA-3", vec![
                        "State commitment quantum analysis",
                        "SHA-3 security margin assessment",
                        "Keccak quantum resistance verified"
                    ]),
                    ("Poseidon/Poseidon2", vec![
                        "ZK-friendly hash quantum analysis",
                        "Large field collision resistance",
                        "Poseidon2 security proof update"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
            ThreatCategory::ConsensusAttacks => {
                let subs = [
                    ("PoS Validator Keys", vec![
                        "Validator impersonation via key forgery",
                        "Slashing fraud technique",
                        "Validator key quantum extraction"
                    ]),
                    ("VRF Randomness", vec![
                        "EC-VRF discrete log vulnerability",
                        "Leader election manipulation",
                        "Randomness beacon prediction"
                    ]),
                    ("Finality Signatures", vec![
                        "Block finality manipulation",
                        "Aggregate finality signature forgery",
                        "Checkpoint signature attack"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
            ThreatCategory::CrossChainBridge => {
                let subs = [
                    ("Light Client Proofs", vec![
                        "SPV proof forgery technique",
                        "Header signature chain break",
                        "Light client verification bypass"
                    ]),
                    ("Relay Authentication", vec![
                        "Relayer signature forgery",
                        "Cross-chain message manipulation",
                        "Relay impersonation attack"
                    ]),
                    ("IBC Protocol", vec![
                        "IBC packet signature forgery",
                        "Inter-Blockchain Communication takeover",
                        "Cosmos IBC vulnerability"
                    ]),
                    ("Rollup Sequencer", vec![
                        "L2 batch signature forgery",
                        "Sequencer attestation bypass",
                        "Rollup commitment attack"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
            ThreatCategory::NetworkLayer => {
                let subs = [
                    ("Node Discovery", vec![
                        "Sybil attack via forged node IDs",
                        "Discv5 signature forgery",
                        "Node identity impersonation"
                    ]),
                    ("TLS/QUIC", vec![
                        "ECDHE key exchange attack",
                        "P2P MITM via TLS break",
                        "QUIC handshake vulnerability"
                    ]),
                    ("Libp2p Identity", vec![
                        "Peer ID forgery technique",
                        "Libp2p authentication bypass",
                        "qp-libp2p-identity recommendation"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
            ThreatCategory::KeyManagement => {
                let subs = [
                    ("HD Wallets BIP-32/39", vec![
                        "Master seed derivation attack",
                        "BIP-32 EC operation exploitation",
                        "Hierarchical key reconstruction"
                    ]),
                    ("MPC/TSS Shares", vec![
                        "Threshold secret reconstruction",
                        "MPC share combination attack",
                        "TSS key recovery technique"
                    ]),
                    ("Key Rotation", vec![
                        "Rotation ceremony attack window",
                        "ECDH key exchange vulnerability",
                        "Transition period exploitation"
                    ]),
                    ("Custodial Wallets", vec![
                        "Exchange hot wallet exposure",
                        "Centralized key compromise",
                        "Multi-tenant isolation break"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
            ThreatCategory::MevOrdering => {
                let subs = [
                    ("Encrypted Mempool Bypass", vec![
                        "Front-running despite encryption",
                        "Threshold decryption key recovery",
                        "MEV via mempool decryption"
                    ]),
                    ("PBS Attack", vec![
                        "Builder collusion via key compromise",
                        "Proposer-Builder separation break",
                        "Block builder key extraction"
                    ]),
                    ("Sealed Auctions", vec![
                        "Bid commitment scheme broken",
                        "Auction manipulation via decryption",
                        "Sealed bid early reveal"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
            ThreatCategory::SmartContracts => {
                let subs = [
                    ("ecrecover Bypass", vec![
                        "On-chain ECDSA verification failure",
                        "Contract signature bypass",
                        "ecrecover precompile attack"
                    ]),
                    ("Access Control", vec![
                        "Owner key takeover technique",
                        "Admin role hijacking",
                        "Privileged function exploitation"
                    ]),
                    ("Governance", vec![
                        "Governance signature forgery",
                        "Voting authorization bypass",
                        "Proposal signature manipulation"
                    ]),
                    ("Upgradeable Proxies", vec![
                        "Proxy admin key attack",
                        "Malicious implementation swap",
                        "Proxy ownership takeover"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
            ThreatCategory::SideChannel => {
                let subs = [
                    ("Timing Attacks", vec![
                        "Lattice operation timing leakage",
                        "Non-constant-time PQC implementation",
                        "ML-DSA rejection sampling timing"
                    ]),
                    ("Power Analysis", vec![
                        "ML-KEM decapsulation power trace",
                        "Key extraction via DPA",
                        "Hardware PQC power vulnerability"
                    ]),
                    ("TEE Side-Channels", vec![
                        "SGX memory access pattern leak",
                        "SEV attestation key extraction",
                        "Enclave side-channel attack"
                    ]),
                    ("Fault Injection", vec![
                        "Glitching during PQC operations",
                        "Induced error key revelation",
                        "Fault attack on lattice signing"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
            ThreatCategory::MigrationAgility => {
                let subs = [
                    ("Algorithm Downgrade", vec![
                        "Forcing legacy crypto use",
                        "Backward compatibility exploitation",
                        "Protocol downgrade attack"
                    ]),
                    ("Hybrid Bypass", vec![
                        "Attacking weaker hybrid component",
                        "Insufficient combiner security",
                        "OR-mode hybrid exploitation"
                    ]),
                    ("Incomplete Migration", vec![
                        "Legacy endpoint exploitation",
                        "Partial upgrade vulnerability",
                        "Migration gap attack"
                    ]),
                    ("Parameter Confusion", vec![
                        "Wrong PQC security level",
                        "ML-DSA-44 vs ML-DSA-87 confusion",
                        "Insufficient parameter selection"
                    ]),
                ];
                let idx = rng.gen_range(0..subs.len());
                (subs[idx].0.to_string(), subs[idx].1.clone())
            },
        };

        let base_severity = category.weight() * 2.0;
        let severity = (base_severity + rng.gen_range(-0.2..0.4)).clamp(0.1, 1.0);

        let era_relevance = match rng.gen_range(0..10) {
            0..=2 => QuantumEra::PreQuantum,
            3..=6 => QuantumEra::Nisq,
            _ => QuantumEra::FaultTolerant,
        };

        let indicator = ThreatIndicator {
            category,
            sub_category,
            severity,
            confidence: rng.gen_range(0.5..1.0),
            source: sources[rng.gen_range(0..sources.len())].to_string(),
            timestamp: Utc::now(),
            description: descriptions[rng.gen_range(0..descriptions.len())].to_string(),
            era_relevance,
            references: vec![],
        };

        self.add_indicator(indicator.clone());
        indicator
    }
}

impl Default for QuantumResistanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weights_sum_to_one() {
        let sum: f64 = ThreatCategory::all().iter().map(|c| c.weight()).sum();
        assert!((sum - 1.0).abs() < 0.001, "Weights should sum to 1.0, got {}", sum);
    }

    #[test]
    fn test_category_count() {
        assert_eq!(ThreatCategory::all().len(), 12);
    }
}
