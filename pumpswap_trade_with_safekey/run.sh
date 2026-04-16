#!/usr/bin/env bash
# =============================================================================
# pumpswap_trade_with_safekey 运行脚本（需要 keystore.json 钱包）
# =============================================================================
# 用法:
#   ./run.sh                     # 无参数：交互输入 mint，直接回车则用默认地址
#   ./run.sh <MINT>              # 第一个参数为代币 mint 地址
#   MINT=xxx ./run.sh            # 或通过环境变量
#   KEYSTORE_PASSWORD=xxx ./run.sh <MINT>   # 非交互时传入 keystore 密码
#
# 默认 mint（无参数且交互时直接回车）:
#   Ce2gx9KGXJ6C9Mp5b5x1sn9Mg87JwEbrQby4Zqo3pump
#
# 环境变量（可选）:
#   APP_ENV=dev|prod    默认 dev，决定读取 config/dev 或 config/prod
#   MINT                代币 mint 地址（也可用第一个参数）
#   KEYSTORE_PASSWORD   keystore.json 密码，不设则运行时交互输入
#   SOLANA_RPC_URL / CONFIG_FILE 等见 config 与程序说明
# =============================================================================

# 默认 mint 地址（无参数且交互回车时使用）
DEFAULT_MINT="Ce2gx9KGXJ6C9Mp5b5x1sn9Mg87JwEbrQby4Zqo3pump"

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 加载同目录或项目根目录的 .env（可选）
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

# keystore.json 必须放在本目录 (pumpswap_trade_with_safekey) 下（写死路径，避免被 .env 覆盖）
KEYSTORE_FILE="${SCRIPT_DIR}/keystore.json"
if [ ! -f "$KEYSTORE_FILE" ]; then
    echo "错误: 未找到 keystore.json（请将 keystore.json 放在本目录下）"
    echo "  路径: $KEYSTORE_FILE"
    exit 1
fi

# 可选：检查 config 存在
if [ ! -f "$CONFIG_DIR/solana.yaml" ]; then
    echo "警告: 未找到 $CONFIG_DIR/solana.yaml，将回退到环境变量 / 默认配置"
fi

# MINT：第一个参数 > 环境变量 > 交互输入（回车用默认）
if [ -n "${1:-}" ]; then
    MINT="$1"
elif [ -n "${MINT:-}" ]; then
    : # 已从环境变量或 .env 得到
else
    echo "未传入 mint 地址，使用默认或输入后回车。"
    printf "Mint address [%s]: " "$DEFAULT_MINT"
    read -r line
    MINT="${line:-$DEFAULT_MINT}"
fi
if [ -z "$MINT" ]; then
    echo "错误: mint 地址为空"
    echo "用法: $0 [MINT]  或  MINT=<代币地址> $0  或直接运行 $0 后按提示输入"
    exit 1
fi
export MINT

echo "环境: APP_ENV=$APP_ENV  配置: $CONFIG_DIR  keystore: $KEYSTORE_FILE  MINT: $MINT"
echo ""

exec cargo run --release -- "$MINT"
