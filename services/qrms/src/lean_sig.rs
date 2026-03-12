//! LeanSig Integration for QuantumAegis
//!
//! Provides integration with leanSig (hash-based post-quantum signatures)
//! and hybrid signatures combining leanSig with ML-DSA-87 and SLH-DSA-256.
//!
//! leanSig is a synchronized hash-based signature scheme using Poseidon2,
//! designed for post-quantum Ethereum consensus.

use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::apqc::AdaptivePqcLayer;

/// leanSig key pair (placeholder until leanSig crate is integrated)
///
/// Note: This is a placeholder structure. When leanSig crate is available,
/// this will wrap the actual leanSig key pair types.
#[derive(Debug, Clone)]
pub struct LeanSigKeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub epoch: u32,
    pub lifetime: u32,
}

impl LeanSigKeyPair {
    /// Generate a new leanSig key pair
    ///
    /// In production, this will use the actual leanSig::key_gen function
    pub fn generate(epoch: u32, lifetime: u32) -> Self {
        // Placeholder: In production, this will call:
        // let (pk, sk) = leanSig::key_gen(&mut rng, epoch, lifetime);

        // For now, generate placeholder keys
        // In production, this will use: let (pk, sk) = leanSig::key_gen(&mut rng, epoch, lifetime);
        use rand::RngCore;
        let mut rng = rand::rngs::OsRng;
        let mut pk = vec![0u8; 64];
        rng.fill_bytes(&mut pk);
        let mut sk = vec![0u8; 128];
        rng.fill_bytes(&mut sk);

        Self {
            public_key: pk,
            secret_key: sk,
            epoch,
            lifetime,
        }
    }

    /// Sign a message with leanSig
    ///
    /// In production, this will use the actual leanSig::sign function
    pub fn sign(&self, epoch: u32, message: &[u8]) -> LeanSigSignature {
        // Placeholder: In production, this will call:
        // let sig = leanSig::sign(&self.secret_key, epoch, message);

        // For now, create a placeholder signature
        // In production, this will use: let sig = leanSig::sign(&self.secret_key, epoch, message);
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(&self.secret_key);
        hasher.update(&epoch.to_le_bytes());
        hasher.update(message);
        let hash = hasher.finalize();

        LeanSigSignature {
            signature: hash.to_vec(),
            epoch,
            size_bytes: 32, // Placeholder size
        }
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.public_key
    }
}

/// leanSig signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanSigSignature {
    pub signature: Vec<u8>,
    pub epoch: u32,
    pub size_bytes: usize,
}

/// Verification result for leanSig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanSigVerification {
    pub valid: bool,
    pub epoch: u32,
    pub verify_time_ms: f64,
}

impl LeanSigSignature {
    /// Verify a leanSig signature
    ///
    /// In production, this will use the actual leanSig::verify function
    pub fn verify(
        public_key: &[u8],
        epoch: u32,
        message: &[u8],
        signature: &[u8],
    ) -> LeanSigVerification {
        let start = Instant::now();

        // Placeholder: In production, this will call:
        // let valid = leanSig::verify(public_key, epoch, message, signature);

        // For now, do a simple hash verification
        // In production, this will use: let valid = leanSig::verify(public_key, epoch, message, signature);
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(public_key);
        hasher.update(&epoch.to_le_bytes());
        hasher.update(message);
        let expected = hasher.finalize();

        let valid = expected.as_slice() == signature;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        LeanSigVerification {
            valid,
            epoch,
            verify_time_ms: elapsed,
        }
    }
}

/// Hybrid signature combining leanSig with PQC algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridLeanSigSignature {
    pub lean_sig: LeanSigSignature,
    pub ml_dsa: Option<Vec<u8>>,  // ML-DSA-87 signature
    pub slh_dsa: Option<Vec<u8>>, // SLH-DSA-256 signature
    pub combined_size_bytes: usize,
}

/// LeanSig integration layer
pub struct LeanSigIntegration {
    apqc: Arc<Mutex<AdaptivePqcLayer>>,
    lean_sig_keys: Arc<Mutex<Option<LeanSigKeyPair>>>,
}

impl LeanSigIntegration {
    /// Create a new LeanSig integration
    pub fn new(apqc: Arc<Mutex<AdaptivePqcLayer>>) -> Self {
        Self {
            apqc,
            lean_sig_keys: Arc::new(Mutex::new(None)),
        }
    }

    /// Generate leanSig keys
    pub async fn generate_keys(&self, epoch: u32, lifetime: u32) -> Result<LeanSigKeyPair, String> {
        let keys = LeanSigKeyPair::generate(epoch, lifetime);
        *self.lean_sig_keys.lock().await = Some(keys.clone());
        Ok(keys)
    }

