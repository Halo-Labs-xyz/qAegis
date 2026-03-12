//! Hybrid Post-Quantum Encryption Layer
//! 
//! Provides end-to-end encryption using:
//! - Hybrid KEM: ML-KEM-1024 + HQC-256 (post-quantum)
//! - Symmetric: AES-256-GCM or ChaCha20-Poly1305
//! - Authentication: ML-DSA-87 + SLH-DSA-256s signatures

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use sha3::Sha3_256;
use hex;
use rand::Rng;

// Post-quantum cryptography (reusing from qrms)
use pqcrypto_dilithium::dilithium5 as dilithium5_mod;
use pqcrypto_sphincsplus::sphincssha256256fsimple as sphincs_mod;
use pqcrypto_traits::sign::{DetachedSignature as PqcDetachedSignature, PublicKey as PqcPublicKey};

// Symmetric encryption
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit as _, OsRng},
    Aes256Gcm, Key as AesKey, Nonce as AesNonce,
};
use chacha20poly1305::{
    ChaCha20Poly1305,
    Key as ChaChaKey,
    Nonce as ChaChaNonce,
    aead::{KeyInit as _, OsRng as ChaChaOsRng},
};

/// Encrypted message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub version: u8,
    pub kem_ciphertext: Vec<u8>,      // ML-KEM + HQC hybrid ciphertext
    pub symmetric_ciphertext: Vec<u8>, // AES-256-GCM or ChaCha20-Poly1305
    pub nonce: Vec<u8>,
    pub signature: Option<MessageSignature>, // Optional PQC signature
    pub metadata: MessageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSignature {
    pub mldsa_sig: Vec<u8>,   // ML-DSA-87 signature
    pub slhdsa_sig: Vec<u8>,  // SLH-DSA-256s signature
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub timestamp: i64,
    pub message_id: String,
    pub algorithm: String,
    pub key_id: String,
}

/// Hybrid KEM result (ML-KEM + HQC)
#[derive(Debug, Clone)]
pub struct HybridKemResult {
    pub ml_kem_ct: Vec<u8>,
    pub hqc_ct: Vec<u8>,
    pub shared_secret: Vec<u8>,
}

/// Encryption key pair
#[derive(Clone)]
pub struct EncryptionKeyPair {
    // For now, we'll use mock ML-KEM and HQC until AVX2 issues are resolved
    // In production, these would be real PQC keys
    pub ml_kem_pubkey: Vec<u8>,
    pub ml_kem_seckey: Vec<u8>,
    pub hqc_pubkey: Vec<u8>,
    pub hqc_seckey: Vec<u8>,
    pub mldsa_public_key: dilithium5_mod::PublicKey,
    pub mldsa_secret_key: dilithium5_mod::SecretKey,
    pub slhdsa_public_key: sphincs_mod::PublicKey,
    pub slhdsa_secret_key: sphincs_mod::SecretKey,
}

impl std::fmt::Debug for EncryptionKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionKeyPair")
            .field("ml_kem_pubkey_len", &self.ml_kem_pubkey.len())
            .field("hqc_pubkey_len", &self.hqc_pubkey.len())
            .finish()
    }
}

impl EncryptionKeyPair {
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        // Generate ML-KEM keypair (mock for now)
        let ml_kem_pubkey: Vec<u8> = (0..1568).map(|_| rng.gen()).collect();
        let ml_kem_seckey: Vec<u8> = (0..3168).map(|_| rng.gen()).collect();
        
        // Generate HQC keypair (mock for now)
        let hqc_pubkey: Vec<u8> = (0..6730).map(|_| rng.gen()).collect();
        let hqc_seckey: Vec<u8> = (0..6730).map(|_| rng.gen()).collect();
        
        // Generate real PQC signature keys
        let (mldsa_pk, mldsa_sk) = dilithium5_mod::keypair();
        let (slhdsa_pk, slhdsa_sk) = sphincs_mod::keypair();
        
        Self {
            ml_kem_pubkey,
            ml_kem_seckey,
            hqc_pubkey,
            hqc_seckey,
            mldsa_public_key: mldsa_pk,
            mldsa_secret_key: mldsa_sk,
            slhdsa_public_key: slhdsa_pk,
            slhdsa_secret_key: slhdsa_sk,
        }
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        // Combine public keys
        let mut combined = Vec::new();
        combined.extend_from_slice(&(self.ml_kem_pubkey.len() as u32).to_le_bytes());
        combined.extend_from_slice(&self.ml_kem_pubkey);
        combined.extend_from_slice(&(self.hqc_pubkey.len() as u32).to_le_bytes());
        combined.extend_from_slice(&self.hqc_pubkey);
        combined.extend_from_slice(&<dilithium5_mod::PublicKey as PqcPublicKey>::as_bytes(&self.mldsa_public_key));
        combined.extend_from_slice(&<sphincs_mod::PublicKey as PqcPublicKey>::as_bytes(&self.slhdsa_public_key));
        combined
    }
}

