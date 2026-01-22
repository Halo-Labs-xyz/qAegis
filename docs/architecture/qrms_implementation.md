# QRMS Implementation Specification

Quantum Resistance Model System (QRMS) - Implementation guide for chain development kit deployment.

---

## Project Structure

```
qrms/
├── crates/
│   ├── qrms-core/              # Core library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── apqc/           # Adaptive PQC
│   │   │   │   ├── mod.rs
│   │   │   │   ├── dual_sign.rs
│   │   │   │   ├── hybrid_kem.rs
│   │   │   │   └── rotation.rs
│   │   │   ├── qrm/            # Quantum Resistance Monitor
│   │   │   │   ├── mod.rs
│   │   │   │   ├── threat.rs
│   │   │   │   ├── risk.rs
│   │   │   │   └── signal.rs
│   │   │   └── tee/            # TEE abstraction
│   │   │       ├── mod.rs
│   │   │       ├── sgx.rs
│   │   │       ├── tdx.rs
│   │   │       └── sev.rs
│   │   └── Cargo.toml
│   │
│   ├── qrms-sequencer/         # TEE Sequencer
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── enclave/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── ordering.rs
│   │   │   │   └── attestation.rs
│   │   │   ├── mempool/
│   │   │   │   ├── mod.rs
│   │   │   │   └── encrypted.rs
│   │   │   └── batch.rs
│   │   └── Cargo.toml
│   │
│   ├── qrms-substrate/         # Substrate pallets
│   │   ├── pallet-apqc/
│   │   │   ├── src/lib.rs
│   │   │   └── Cargo.toml
│   │   ├── pallet-qrm/
│   │   │   ├── src/lib.rs
│   │   │   └── Cargo.toml
│   │   └── runtime/
│   │       ├── src/lib.rs
│   │       └── Cargo.toml
│   │
│   ├── qrms-cosmos/            # Cosmos SDK module
│   │   ├── x/
│   │   │   ├── apqc/
│   │   │   │   ├── keeper/
│   │   │   │   ├── types/
│   │   │   │   └── module.go
│   │   │   └── qrm/
│   │   │       ├── keeper/
│   │   │       ├── types/
│   │   │       └── module.go
│   │   ├── app/
│   │   │   └── app.go
│   │   └── go.mod
│   │
│   └── qrms-opstack/           # OP Stack integration
│       ├── contracts/
│       │   ├── PQCVerifier.sol
│       │   └── SequencerAttestation.sol
│       ├── op-node/
│       │   └── rollup/
│       │       └── derive/
│       │           └── pqc_batch.go
│       └── op-batcher/
│           └── batcher/
│               └── tee_driver.go
│
├── enclave/                    # SGX enclave code
│   ├── Enclave.edl
│   ├── Enclave.config.xml
│   └── src/
│       ├── lib.rs
│       └── ecalls.rs
│
├── qrm-agent/                  # QRM monitoring agent
│   ├── src/
│   │   ├── main.rs
│   │   ├── sources/
│   │   │   ├── arxiv.rs
│   │   │   ├── nist.rs
│   │   │   └── iacr.rs
│   │   ├── classifier.rs
│   │   └── scorer.rs
│   └── Cargo.toml
│
├── docker/
│   ├── Dockerfile.sequencer
│   ├── Dockerfile.node
│   └── docker-compose.yml
│
└── specs/
    ├── apqc.md
    ├── qrm.md
    └── sequencer.md
```

---

## Core Library: qrms-core

### Dual Signature Implementation

