//! QRMS CLI - Terminal User Interface
//! 
//! Multi-pane view of all QRMS processes:
//! - QRM threat indicators + risk scores
//! - APQC algorithm status + signatures
//! - Sequencer mempool + batches
//! - Chain blocks + state
//! - Event stream

use std::io::{self, stdout};
use std::sync::Arc;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Gauge, Tabs, Wrap},
    Frame, Terminal,
};
use tokio::sync::mpsc;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
struct StatusResponse {
    qrm: QrmStatus,
    apqc: ApqcStatus,
    sequencer: SequencerStatus,
    chain: ChainStatus,
}

#[derive(Debug, Clone, Deserialize)]
struct QrmStatus {
    risk_score: u32,
    recommendation: String,
    indicator_count: usize,
    thresholds: Thresholds,
}

#[derive(Debug, Clone, Deserialize)]
struct Thresholds {
    scheduled: u32,
    emergency: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct ApqcStatus {
    signatures: Vec<String>,
    kems: Vec<String>,
    rotation_pending: bool,
    rotation_block: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct SequencerStatus {
    mempool_size: usize,
    ordered_queue: usize,
    batch_count: usize,
    tee_platform: String,
    mrenclave: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ChainStatus {
    height: u64,
    algorithm_set: AlgorithmSet,
    risk_score: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct AlgorithmSet {
    signatures: Vec<String>,
    kems: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ThreatIndicator {
    category: String,
    sub_category: String,
    severity: f64,
    confidence: f64,
    source: String,
    timestamp: String,
    description: String,
    era_relevance: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RiskAssessment {
    score: u32,
    recommendation: String,
    category_breakdown: Vec<CategoryRisk>,
}

#[derive(Debug, Clone, Deserialize)]
struct CategoryRisk {
    category: String,
    score: u32,
    indicator_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct Transaction {
    tx_id: String,
    sender: String,
    data: String,
    priority_fee: u64,
    status: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Batch {
    batch_id: String,
    transactions: Vec<Transaction>,
    ml_dsa_sig: String,
    slh_dsa_sig: String,
    timestamp: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BlockInfo {
    height: u64,
    batch_id: String,
    tx_count: usize,
    risk_score: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "data")]
enum WsEvent {
    #[serde(rename = "qrm_update")]
    QrmUpdate { indicator: ThreatIndicator, risk: RiskAssessment },
    #[serde(rename = "tx_submitted")]
    TxSubmitted(Transaction),
    #[serde(rename = "txs_ordered")]
    TxsOrdered { count: usize, txs: Vec<Transaction> },
    #[serde(rename = "batch_created")]
    BatchCreated { batch: Batch, block: BlockInfo },
    #[serde(rename = "rotation_scheduled")]
    RotationScheduled { effective_block: u64 },
    #[serde(rename = "rotation_executed")]
    RotationExecuted { rotation_type: String },
    #[serde(rename = "simulation_started")]
    SimulationStarted,
    #[serde(rename = "simulation_stopped")]
    SimulationStopped,
}

// ============================================================================
// App State
// ============================================================================

#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: String,
    level: LogLevel,
    component: String,
    message: String,
}

#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
    Threat,
    Block,
    Tx,
    Rotation,
}

impl LogLevel {
    fn color(&self) -> Color {
        match self {
            Self::Info => Color::Cyan,
            Self::Warn => Color::Yellow,
            Self::Error => Color::Red,
            Self::Debug => Color::DarkGray,
            Self::Threat => Color::Magenta,
            Self::Block => Color::Green,
            Self::Tx => Color::Blue,
            Self::Rotation => Color::LightRed,
        }
    }
    
    fn label(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERR ",
            Self::Debug => "DBG ",
            Self::Threat => "THRT",
            Self::Block => "BLK ",
            Self::Tx => "TX  ",
            Self::Rotation => "ROT ",
        }
    }
}

struct App {
    // Status
    status: Option<StatusResponse>,
    
    // Threat data
    indicators: Vec<ThreatIndicator>,
    category_risks: Vec<CategoryRisk>,
    current_risk: u32,
    
    // Transaction data
    pending_txs: Vec<Transaction>,
    recent_batches: Vec<Batch>,
    
    // Blocks
    blocks: Vec<BlockInfo>,
    
    // Logs
    logs: Vec<LogEntry>,
    
    // UI state
    active_tab: usize,
    scroll_offset: usize,
    running: bool,
    connected: bool,
    
    // Stats
    total_indicators: u64,
    total_txs: u64,
    total_blocks: u64,
    rotations: u64,
}

impl App {
    fn new() -> Self {
        Self {
            status: None,
            indicators: Vec::new(),
            category_risks: Vec::new(),
            current_risk: 0,
            pending_txs: Vec::new(),
            recent_batches: Vec::new(),
            blocks: Vec::new(),
            logs: Vec::new(),
            active_tab: 0,
            scroll_offset: 0,
            running: false,
            connected: false,
            total_indicators: 0,
            total_txs: 0,
            total_blocks: 0,
            rotations: 0,
        }
    }
    
    fn log(&mut self, level: LogLevel, component: &str, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        self.logs.push(LogEntry {
            timestamp,
            level,
            component: component.to_string(),
            message,
        });
        // Keep last 500 logs
        if self.logs.len() > 500 {
            self.logs.remove(0);
        }
    }
    
    fn handle_event(&mut self, event: WsEvent) {
        match event {
            WsEvent::QrmUpdate { indicator, risk } => {
                self.total_indicators += 1;
                self.current_risk = risk.score;
                self.category_risks = risk.category_breakdown;
                
                self.log(
                    LogLevel::Threat,
                    "QRM",
                    format!(
                        "[{}] {} | sev={:.2} conf={:.2} | {}",
                        indicator.category,
                        indicator.sub_category,
                        indicator.severity,
                        indicator.confidence,
                        indicator.description
                    ),
                );
                
                self.indicators.push(indicator);
                if self.indicators.len() > 100 {
                    self.indicators.remove(0);
                }
            }
            WsEvent::TxSubmitted(tx) => {
                self.total_txs += 1;
                self.log(
                    LogLevel::Tx,
                    "SEQ",
                    format!("{} from {} | fee={} | {}", tx.tx_id, &tx.sender[..10], tx.priority_fee, tx.data),
                );
                self.pending_txs.push(tx);
                if self.pending_txs.len() > 50 {
                    self.pending_txs.remove(0);
                }
            }
            WsEvent::TxsOrdered { count, txs: _ } => {
                self.log(LogLevel::Info, "SEQ", format!("Ordered {} transactions", count));
            }
            WsEvent::BatchCreated { batch, block } => {
                self.total_blocks += 1;
                self.log(
                    LogLevel::Block,
                    "CHAIN",
                    format!(
                        "Block #{} | batch={} | txs={} | risk={}",
                        block.height, batch.batch_id, block.tx_count, block.risk_score
                    ),
                );
                self.log(
                    LogLevel::Debug,
                    "APQC",
                    format!("ML-DSA: {}...", &batch.ml_dsa_sig[..16]),
                );
                self.log(
                    LogLevel::Debug,
                    "APQC",
                    format!("SLH-DSA: {}...", &batch.slh_dsa_sig[..16]),
                );
                
                self.recent_batches.push(batch);
                self.blocks.push(block);
                if self.recent_batches.len() > 20 {
                    self.recent_batches.remove(0);
                }
                if self.blocks.len() > 50 {
                    self.blocks.remove(0);
                }
            }
            WsEvent::RotationScheduled { effective_block } => {
                self.log(
                    LogLevel::Rotation,
                    "APQC",
                    format!("ROTATION SCHEDULED for block {}", effective_block),
                );
            }
            WsEvent::RotationExecuted { rotation_type } => {
                self.rotations += 1;
                self.log(
                    LogLevel::Rotation,
                    "APQC",
                    format!("ROTATION EXECUTED: {}", rotation_type),
                );
            }
            WsEvent::SimulationStarted => {
                self.running = true;
                self.log(LogLevel::Info, "SYS", "Simulation STARTED".to_string());
            }
            WsEvent::SimulationStopped => {
                self.running = false;
                self.log(LogLevel::Info, "SYS", "Simulation STOPPED".to_string());
            }
        }
    }
    
    fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % 5;
        self.scroll_offset = 0;
    }
    
    fn prev_tab(&mut self) {
        self.active_tab = if self.active_tab == 0 { 4 } else { self.active_tab - 1 };
        self.scroll_offset = 0;
    }
    
    fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
    
    fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }
}

// ============================================================================
// UI Rendering
// ============================================================================

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Stats bar
            Constraint::Min(10),    // Main content
            Constraint::Length(12), // Log panel
            Constraint::Length(1),  // Footer
        ])
        .split(f.area());
    
