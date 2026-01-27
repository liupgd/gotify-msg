#!/bin/bash
# 启动开发服务器脚本

# 在后台启动 HTTP 服务器
cd "$(dirname "$0")/src"
python3 -m http.server 1420 > /dev/null 2>&1 &
SERVER_PID=$!

# 等待服务器启动
sleep 1

# 启动 Tauri 应用
cd ../src-tauri
cargo run

# 清理：应用退出后停止服务器
kill $SERVER_PID 2>/dev/null