```rust
// crates/qrms-core/src/apqc/dual_sign.rs

use ml_dsa::{MlDsa87, SigningKey as MlDsaSk, VerifyingKey as MlDsaVk};
use slh_dsa::{SlhDsa256s, SigningKey as SlhDsaSk, VerifyingKey as SlhDsaVk};

/// Dual signature combining ML-DSA-87 and SLH-DSA-256s
#[derive(Clone, Debug)]
pub struct DualSignature {
    pub ml_dsa: Vec<u8>,    // 4595 bytes
    pub slh_dsa: Vec<u8>,   // 29792 bytes
}

/// Dual public key
#[derive(Clone, Debug)]
pub struct DualPublicKey {
    pub ml_dsa: MlDsaVk,
    pub slh_dsa: SlhDsaVk,
}

/// Dual signing key (sealed in TEE)
pub struct DualSigningKey {
    ml_dsa: MlDsaSk,
    slh_dsa: SlhDsaSk,
}

impl DualSigningKey {
    /// Generate new key pair from QRNG seed
    pub fn generate(seed: &[u8; 64]) -> Self {
        let (ml_seed, slh_seed) = seed.split_at(32);
        Self {
            ml_dsa: MlDsa87::keygen_from_seed(ml_seed.try_into().unwrap()),
            slh_dsa: SlhDsa256s::keygen_from_seed(slh_seed.try_into().unwrap()),
        }
    }
    
    /// Sign message with both algorithms (parallel execution)
    pub fn sign(&self, msg: &[u8]) -> DualSignature {
        use rayon::prelude::*;
        
        let (ml_sig, slh_sig) = rayon::join(
            || self.ml_dsa.sign(msg),
            || self.slh_dsa.sign(msg),
        );
        
        DualSignature {
            ml_dsa: ml_sig.to_bytes().to_vec(),
            slh_dsa: slh_sig.to_bytes().to_vec(),
        }
    }
    
    pub fn public_key(&self) -> DualPublicKey {
        DualPublicKey {
            ml_dsa: self.ml_dsa.verifying_key(),
            slh_dsa: self.slh_dsa.verifying_key(),
        }
    }
}

impl DualPublicKey {
    /// Verify signature (AND combiner for security guarantee)
    pub fn verify_and(&self, msg: &[u8], sig: &DualSignature) -> Result<(), VerifyError> {
        self.ml_dsa.verify(msg, &sig.ml_dsa)?;
        self.slh_dsa.verify(msg, &sig.slh_dsa)?;
        Ok(())
    }
    
    /// Verify signature (OR combiner for availability)
    pub fn verify_or(&self, msg: &[u8], sig: &DualSignature) -> Result<(), VerifyError> {
        if self.ml_dsa.verify(msg, &sig.ml_dsa).is_ok() {
            return Ok(());
        }
        self.slh_dsa.verify(msg, &sig.slh_dsa)
    }
}
```

### Hybrid KEM Implementation

```rust
// crates/qrms-core/src/apqc/hybrid_kem.rs

use ml_kem::{MlKem1024, DecapsulationKey, EncapsulationKey};
use hqc::{Hqc256, HqcDecapsKey, HqcEncapsKey};
use hkdf::Hkdf;
use sha3::Sha3_256;

pub struct HybridKemPublicKey {
    pub ml_kem: EncapsulationKey,
    pub hqc: HqcEncapsKey,
}

pub struct HybridKemSecretKey {
    ml_kem: DecapsulationKey,
    hqc: HqcDecapsKey,
}

pub struct HybridCiphertext {
    pub ml_kem: Vec<u8>,  // 1568 bytes
    pub hqc: Vec<u8>,     // ~6730 bytes
}

impl HybridKemPublicKey {
    /// Encapsulate to both KEMs, combine shared secrets
    pub fn encapsulate(&self) -> (HybridCiphertext, [u8; 32]) {
        let (ct1, ss1) = self.ml_kem.encapsulate();
        let (ct2, ss2) = self.hqc.encapsulate();
        
        // Combine with HKDF
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(&ss1);
        combined[32..].copy_from_slice(&ss2);
        
        let hk = Hkdf::<Sha3_256>::new(None, &combined);
        let mut shared_secret = [0u8; 32];
        hk.expand(b"QRMS-HybridKEM-v1", &mut shared_secret).unwrap();
        
        (HybridCiphertext { ml_kem: ct1, hqc: ct2 }, shared_secret)
    }
}

impl HybridKemSecretKey {
    pub fn decapsulate(&self, ct: &HybridCiphertext) -> Result<[u8; 32], DecapError> {
        let ss1 = self.ml_kem.decapsulate(&ct.ml_kem)?;
        let ss2 = self.hqc.decapsulate(&ct.hqc)?;
        
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(&ss1);
        combined[32..].copy_from_slice(&ss2);
        
        let hk = Hkdf::<Sha3_256>::new(None, &combined);
        let mut shared_secret = [0u8; 32];
        hk.expand(b"QRMS-HybridKEM-v1", &mut shared_secret).unwrap();
        
        Ok(shared_secret)
    }
}
```

### QRM Risk Scorer

