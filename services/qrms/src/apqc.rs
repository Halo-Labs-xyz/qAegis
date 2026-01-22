//! Adaptive PQC Layer (APQC)
//! Manages concurrent redundant post-quantum cryptographic operations

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;
use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::crypto::{
    MldsaKeyPair, SlhDsaKeyPair, MlKemKeyPair, HqcKeyPair, EcdsaKeyPair,
    HybridSignature,
};

/// Signature algorithms
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    #[serde(rename = "ML-DSA-87")]
    MlDsa87,
    #[serde(rename = "SLH-DSA-256s")]
    SlhDsa256s,
}

impl SignatureAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            Self::MlDsa87 => "ML-DSA-87",
            Self::SlhDsa256s => "SLH-DSA-256s",
        }
    }

    pub fn signature_size(&self) -> usize {
        match self {
            Self::MlDsa87 => 4595,
            Self::SlhDsa256s => 29792,
        }
    }

    pub fn public_key_size(&self) -> usize {
        match self {
            Self::MlDsa87 => 2592,
            Self::SlhDsa256s => 64,
        }
    }
}

/// KEM algorithms
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KemAlgorithm {
    #[serde(rename = "ML-KEM-1024")]
    MlKem1024,
    #[serde(rename = "HQC-256")]
    Hqc256,
}

impl KemAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            Self::MlKem1024 => "ML-KEM-1024",
            Self::Hqc256 => "HQC-256",
        }
    }

    pub fn ciphertext_size(&self) -> usize {
        match self {
            Self::MlKem1024 => 1568,
            Self::Hqc256 => 6730,
        }
    }
}

/// A single algorithm signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleSignature {
    pub algorithm: String,
    pub signature: String,
    pub size_bytes: usize,
    pub sign_time_ms: f64,
}

/// Dual signature combining ML-DSA and SLH-DSA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualSignature {
    pub ml_dsa: SingleSignature,
    pub slh_dsa: SingleSignature,
    pub combined_size_bytes: usize,
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub mode: String,
    pub ml_dsa_valid: bool,
    pub slh_dsa_valid: bool,
    pub verify_time_ms: f64,
}

/// Hybrid KEM encapsulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridKemResult {
    pub ml_kem: KemPartResult,
    pub hqc: KemPartResult,
    pub shared_secret: String,
    pub combined_ct_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KemPartResult {
    pub algorithm: String,
    pub ciphertext_size: usize,
    pub encaps_time_ms: f64,
}

/// Combiner mode for signature verification
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CombinerMode {
    And,  // Both must be valid (security)
    Or,   // Either valid (availability)
}

/// Adaptive PQC Layer
pub struct AdaptivePqcLayer {
    pub active_signatures: Vec<SignatureAlgorithm>,
    pub active_kems: Vec<KemAlgorithm>,
    pub rotation_pending: bool,
    pub rotation_block: Option<u64>,
    pub key_generation_count: u64,
    
    // Real PQC key pairs
    mldsa_keys: Arc<Mutex<MldsaKeyPair>>,
    slhdsa_keys: Arc<Mutex<SlhDsaKeyPair>>,
    mlkem_keys: Arc<Mutex<MlKemKeyPair>>,
    hqc_keys: Arc<Mutex<HqcKeyPair>>,
    ecdsa_keys: Arc<Mutex<EcdsaKeyPair>>,
    
    // Pending keys for rotation
    pending_mldsa_keys: Arc<Mutex<Option<MldsaKeyPair>>>,
    pending_slhdsa_keys: Arc<Mutex<Option<SlhDsaKeyPair>>>,
}

impl AdaptivePqcLayer {
    pub fn new() -> Self {
        Self {
            active_signatures: vec![SignatureAlgorithm::MlDsa87, SignatureAlgorithm::SlhDsa256s],
            active_kems: vec![KemAlgorithm::MlKem1024, KemAlgorithm::Hqc256],
            rotation_pending: false,
            rotation_block: None,
            key_generation_count: 0,
            mldsa_keys: Arc::new(Mutex::new(MldsaKeyPair::generate())),
            slhdsa_keys: Arc::new(Mutex::new(SlhDsaKeyPair::generate())),
            mlkem_keys: Arc::new(Mutex::new(MlKemKeyPair::generate())),
            hqc_keys: Arc::new(Mutex::new(HqcKeyPair::generate())),
            ecdsa_keys: Arc::new(Mutex::new(EcdsaKeyPair::generate())),
            pending_mldsa_keys: Arc::new(Mutex::new(None)),
            pending_slhdsa_keys: Arc::new(Mutex::new(None)),
        }
    }

