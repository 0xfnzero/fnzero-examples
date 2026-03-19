#!/usr/bin/env bash
# =============================================================================
# 服务器用：直接运行已编译的 pumpswap_trade 二进制（不调用 cargo）
# =============================================================================
# 用法（在部署目录 ~/pumpswap_trade 下）:
#   ./startup-linux.sh              # 交互输入 mint 或回车用默认
#   ./startup-linux.sh <MINT>       # 指定 mint
#   APP_ENV=prod ./startup-linux.sh <MINT>
#
# 环境变量: APP_ENV, MINT, PRIVATE_KEY, SOLANA_RPC_URL 等同 run.sh
# =============================================================================

DEFAULT_MINT="Cm6fNnMk7NfzStP9CZpsQA2v3jjzbcYGAxdJySmHpump"
BINARY_NAME="pumpswap_trade"

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 加载 .env
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

if [ ! -f "$CONFIG_DIR/solana.yaml" ]; then
    echo "警告: 未找到 $CONFIG_DIR/solana.yaml，将回退到环境变量 / 默认配置"
fi

# MINT：第一个参数 > 环境变量 > 交互输入（回车用默认）
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
    echo "用法: $0 [MINT]  或  MINT=<代币地址> $0"
    exit 1
fi
export MINT

if [ ! -x "$SCRIPT_DIR/$BINARY_NAME" ]; then
    echo "错误: 未找到可执行文件 $SCRIPT_DIR/$BINARY_NAME"
    exit 1
fi

echo "环境: APP_ENV=$APP_ENV  配置: $CONFIG_DIR  MINT: $MINT"
echo ""

exec "$SCRIPT_DIR/$BINARY_NAME" "$MINT"