```rust
// crates/qrms-core/src/qrm/risk.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    pub category: ThreatCategory,
    pub severity: f64,      // 0.0 - 1.0
    pub confidence: f64,    // 0.0 - 1.0
    pub timestamp: u64,
    pub source: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ThreatCategory {
    QuantumHardware,
    CryptanalysisLattice,
    CryptanalysisHash,
    CryptanalysisCode,
    SideChannel,
    StandardsGuidance,
}

impl ThreatCategory {
    fn weight(&self) -> f64 {
        match self {
            Self::QuantumHardware => 0.30,
            Self::CryptanalysisLattice => 0.25,
            Self::CryptanalysisHash => 0.20,
            Self::CryptanalysisCode => 0.15,
            Self::SideChannel => 0.05,
            Self::StandardsGuidance => 0.05,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub score: u32,         // 0-10000 basis points
    pub indicators: Vec<ThreatIndicator>,
    pub recommendation: RiskRecommendation,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum RiskRecommendation {
    Continue,
    MonitorClosely,
    ScheduleRotation,
    EmergencyRotation,
}

pub struct RiskScorer {
    threshold_scheduled: u32,   // e.g., 6000
    threshold_emergency: u32,   // e.g., 9000
}

impl RiskScorer {
    pub fn new(threshold_scheduled: u32, threshold_emergency: u32) -> Self {
        Self { threshold_scheduled, threshold_emergency }
    }
    
    pub fn assess(&self, indicators: &[ThreatIndicator]) -> RiskAssessment {
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;
        
        for ind in indicators {
            let w = ind.category.weight() * ind.confidence;
            weighted_sum += ind.severity * w;
            weight_total += w;
        }
        
        let score = if weight_total > 0.0 {
            ((weighted_sum / weight_total) * 10000.0) as u32
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
        
        RiskAssessment {
            score,
            indicators: indicators.to_vec(),
            recommendation,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
```

---

## TEE Sequencer

### Enclave Definition (SGX EDL)

```c
// enclave/Enclave.edl

enclave {
    include "sgx_key_exchange.h"
    
    trusted {
        // Transaction processing
        public sgx_status_t ecall_decrypt_tx(
            [in, size=ct_len] uint8_t* ciphertext,
            size_t ct_len,
            [out, size=pt_len] uint8_t* plaintext,
            size_t pt_len
        );
        
        // Batch ordering
        public sgx_status_t ecall_order_batch(
            [in, count=tx_count] tx_entry_t* transactions,
            size_t tx_count,
            [out] ordered_batch_t* batch
        );
        
        // PQC signing
        public sgx_status_t ecall_sign_batch(
            [in, size=batch_len] uint8_t* batch_data,
            size_t batch_len,
            [out, size=sig_len] uint8_t* signature,
            size_t sig_len
        );
        
        // Generate attestation
        public sgx_status_t ecall_generate_attestation(
            [out, size=report_len] uint8_t* report,
            size_t report_len
        );
        
        // Key rotation
        public sgx_status_t ecall_rotate_keys(
            [in, size=seed_len] uint8_t* seed,
            size_t seed_len
        );
    };
    
    untrusted {
        // Logging (sanitized)
        void ocall_log(
            [in, string] const char* message
        );
        
        // Persistent sealed storage
        sgx_status_t ocall_seal_write(
            [in, size=data_len] uint8_t* data,
            size_t data_len
        );
        
        sgx_status_t ocall_seal_read(
            [out, size=data_len] uint8_t* data,
            size_t data_len
        );
    };
};
```

### Ordering Engine

```rust
// crates/qrms-sequencer/src/enclave/ordering.rs

use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct DecryptedTx {
    pub hash: [u8; 32],
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub priority_fee: u64,
}

impl Ord for DecryptedTx {
    fn cmp(&self, other: &Self) -> Ordering {
        // FCFS: earlier timestamp wins
        other.timestamp.cmp(&self.timestamp)
            .then_with(|| other.hash.cmp(&self.hash))
    }
}

pub enum OrderingMode {
    Fcfs,
    BatchAuction { interval_ms: u64 },
    ThresholdDecrypt { threshold: u32, parties: u32 },
}

pub struct OrderingEngine {
    mode: OrderingMode,
    pending: BinaryHeap<DecryptedTx>,
    batch_start: u64,
}

impl OrderingEngine {
    pub fn new(mode: OrderingMode) -> Self {
        Self {
            mode,
            pending: BinaryHeap::new(),
            batch_start: 0,
        }
    }
    
    pub fn submit(&mut self, tx: DecryptedTx) {
        self.pending.push(tx);
    }
    
    pub fn finalize_batch(&mut self, max_size: usize) -> Vec<DecryptedTx> {
        let mut batch = Vec::with_capacity(max_size);
        
        while batch.len() < max_size {
            match self.pending.pop() {
                Some(tx) => batch.push(tx),
                None => break,
            }
        }
        
        // For batch auction: sort by uniform clearing price
        if matches!(self.mode, OrderingMode::BatchAuction { .. }) {
            batch.sort_by(|a, b| b.priority_fee.cmp(&a.priority_fee));
        }
        
        batch
    }
}
```

