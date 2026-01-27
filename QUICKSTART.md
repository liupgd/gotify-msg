# 快速开始指南

## 安装依赖

### Rust 环境

确保已安装 Rust：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Tauri CLI（可选）

如果需要使用 Tauri CLI：

```bash
cargo install tauri-cli
```

## 开发运行

进入 `src-tauri` 目录并运行：

```bash
cd src-tauri
cargo tauri dev
```

或者如果安装了 Tauri CLI：

```bash
cargo tauri dev
```

## 构建发布版本

```bash
cd src-tauri
cargo tauri build
```

构建产物会在 `src-tauri/target/release/bundle/` 目录中。

## 配置应用

1. 启动应用后，在主界面填写：
   - **服务器地址**：您的 Gotify 服务器地址（例如：`https://gotify.example.com`）
   - **Token**：Gotify 应用的 Token（在 Gotify 管理界面创建应用后获取）
   - **通知超时时间**：通知窗口自动关闭的时间（秒，默认 5 秒）

2. 点击"保存配置"保存设置

3. 点击"启动连接"开始接收消息

## 功能说明

- **系统托盘**：关闭主窗口时，应用会最小化到系统托盘
- **实时通知**：收到 Gotify 消息后，会在桌面右下角弹出通知窗口
- **自动关闭**：通知窗口会在倒计时结束后自动关闭
- **手动关闭**：点击通知窗口的"确定"按钮可立即关闭

## 故障排除

### 连接失败

- 检查服务器地址格式是否正确（应包含协议，如 `https://`）
- 确认 Token 是否正确
- 检查网络连接和防火墙设置

### 通知窗口不显示

- 检查浏览器控制台是否有错误信息
- 确认 Gotify 服务器正在运行
- 检查消息格式是否正确

## 配置文件位置

配置文件保存在：`~/.config/gotify-client/config.yaml`

格式示例：

```yaml
server_url: "https://gotify.example.com"
token: "your-token-here"
timeout_seconds: 5
```