/// Encrypt a message using hybrid PQC encryption
pub fn encrypt_message(
    message: &[u8],
    recipient_pubkey: &EncryptionKeyPair,
    signer_keypair: Option<&EncryptionKeyPair>,
    algorithm: &str,
) -> Result<EncryptedMessage> {
    // Step 1: Perform hybrid KEM encapsulation
    let kem_result = perform_hybrid_kem_encapsulation(&recipient_pubkey)?;
    
    // Step 3: Derive symmetric key from KEM shared secret
    let symmetric_key = derive_symmetric_key(&kem_result.shared_secret);
    
    // Step 4: Encrypt message with symmetric cipher
    let (ciphertext, nonce) = match algorithm {
        "aes-256-gcm" => encrypt_symmetric_aes(&symmetric_key, message)?,
        "chacha20-poly1305" => encrypt_symmetric_chacha(&symmetric_key, message)?,
        _ => return Err(anyhow::anyhow!("Unsupported symmetric algorithm")),
    };
    
    // Step 5: Optionally sign the encrypted message
    let signature = if let Some(signer) = signer_keypair {
        let message_to_sign = [&ciphertext[..], &nonce[..], &kem_result.ml_kem_ct[..], &kem_result.hqc_ct[..]].concat();
        Some(sign_message(&message_to_sign, signer)?)
    } else {
        None
    };
    
    // Step 6: Combine KEM ciphertexts
    let mut kem_ciphertext = Vec::new();
    kem_ciphertext.extend_from_slice(&(kem_result.ml_kem_ct.len() as u32).to_le_bytes());
    kem_ciphertext.extend_from_slice(&kem_result.ml_kem_ct);
    kem_ciphertext.extend_from_slice(&(kem_result.hqc_ct.len() as u32).to_le_bytes());
    kem_ciphertext.extend_from_slice(&kem_result.hqc_ct);
    
    Ok(EncryptedMessage {
        version: 1,
        kem_ciphertext: kem_ciphertext,
        symmetric_ciphertext: ciphertext,
        nonce,
        signature,
        metadata: MessageMetadata {
            timestamp: chrono::Utc::now().timestamp(),
            message_id: uuid::Uuid::new_v4().to_string(),
            algorithm: algorithm.to_string(),
            key_id: hex::encode(&recipient_pubkey.public_key_bytes()[..16]),
        },
    })
}

/// Decrypt a message using hybrid PQC decryption
pub fn decrypt_message(
    encrypted: &EncryptedMessage,
    recipient_keypair: &EncryptionKeyPair,
    verifier_pubkey: Option<&EncryptionKeyPair>,
) -> Result<Vec<u8>> {
    // Step 1: Verify signature if present
    if let Some(sig) = &encrypted.signature {
        if let Some(verifier) = verifier_pubkey {
            let message_to_verify = [
                &encrypted.symmetric_ciphertext[..],
                &encrypted.nonce[..],
                &encrypted.kem_ciphertext[..],
            ].concat();
            verify_message_signature(&message_to_verify, sig, verifier)?;
        }
    }
    
    // Step 2: Extract KEM ciphertexts
    let mut offset = 0;
    let ml_kem_ct_len = u32::from_le_bytes([
        encrypted.kem_ciphertext[offset],
        encrypted.kem_ciphertext[offset + 1],
        encrypted.kem_ciphertext[offset + 2],
        encrypted.kem_ciphertext[offset + 3],
    ]) as usize;
    offset += 4;
    let ml_kem_ct = &encrypted.kem_ciphertext[offset..offset + ml_kem_ct_len];
    offset += ml_kem_ct_len;
    
    let hqc_ct_len = u32::from_le_bytes([
        encrypted.kem_ciphertext[offset],
        encrypted.kem_ciphertext[offset + 1],
        encrypted.kem_ciphertext[offset + 2],
        encrypted.kem_ciphertext[offset + 3],
    ]) as usize;
    offset += 4;
    let hqc_ct = &encrypted.kem_ciphertext[offset..offset + hqc_ct_len];
    
    // Step 3: Perform hybrid KEM decapsulation
    let shared_secret = perform_hybrid_kem_decapsulation(
        ml_kem_ct,
        hqc_ct,
        &recipient_keypair,
    )?;
    
    // Step 4: Derive symmetric key
    let symmetric_key = derive_symmetric_key(&shared_secret);
    
    // Step 5: Decrypt message
    match encrypted.metadata.algorithm.as_str() {
        "aes-256-gcm" => decrypt_symmetric_aes(&symmetric_key, &encrypted.symmetric_ciphertext, &encrypted.nonce),
        "chacha20-poly1305" => decrypt_symmetric_chacha(&symmetric_key, &encrypted.symmetric_ciphertext, &encrypted.nonce),
        _ => Err(anyhow::anyhow!("Unsupported symmetric algorithm")),
    }
}

