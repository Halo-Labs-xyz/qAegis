//! End-to-end hybrid execution engine with security, governance, and claims gating.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::governance_plane::{AuditLog, GovernanceAdjudicator, GovernanceDecision};
use crate::hybrid_quantum::{ExecutionMode, HybridQuantumPolicy, QuantumConfig, RoutingDecision};
use crate::security_plane::{
    PersistenceConsent, PrivacyMode, RetentionPolicy, SecurityConfig, TransientStore,
};

#[derive(Debug, Clone, Deserialize)]
pub struct HybridSolveRequest {
    pub goal: String,
    #[serde(default)]
    pub max_depth: u32,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    #[serde(default)]
    pub privacy_mode: PrivacyMode,
    #[serde(default)]
    pub persistence_consent: PersistenceConsent,
    #[serde(default = "default_governance_profile")]
    pub governance_profile: String,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_governance_profile() -> String {
    "freedom_v1".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct HybridSolveResponse {
    pub execution_id: String,
    pub status: String,
    pub result: String,
    pub governance_decision_id: String,
    pub audit_hash: String,
    pub retention_policy_applied: String,
    pub quantum_backend: String,
    pub fallback_used: bool,
    pub claims_manifest: ClaimsManifest,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaimsManifest {
    pub config_digest: String,
    pub seed: u64,
    pub commit_sha: String,
    pub task_set_digest: String,
    pub reproducible_execution_manifest: String,
    pub public_claim_allowed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HybridExecutionRecord {
    pub execution_id: String,
    pub status: String,
    pub stored_goal: String,
    pub stored_result: String,
    pub created_at: DateTime<Utc>,
    pub governance_decision_id: String,
    pub audit_hash: String,
    pub retention_policy_applied: String,
    pub quantum_backend: String,
    pub fallback_used: bool,
    pub claims_manifest: ClaimsManifest,
}

pub struct HybridExecutionEngine {
    policy: HybridQuantumPolicy,
    audit_log: AuditLog,
    records: HashMap<String, HybridExecutionRecord>,
    transient_store: TransientStore,
    default_security: SecurityConfig,
}

impl HybridExecutionEngine {
    pub fn new(audit_log_path: impl Into<std::path::PathBuf>) -> Self {
        let policy = HybridQuantumPolicy::new(load_quantum_config_from_env());
        Self {
            policy,
            audit_log: AuditLog::new(audit_log_path),
            records: HashMap::new(),
            transient_store: TransientStore::new(),
            default_security: SecurityConfig::default(),
        }
    }

    pub fn execute(&mut self, request: HybridSolveRequest) -> Result<HybridSolveResponse, String> {
        if request.goal.trim().is_empty() {
            return Err("goal cannot be empty".to_string());
        }

        let execution_id = Uuid::new_v4().to_string();
        let retention = RetentionPolicy::new(request.privacy_mode, request.persistence_consent);

        let routing = self.route_task(&request);
        let raw_output = self.execute_task_payload(&request, &routing);

        let adjudicator = GovernanceAdjudicator::new(&request.governance_profile);
        let mut decision = adjudicator.adjudicate(&raw_output);

        let final_output = if decision.blocked {
            "Output blocked by governance policy.".to_string()
        } else {
            raw_output
        };

        let audit_hash = self.append_audit_log(&mut decision)?;
        let claims_manifest = self.build_claims_manifest(&request);

        let stored_goal = retention.sanitize_text_for_persistence(&request.goal);
        let stored_result = retention.sanitize_text_for_persistence(&final_output);

        let record = HybridExecutionRecord {
            execution_id: execution_id.clone(),
            status: if decision.blocked {
                "blocked".to_string()
            } else {
                "completed".to_string()
            },
            stored_goal,
            stored_result,
            created_at: Utc::now(),
            governance_decision_id: decision.decision_id.clone(),
            audit_hash: audit_hash.clone(),
            retention_policy_applied: retention.retention_policy_applied(),
            quantum_backend: routing.quantum_backend.clone(),
            fallback_used: routing.fallback_used,
            claims_manifest: claims_manifest.clone(),
        };

        self.transient_store.track(execution_id.clone());
        self.records.insert(execution_id.clone(), record.clone());

        Ok(HybridSolveResponse {
            execution_id,
            status: record.status,
            result: final_output,
            governance_decision_id: record.governance_decision_id,
            audit_hash,
            retention_policy_applied: record.retention_policy_applied,
            quantum_backend: record.quantum_backend,
            fallback_used: record.fallback_used,
            claims_manifest,
        })
    }

    fn route_task(&self, request: &HybridSolveRequest) -> RoutingDecision {
        match request.execution_mode {
            ExecutionMode::Classical => RoutingDecision {
                selected_task_type: "THINK".to_string(),
                selected_tool: "terminal".to_string(),
                task_routing_logits: HashMap::from([("THINK".to_string(), 1.0)]),
                tool_selection_logits: HashMap::from([("terminal".to_string(), 1.0)]),
                quantum_backend: "classical".to_string(),
                fallback_used: false,
                policy_parameters: self.policy.config.policy_params.clone(),
                feature_digest: hash_text(&request.goal),
            },
            ExecutionMode::HybridQuantum => self.policy.route(
                &request.goal,
                request.max_depth,
                request.metadata.len().max(1),
            ),
        }
    }

    fn execute_task_payload(
        &self,
        request: &HybridSolveRequest,
        routing: &RoutingDecision,
    ) -> String {
        format!(
            "goal='{}' routed_task_type={} routed_tool={} mode={:?} profile={} metadata_keys={}",
            request.goal,
            routing.selected_task_type,
            routing.selected_tool,
            request.execution_mode,
            request.governance_profile,
            request.metadata.len()
        )
    }

    fn append_audit_log(&mut self, decision: &mut GovernanceDecision) -> Result<String, String> {
        let hash = self.audit_log.append(decision)?;
        decision.audit_hash = hash.clone();
        Ok(hash)
    }

    fn build_claims_manifest(&self, request: &HybridSolveRequest) -> ClaimsManifest {
        let config_payload = json!({
            "execution_mode": request.execution_mode,
            "privacy_mode": request.privacy_mode,
            "persistence_consent": request.persistence_consent,
            "governance_profile": request.governance_profile,
            "max_depth": request.max_depth,
            "policy_config": self.policy.config,
        });

        let config_digest = hash_text(&config_payload.to_string());
        let task_set_digest = hash_text(&request.goal);
        let commit_sha = std::env::var("GIT_COMMIT_SHA").unwrap_or_else(|_| "unknown".to_string());
        let reproducible_execution_manifest = hash_text(&format!(
            "{}:{}:{}:{}",
            config_digest, self.policy.config.seed, task_set_digest, commit_sha,
        ));

        let public_claim_allowed = !config_digest.is_empty()
            && self.policy.config.seed > 0
            && !commit_sha.is_empty()
            && commit_sha != "unknown"
            && !task_set_digest.is_empty()
            && !reproducible_execution_manifest.is_empty();

        ClaimsManifest {
            config_digest,
            seed: self.policy.config.seed,
            commit_sha,
            task_set_digest,
            reproducible_execution_manifest,
            public_claim_allowed,
        }
    }

    pub fn get_execution(&self, execution_id: &str) -> Option<HybridExecutionRecord> {
        self.records.get(execution_id).cloned()
    }

    pub fn transparency_log(&self, max_lines: usize) -> Vec<String> {
        self.audit_log.read_recent(max_lines)
    }

    pub fn purge_transient(&mut self) -> usize {
        let ttl = Duration::from_secs(self.default_security.transient_ttl_minutes * 60);
        self.transient_store.purge_expired(ttl)
    }
}

fn hash_text(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn load_quantum_config_from_env() -> QuantumConfig {
    let maybe_path = std::env::var("HYBRID_POLICY_CONFIG_PATH").ok();
    match maybe_path {
        Some(path) => match QuantumConfig::from_json_file(Path::new(&path)) {
            Ok(config) => {
                tracing::info!("Loaded hybrid policy config from {}", path);
                config
            }
            Err(err) => {
                tracing::warn!(
                    "Failed to load HYBRID_POLICY_CONFIG_PATH={} ({}) - using defaults",
                    path,
                    err
                );
                QuantumConfig::default()
            }
        },
        None => QuantumConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claims_manifest_has_required_fields() {
        let mut engine =
            HybridExecutionEngine::new(std::env::temp_dir().join("qaegis_claims_audit.log"));
        let req = HybridSolveRequest {
            goal: "Fix flaky parser tests".to_string(),
            max_depth: 2,
            execution_mode: ExecutionMode::HybridQuantum,
            privacy_mode: PrivacyMode::ZeroRetention,
            persistence_consent: PersistenceConsent::None,
            governance_profile: "freedom_v1".to_string(),
            metadata: HashMap::new(),
        };

        let result = engine.execute(req).expect("execution should succeed");
        assert!(!result.claims_manifest.config_digest.is_empty());
        assert!(!result.claims_manifest.task_set_digest.is_empty());
        assert!(!result
            .claims_manifest
            .reproducible_execution_manifest
            .is_empty());
    }

    #[test]
    fn claims_gate_blocks_unknown_commit_sha() {
        std::env::remove_var("GIT_COMMIT_SHA");
        let mut engine =
            HybridExecutionEngine::new(std::env::temp_dir().join("qaegis_claims_unknown.log"));
        let req = HybridSolveRequest {
            goal: "Fix flaky parser tests".to_string(),
            max_depth: 1,
            execution_mode: ExecutionMode::HybridQuantum,
            privacy_mode: PrivacyMode::ZeroRetention,
            persistence_consent: PersistenceConsent::None,
            governance_profile: "freedom_v1".to_string(),
            metadata: HashMap::new(),
        };

        let result = engine.execute(req).expect("execution should succeed");
        assert!(!result.claims_manifest.public_claim_allowed);
        assert_eq!(result.claims_manifest.commit_sha, "unknown");
    }
}