### Attestation Generation

```rust
// crates/qrms-sequencer/src/enclave/attestation.rs

use crate::apqc::DualSigningKey;

#[derive(Debug, Clone)]
pub struct TeeAttestation {
    pub platform: Platform,
    pub mrenclave: [u8; 32],
    pub mrsigner: [u8; 32],
    pub report_data: [u8; 64],
    pub platform_report: Vec<u8>,
    pub pqc_signature: DualSignature,
    pub timestamp: u64,
    pub nonce: [u8; 32],
}

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    Sgx,
    Tdx,
    SevSnp,
}

impl TeeAttestation {
    pub fn generate(
        signing_key: &DualSigningKey,
        batch_commitment: &[u8; 32],
    ) -> Result<Self, AttestationError> {
        let nonce = generate_random_nonce();
        let timestamp = current_timestamp();
        
        // Get platform-specific attestation
        #[cfg(feature = "sgx")]
        let (platform_report, mrenclave, mrsigner) = sgx_generate_report(batch_commitment)?;
        
        #[cfg(feature = "tdx")]
        let (platform_report, mrenclave, mrsigner) = tdx_generate_report(batch_commitment)?;
        
        #[cfg(feature = "sev")]
        let (platform_report, mrenclave, mrsigner) = sev_generate_report(batch_commitment)?;
        
        // Create report data
        let mut report_data = [0u8; 64];
        report_data[..32].copy_from_slice(batch_commitment);
        report_data[32..].copy_from_slice(&nonce);
        
        // Sign attestation with PQC
        let attestation_msg = Self::attestation_message(
            &mrenclave,
            &mrsigner,
            &report_data,
            timestamp,
        );
        let pqc_signature = signing_key.sign(&attestation_msg);
        
        Ok(Self {
            platform: Platform::Sgx, // or Tdx/SevSnp
            mrenclave,
            mrsigner,
            report_data,
            platform_report,
            pqc_signature,
            timestamp,
            nonce,
        })
    }
    
    fn attestation_message(
        mrenclave: &[u8; 32],
        mrsigner: &[u8; 32],
        report_data: &[u8; 64],
        timestamp: u64,
    ) -> Vec<u8> {
        let mut msg = Vec::with_capacity(136);
        msg.extend_from_slice(mrenclave);
        msg.extend_from_slice(mrsigner);
        msg.extend_from_slice(report_data);
        msg.extend_from_slice(&timestamp.to_le_bytes());
        msg
    }
}
```

---

## Substrate Pallet