    /// Sign message with dual PQC signatures (real implementation)
    pub async fn sign_dual(&mut self, message: &[u8]) -> DualSignature {
        self.key_generation_count += 1;

        // Real ML-DSA signature
        let mldsa_keys = self.mldsa_keys.lock().await;
        let (ml_sig_bytes, ml_time) = mldsa_keys.sign(message);
        let ml_sig = hex::encode(&ml_sig_bytes);
        drop(mldsa_keys);

        // Real SLH-DSA signature
        let slhdsa_keys = self.slhdsa_keys.lock().await;
        let (slh_sig_bytes, slh_time) = slhdsa_keys.sign(message);
        let slh_sig = hex::encode(&slh_sig_bytes);
        drop(slhdsa_keys);

        DualSignature {
            ml_dsa: SingleSignature {
                algorithm: SignatureAlgorithm::MlDsa87.name().to_string(),
                signature: ml_sig,
                size_bytes: ml_sig_bytes.len(),
                sign_time_ms: ml_time,
            },
            slh_dsa: SingleSignature {
                algorithm: SignatureAlgorithm::SlhDsa256s.name().to_string(),
                signature: slh_sig,
                size_bytes: slh_sig_bytes.len(),
                sign_time_ms: slh_time,
            },
            combined_size_bytes: ml_sig_bytes.len() + slh_sig_bytes.len(),
        }
    }

    /// Sign with hybrid scheme (ECDSA + PQC dual)
    pub async fn sign_hybrid(&mut self, message: &[u8]) -> HybridSignature {
        // ECDSA signature
        let ecdsa_keys = self.ecdsa_keys.lock().await;
        let (ecdsa_sig, _) = ecdsa_keys.sign(message);
        drop(ecdsa_keys);

        // PQC dual signatures
        let dual = self.sign_dual(message).await;

        HybridSignature::new(
            ecdsa_sig,
            hex::decode(&dual.ml_dsa.signature).unwrap_or_default(),
            hex::decode(&dual.slh_dsa.signature).unwrap_or_default(),
        )
    }

    /// Verify dual signature (real implementation)
    pub async fn verify_dual(&self, message: &[u8], signature: &DualSignature, mode: CombinerMode) -> VerificationResult {
        // Verify ML-DSA
        let mldsa_keys = self.mldsa_keys.lock().await;
        let ml_sig_bytes = hex::decode(&signature.ml_dsa.signature).unwrap_or_default();
        let (ml_dsa_valid, ml_time) = if !ml_sig_bytes.is_empty() {
            MldsaKeyPair::verify(message, &ml_sig_bytes, &mldsa_keys.public_key)
        } else {
            (false, 0.0)
        };
        drop(mldsa_keys);

        // Verify SLH-DSA
        let slhdsa_keys = self.slhdsa_keys.lock().await;
        let slh_sig_bytes = hex::decode(&signature.slh_dsa.signature).unwrap_or_default();
        let (slh_dsa_valid, slh_time) = if !slh_sig_bytes.is_empty() {
            SlhDsaKeyPair::verify(message, &slh_sig_bytes, &slhdsa_keys.public_key)
        } else {
            (false, 0.0)
        };
        drop(slhdsa_keys);

        let valid = match mode {
            CombinerMode::And => ml_dsa_valid && slh_dsa_valid,
            CombinerMode::Or => ml_dsa_valid || slh_dsa_valid,
        };

        VerificationResult {
            valid,
            mode: format!("{:?}", mode).to_lowercase(),
            ml_dsa_valid,
            slh_dsa_valid,
            verify_time_ms: ml_time + slh_time,
        }
    }

