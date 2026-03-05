//! Governance constitution checks, adjudication, and transparency log.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    pub version: String,
    pub blocked_phrases: Vec<String>,
}

impl Constitution {
    pub fn freedom_v1() -> Self {
        Self {
            version: "freedom_v1".to_string(),
            blocked_phrases: vec![
                "BEGIN_PRIVATE_KEY".to_string(),
                "aws_secret_access_key".to_string(),
                "openai_api_key".to_string(),
            ],
        }
    }

    pub fn evaluate(&self, output: &str) -> (bool, String) {
        let output_lower = output.to_lowercase();
        for blocked in &self.blocked_phrases {
            if output_lower.contains(&blocked.to_lowercase()) {
                return (false, "secret_leak_prevented".to_string());
            }
        }
        (true, "policy_compliant".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceDecision {
    pub decision_id: String,
    pub policy_version: String,
    pub rationale_class: String,
    pub allowed: bool,
    pub blocked: bool,
    pub created_at: DateTime<Utc>,
    pub audit_hash: String,
}

#[derive(Debug, Clone)]
pub struct AuditLog {
    path: PathBuf,
    last_hash: String,
}

impl AuditLog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let last_hash = load_last_hash(&path).unwrap_or_else(|| "GENESIS".to_string());
        Self {
            path,
            last_hash,
        }
    }

    pub fn append(&mut self, decision: &GovernanceDecision) -> Result<String, String> {
        let payload = serde_json::to_string(decision).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        hasher.update(self.last_hash.as_bytes());
        hasher.update(payload.as_bytes());
        let chain_hash = hex::encode(hasher.finalize());

        let line = serde_json::json!({
            "decision": decision,
            "prev_hash": self.last_hash,
            "hash": chain_hash,
        });

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| e.to_string())?;

        writeln!(file, "{}", line).map_err(|e| e.to_string())?;
        self.last_hash = chain_hash.clone();
        Ok(chain_hash)
    }

    pub fn read_recent(&self, max_lines: usize) -> Vec<String> {
        if !self.path.exists() {
            return Vec::new();
        }

        let content = fs::read_to_string(&self.path).unwrap_or_default();
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        if lines.len() > max_lines {
            lines = lines.split_off(lines.len() - max_lines);
        }
        lines
    }
}

fn load_last_hash(path: &PathBuf) -> Option<String> {
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let last = content.lines().last()?;
    let parsed: serde_json::Value = serde_json::from_str(last).ok()?;
    parsed
        .get("hash")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[derive(Debug, Clone)]
pub struct GovernanceAdjudicator {
    constitution: Constitution,
    policy_version: String,
}

impl GovernanceAdjudicator {
    pub fn new(profile: &str) -> Self {
        let constitution = match profile {
            "freedom_v1" => Constitution::freedom_v1(),
            _ => Constitution::freedom_v1(),
        };

        Self {
            policy_version: constitution.version.clone(),
            constitution,
        }
    }

    pub fn adjudicate(&self, output: &str) -> GovernanceDecision {
        let (allowed, rationale_class) = self.constitution.evaluate(output);
        GovernanceDecision {
            decision_id: Uuid::new_v4().to_string(),
            policy_version: self.policy_version.clone(),
            rationale_class,
            allowed,
            blocked: !allowed,
            created_at: Utc::now(),
            audit_hash: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjudicator_is_deterministic_for_blocked_output() {
        let adjudicator = GovernanceAdjudicator::new("freedom_v1");
        let d1 = adjudicator.adjudicate("leak BEGIN_PRIVATE_KEY now");
        let d2 = adjudicator.adjudicate("leak BEGIN_PRIVATE_KEY now");
        assert!(d1.blocked);
        assert!(d2.blocked);
        assert_eq!(d1.rationale_class, d2.rationale_class);
    }
}
