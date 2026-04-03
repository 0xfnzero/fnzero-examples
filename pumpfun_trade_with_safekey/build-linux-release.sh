#!/bin/bash
set -e
BINARY_NAME="pumpfun_trade_with_safekey"
TARGET="x86_64-unknown-linux-gnu"
TARGET_DIR="linux-release"
DEPLOY_ARCHIVE="deploy.tar.gz"
STAGING_DIR=".deploy_staging"
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
echo -e "${GREEN}=== Building ${BINARY_NAME} for Linux ===${NC}"
if ! command -v x86_64-unknown-linux-gnu-gcc &> /dev/null; then
    command -v brew &> /dev/null && brew install messense/macos-cross-toolchains/x86_64-unknown-linux-gnu || { echo -e "${RED}Need x86_64-unknown-linux-gnu-gcc${NC}"; exit 1; }
fi
rustup target add ${TARGET} 2>/dev/null || true
cargo build --release --target ${TARGET}
[ ! -f "${TARGET_DIR}/${TARGET}/release/${BINARY_NAME}" ] && { echo -e "${RED}Build failed.${NC}"; exit 1; }
rm -rf "${STAGING_DIR}"
mkdir -p "${STAGING_DIR}"
cp "${TARGET_DIR}/${TARGET}/release/${BINARY_NAME}" "${STAGING_DIR}/"
cp run.sh startup-linux.sh "${STAGING_DIR}/"
cp -r config "${STAGING_DIR}/"
[ -f ".env.example" ] && cp .env.example "${STAGING_DIR}/"
rm -f "${DEPLOY_ARCHIVE}"
tar -czf "${DEPLOY_ARCHIVE}" -C "${STAGING_DIR}" .
rm -rf "${STAGING_DIR}"
mkdir -p "${TARGET_DIR}"
mv "${DEPLOY_ARCHIVE}" "${TARGET_DIR}/"
echo -e "${GREEN}Done ${TARGET_DIR}/${DEPLOY_ARCHIVE}${NC}"