```rust
// crates/qrms-substrate/pallet-apqc/src/lib.rs

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
    
    #[pallet::pallet]
    pub struct Pallet<T>(_);
    
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type PqcVerifier: PqcVerify;
        type QrmOracle: QrmRiskOracle;
        type RotationOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }
    
    #[pallet::storage]
    #[pallet::getter(fn active_algorithms)]
    pub type ActiveAlgorithms<T> = StorageValue<_, AlgorithmSet, ValueQuery>;
    
    #[pallet::storage]
    #[pallet::getter(fn risk_score)]
    pub type RiskScore<T> = StorageValue<_, u32, ValueQuery>;
    
    #[pallet::storage]
    #[pallet::getter(fn pending_rotation)]
    pub type PendingRotation<T: Config> = StorageValue<_, (AlgorithmSet, BlockNumberFor<T>)>;
    
    #[pallet::storage]
    #[pallet::getter(fn sequencer_key)]
    pub type SequencerPublicKey<T> = StorageValue<_, DualPublicKey>;
    
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
    pub struct AlgorithmSet {
        pub signatures: BoundedVec<SignatureAlgorithm, ConstU32<4>>,
        pub kems: BoundedVec<KemAlgorithm, ConstU32<4>>,
    }
    
    #[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum SignatureAlgorithm {
        MlDsa44,
        MlDsa65,
        MlDsa87,
        SlhDsa128s,
        SlhDsa128f,
        SlhDsa256s,
    }
    
    #[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum KemAlgorithm {
        MlKem512,
        MlKem768,
        MlKem1024,
        Hqc128,
        Hqc192,
        Hqc256,
    }
    
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        RiskScoreUpdated { score: u32 },
        RotationScheduled { new_set: AlgorithmSet, effective_block: BlockNumberFor<T> },
        RotationExecuted { new_set: AlgorithmSet },
        EmergencyRotation { new_set: AlgorithmSet },
        SequencerKeyUpdated { new_key: DualPublicKey },
    }
    
    #[pallet::error]
    pub enum Error<T> {
        InvalidQrmProof,
        RotationAlreadyPending,
        RiskBelowThreshold,
        InvalidSignature,
        SequencerNotRegistered,
    }
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            // Check if pending rotation should execute
            if let Some((new_set, effective_block)) = PendingRotation::<T>::get() {
                if n >= effective_block {
                    ActiveAlgorithms::<T>::put(new_set.clone());
                    PendingRotation::<T>::kill();
                    Self::deposit_event(Event::RotationExecuted { new_set });
                }
            }
            
            // Update risk score from oracle
            let risk = T::QrmOracle::get_current_risk();
            RiskScore::<T>::put(risk);
            Self::deposit_event(Event::RiskScoreUpdated { score: risk });
            
            Weight::from_parts(10_000, 0)
        }
    }
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(100_000)]
        pub fn schedule_rotation(
            origin: OriginFor<T>,
            new_set: AlgorithmSet,
            qrm_proof: QrmSignal,
        ) -> DispatchResult {
            T::RotationOrigin::ensure_origin(origin)?;
            
            ensure!(qrm_proof.verify(), Error::<T>::InvalidQrmProof);
            ensure!(PendingRotation::<T>::get().is_none(), Error::<T>::RotationAlreadyPending);
            
            let current_block = frame_system::Pallet::<T>::block_number();
            let grace_period: BlockNumberFor<T> = 1000u32.into();
            let effective_block = current_block + grace_period;
            
            PendingRotation::<T>::put((new_set.clone(), effective_block));
            Self::deposit_event(Event::RotationScheduled { new_set, effective_block });
            
            Ok(())
        }
        
        #[pallet::call_index(1)]
        #[pallet::weight(50_000)]
        pub fn emergency_rotation(
            origin: OriginFor<T>,
            new_set: AlgorithmSet,
            qrm_proof: QrmSignal,
        ) -> DispatchResult {
            T::RotationOrigin::ensure_origin(origin)?;
            
            ensure!(qrm_proof.verify(), Error::<T>::InvalidQrmProof);
            ensure!(
                qrm_proof.risk_score >= 9000,
                Error::<T>::RiskBelowThreshold
            );
            
            // Immediate rotation
            ActiveAlgorithms::<T>::put(new_set.clone());
            PendingRotation::<T>::kill();
            Self::deposit_event(Event::EmergencyRotation { new_set });
            
            Ok(())
        }
        
        #[pallet::call_index(2)]
        #[pallet::weight(20_000)]
        pub fn register_sequencer_key(
            origin: OriginFor<T>,
            public_key: DualPublicKey,
            attestation: TeeAttestation,
        ) -> DispatchResult {
            T::RotationOrigin::ensure_origin(origin)?;
            
            // Verify TEE attestation
            ensure!(
                attestation.verify(&public_key),
                Error::<T>::InvalidSignature
            );
            
            SequencerPublicKey::<T>::put(public_key.clone());
            Self::deposit_event(Event::SequencerKeyUpdated { new_key: public_key });
            
            Ok(())
        }
    }
}
```

---

## Cosmos SDK Module

