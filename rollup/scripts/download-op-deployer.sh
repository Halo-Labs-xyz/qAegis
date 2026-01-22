#!/bin/bash
# ============================================================================
# Download op-deployer binary
# From: opstack/docs/create-l2-rollup-example/scripts/download-op-deployer.sh
# ============================================================================

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

detect_platform() {
    case "$(uname -s)" in
        Darwin)
            case "$(uname -m)" in
                arm64) echo "darwin-arm64" ;;
                x86_64) echo "darwin-amd64" ;;
                *) log_error "Unsupported macOS arch: $(uname -m)"; exit 1 ;;
            esac
            ;;
        Linux) echo "linux-amd64" ;;
        *) log_error "Unsupported platform: $(uname -s)"; exit 1 ;;
    esac
}

download_op_deployer() {
    local os arch platform
    case "$(uname -s)" in
        Darwin) os="darwin" ;;
        Linux) os="linux" ;;
        *) log_error "Unsupported OS"; exit 1 ;;
    esac
    case "$(uname -m)" in
        aarch64|arm64) arch="arm64" ;;
        x86_64|amd64) arch="amd64" ;;
        *) log_error "Unsupported arch"; exit 1 ;;
    esac
    platform="$os-$arch"
    local releases_url="https://api.github.com/repos/ethereum-optimism/optimism/releases"

    log_info "Platform: $platform"
    log_info "Finding latest op-deployer release..."

    local latest_release
    latest_release=$(curl -s "$releases_url?per_page=50" | jq -r '.[] | select(.tag_name | startswith("op-deployer/")) | .tag_name' | sort -V | tail -1)

    if [ -z "$latest_release" ]; then
        log_error "Could not find op-deployer releases"
        exit 1
    fi

    log_info "Found: $latest_release"

    local release_info
    release_info=$(curl -s "$releases_url/tags/$latest_release")

    local asset_name
    asset_name=$(echo "$release_info" | jq -r ".assets[] | select(.name | contains(\"op-deployer\") and contains(\"$platform\")) | .name")

    if [ -z "$asset_name" ]; then
        log_error "No asset for $platform"
        exit 1
    fi

    local download_url="https://github.com/ethereum-optimism/optimism/releases/download/$latest_release/$asset_name"

    log_info "Downloading: $download_url"

    curl -L -o "op-deployer.tar.gz" "$download_url"
    tar -xzf op-deployer.tar.gz

    local binary_path
    binary_path=$(find . -name "op-deployer" -type f | head -1)
    if [ -z "$binary_path" ]; then
        binary_path=$(find . -name "op-deployer*" -type f -perm +111 | head -1)
    fi

    if [ -z "$binary_path" ]; then
        log_error "Could not find binary"
        exit 1
    fi

    chmod +x "$binary_path"
    mv "$binary_path" ./op-deployer

    rm -f op-deployer.tar.gz
    rm -rf op-deployer-* 2>/dev/null || true

    if ./op-deployer --version >/dev/null 2>&1; then
        log_success "op-deployer ready: $(./op-deployer --version)"
    else
        log_success "op-deployer downloaded"
    fi
}

main() {
    log_info "Downloading op-deployer..."
    command -v curl &>/dev/null || { log_error "curl required"; exit 1; }
    command -v jq &>/dev/null || { log_error "jq required"; exit 1; }
    download_op_deployer
}

main "$@"