    render_header(f, app, chunks[0]);
    render_stats(f, app, chunks[1]);
    render_main(f, app, chunks[2]);
    render_logs(f, app, chunks[3]);
    render_footer(f, chunks[4]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = ["QRM", "APQC", "SEQ", "CHAIN", "ALL"];
    let tabs = Tabs::new(titles.iter().map(|t| Line::from(*t)).collect::<Vec<_>>())
        .block(Block::default()
            .title(" QRMS CLI ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .select(app.active_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, area);
}

fn render_stats(f: &mut Frame, app: &App, area: Rect) {
    let status_color = if app.connected {
        if app.running { Color::Green } else { Color::Yellow }
    } else {
        Color::Red
    };
    
    let risk_color = if app.current_risk < 3000 {
        Color::Green
    } else if app.current_risk < 6000 {
        Color::Yellow
    } else {
        Color::Red
    };
    
    let stats = Line::from(vec![
        Span::styled(" ● ", Style::default().fg(status_color)),
        Span::raw(if app.connected { if app.running { "RUNNING" } else { "IDLE   " } } else { "DISCONN" }),
        Span::raw(" │ "),
        Span::styled(format!("RISK: {:>5}", app.current_risk), Style::default().fg(risk_color).add_modifier(Modifier::BOLD)),
        Span::raw(" │ "),
        Span::raw(format!("IND: {:>4}", app.total_indicators)),
        Span::raw(" │ "),
        Span::raw(format!("TXS: {:>5}", app.total_txs)),
        Span::raw(" │ "),
        Span::raw(format!("BLKS: {:>4}", app.total_blocks)),
        Span::raw(" │ "),
        Span::styled(format!("ROT: {:>2}", app.rotations), Style::default().fg(if app.rotations > 0 { Color::LightRed } else { Color::DarkGray })),
        Span::raw(" │ "),
        Span::raw(format!("HEIGHT: {:>6}", app.status.as_ref().map(|s| s.chain.height).unwrap_or(0))),
    ]);
    
    let para = Paragraph::new(stats)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(para, area);
}

fn render_main(f: &mut Frame, app: &App, area: Rect) {
    match app.active_tab {
        0 => render_qrm(f, app, area),
        1 => render_apqc(f, app, area),
        2 => render_sequencer(f, app, area),
        3 => render_chain(f, app, area),
        4 => render_all(f, app, area),
        _ => {}
    }
}

fn render_qrm(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);
    
    // Left: Category breakdown
    let cat_items: Vec<ListItem> = app.category_risks.iter().map(|c| {
        let color = if c.score < 3000 {
            Color::Green
        } else if c.score < 6000 {
            Color::Yellow
        } else {
            Color::Red
        };
        let bar_len = (c.score as usize * 20) / 10000;
        let bar: String = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:>20}", c.category), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(bar, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(format!("{:>5}", c.score), Style::default().fg(color)),
            Span::raw(format!(" ({:>2})", c.indicator_count)),
        ]))
    }).collect();
    
    let cat_list = List::new(cat_items)
        .block(Block::default()
            .title(" Category Risk ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)));
    f.render_widget(cat_list, chunks[0]);
    
    // Right: Recent indicators
    let skip = app.scroll_offset.min(app.indicators.len().saturating_sub(1));
    let ind_items: Vec<ListItem> = app.indicators.iter().rev().skip(skip).take(15).map(|i| {
        let sev_color = if i.severity < 0.4 {
            Color::Green
        } else if i.severity < 0.7 {
            Color::Yellow
        } else {
            Color::Red
        };
        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("[{}]", i.category), Style::default().fg(Color::Magenta)),
                Span::raw(" "),
                Span::styled(&i.sub_category, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("sev={:.2}", i.severity), Style::default().fg(sev_color)),
                Span::raw(format!(" conf={:.2} ", i.confidence)),
                Span::styled(&i.source, Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw(&i.description),
            ]),
        ])
    }).collect();
    
    let ind_list = List::new(ind_items)
        .block(Block::default()
            .title(format!(" Indicators ({}) ", app.indicators.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)));
    f.render_widget(ind_list, chunks[1]);
}