```go
// crates/qrms-cosmos/x/apqc/keeper/keeper.go

package keeper

import (
    "github.com/cosmos/cosmos-sdk/codec"
    storetypes "github.com/cosmos/cosmos-sdk/store/types"
    sdk "github.com/cosmos/cosmos-sdk/types"
    
    "qrms/x/apqc/types"
    "qrms/pqc"
)

type Keeper struct {
    cdc      codec.BinaryCodec
    storeKey storetypes.StoreKey
    
    pqcSigner  pqc.DualSigner
    pqcVerify  pqc.DualVerifier
    qrmOracle  types.QrmOracle
}

func NewKeeper(
    cdc codec.BinaryCodec,
    storeKey storetypes.StoreKey,
    pqcSigner pqc.DualSigner,
    qrmOracle types.QrmOracle,
) Keeper {
    return Keeper{
        cdc:       cdc,
        storeKey:  storeKey,
        pqcSigner: pqcSigner,
        qrmOracle: qrmOracle,
    }
}

// GetActiveAlgorithms returns current algorithm set
func (k Keeper) GetActiveAlgorithms(ctx sdk.Context) types.AlgorithmSet {
    store := ctx.KVStore(k.storeKey)
    bz := store.Get(types.ActiveAlgorithmsKey)
    
    var algSet types.AlgorithmSet
    k.cdc.MustUnmarshal(bz, &algSet)
    return algSet
}

// VerifyDualSignature verifies a dual PQC signature
func (k Keeper) VerifyDualSignature(
    ctx sdk.Context,
    msg []byte,
    sig types.DualSignature,
    pubkey types.DualPublicKey,
    mode types.CombinerMode,
) error {
    switch mode {
    case types.CombinerModeAnd:
        // Both must be valid (security)
        if err := k.pqcVerify.VerifyMLDSA(msg, sig.MlDsa, pubkey.MlDsa); err != nil {
            return types.ErrInvalidMLDSASignature
        }
        if err := k.pqcVerify.VerifySLHDSA(msg, sig.SlhDsa, pubkey.SlhDsa); err != nil {
            return types.ErrInvalidSLHDSASignature
        }
        
    case types.CombinerModeOr:
        // Either valid (availability)
        mlErr := k.pqcVerify.VerifyMLDSA(msg, sig.MlDsa, pubkey.MlDsa)
        slhErr := k.pqcVerify.VerifySLHDSA(msg, sig.SlhDsa, pubkey.SlhDsa)
        if mlErr != nil && slhErr != nil {
            return types.ErrInvalidSignature
        }
    }
    
    return nil
}

// ScheduleRotation schedules an algorithm rotation
func (k Keeper) ScheduleRotation(
    ctx sdk.Context,
    newSet types.AlgorithmSet,
    qrmProof types.QrmSignal,
) error {
    if err := qrmProof.Verify(); err != nil {
        return types.ErrInvalidQrmProof
    }
    
    effectiveHeight := ctx.BlockHeight() + types.GracePeriodBlocks
    
    rotation := types.PendingRotation{
        NewSet:          newSet,
        EffectiveHeight: effectiveHeight,
    }
    
    store := ctx.KVStore(k.storeKey)
    store.Set(types.PendingRotationKey, k.cdc.MustMarshal(&rotation))
    
    ctx.EventManager().EmitEvent(
        sdk.NewEvent(
            types.EventTypeRotationScheduled,
            sdk.NewAttribute(types.AttributeKeyEffectiveHeight, fmt.Sprint(effectiveHeight)),
        ),
    )
    
    return nil
}

// BeginBlocker executes pending rotations
func (k Keeper) BeginBlocker(ctx sdk.Context) {
    store := ctx.KVStore(k.storeKey)
    
    // Check pending rotation
    bz := store.Get(types.PendingRotationKey)
    if bz != nil {
        var rotation types.PendingRotation
        k.cdc.MustUnmarshal(bz, &rotation)
        
        if ctx.BlockHeight() >= rotation.EffectiveHeight {
            store.Set(types.ActiveAlgorithmsKey, k.cdc.MustMarshal(&rotation.NewSet))
            store.Delete(types.PendingRotationKey)
            
            ctx.EventManager().EmitEvent(
                sdk.NewEvent(types.EventTypeRotationExecuted),
            )
        }
    }
    
    // Update risk score
    risk := k.qrmOracle.GetCurrentRisk(ctx)
    store.Set(types.RiskScoreKey, sdk.Uint64ToBigEndian(uint64(risk)))
}
```

---

## OP Stack Integration

### PQC Batch Derivation