    /// Sign message with leanSig + PQC hybrid
    pub async fn sign_hybrid(
        &self,
        message: &[u8],
        epoch: u32,
    ) -> Result<HybridLeanSigSignature, String> {
        let start = Instant::now();

        // Get or generate leanSig keys
        let lean_keys = {
            let mut keys_guard = self.lean_sig_keys.lock().await;
            if keys_guard.is_none() {
                *keys_guard = Some(LeanSigKeyPair::generate(epoch, 1000000)); // Default lifetime
            }
            keys_guard.as_ref().unwrap().clone()
        };

        // Sign with leanSig
        let lean_sig = lean_keys.sign(epoch, message);

        // Sign with PQC algorithms via APQC
        let dual_sig = {
            let mut apqc_guard = self.apqc.lock().await;
            apqc_guard.sign_dual(message).await
        };

        let ml_dsa_sig = hex::decode(&dual_sig.ml_dsa.signature).ok();
        let slh_dsa_sig = hex::decode(&dual_sig.slh_dsa.signature).ok();

        let combined_size = lean_sig.size_bytes
            + ml_dsa_sig.as_ref().map(|s| s.len()).unwrap_or(0)
            + slh_dsa_sig.as_ref().map(|s| s.len()).unwrap_or(0);

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        tracing::debug!("Hybrid leanSig signing took {:.2}ms", elapsed);

        Ok(HybridLeanSigSignature {
            lean_sig,
            ml_dsa: ml_dsa_sig,
            slh_dsa: slh_dsa_sig,
            combined_size_bytes: combined_size,
        })
    }

    /// Verify hybrid signature (leanSig + PQC)
    pub async fn verify_hybrid(
        &self,
        message: &[u8],
        signature: &HybridLeanSigSignature,
        lean_public_key: &[u8],
        ml_dsa_public_key: Option<&[u8]>,
        slh_dsa_public_key: Option<&[u8]>,
    ) -> Result<bool, String> {
        let start = Instant::now();

        // Verify leanSig component
        let lean_verify = LeanSigSignature::verify(
            lean_public_key,
            signature.lean_sig.epoch,
            message,
            &signature.lean_sig.signature,
        );

        if !lean_verify.valid {
            return Ok(false);
        }

        // Verify PQC components if present
        let mut all_valid = true;

        if let Some(_ml_dsa_sig) = &signature.ml_dsa {
            if let Some(_ml_pk) = ml_dsa_public_key {
                // Verify ML-DSA signature
                // In production, use actual ML-DSA verification via APQC
                // For now, assume valid if present
                all_valid = all_valid && true; // Placeholder
            }
        }

        if let Some(_slh_dsa_sig) = &signature.slh_dsa {
            if let Some(_slh_pk) = slh_dsa_public_key {
                // Verify SLH-DSA signature
                // In production, use actual SLH-DSA verification via APQC
                // For now, assume valid if present
                all_valid = all_valid && true; // Placeholder
            }
        }

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        tracing::debug!("Hybrid leanSig verification took {:.2}ms", elapsed);

        Ok(all_valid && lean_verify.valid)
    }

    /// Get current leanSig keys
    pub async fn get_keys(&self) -> Option<LeanSigKeyPair> {
        self.lean_sig_keys.lock().await.clone()
    }

    /// Export public key for on-chain use
    pub async fn export_public_key(&self) -> Option<Vec<u8>> {
        self.lean_sig_keys
            .lock()
            .await
            .as_ref()
            .map(|k| k.public_key_bytes().to_vec())
    }
}

/// Request to sign with leanSig
#[derive(Debug, Deserialize)]
pub struct LeanSigSignRequest {
    pub message: String,
    pub epoch: Option<u32>,
    pub include_pqc: Option<bool>, // Whether to include PQC signatures
}

/// Response from leanSig signing
#[derive(Debug, Serialize)]
pub struct LeanSigSignResponse {
    pub signature: HybridLeanSigSignature,
    pub public_key: Vec<u8>,
    pub epoch: u32,
    pub sign_time_ms: f64,
}

/// Request to verify leanSig signature
#[derive(Debug, Deserialize)]
pub struct LeanSigVerifyRequest {
    pub message: String,
    pub signature: HybridLeanSigSignature,
    pub lean_public_key: String,            // Hex encoded
    pub ml_dsa_public_key: Option<String>,  // Hex encoded
    pub slh_dsa_public_key: Option<String>, // Hex encoded
}

/// Response from leanSig verification
#[derive(Debug, Serialize)]
pub struct LeanSigVerifyResponse {
    pub valid: bool,
    pub lean_sig_valid: bool,
    pub ml_dsa_valid: Option<bool>,
    pub slh_dsa_valid: Option<bool>,
    pub verify_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apqc::AdaptivePqcLayer;

    #[tokio::test]
    async fn test_lean_sig_key_generation() {
        let keys = LeanSigKeyPair::generate(0, 1000000);
        assert_eq!(keys.epoch, 0);
        assert_eq!(keys.lifetime, 1000000);
        assert!(!keys.public_key.is_empty());
        assert!(!keys.secret_key.is_empty());
    }

    #[tokio::test]
    async fn test_lean_sig_sign_verify() {
        let keys = LeanSigKeyPair::generate(0, 1000000);
        let message = b"test message";
        let epoch = 0;

        let signature = keys.sign(epoch, message);
        let verification =
            LeanSigSignature::verify(&keys.public_key, epoch, message, &signature.signature);

        assert!(verification.valid);
    }

    #[tokio::test]
    async fn test_hybrid_lean_sig() {
        let apqc = Arc::new(Mutex::new(AdaptivePqcLayer::new()));
        let integration = LeanSigIntegration::new(apqc);

        let message = b"test message for hybrid signature";
        let epoch = 0;

        // Generate keys
        integration.generate_keys(epoch, 1000000).await.unwrap();

        // Sign
        let signature = integration.sign_hybrid(message, epoch).await.unwrap();
        assert!(!signature.lean_sig.signature.is_empty());

        // Get public key
        let public_key = integration.export_public_key().await.unwrap();

        // Verify
        let valid = integration
            .verify_hybrid(message, &signature, &public_key, None, None)
            .await
            .unwrap();

        assert!(valid);
    }
}
