//! Zero-retention security plane and consent controls.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyMode {
    ZeroRetention,
}

impl Default for PrivacyMode {
    fn default() -> Self {
        Self::ZeroRetention
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PersistenceConsent {
    None,
    MetadataOnly,
    Full,
}

impl Default for PersistenceConsent {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub privacy_mode: PrivacyMode,
    pub persistence_consent: PersistenceConsent,
    pub transient_ttl_minutes: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            privacy_mode: PrivacyMode::ZeroRetention,
            persistence_consent: PersistenceConsent::None,
            transient_ttl_minutes: 15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RedactionGuard {
    patterns: Vec<Regex>,
}

impl RedactionGuard {
    pub fn new() -> Self {
        let patterns = vec![
            Regex::new(r"(?i)api[_-]?key\s*[:=]\s*[A-Za-z0-9_\-]{10,}").unwrap(),
            Regex::new(r"(?i)secret\s*[:=]\s*[A-Za-z0-9_\-]{8,}").unwrap(),
            Regex::new(r"(?i)token\s*[:=]\s*[A-Za-z0-9_\-]{10,}").unwrap(),
            Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
        ];
        Self { patterns }
    }

    pub fn contains_sensitive_data(&self, input: &str) -> bool {
        if self.patterns.iter().any(|p| p.is_match(input)) {
            return true;
        }

        // High-entropy token check for long strings
        let compact: String = input.chars().filter(|c| !c.is_whitespace()).collect();
        if compact.len() < 20 {
            return false;
        }

        shannon_entropy(&compact) > 4.2
    }

    pub fn redact_text(&self, input: &str) -> String {
        let mut out = input.to_string();
        for pattern in &self.patterns {
            out = pattern.replace_all(&out, "[REDACTED]").to_string();
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    pub privacy_mode: PrivacyMode,
    pub persistence_consent: PersistenceConsent,
    guard: RedactionGuard,
}

impl RetentionPolicy {
    pub fn new(privacy_mode: PrivacyMode, persistence_consent: PersistenceConsent) -> Self {
        Self {
            privacy_mode,
            persistence_consent,
            guard: RedactionGuard::new(),
        }
    }

    pub fn retention_policy_applied(&self) -> String {
        format!("{:?}:{:?}", self.privacy_mode, self.persistence_consent).to_lowercase()
    }

    pub fn allows_plaintext_persistence(&self) -> bool {
        self.persistence_consent == PersistenceConsent::Full
    }

    pub fn sanitize_text_for_persistence(&self, text: &str) -> String {
        match self.persistence_consent {
            PersistenceConsent::Full => self.guard.redact_text(text),
            PersistenceConsent::MetadataOnly => {
                format!("[REDACTED_SHA256:{}]", hash_text(text))
            }
            PersistenceConsent::None => hash_text(text),
        }
    }

    pub fn sanitize_json_for_persistence(&self, value: Value) -> Value {
        match value {
            Value::String(s) => Value::String(self.sanitize_text_for_persistence(&s)),
            Value::Array(items) => Value::Array(
                items
                    .into_iter()
                    .map(|v| self.sanitize_json_for_persistence(v))
                    .collect(),
            ),
            Value::Object(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map {
                    out.insert(k, self.sanitize_json_for_persistence(v));
                }
                Value::Object(out)
            }
            other => other,
        }
    }

    pub fn should_block_write(&self, text: &str) -> bool {
        self.persistence_consent != PersistenceConsent::Full
            && self.guard.contains_sensitive_data(text)
    }
}

#[derive(Debug, Clone)]
pub struct TransientStoreEntry {
    pub created_at: Instant,
}

#[derive(Debug, Clone)]
pub struct TransientStore {
    entries: HashMap<String, TransientStoreEntry>,
}

impl TransientStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn track(&mut self, id: impl Into<String>) {
        self.entries.insert(
            id.into(),
            TransientStoreEntry {
                created_at: Instant::now(),
            },
        );
    }

    pub fn purge_expired(&mut self, ttl: Duration) -> usize {
        let before = self.entries.len();
        self.entries
            .retain(|_, entry| entry.created_at.elapsed() < ttl);
        before.saturating_sub(self.entries.len())
    }
}

fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

fn shannon_entropy(input: &str) -> f64 {
    if input.is_empty() {
        return 0.0;
    }
    let mut map: HashMap<char, usize> = HashMap::new();
    for c in input.chars() {
        *map.entry(c).or_insert(0) += 1;
    }
    let len = input.len() as f64;
    map.values()
        .map(|count| {
            let p = *count as f64 / len;
            -p * p.log2()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_hashes_plaintext_when_consent_none() {
        let policy = RetentionPolicy::new(PrivacyMode::ZeroRetention, PersistenceConsent::None);
        let sanitized = policy.sanitize_text_for_persistence("sensitive prompt text");
        assert!(sanitized.starts_with("sha256:"));
        assert!(!sanitized.contains("sensitive"));
    }

    #[test]
    fn redaction_guard_blocks_secrets_without_full_consent() {
        let policy =
            RetentionPolicy::new(PrivacyMode::ZeroRetention, PersistenceConsent::MetadataOnly);
        assert!(policy.should_block_write("api_key=sk_test_1234567890abcdef"));
    }
}