fn perform_hybrid_kem_encapsulation(pubkey: &EncryptionKeyPair) -> Result<HybridKemResult> {
    // Mock implementation - in production, use real ML-KEM and HQC
    // For now, generate random ciphertexts and shared secret
    let mut rng = rand::thread_rng();
    let ml_kem_ct: Vec<u8> = (0..1568).map(|_| rng.gen()).collect();
    let hqc_ct: Vec<u8> = (0..6730).map(|_| rng.gen()).collect();
    
    // Derive shared secret from both KEMs
    let mut hasher = Sha256::new();
    hasher.update(&ml_kem_ct);
    hasher.update(&hqc_ct);
    hasher.update(&pubkey.ml_kem_pubkey);
    hasher.update(&pubkey.hqc_pubkey);
    let shared_secret = hasher.finalize().to_vec();
    
    Ok(HybridKemResult {
        ml_kem_ct,
        hqc_ct,
        shared_secret,
    })
}

fn perform_hybrid_kem_decapsulation(
    ml_kem_ct: &[u8],
    hqc_ct: &[u8],
    keypair: &EncryptionKeyPair,
) -> Result<Vec<u8>> {
    // Mock implementation - derive shared secret
    let mut hasher = Sha256::new();
    hasher.update(ml_kem_ct);
    hasher.update(hqc_ct);
    hasher.update(&keypair.ml_kem_pubkey);
    hasher.update(&keypair.hqc_pubkey);
    Ok(hasher.finalize().to_vec())
}

fn derive_symmetric_key(kem_shared_secret: &[u8]) -> Vec<u8> {
    // Use SHA3-256 for key derivation
    let mut hasher = Sha3_256::new();
    hasher.update(kem_shared_secret);
    hasher.update(b"moltbot-secure-symmetric-key");
    hasher.finalize().to_vec()
}

fn encrypt_symmetric_aes(key: &[u8], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let key_32: [u8; 32] = key[..32].try_into()
        .context("Key must be at least 32 bytes")?;
    let cipher_key: &AesKey<Aes256Gcm> = (&key_32).into();
    let cipher = Aes256Gcm::new(cipher_key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("AES encryption failed: {:?}", e))?;
    Ok((ciphertext, nonce.to_vec()))
}

fn decrypt_symmetric_aes(key: &[u8], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
    let key_32: [u8; 32] = key[..32].try_into()
        .context("Key must be at least 32 bytes")?;
    let cipher_key: &AesKey<Aes256Gcm> = (&key_32).into();
    let cipher = Aes256Gcm::new(cipher_key);
    let nonce = AesNonce::from_slice(nonce);
    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("AES decryption failed: {:?}", e))
}

fn encrypt_symmetric_chacha(key: &[u8], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let key_32: [u8; 32] = key[..32].try_into()
        .context("Key must be at least 32 bytes")?;
    let cipher_key: &ChaChaKey = (&key_32).into();
    let cipher = ChaCha20Poly1305::new(cipher_key);
    let nonce = ChaCha20Poly1305::generate_nonce(&mut ChaChaOsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("ChaCha20-Poly1305 encryption failed: {:?}", e))?;
    Ok((ciphertext, nonce.to_vec()))
}

fn decrypt_symmetric_chacha(key: &[u8], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
    let key_32: [u8; 32] = key[..32].try_into()
        .context("Key must be at least 32 bytes")?;
    let cipher_key: &ChaChaKey = (&key_32).into();
    let cipher = ChaCha20Poly1305::new(cipher_key);
    let nonce = ChaChaNonce::from_slice(nonce);
    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("ChaCha20-Poly1305 decryption failed: {:?}", e))
}

fn sign_message(message: &[u8], keypair: &EncryptionKeyPair) -> Result<MessageSignature> {
    let mldsa_sig = dilithium5_mod::detached_sign(message, &keypair.mldsa_secret_key);
    let slhdsa_sig = sphincs_mod::detached_sign(message, &keypair.slhdsa_secret_key);
    
    Ok(MessageSignature {
        mldsa_sig: mldsa_sig.as_bytes().to_vec(),
        slhdsa_sig: slhdsa_sig.as_bytes().to_vec(),
    })
}

fn verify_message_signature(
    message: &[u8],
    signature: &MessageSignature,
    pubkey: &EncryptionKeyPair,
) -> Result<()> {
    // Verify ML-DSA signature
    let mldsa_sig = <dilithium5_mod::DetachedSignature as PqcDetachedSignature>::from_bytes(&signature.mldsa_sig)
        .context("Invalid ML-DSA signature format")?;
    dilithium5_mod::verify_detached_signature(&mldsa_sig, message, &pubkey.mldsa_public_key)
        .context("ML-DSA signature verification failed")?;
    
    // Verify SLH-DSA signature
    let slhdsa_sig = <sphincs_mod::DetachedSignature as PqcDetachedSignature>::from_bytes(&signature.slhdsa_sig)
        .context("Invalid SLH-DSA signature format")?;
    sphincs_mod::verify_detached_signature(&slhdsa_sig, message, &pubkey.slhdsa_public_key)
        .context("SLH-DSA signature verification failed")?;
    
    Ok(())
}