    /// Verify hybrid signature (ECDSA + PQC)
    pub async fn verify_hybrid(&self, message: &[u8], hybrid_sig: &HybridSignature) -> bool {
        // Verify ECDSA
        let ecdsa_keys = self.ecdsa_keys.lock().await;
        let (ecdsa_valid, _) = EcdsaKeyPair::verify(message, &hybrid_sig.ecdsa_sig, &ecdsa_keys.verifying_key);
        drop(ecdsa_keys);

        // Verify PQC dual
        let dual_sig = DualSignature {
            ml_dsa: SingleSignature {
                algorithm: "ML-DSA-87".to_string(),
                signature: hex::encode(&hybrid_sig.mldsa_sig),
                size_bytes: hybrid_sig.mldsa_sig.len(),
                sign_time_ms: 0.0,
            },
            slh_dsa: SingleSignature {
                algorithm: "SLH-DSA-256s".to_string(),
                signature: hex::encode(&hybrid_sig.slhdsa_sig),
                size_bytes: hybrid_sig.slhdsa_sig.len(),
                sign_time_ms: 0.0,
            },
            combined_size_bytes: hybrid_sig.total_size(),
        };
        let pqc_result = self.verify_dual(message, &dual_sig, CombinerMode::And).await;

        // Both ECDSA and PQC must be valid
        ecdsa_valid && pqc_result.valid
    }

    /// Hybrid KEM encapsulation (real implementation)
    pub async fn encapsulate_hybrid(&self) -> HybridKemResult {
        // ML-KEM encapsulation
        let mlkem_keys = self.mlkem_keys.lock().await;
        let (ml_ct, ml_ss, ml_time) = mlkem_keys.encapsulate();
        drop(mlkem_keys);

        // HQC encapsulation
        let hqc_keys = self.hqc_keys.lock().await;
        let (hqc_ct, hqc_ss, hqc_time) = hqc_keys.encapsulate();
        drop(hqc_keys);

        // Combine shared secrets
        let mut hasher = Sha256::new();
        hasher.update(&ml_ss);
        hasher.update(&hqc_ss);
        let shared_secret = hex::encode(&hasher.finalize()[..16]);

        HybridKemResult {
            ml_kem: KemPartResult {
                algorithm: KemAlgorithm::MlKem1024.name().to_string(),
                ciphertext_size: ml_ct.len(),
                encaps_time_ms: ml_time,
            },
            hqc: KemPartResult {
                algorithm: KemAlgorithm::Hqc256.name().to_string(),
                ciphertext_size: hqc_ct.len(),
                encaps_time_ms: hqc_time,
            },
            shared_secret,
            combined_ct_size: ml_ct.len() + hqc_ct.len(),
        }
    }

    /// Generate new key pairs for rotation
    pub async fn generate_rotation_keys(&mut self) {
        *self.pending_mldsa_keys.lock().await = Some(MldsaKeyPair::generate());
        *self.pending_slhdsa_keys.lock().await = Some(SlhDsaKeyPair::generate());
        self.key_generation_count += 2;
    }

    /// Schedule algorithm rotation
    pub fn schedule_rotation(&mut self, effective_block: u64) {
        self.rotation_pending = true;
        self.rotation_block = Some(effective_block);
    }

    /// Execute rotation (swap to pending keys)
    pub async fn execute_rotation(&mut self) -> RotationResult {
        if let Some(new_mldsa) = self.pending_mldsa_keys.lock().await.take() {
            *self.mldsa_keys.lock().await = new_mldsa;
        }
        if let Some(new_slhdsa) = self.pending_slhdsa_keys.lock().await.take() {
            *self.slhdsa_keys.lock().await = new_slhdsa;
        }
        
        self.rotation_pending = false;
        self.rotation_block = None;
        
        RotationResult {
            status: "rotated".to_string(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get public keys for on-chain registration
    pub async fn get_public_keys(&self) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let mldsa = self.mldsa_keys.lock().await.public_key_bytes();
        let slhdsa = self.slhdsa_keys.lock().await.public_key_bytes();
        let ecdsa = self.ecdsa_keys.lock().await.public_key_bytes();
        (mldsa, slhdsa, ecdsa)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationResult {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for AdaptivePqcLayer {
    fn default() -> Self {
        Self::new()
    }
}