fn render_apqc(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(5)])
        .split(area);
    
    // Top: Algorithm status
    let algo_text = if let Some(ref status) = app.status {
        vec![
            Line::from(vec![
                Span::styled(" SIGNATURES: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("   "),
                Span::styled("● ML-DSA-87    ", Style::default().fg(Color::Green)),
                Span::raw("4,595 bytes │ Lattice-based │ NIST FIPS 204"),
            ]),
            Line::from(vec![
                Span::raw("   "),
                Span::styled("● SLH-DSA-256s ", Style::default().fg(Color::Green)),
                Span::raw("29,792 bytes │ Hash-based │ NIST FIPS 205"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" KEMS: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("   "),
                Span::styled("● ML-KEM-1024  ", Style::default().fg(Color::Green)),
                Span::raw("1,568 bytes CT │ Lattice-based │ NIST FIPS 203"),
            ]),
            Line::from(vec![
                Span::raw("   "),
                Span::styled("● HQC-256      ", Style::default().fg(Color::Green)),
                Span::raw("6,730 bytes CT │ Code-based │ Backup family"),
            ]),
        ]
    } else {
        vec![Line::from("Awaiting connection...")]
    };
    
    let algo_para = Paragraph::new(algo_text)
        .block(Block::default()
            .title(" Active Algorithms ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)));
    f.render_widget(algo_para, chunks[0]);
    
    // Bottom: Recent signatures
    let sig_items: Vec<ListItem> = app.recent_batches.iter().rev().take(8).map(|b| {
        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("Batch {} ", b.batch_id), Style::default().fg(Color::Cyan)),
                Span::raw(format!("| {} txs", b.transactions.len())),
            ]),
            Line::from(vec![
                Span::raw("  ML-DSA:  "),
                Span::styled(&b.ml_dsa_sig[..32], Style::default().fg(Color::Green)),
                Span::raw("..."),
            ]),
            Line::from(vec![
                Span::raw("  SLH-DSA: "),
                Span::styled(&b.slh_dsa_sig[..32], Style::default().fg(Color::Green)),
                Span::raw("..."),
            ]),
        ])
    }).collect();
    
    let sig_list = List::new(sig_items)
        .block(Block::default()
            .title(" Recent Dual Signatures ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)));
    f.render_widget(sig_list, chunks[1]);
}

fn render_sequencer(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    // Left: TEE info + mempool
    let tee_text = if let Some(ref status) = app.status {
        vec![
            Line::from(vec![
                Span::styled(" TEE Platform: ", Style::default().fg(Color::Cyan)),
                Span::raw(&status.sequencer.tee_platform),
            ]),
            Line::from(vec![
                Span::styled(" MRENCLAVE:    ", Style::default().fg(Color::Cyan)),
                Span::styled(&status.sequencer.mrenclave, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled(" Attestation:  ", Style::default().fg(Color::Cyan)),
                Span::styled("PQC-Signed", Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!(" Mempool:  {:>4} ", status.sequencer.mempool_size), Style::default()),
                Span::styled(format!(" Ordered:  {:>4} ", status.sequencer.ordered_queue), Style::default()),
                Span::styled(format!(" Batches:  {:>4} ", status.sequencer.batch_count), Style::default()),
            ]),
        ]
    } else {
        vec![Line::from("Awaiting connection...")]
    };
    
    let tee_para = Paragraph::new(tee_text)
        .block(Block::default()
            .title(" TEE Sequencer (SGX) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)));
    f.render_widget(tee_para, chunks[0]);
    
    // Right: Recent transactions
    let tx_items: Vec<ListItem> = app.pending_txs.iter().rev().take(12).map(|tx| {
        let status_color = match tx.status.as_str() {
            "pending" => Color::Yellow,
            "ordered" => Color::Cyan,
            "committed" => Color::Green,
            _ => Color::DarkGray,
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:>8}", &tx.tx_id[..8]), Style::default().fg(Color::Cyan)),
            Span::raw(" │ "),
            Span::raw(format!("{:.10}...", tx.sender)),
            Span::raw(" │ "),
            Span::styled(format!("fee={:>3}", tx.priority_fee), Style::default().fg(Color::Yellow)),
            Span::raw(" │ "),
            Span::styled(format!("{:>9}", tx.status), Style::default().fg(status_color)),
        ]))
    }).collect();
    
    let tx_list = List::new(tx_items)
        .block(Block::default()
            .title(format!(" Transactions ({}) ", app.pending_txs.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)));
    f.render_widget(tx_list, chunks[1]);
}

fn render_chain(f: &mut Frame, app: &App, area: Rect) {
    let block_items: Vec<ListItem> = app.blocks.iter().rev().take(15).map(|b| {
        let risk_color = if b.risk_score < 3000 {
            Color::Green
        } else if b.risk_score < 6000 {
            Color::Yellow
        } else {
            Color::Red
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!("#{:>6}", b.height), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" │ "),
            Span::raw(format!("batch={}", b.batch_id)),
            Span::raw(" │ "),
            Span::raw(format!("txs={:>2}", b.tx_count)),
            Span::raw(" │ "),
            Span::styled(format!("risk={:>5}", b.risk_score), Style::default().fg(risk_color)),
        ]))
    }).collect();
    
    let block_list = List::new(block_items)
        .block(Block::default()
            .title(format!(" Blocks ({}) ", app.blocks.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)));
    f.render_widget(block_list, area);
}

