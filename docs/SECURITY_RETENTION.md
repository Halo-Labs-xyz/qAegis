# Security Retention Policy (qAegis v1)

## Default
`privacy_mode=zero_retention` and `persistence_consent=none`.

## Consent Model
- `none`: no plaintext persistence; stored payloads are SHA-256 digests.
- `metadata_only`: persisted strings are redacted digest markers.
- `full`: plaintext persistence is allowed, but redaction guard still masks known secret patterns.

## Components
- `services/qrms/src/security_plane.rs`
  - `RetentionPolicy`
  - `RedactionGuard` (secret pattern + entropy heuristic)
  - `TransientStore` with TTL purge support

## Purge Worker
`state::run_hybrid_purge` executes every 60 seconds and removes transient execution records older than 15 minutes.

## Export/Claims
Public-claim artifacts must pass claims-gate requirements before the run can be declared claim-ready.
