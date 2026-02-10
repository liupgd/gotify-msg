# Gotify 客户端

基于 Rust + Tauri 的 Gotify 桌面客户端应用。

## 🚀 项目说明

本项目通过 **AI 编程** 创建，使用 OpenCode 助手辅助开发完成。

## 📸 软件界面

![软件界面截图](./gotify-msg-screenshot.png)

## 💻 跨平台支持

- ✅ **Windows** (支持，待测试)
- ✅ **macOS** (支持，待测试)  
- ✅ **Linux** (已测试验证)

> **注意**：该应用理论支持 Windows、macOS 和 Linux 三大平台，但目前仅在 **Linux** 环境上进行了实际测试验证。

## 功能特性

- 📝 配置界面：简单易用的配置界面，支持配置 Gotify 服务器地址、Token 和通知超时时间
- 💾 本地配置：配置保存为 YAML 文件，存储在 `~/.config/gotify-client/config.yaml`
- 🔔 系统托盘：支持最小化到系统托盘，点击托盘图标可显示主窗口
- 📨 实时通知：通过 WebSocket 实时接收 Gotify 消息
- 🎨 自定义通知窗口：收到消息后在桌面右下角弹出通知窗口
- ⏱️ 自动关闭：通知窗口支持倒计时自动关闭和手动关闭
- 🎯 优先级显示：根据消息优先级显示不同的颜色主题

## 开发环境要求

- Rust (最新稳定版)
- Python 3 (用于启动开发服务器)
- 系统依赖（根据平台不同）

## 构建和运行

### 开发模式

**方法 1：使用启动脚本（推荐）**

```bash
./start-dev.sh
```

**方法 2：手动启动**

1. 在一个终端启动 HTTP 服务器：
```bash
cd src
python3 -m http.server 1420
```

2. 在另一个终端运行应用：
```bash
cd src-tauri
cargo run
```

### 构建发布版本

```bash
cd src-tauri
cargo tauri build
```

## 配置说明

应用首次启动后，需要在配置界面填写：

1. **服务器地址**：Gotify 服务器的地址（例如：`https://gotify.example.com`）
2. **Token**：Gotify 应用的 Token
3. **通知超时时间**：通知窗口自动关闭的时间（秒）

配置会自动保存到 `~/.config/gotify-client/config.yaml`。

## 使用说明

1. 启动应用后，在配置界面填写 Gotify 服务器信息
2. 点击"保存配置"保存设置
3. 点击"启动连接"开始接收消息
4. 收到消息后，会在桌面右下角弹出通知窗口
5. 通知窗口会在倒计时结束后自动关闭，也可以点击"确定"按钮手动关闭
6. 点击窗口关闭按钮会将应用最小化到系统托盘

## 项目结构

```
gotify-msg/
├── src-tauri/          # Rust 后端
│   ├── src/
│   │   ├── main.rs     # 应用入口
│   │   ├── config.rs   # 配置管理
│   │   ├── gotify.rs   # Gotify WebSocket 连接
│   │   └── commands.rs # Tauri 命令
│   └── Cargo.toml      # Rust 依赖
├── src/                # 前端代码
│   ├── index.html      # 主配置界面
│   ├── styles.css      # 样式文件
│   ├── main.js         # 前端逻辑
│   └── notification.html # 通知窗口
├── start-dev.sh        # 开发启动脚本
└── README.md           # 项目说明
```

## 技术栈

- **后端**：Rust + Tauri
- **前端**：HTML + CSS + JavaScript (Vanilla JS)
- **WebSocket**：tokio-tungstenite
- **配置存储**：serde_yaml

## 故障排除

### 前端文件无法加载

如果看到 "asset not found: index.html" 错误：

1. 确保 HTTP 服务器正在运行（端口 1420）
2. 检查 `src` 目录中是否存在 `index.html` 文件
3. 使用 `./start-dev.sh` 脚本自动启动服务器

### GLib 警告信息

运行时出现的 GLib-GObject-CRITICAL 警告通常是 GTK 库的版本兼容性问题，不影响功能使用。可以忽略这些警告。

## 许可证

MIT License
