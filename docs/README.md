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
# Install markdown converter (if not already installed)
pip install markdown

# Build documentation
./scripts/docs/build-docs.sh
cd _site
python3 -m http.server 8000
# Open http://localhost:8000 in browser
```

**Note:** The build script converts all markdown files to HTML automatically. If the `markdown` Python library is not installed, markdown files will be copied as-is.

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
- Architecture documentation (converted from markdown to HTML)
  - All `.md` files in `docs/` are converted to `.html`
- Project documentation (README, CONTRIBUTING, etc., converted to HTML)

### Access

After deployment, documentation is available at:
```
https://<username>.github.io/qAegis/
```

See [docs/DEPLOYMENT.md](../docs/DEPLOYMENT.md) for detailed deployment guide.
