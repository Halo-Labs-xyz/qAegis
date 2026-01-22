# Documentation Build Scripts

Scripts for building and deploying documentation.

## build-docs.sh

Builds complete documentation site including:
- Rust API documentation (from `cargo doc`)
- Markdown documentation files
- Generated index page

### Usage

```bash
./scripts/docs/build-docs.sh
```

This creates a `_site/` directory with all documentation ready for deployment.

### Local Preview

After building, preview locally:

```bash
cd _site
python3 -m http.server 8000
# Open http://localhost:8000
```

## GitHub Pages Deployment

Documentation is automatically deployed via GitHub Actions workflow (`.github/workflows/docs.yml`) on pushes to `main` branch.

The workflow:
1. Builds Rust documentation with `cargo doc`
2. Copies markdown documentation
3. Generates index page
4. Deploys to GitHub Pages

### Manual Deployment

If you need to manually deploy:

1. Build documentation: `./scripts/docs/build-docs.sh`
2. Push `_site/` contents to `gh-pages` branch (or use GitHub Actions)
