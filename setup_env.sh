#!/bin/bash

# ==========================================
# Agent Hub 环境依赖一键安装与补充脚本
# ==========================================

echo "=========================================="
echo "🚀 开始检查并补充 Agent Hub 环境依赖..."
echo "=========================================="

# 1. 检查 Homebrew (macOS 必备包管理器)
if ! command -v brew &> /dev/null; then
    echo "❌ 未检测到 Homebrew。正在安装 Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
else
    echo "✅ Homebrew 已安装。"
fi

# 2. 检查 Rust 编译环境
if ! command -v cargo &> /dev/null; then
    if [ ! -f "$HOME/.cargo/bin/cargo" ]; then
        echo "❌ 未检测到 Rust (cargo)。正在安装 Rust 工具链..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        echo "加载 Rust 环境变量..."
        source "$HOME/.cargo/env"
    else
        echo "✅ Rust (cargo) 已安装，但未在 PATH 中。临时加载环境变量..."
        source "$HOME/.cargo/env"
    fi
else
    echo "✅ Rust (cargo) 已安装。"
fi

# 3. 检查 ripgrep (Obsidian 搜索核心依赖)
if ! command -v rg &> /dev/null; then
    echo "❌ 未检测到 ripgrep (rg)。正在通过 Homebrew 安装..."
    brew install ripgrep
else
    echo "✅ ripgrep (rg) 已安装。"
fi

# 4. 检查 jq (测试脚本依赖)
if ! command -v jq &> /dev/null; then
    echo "❌ 未检测到 jq (JSON解析工具)。正在通过 Homebrew 安装..."
    brew install jq
else
    echo "✅ jq 已安装。"
fi

# 5. 补充项目 .env 环境变量模板
ENV_FILE="src-tauri/.env"
if [ ! -f "$ENV_FILE" ]; then
    echo "📝 正在生成环境变量模板文件 ($ENV_FILE)..."
    cat <<EOF > "$ENV_FILE"
# Agent Hub 环境变量配置 (自动加载需要第三方工具，或手动 source)

# SQLite 数据库路径 (可选，默认 ~/.agent-hub/agent_hub.sqlite)
# AGENT_HUB_DB_PATH=~/.agent-hub/agent_hub.sqlite

# Obsidian 本地知识库绝对路径 (必需，替换为你的实际路径)
OBSIDIAN_VAULT_PATH="/Users/cherrych/Documents/Mynotes"

# 豆包 API Key 与 模型配置 (必需，用于复杂意图识别)
DOUBAO_API_KEY="your_doubao_api_key_here"
DOUBAO_MODEL="ep-20241225134106-kpsps"
EOF
    echo "✅ 环境变量模板已生成。请在运行前修改 src-tauri/.env 中的实际路径和 Key！"
else
    echo "✅ 环境变量配置文件 ($ENV_FILE) 已存在。"
fi

echo "=========================================="
echo "🎉 环境依赖补充完毕！"
echo "👉 下一步建议："
echo "   1. 检查并修改 src-tauri/.env 文件中的配置项。"
echo "   2. 进入 src-tauri 目录执行: cargo build 测试编译。"
echo "=========================================="
