# QuantumAegis API Documentation

## REST API

### Base URL
- **Local**: `http://localhost:5050`
- **Production**: TBD

### Endpoints

#### GET `/api/status`
Get current QRMS status including QRM, APQC, Sequencer, and Chain state.

**Response:**
```json
{
  "qrm": {
    "risk_score": 0,
    "recommendation": "continue",
    "indicator_count": 0,
    "thresholds": {
      "scheduled": 6000,
      "emergency": 9000
    }
  },
  "apqc": {
    "signatures": ["ML-DSA-87", "SLH-DSA-256s"],
    "kems": ["ML-KEM-1024", "HQC-256"],
    "rotation_pending": false,
    "rotation_block": null
  },
  "sequencer": {
    "mempool_size": 0,
    "ordered_queue": 0,
    "batch_count": 0,
    "tee_platform": "SGX",
    "mrenclave": "2c2ef428332cb45f"
  },
  "chain": {
    "height": 0,
    "algorithm_set": {
      "signatures": ["ML-DSA-87", "SLH-DSA-256s"],
      "kems": ["ML-KEM-1024", "HQC-256"]
    },
    "risk_score": 0
  }
}
```

## WebSocket API

### Endpoint
`ws://localhost:5050/ws`

### Events

#### `status_update`
Real-time status updates from QRMS components.

#### `threat_alert`
Alerts when threat indicators are detected.

#### `rotation_signal`
Notifications when algorithm rotation is triggered.

## Contract APIs

See Solidity contract interfaces in `contracts/src/` for on-chain API documentation.
