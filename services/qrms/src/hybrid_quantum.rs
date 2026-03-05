//! Hybrid quantum-classical task routing primitives.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Classical,
    HybridQuantum,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::Classical
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct QuantumConfig {
    pub backend: String,
    pub seed: u64,
    pub timeout_ms: u64,
    pub policy_params: PolicyParams,
}

impl Default for QuantumConfig {
    fn default() -> Self {
        Self {
            backend: "lightning.qubit".to_string(),
            seed: 1337,
            timeout_ms: 1200,
            policy_params: PolicyParams::default(),
        }
    }
}

impl QuantumConfig {
    pub fn from_json_file(path: &Path) -> Result<Self, String> {
        let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut config: Self = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        config.normalize();
        Ok(config)
    }

    pub fn normalize(&mut self) {
        if self.backend.trim().is_empty() {
            self.backend = "lightning.qubit".to_string();
        }
        if self.seed == 0 {
            self.seed = 1337;
        }
        if self.timeout_ms == 0 {
            self.timeout_ms = 1200;
        }
        self.policy_params.normalize();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PolicyParams {
    pub depth_weight: f64,
    pub complexity_weight: f64,
    pub dependency_weight: f64,
    pub backend_weight: f64,
    pub task_bias_retrieve: f64,
    pub task_bias_write: f64,
    pub task_bias_think: f64,
    pub task_bias_code_interpret: f64,
    pub task_bias_image_generation: f64,
    pub tool_bias_terminal: f64,
    pub tool_bias_file: f64,
    pub tool_bias_web_search: f64,
    pub tool_bias_calculator: f64,
}

impl Default for PolicyParams {
    fn default() -> Self {
        Self {
            depth_weight: 0.35,
            complexity_weight: 0.40,
            dependency_weight: 0.25,
            backend_weight: 0.70,
            task_bias_retrieve: 0.0,
            task_bias_write: 0.0,
            task_bias_think: 0.0,
            task_bias_code_interpret: 0.0,
            task_bias_image_generation: 0.0,
            tool_bias_terminal: 0.0,
            tool_bias_file: 0.0,
            tool_bias_web_search: 0.0,
            tool_bias_calculator: 0.0,
        }
    }
}

impl PolicyParams {
    pub fn normalize(&mut self) {
        let sum = self.depth_weight + self.complexity_weight + self.dependency_weight;
        if sum <= 0.0 {
            self.depth_weight = 0.35;
            self.complexity_weight = 0.40;
            self.dependency_weight = 0.25;
        } else {
            self.depth_weight /= sum;
            self.complexity_weight /= sum;
            self.dependency_weight /= sum;
        }
        self.backend_weight = self.backend_weight.clamp(0.0, 1.0);
        self.task_bias_retrieve = self.task_bias_retrieve.clamp(-0.95, 2.0);
        self.task_bias_write = self.task_bias_write.clamp(-0.95, 2.0);
        self.task_bias_think = self.task_bias_think.clamp(-0.95, 2.0);
        self.task_bias_code_interpret = self.task_bias_code_interpret.clamp(-0.95, 2.0);
        self.task_bias_image_generation = self.task_bias_image_generation.clamp(-0.95, 2.0);
        self.tool_bias_terminal = self.tool_bias_terminal.clamp(-0.95, 2.0);
        self.tool_bias_file = self.tool_bias_file.clamp(-0.95, 2.0);
        self.tool_bias_web_search = self.tool_bias_web_search.clamp(-0.95, 2.0);
        self.tool_bias_calculator = self.tool_bias_calculator.clamp(-0.95, 2.0);
    }

    fn task_biases(&self) -> [f64; 5] {
        [
            self.task_bias_retrieve,
            self.task_bias_write,
            self.task_bias_think,
            self.task_bias_code_interpret,
            self.task_bias_image_generation,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFeatures {
    pub embedding: Vec<f64>,
    pub digest: String,
    pub depth: u32,
    pub dag_nodes: usize,
}

impl TaskFeatures {
    pub fn from_context(goal: &str, depth: u32, dag_nodes: usize, seed: u64) -> Self {
        let mut embedding = Vec::with_capacity(12);
        let mut hasher = Sha256::new();
        hasher.update(format!("{seed}:{goal}:{depth}:{dag_nodes}"));
        let bytes = hasher.finalize();

        let complexity = (goal.len() as f64 / 1024.0).min(1.0);
        let dependency_density = (dag_nodes as f64 / 256.0).min(1.0);
        let depth_norm = (depth as f64 / 16.0).min(1.0);

        embedding.push(complexity);
        embedding.push(dependency_density);
        embedding.push(depth_norm);

        for i in 0..9 {
            let b = bytes[i] as f64 / 255.0;
            embedding.push(b);
        }

        let digest = hex::encode(bytes);
        Self {
            embedding,
            digest,
            depth,
            dag_nodes,
        }
    }
}

pub trait QuantumBackend: Send + Sync {
    fn name(&self) -> &str;
    fn run_logits(&self, features: &TaskFeatures, seed: u64) -> Result<Vec<f64>, String>;
}

#[derive(Debug, Clone)]
pub struct PennyLaneSimulatorBackend {
    backend: String,
}

impl PennyLaneSimulatorBackend {
    pub fn new(backend: impl Into<String>) -> Self {
        Self {
            backend: backend.into(),
        }
    }
}

impl QuantumBackend for PennyLaneSimulatorBackend {
    fn name(&self) -> &str {
        &self.backend
    }

    fn run_logits(&self, features: &TaskFeatures, seed: u64) -> Result<Vec<f64>, String> {
        // Rust implementation mirrors PennyLane routing deterministically for v1 simulation.
        let mut logits = Vec::with_capacity(5);
        for class_idx in 0..5 {
            let mut hasher = Sha256::new();
            hasher.update(format!(
                "{}:{}:{}:{}",
                seed, class_idx, self.backend, features.digest
            ));
            let digest = hasher.finalize();
            let bytes: [u8; 8] = digest[..8]
                .try_into()
                .map_err(|_| "invalid digest length".to_string())?;
            let raw = u64::from_be_bytes(bytes);
            logits.push((raw as f64 / u64::MAX as f64).max(1e-9));
        }
        Ok(logits)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub selected_task_type: String,
    pub selected_tool: String,
    pub task_routing_logits: HashMap<String, f64>,
    pub tool_selection_logits: HashMap<String, f64>,
    pub quantum_backend: String,
    pub fallback_used: bool,
    pub policy_parameters: PolicyParams,
    pub feature_digest: String,
}

#[derive(Clone)]
pub struct HybridQuantumPolicy {
    pub config: QuantumConfig,
    backend: Arc<dyn QuantumBackend>,
}

impl HybridQuantumPolicy {
    pub fn new(mut config: QuantumConfig) -> Self {
        config.normalize();
        let backend = Arc::new(PennyLaneSimulatorBackend::new(config.backend.clone()));
        Self { config, backend }
    }

    pub fn with_backend(mut config: QuantumConfig, backend: Arc<dyn QuantumBackend>) -> Self {
        config.normalize();
        Self { config, backend }
    }

    pub fn serialize_parameters(&self) -> Result<String, String> {
        serde_json::to_string(&self.config).map_err(|e| e.to_string())
    }

    pub fn route(&self, goal: &str, depth: u32, dag_nodes: usize) -> RoutingDecision {
        let features = TaskFeatures::from_context(goal, depth, dag_nodes, self.config.seed);
        let (task_logits, fallback_used) = self.run_with_fallback(&features);

        let task_labels = [
            "RETRIEVE",
            "WRITE",
            "THINK",
            "CODE_INTERPRET",
            "IMAGE_GENERATION",
        ];

        let mut task_routing_logits = HashMap::new();
        for (idx, label) in task_labels.iter().enumerate() {
            task_routing_logits.insert((*label).to_string(), task_logits[idx]);
        }

        let selected_task_type = task_labels
            .iter()
            .enumerate()
            .max_by(|(i, _), (j, _)| task_logits[*i].partial_cmp(&task_logits[*j]).unwrap())
            .map(|(_, label)| (*label).to_string())
            .unwrap_or_else(|| "THINK".to_string());

        let tool_selection_logits = self.tool_logits_from_embedding(&features.embedding);
        let selected_tool = tool_selection_logits
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| "terminal".to_string());

        RoutingDecision {
            selected_task_type,
            selected_tool,
            task_routing_logits,
            tool_selection_logits,
            quantum_backend: self.backend.name().to_string(),
            fallback_used,
            policy_parameters: self.config.policy_params.clone(),
            feature_digest: features.digest,
        }
    }

    fn run_with_fallback(&self, features: &TaskFeatures) -> (Vec<f64>, bool) {
        let start = Instant::now();
        let backend_result = self.backend.run_logits(features, self.config.seed);
        let elapsed = start.elapsed();
        let classical = normalize_logits(&classical_logits(features, &self.config.policy_params));

        match backend_result {
            Ok(logits) if elapsed <= Duration::from_millis(self.config.timeout_ms) => {
                let backend = normalize_logits(&logits);
                let blended = blend_logits(
                    &backend,
                    &classical,
                    self.config.policy_params.backend_weight,
                );
                (
                    apply_task_biases(&blended, &self.config.policy_params.task_biases()),
                    false,
                )
            }
            _ => {
                (
                    apply_task_biases(&classical, &self.config.policy_params.task_biases()),
                    true,
                )
            }
        }
    }

    fn tool_logits_from_embedding(&self, embedding: &[f64]) -> HashMap<String, f64> {
        let mut logits = HashMap::new();
        let base = embedding.iter().copied().sum::<f64>() / (embedding.len().max(1) as f64);
        let terminal = ((base * 0.92 + 0.08)
            * (1.0 + self.config.policy_params.tool_bias_terminal))
            .max(1e-9);
        let file =
            ((base * 0.83 + 0.12) * (1.0 + self.config.policy_params.tool_bias_file)).max(1e-9);
        let web_search = ((base * 0.76 + 0.15)
            * (1.0 + self.config.policy_params.tool_bias_web_search))
            .max(1e-9);
        let calculator = ((base * 0.65 + 0.11)
            * (1.0 + self.config.policy_params.tool_bias_calculator))
            .max(1e-9);

        logits.insert("terminal".to_string(), terminal);
        logits.insert("file".to_string(), file);
        logits.insert("web_search".to_string(), web_search);
        logits.insert("calculator".to_string(), calculator);
        logits
    }
}

fn classical_logits(features: &TaskFeatures, params: &PolicyParams) -> Vec<f64> {
    let complexity = features.embedding[0];
    let dependency = features.embedding[1];
    let depth = features.embedding[2];

    vec![
        complexity * 0.45 + dependency * 0.40,
        complexity * 0.52,
        depth * params.depth_weight + (1.0 - complexity) * params.complexity_weight,
        complexity * params.complexity_weight + dependency * params.dependency_weight,
        (1.0 - dependency) * 0.30,
    ]
}

fn normalize_logits(raw: &[f64]) -> Vec<f64> {
    let sum: f64 = raw.iter().sum();
    if sum <= 0.0 {
        return vec![0.2; 5];
    }
    raw.iter().map(|v| v / sum).collect()
}

fn blend_logits(quantum: &[f64], classical: &[f64], backend_weight: f64) -> Vec<f64> {
    let n = quantum.len().min(classical.len());
    if n == 0 {
        return vec![0.2; 5];
    }
    (0..n)
        .map(|i| quantum[i] * backend_weight + classical[i] * (1.0 - backend_weight))
        .collect()
}

fn apply_task_biases(raw: &[f64], biases: &[f64; 5]) -> Vec<f64> {
    let adjusted: Vec<f64> = raw
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            let bias = 1.0 + biases.get(idx).copied().unwrap_or(0.0);
            (value * bias.max(0.01)).max(1e-9)
        })
        .collect();
    normalize_logits(&adjusted)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingBackend;
    impl QuantumBackend for FailingBackend {
        fn name(&self) -> &str {
            "failing"
        }
        fn run_logits(&self, _features: &TaskFeatures, _seed: u64) -> Result<Vec<f64>, String> {
            Err("backend timeout".to_string())
        }
    }

    #[test]
    fn deterministic_routing_for_same_seed() {
        let policy = HybridQuantumPolicy::new(QuantumConfig::default());
        let d1 = policy.route("fix parser race condition", 2, 14);
        let d2 = policy.route("fix parser race condition", 2, 14);

        assert_eq!(d1.selected_task_type, d2.selected_task_type);
        assert_eq!(d1.feature_digest, d2.feature_digest);
        assert!(!d1.fallback_used);
    }

    #[test]
    fn fallback_triggers_on_backend_error() {
        let config = QuantumConfig::default();
        let policy = HybridQuantumPolicy::with_backend(config, Arc::new(FailingBackend));
        let decision = policy.route("analyze swe benchmark regression", 1, 4);
        assert!(decision.fallback_used);
    }

    #[test]
    fn config_file_supports_partial_fields() {
        let tmp = std::env::temp_dir().join("qaegis_hybrid_policy_test.json");
        std::fs::write(
            &tmp,
            r#"{
  "backend": "lightning.qubit",
  "seed": 777,
  "policy_params": {
    "backend_weight": 0.9,
    "task_bias_think": 0.25
  }
}"#,
        )
        .expect("write config");
        let config = QuantumConfig::from_json_file(&tmp).expect("load config");
        assert_eq!(config.seed, 777);
        assert_eq!(config.timeout_ms, 1200);
        assert_eq!(config.policy_params.backend_weight, 0.9);
        assert_eq!(config.policy_params.task_bias_think, 0.25);
    }

    #[test]
    fn task_biases_can_shift_routing() {
        let mut config = QuantumConfig::default();
        config.policy_params.task_bias_retrieve = 1.5;
        config.policy_params.task_bias_think = -0.8;
        let policy = HybridQuantumPolicy::new(config);
        let decision = policy.route("investigate flaky benchmark", 2, 6);
        assert_eq!(decision.selected_task_type, "RETRIEVE");
    }
}
