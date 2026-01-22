# Documentation

This directory contains all project documentation.

## Structure

- `architecture/` - System architecture and design documents
- `deployment/` - Deployment guides and procedures
- `api/` - API documentation

## Building Documentation

### Rust API Documentation

Build Rust documentation locally:

```bash
cd services/qrms
cargo doc --no-deps --all-features
# Open target/doc/qrms/index.html in browser
```

### Full Documentation Site

Build the complete documentation site:

```bash
./scripts/docs/build-docs.sh
cd _site
python3 -m http.server 8000
# Open http://localhost:8000 in browser
```

## GitHub Pages

Documentation is automatically deployed to GitHub Pages on pushes to `main` branch.

### Setup

1. **Enable GitHub Pages:**
   - Repository Settings → Pages
   - Source: GitHub Actions
   - Save

2. **Push to main branch:**
   - The workflow (`.github/workflows/docs.yml`) will automatically run
   - Or trigger manually: Actions → "Deploy Documentation" → Run workflow

### Deployed Content

The deployed site includes:
- Rust API documentation (from `cargo doc`)
  - Main API: `rust/qrms/index.html`
  - APQC Module: `rust/qrms/apqc/index.html`
  - QRM Module: `rust/qrms/qrm/index.html`
  - Crypto Module: `rust/qrms/crypto/index.html`
- Architecture documentation (markdown files)
- Project documentation (README, CONTRIBUTING, etc.)

### Access

After deployment, documentation is available at:
```
https://<username>.github.io/qAegis/
```

See [docs/DEPLOYMENT.md](../docs/DEPLOYMENT.md) for detailed deployment guide.
