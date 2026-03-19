#!/bin/bash
# 编译 Linux 二进制并打包
# 仅使用本机交叉编译工具链（brew install x86_64-unknown-linux-gnu），不使用 Docker/cross。
# 用法: 在本目录执行 ./build-linux-release.sh

set -e

BINARY_NAME="pumpswap_trade"
TARGET="x86_64-unknown-linux-gnu"
TARGET_DIR="linux-release"
DEPLOY_ARCHIVE="deploy.tar.gz"
STAGING_DIR=".deploy_staging"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo -e "${GREEN}=== 1. Building ${BINARY_NAME} for Linux ===${NC}"
if ! command -v x86_64-unknown-linux-gnu-gcc &> /dev/null; then
    echo -e "${YELLOW}Installing cross-compilation toolchain...${NC}"
    command -v brew &> /dev/null && brew install messense/macos-cross-toolchains/x86_64-unknown-linux-gnu || { echo -e "${RED}Need Homebrew or install x86_64-unknown-linux-gnu${NC}"; exit 1; }
fi
rustup target add ${TARGET} 2>/dev/null || true
cargo build --release --target ${TARGET}
[ ! -f "${TARGET_DIR}/${TARGET}/release/${BINARY_NAME}" ] && { echo -e "${RED}Build failed.${NC}"; exit 1; }
echo -e "${GREEN}Build OK.${NC}"

echo -e "${GREEN}=== 2. Packing ${DEPLOY_ARCHIVE} ===${NC}"
rm -rf "${STAGING_DIR}"
mkdir -p "${STAGING_DIR}"
cp "${TARGET_DIR}/${TARGET}/release/${BINARY_NAME}" "${STAGING_DIR}/"
cp run.sh "${STAGING_DIR}/"
cp startup-linux.sh "${STAGING_DIR}/"
cp -r config "${STAGING_DIR}/"
[ -f ".env.example" ] && cp .env.example "${STAGING_DIR}/"
rm -f "${DEPLOY_ARCHIVE}"
tar -czf "${DEPLOY_ARCHIVE}" -C "${STAGING_DIR}" .
rm -rf "${STAGING_DIR}"
mkdir -p "${TARGET_DIR}"
mv "${DEPLOY_ARCHIVE}" "${TARGET_DIR}/"
echo -e "${GREEN}Created ${TARGET_DIR}/${DEPLOY_ARCHIVE} ($(du -h ${TARGET_DIR}/${DEPLOY_ARCHIVE} | cut -f1))${NC}"

echo ""
echo -e "${GREEN}=== Build done. ===${NC}"
echo ""
echo -e "${YELLOW}Files in ${TARGET_DIR}:"
echo "  ${TARGET_DIR}/${TARGET}/release/${BINARY_NAME}  (binary)"
echo "  ${TARGET_DIR}/${DEPLOY_ARCHIVE}                (archive)"
echo ""
echo -e "${YELLOW}Usage:${NC}"
echo "  Upload ${TARGET_DIR}/${DEPLOY_ARCHIVE} to your server"
echo "  Extract: tar -xzf ${DEPLOY_ARCHIVE}"
echo "  Run: ./startup-linux.sh"
echo ""