```go
// crates/qrms-opstack/op-node/rollup/derive/pqc_batch.go

package derive

import (
    "errors"
    
    "github.com/ethereum-optimism/optimism/op-node/rollup"
    "qrms/pqc"
)

type PQCBatchSubmitter struct {
    cfg        *rollup.Config
    dualSigner *pqc.DualSigner
    verifier   *pqc.DualVerifier
}

func NewPQCBatchSubmitter(cfg *rollup.Config, signer *pqc.DualSigner) *PQCBatchSubmitter {
    return &PQCBatchSubmitter{
        cfg:        cfg,
        dualSigner: signer,
        verifier:   pqc.NewDualVerifier(),
    }
}

// SignBatch signs a batch with dual PQC signatures
func (s *PQCBatchSubmitter) SignBatch(batch *BatchData) (*SignedBatch, error) {
    batchBytes, err := batch.Encode()
    if err != nil {
        return nil, err
    }
    
    sig := s.dualSigner.Sign(batchBytes)
    
    return &SignedBatch{
        Data:      batch,
        Signature: sig,
    }, nil
}

// VerifyBatch verifies a PQC-signed batch
func (s *PQCBatchSubmitter) VerifyBatch(
    signed *SignedBatch,
    pubkey *pqc.DualPublicKey,
) error {
    batchBytes, err := signed.Data.Encode()
    if err != nil {
        return err
    }
    
    // Use AND combiner for security
    if err := s.verifier.VerifyAnd(batchBytes, signed.Signature, pubkey); err != nil {
        return errors.New("invalid batch signature")
    }
    
    return nil
}

type SignedBatch struct {
    Data      *BatchData
    Signature *pqc.DualSignature
}
```

### Solidity Verifier Contract

```solidity
// crates/qrms-opstack/contracts/PQCVerifier.sol

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IPQCVerifier {
    function verifyMLDSA(
        bytes calldata message,
        bytes calldata signature,
        bytes calldata publicKey
    ) external view returns (bool);
    
    function verifySLHDSA(
        bytes calldata message,
        bytes calldata signature,
        bytes calldata publicKey
    ) external view returns (bool);
}

contract PQCBatchVerifier {
    IPQCVerifier public immutable mldsaVerifier;
    IPQCVerifier public immutable slhdsaVerifier;
    
    // ML-DSA-87 sizes
    uint256 public constant MLDSA_PUBKEY_SIZE = 2592;
    uint256 public constant MLDSA_SIG_SIZE = 4595;
    
    // SLH-DSA-256s sizes
    uint256 public constant SLHDSA_PUBKEY_SIZE = 64;
    uint256 public constant SLHDSA_SIG_SIZE = 29792;
    
    struct DualPublicKey {
        bytes mldsa;    // 2592 bytes
        bytes slhdsa;   // 64 bytes
    }
    
    struct DualSignature {
        bytes mldsa;    // 4595 bytes
        bytes slhdsa;   // 29792 bytes
    }
    
    mapping(address => DualPublicKey) public sequencerKeys;
    
    event BatchVerified(bytes32 indexed batchHash, address indexed sequencer);
    
    constructor(address _mldsaVerifier, address _slhdsaVerifier) {
        mldsaVerifier = IPQCVerifier(_mldsaVerifier);
        slhdsaVerifier = IPQCVerifier(_slhdsaVerifier);
    }
    
    /// @notice Verify a dual-signed batch (AND combiner)
    function verifyBatch(
        bytes calldata batchData,
        DualSignature calldata sig,
        address sequencer
    ) external view returns (bool) {
        DualPublicKey storage pk = sequencerKeys[sequencer];
        
        require(pk.mldsa.length == MLDSA_PUBKEY_SIZE, "Invalid ML-DSA key");
        require(pk.slhdsa.length == SLHDSA_PUBKEY_SIZE, "Invalid SLH-DSA key");
        
        // Both must be valid (AND combiner)
        bool mldsaValid = mldsaVerifier.verifyMLDSA(batchData, sig.mldsa, pk.mldsa);
        bool slhdsaValid = slhdsaVerifier.verifySLHDSA(batchData, sig.slhdsa, pk.slhdsa);
        
        return mldsaValid && slhdsaValid;
    }
    
    /// @notice Register sequencer public key with TEE attestation
    function registerSequencer(
        DualPublicKey calldata publicKey,
        bytes calldata teeAttestation
    ) external {
        // Verify TEE attestation (implementation depends on platform)
        require(_verifyTeeAttestation(teeAttestation, publicKey), "Invalid attestation");
        
        sequencerKeys[msg.sender] = publicKey;
    }
    
    function _verifyTeeAttestation(
        bytes calldata attestation,
        DualPublicKey calldata publicKey
    ) internal view returns (bool) {
        // Simplified: verify platform report + PQC signature
        // In production: full SGX/TDX/SEV attestation verification
        return attestation.length > 0;
    }
}
```

