# GitHub Pages Documentation Deployment

## Overview

Documentation is automatically deployed to GitHub Pages via GitHub Actions workflow.

## Initial Setup

1. **Enable GitHub Pages:**
   - Go to repository Settings → Pages
   - Source: Select "GitHub Actions"
   - Save

2. **Workflow will run automatically:**
   - On every push to `main` branch
   - Or manually: Actions → "Deploy Documentation" → Run workflow

## What Gets Deployed

- **Rust API Documentation**: Generated from `cargo doc` in `services/qrms/`
  - Main API: `rust/qrms/index.html`
  - APQC Module: `rust/qrms/apqc/index.html`
  - QRM Module: `rust/qrms/qrm/index.html`
  - Crypto Module: `rust/qrms/crypto/index.html`

- **Markdown Documentation**: Copied from `docs/` directory
  - Architecture docs
  - Deployment guides
  - API documentation

- **Project Files**: README, CONTRIBUTING, DEPLOYMENT, CHANGELOG

## Accessing Documentation

After deployment, documentation is available at:
```
https://<username>.github.io/qAegis/
```

Or if using custom domain:
```
https://<your-domain>/
```

## Local Preview

Build and preview documentation locally:

```bash
./scripts/docs/build-docs.sh
cd _site
python3 -m http.server 8000
# Open http://localhost:8000
```

## Troubleshooting

**Workflow fails:**
- Check Actions tab for error messages
- Ensure GitHub Pages is enabled in repository settings
- Verify Rust toolchain setup in workflow

**Documentation not updating:**
- Wait a few minutes for GitHub Pages to rebuild
- Check Actions tab to see if workflow completed
- Clear browser cache

**Rust docs missing:**
- Ensure `cargo doc` runs successfully
- Check that `services/qrms/Cargo.toml` is valid
- Verify all dependencies are available
