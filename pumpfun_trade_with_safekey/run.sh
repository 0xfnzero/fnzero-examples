#!/usr/bin/env bash
# =============================================================================
# pumpfun_trade_with_safekey — keystore / KEYPAIR_BASE58
# =============================================================================

DEFAULT_MINT="Cm6fNnMk7NfzStP9CZpsQA2v3jjzbcYGAxdJySmHpump"

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ -f .env ]; then
    set -a
    source .env
    set +a
elif [ -f ../../.env ]; then
    set -a
    source ../../.env
    set +a
fi

: "${APP_ENV:=dev}"
export APP_ENV

if [ "$APP_ENV" = "prod" ] || [ "$APP_ENV" = "production" ]; then
    CONFIG_DIR="config/prod"
else
    CONFIG_DIR="config/dev"
fi

if [ ! -f "$CONFIG_DIR/solana.yaml" ]; then
    echo "警告: 未找到 $CONFIG_DIR/solana.yaml"
fi

if [ -n "${1:-}" ]; then
    MINT="$1"
elif [ -n "${MINT:-}" ]; then
    :
else
    echo "未传入 mint 地址，使用默认或输入后回车。"
    printf "Mint address [%s]: " "$DEFAULT_MINT"
    read -r line
    MINT="${line:-$DEFAULT_MINT}"
fi
if [ -z "$MINT" ]; then
    echo "错误: mint 地址为空"
    exit 1
fi
export MINT

echo "环境: APP_ENV=$APP_ENV  配置: $CONFIG_DIR  MINT: $MINT"
echo ""

exec cargo run --release -- "$MINT"