---

## Docker Deployment

```yaml
# docker/docker-compose.yml

version: "3.8"

services:
  qrms-sequencer:
    build:
      context: ..
      dockerfile: docker/Dockerfile.sequencer
    devices:
      - /dev/sgx_enclave:/dev/sgx_enclave
      - /dev/sgx_provision:/dev/sgx_provision
    volumes:
      - sealed_data:/sealed
    environment:
      - TEE_PLATFORM=SGX
      - APQC_PRIMARY_SIG=ML-DSA-87
      - APQC_SECONDARY_SIG=SLH-DSA-256s
      - SEQ_ORDERING_MODE=BATCH_AUCTION
      - SEQ_BATCH_TIMEOUT_MS=500
    ports:
      - "9944:9944"  # RPC
      - "30333:30333"  # P2P
    networks:
      - qrms-net
    depends_on:
      - qrm-agent

  qrm-agent:
    build:
      context: ..
      dockerfile: docker/Dockerfile.qrm
    environment:
      - QRM_ARXIV_POLLING_INTERVAL=86400
      - QRM_RISK_THRESHOLD_SCHEDULED=6000
      - QRM_RISK_THRESHOLD_EMERGENCY=9000
    volumes:
      - qrm_data:/data
    networks:
      - qrms-net

  node:
    build:
      context: ..
      dockerfile: docker/Dockerfile.node
    environment:
      - CHAIN_SPEC=/specs/qrms-local.json
    volumes:
      - chain_data:/data
    ports:
      - "9933:9933"
      - "9934:9934"
    networks:
      - qrms-net
    depends_on:
      - qrms-sequencer

volumes:
  sealed_data:
  qrm_data:
  chain_data:

networks:
  qrms-net:
    driver: bridge
```

### Sequencer Dockerfile

```dockerfile
# docker/Dockerfile.sequencer

FROM gramineproject/gramine:v1.6 AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    protobuf-compiler \
    libprotobuf-dev \
    curl

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build
COPY . .

# Build enclave
RUN cd enclave && make SGX=1

# Build sequencer
RUN cd crates/qrms-sequencer && cargo build --release --features sgx

FROM gramineproject/gramine:v1.6

COPY --from=builder /build/target/release/qrms-sequencer /usr/local/bin/
COPY --from=builder /build/enclave/qrms_enclave.signed.so /enclave/
COPY --from=builder /build/enclave/qrms_enclave.manifest.sgx /enclave/

EXPOSE 9944 30333

ENTRYPOINT ["/usr/local/bin/qrms-sequencer"]
```

---

## Dependencies

### Cargo.toml (qrms-core)

```toml
[package]
name = "qrms-core"
version = "0.1.0"
edition = "2021"

[dependencies]
# PQC
ml-dsa = { git = "https://github.com/RustCrypto/signatures", features = ["ml-dsa-87"] }
slh-dsa = { git = "https://github.com/RustCrypto/signatures", features = ["slh-dsa-shake-256s"] }
ml-kem = { git = "https://github.com/RustCrypto/KEMs", features = ["ml-kem-1024"] }
hqc = { version = "0.1", optional = true }

# Crypto
hkdf = "0.12"
sha3 = "0.10"
rand = "0.8"
rand_chacha = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Async
tokio = { version = "1", features = ["full"] }
rayon = "1.8"

# SGX (optional)
sgx_types = { version = "2.0", optional = true }
sgx_urts = { version = "2.0", optional = true }

[features]
default = []
sgx = ["sgx_types", "sgx_urts"]
tdx = []
sev = []
hqc = ["dep:hqc"]
```

---

## References

1. NIST FIPS 203: ML-KEM
2. NIST FIPS 204: ML-DSA
3. NIST FIPS 205: SLH-DSA
4. Intel SGX Developer Reference
5. AMD SEV-SNP ABI Specification
6. Substrate Developer Hub
7. Cosmos SDK Documentation
8. OP Stack Specifications