fn render_all(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25)])
        .split(area);
    
    // QRM mini
    let qrm_items: Vec<ListItem> = app.indicators.iter().rev().take(8).map(|i| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("[{}]", &i.category[..3].to_uppercase()), Style::default().fg(Color::Magenta)),
            Span::raw(" "),
            Span::raw(&i.description[..i.description.len().min(25)]),
        ]))
    }).collect();
    let qrm_list = List::new(qrm_items)
        .block(Block::default().title(" QRM ").borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)));
    f.render_widget(qrm_list, chunks[0]);
    
    // APQC mini
    let apqc_text = vec![
        Line::from(vec![Span::styled("● ML-DSA-87", Style::default().fg(Color::Green))]),
        Line::from(vec![Span::styled("● SLH-DSA-256s", Style::default().fg(Color::Green))]),
        Line::from(vec![Span::styled("● ML-KEM-1024", Style::default().fg(Color::Green))]),
        Line::from(vec![Span::styled("● HQC-256", Style::default().fg(Color::Green))]),
        Line::from(""),
        Line::from(format!("Sigs: {}", app.recent_batches.len())),
    ];
    let apqc_para = Paragraph::new(apqc_text)
        .block(Block::default().title(" APQC ").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
    f.render_widget(apqc_para, chunks[1]);
    
    // SEQ mini
    let seq_text = if let Some(ref s) = app.status {
        vec![
            Line::from(format!("Mempool: {}", s.sequencer.mempool_size)),
            Line::from(format!("Ordered: {}", s.sequencer.ordered_queue)),
            Line::from(format!("Batches: {}", s.sequencer.batch_count)),
            Line::from(""),
            Line::from(vec![Span::styled(&s.sequencer.mrenclave, Style::default().fg(Color::Green))]),
        ]
    } else {
        vec![Line::from("...")]
    };
    let seq_para = Paragraph::new(seq_text)
        .block(Block::default().title(" SEQ ").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
    f.render_widget(seq_para, chunks[2]);
    
    // Chain mini
    let chain_items: Vec<ListItem> = app.blocks.iter().rev().take(6).map(|b| {
        ListItem::new(Line::from(format!("#{} txs={}", b.height, b.tx_count)))
    }).collect();
    let chain_list = List::new(chain_items)
        .block(Block::default().title(" CHAIN ").borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
    f.render_widget(chain_list, chunks[3]);
}

fn render_logs(f: &mut Frame, app: &App, area: Rect) {
    let log_items: Vec<ListItem> = app.logs.iter().rev().take(10).map(|log| {
        ListItem::new(Line::from(vec![
            Span::styled(&log.timestamp, Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled(log.level.label(), Style::default().fg(log.level.color())),
            Span::raw(" "),
            Span::styled(format!("{:>5}", log.component), Style::default().fg(Color::Cyan)),
            Span::raw(" │ "),
            Span::raw(&log.message),
        ]))
    }).collect();
    
    let log_list = List::new(log_items)
        .block(Block::default()
            .title(format!(" Event Log ({}) ", app.logs.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(log_list, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(" Tab", Style::default().fg(Color::Yellow)),
        Span::raw(":switch "),
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(":scroll "),
        Span::styled("s", Style::default().fg(Color::Yellow)),
        Span::raw(":start "),
        Span::styled("x", Style::default().fg(Color::Yellow)),
        Span::raw(":stop "),
        Span::styled("h", Style::default().fg(Color::Yellow)),
        Span::raw(":inject "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(":quit "),
    ]);
    let para = Paragraph::new(help);
    f.render_widget(para, area);
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let host = args.get(1).map(|s| s.as_str()).unwrap_or("localhost:5050");
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::new();
    app.log(LogLevel::Info, "SYS", format!("Connecting to ws://{}...", host));
    
    // WebSocket connection
    let ws_url = format!("ws://{}/ws", host);
    let (tx, mut rx) = mpsc::channel::<WsEvent>(100);
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<String>(10);
    
    // Spawn WebSocket task
    let ws_handle = tokio::spawn(async move {
        loop {
            match connect_async(&ws_url).await {
                Ok((ws_stream, _)) => {
                    let (mut write, mut read) = ws_stream.split();
                    
                    loop {
                        tokio::select! {
                            Some(msg) = read.next() => {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        if let Ok(event) = serde_json::from_str::<WsEvent>(&text) {
                                            let _ = tx.send(event).await;
                                        }
                                    }
                                    Err(_) => break,
                                    _ => {}
                                }
                            }
                            Some(cmd) = cmd_rx.recv() => {
                                let _ = write.send(Message::Text(cmd)).await;
                            }
                        }
                    }
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    });
    
    // Fetch initial status
    let status_host = host.to_string();
    let (status_tx, mut status_rx) = mpsc::channel::<StatusResponse>(10);
    tokio::spawn(async move {
        loop {
            if let Ok(resp) = reqwest::get(format!("http://{}/api/status", status_host)).await {
                if let Ok(status) = resp.json::<StatusResponse>().await {
                    let _ = status_tx.send(status).await;
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    
    // Main loop
    loop {
        terminal.draw(|f| ui(f, &app))?;
        
        // Handle WebSocket events
        while let Ok(event) = rx.try_recv() {
            app.connected = true;
            app.handle_event(event);
        }
        
        // Handle status updates
        while let Ok(status) = status_rx.try_recv() {
            app.connected = true;
            app.status = Some(status);
        }
        
        // Handle input
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.prev_tab(),
                    KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                    KeyCode::Char('s') => {
                        let _ = cmd_tx.send(r#"{"command":"start"}"#.to_string()).await;
                        app.log(LogLevel::Info, "CMD", "Sent START command".to_string());
                    }
                    KeyCode::Char('x') => {
                        let _ = cmd_tx.send(r#"{"command":"stop"}"#.to_string()).await;
                        app.log(LogLevel::Info, "CMD", "Sent STOP command".to_string());
                    }
                    KeyCode::Char('h') => {
                        let _ = cmd_tx.send(r#"{"command":"inject_high"}"#.to_string()).await;
                        app.log(LogLevel::Warn, "CMD", "Sent INJECT HIGH THREAT command".to_string());
                    }
                    KeyCode::Char('1') => app.active_tab = 0,
                    KeyCode::Char('2') => app.active_tab = 1,
                    KeyCode::Char('3') => app.active_tab = 2,
                    KeyCode::Char('4') => app.active_tab = 3,
                    KeyCode::Char('5') => app.active_tab = 4,
                    _ => {}
                }
            }
        }
    }
    
    // Cleanup
    ws_handle.abort();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    
    Ok(())
}
