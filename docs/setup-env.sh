#!/bin/bash
# ============================================
# OpsPilot 环境变量一键设置脚本
# ============================================
# 使用方式: bash setup-env.sh
# 不要用 source，直接执行

# 移除 set -e，避免任何命令失败就退出

echo "OpsPilot 环境变量设置"
echo "========================"
echo ""

# ─── 检测 shell rc 文件 ───
SHELL_RC=""
if [ -f "$HOME/.zshrc" ]; then
    SHELL_RC="$HOME/.zshrc"
elif [ -f "$HOME/.bashrc" ]; then
    SHELL_RC="$HOME/.bashrc"
elif [ -f "$HOME/.bash_profile" ]; then
    SHELL_RC="$HOME/.bash_profile"
fi

if [ -z "$SHELL_RC" ]; then
    echo "未检测到 shell 配置文件，将只输出临时命令。"
fi

# ─── 询问是否永久写入 ───
PERMANENT=""
if [ -n "$SHELL_RC" ]; then
    printf "是否永久写入 %s？(y/N): " "$SHELL_RC"
    read -r PERMANENT
fi

echo ""
echo "需要配置以下环境变量："
echo ""
echo "  1. OPENCODE_API_KEY  - OpenCode Go 订阅的 API Key"
echo "  2. JWT_SECRET        - JWT 认证密钥（自动生成）"
echo ""

# ─── OpenCode API Key ───
OPENCODE_KEY=""
printf "请输入你的 OpenCode Go API Key（留空跳过）: "
read -r OPENCODE_KEY
if [ -z "$OPENCODE_KEY" ]; then
    echo "未输入 Key，跳过。你可以稍后手动设置："
    echo "  export OPENCODE_API_KEY='your-key-here'"
fi

# ─── 自动生成 JWT Secret ───
JWT_SECRET=""
if command -v openssl >/dev/null 2>&1; then
    JWT_SECRET=$(openssl rand -hex 32)
fi
if [ -z "$JWT_SECRET" ]; then
    # fallback: 用 /dev/urandom
    JWT_SECRET=$(tr -dc 'a-f0-9' < /dev/urandom | head -c 64)
fi
if [ -z "$JWT_SECRET" ]; then
    # 再 fallback: 用 date + random
    JWT_SECRET=$(printf '%s%s%s%s' "$(date +%s)" "$$" "$RANDOM" "$RANDOM" | sha256sum 2>/dev/null | head -c 64 || echo "please-change-me-to-a-random-string-64chars")
fi
echo ""
echo "已自动生成 JWT_SECRET: ${JWT_SECRET:0:8}...${JWT_SECRET: -8}"

# ─── 写入环境变量 ───
if [ "$PERMANENT" = "y" ] || [ "$PERMANENT" = "Y" ]; then
    echo ""
    echo "写入 $SHELL_RC ..."
    
    # 移除旧的 OpsPilot 配置（如果有）
    if [ -f "$SHELL_RC" ]; then
        # 用临时文件方式删除，兼容 macOS 和 Linux
        tmpfile=$(mktemp)
        awk '/# OpsPilot env/,/# End OpsPilot env/ { next } { print }' "$SHELL_RC" > "$tmpfile" && mv "$tmpfile" "$SHELL_RC"
    fi
    
    cat >> "$SHELL_RC" << ENVEOF

# ─── OpsPilot env (auto-generated) ───
export OPENCODE_API_KEY="${OPENCODE_KEY}"
export JWT_SECRET="${JWT_SECRET}"
export OPENCODE_BASE_URL="https://opencode.ai/zen/go/v1"
# End OpsPilot env
ENVEOF
    
    echo "已写入 $SHELL_RC"
    echo "运行 'source $SHELL_RC' 使其生效"
else
    echo ""
    echo "临时设置（仅当前终端生效）："
    echo ""
    if [ -n "$OPENCODE_KEY" ]; then
        echo "export OPENCODE_API_KEY=\"$OPENCODE_KEY\""
    fi
    echo "export JWT_SECRET=\"$JWT_SECRET\""
    echo "export OPENCODE_BASE_URL=\"https://opencode.ai/zen/go/v1\""
    echo ""
    echo "复制上面的命令到终端执行即可。"
fi

echo ""
echo "配置完成！"
echo ""
echo "可用模型（通过 OpenCode Go 订阅）："
echo "  - mimo-v2.5-pro     MiMo V2.5 Pro (最强编码)"
echo "  - mimo-v2.5         MiMo V2.5 (性价比高)"
echo "  - deepseek-v4-pro   DeepSeek V4 Pro"
echo "  - deepseek-v4-flash DeepSeek V4 Flash (最快)"
echo "  - kimi-k2.7-code    Kimi K2.7 Code"
echo "  - qwen3.7-max       Qwen 3.7 Max"
echo "  - qwen3.7-plus      Qwen 3.7 Plus"
echo "  - minimax-m3        MiniMax M3"
echo "  - glm-5.2           GLM 5.2"
echo ""
echo "下一步："
echo "  1. cd ops-pilot"
echo "  2. mimo              # 用 MiMoCode 编辑器开发"
echo "  3. opencode          # 用 OpenCode CLI 开发"
