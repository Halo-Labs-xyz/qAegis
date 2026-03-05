//! Hybrid QRMS CLI for API + governance + retention aware execution.

use std::collections::HashMap;

use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 || args[1] != "solve" {
        eprintln!(
            "Usage: hybrid-cli solve \"<goal>\" [--server <url>] [--execution-mode classical|hybrid_quantum] [--privacy-mode zero_retention] [--persistence-consent none|metadata_only|full] [--governance-profile freedom_v1]"
        );
        std::process::exit(2);
    }

    let goal = args[2].clone();
    let mut server = "http://localhost:5050".to_string();
    let mut execution_mode = "classical".to_string();
    let mut privacy_mode = "zero_retention".to_string();
    let mut persistence_consent = "none".to_string();
    let mut governance_profile = "freedom_v1".to_string();

    let mut idx = 3;
    while idx < args.len() {
        match args[idx].as_str() {
            "--server" if idx + 1 < args.len() => {
                server = args[idx + 1].clone();
                idx += 2;
            }
            "--execution-mode" if idx + 1 < args.len() => {
                execution_mode = args[idx + 1].clone();
                idx += 2;
            }
            "--privacy-mode" if idx + 1 < args.len() => {
                privacy_mode = args[idx + 1].clone();
                idx += 2;
            }
            "--persistence-consent" if idx + 1 < args.len() => {
                persistence_consent = args[idx + 1].clone();
                idx += 2;
            }
            "--governance-profile" if idx + 1 < args.len() => {
                governance_profile = args[idx + 1].clone();
                idx += 2;
            }
            _ => {
                idx += 1;
            }
        }
    }

    let payload = json!({
        "goal": goal,
        "max_depth": 2,
        "execution_mode": execution_mode,
        "privacy_mode": privacy_mode,
        "persistence_consent": persistence_consent,
        "governance_profile": governance_profile,
        "metadata": HashMap::<String, String>::new(),
    });

    let client = Client::new();
    let url = format!("{}/api/hybrid/solve", server.trim_end_matches('/'));

    match client.post(url).json(&payload).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "{}".to_string());
            if !status.is_success() {
                eprintln!("HTTP {}: {}", status, body);
                std::process::exit(1);
            }
            println!("{}", body);
        }
        Err(err) => {
            eprintln!("Request failed: {}", err);
            std::process::exit(1);
        }
    }
}
