#!/usr/bin/env bash
# 服务器用：直接运行 pumpfun_trade_with_safekey 二进制

DEFAULT_MINT="Cm6fNnMk7NfzStP9CZpsQA2v3jjzbcYGAxdJySmHpump"
BINARY_NAME="pumpfun_trade_with_safekey"

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ -f .env ]; then
    set -a
    source .env
    set +a
fi

: "${APP_ENV:=prod}"
export APP_ENV

if [ "$APP_ENV" = "prod" ] || [ "$APP_ENV" = "production" ]; then
    CONFIG_DIR="config/prod"
else
    CONFIG_DIR="config/dev"
fi

if [ -n "${1:-}" ]; then
    MINT="$1"
elif [ -n "${MINT:-}" ]; then
    :
else
    printf "Mint address [%s]: " "$DEFAULT_MINT"
    read -r line
    MINT="${line:-$DEFAULT_MINT}"
fi
if [ -z "$MINT" ]; then
    exit 1
fi
export MINT

if [ ! -x "$SCRIPT_DIR/$BINARY_NAME" ]; then
    echo "错误: 未找到可执行文件 $SCRIPT_DIR/$BINARY_NAME"
    exit 1
fi

exec "$SCRIPT_DIR/$BINARY_NAME" "$MINT"
