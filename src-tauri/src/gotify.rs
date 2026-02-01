use crate::config::AppConfig;
use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct GotifyConnection {
    app_handle: AppHandle,
    config: AppConfig,
    running: Arc<Mutex<bool>>,
}

impl GotifyConnection {
    pub fn new(app_handle: AppHandle, config: AppConfig) -> Self {
        Self {
            app_handle,
            config,
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut running = self.running.lock().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        let app_handle = self.app_handle.clone();
        let config = self.config.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            loop {
                let is_running = *running.lock().await;
                if !is_running {
                    break;
                }

                // 检查 token 是否为空
                if config.token.is_empty() {
                    eprintln!("错误: Token 为空，无法连接");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
                
                // 确保 URL 格式正确，并对 token 进行 URL 编码
                let base_url = config.server_url.trim_end_matches('/');
                let encoded_token = urlencoding::encode(&config.token).to_string();
                
                let ws_url = if base_url.starts_with("https://") {
                    format!("{}/stream?token={}", base_url.replace("https://", "wss://"), encoded_token)
                } else if base_url.starts_with("http://") {
                    format!("{}/stream?token={}", base_url.replace("http://", "ws://"), encoded_token)
                } else {
                    format!("wss://{}/stream?token={}", base_url, encoded_token)
                };
                
                eprintln!("尝试连接到 Gotify WebSocket");
                eprintln!("服务器地址: {}", base_url);
                eprintln!("Token 长度: {} 字符", config.token.len());
                if config.token.len() >= 4 {
                    eprintln!("Token 前缀: {}...", &config.token.chars().take(4).collect::<String>());
                }
                // 在日志中隐藏完整 token
                let safe_url = ws_url.replace(&encoded_token, "***");
                eprintln!("WebSocket URL: {}", safe_url);
                
                match connect_async(&ws_url).await {
                    Ok((ws_stream, _)) => {
                        eprintln!("成功连接到 Gotify WebSocket");
                        let (mut _write, mut read) = ws_stream.split();
                        
                        while let Some(message) = read.next().await {
                            let is_running = *running.lock().await;
                            if !is_running {
                                break;
                            }

                            match message {
                                Ok(Message::Text(text)) => {
                                    eprintln!("收到 WebSocket 消息: {}", text);
                                    if let Ok(json_value) = serde_json::from_str::<Value>(&text) {
                                        eprintln!("解析后的 JSON: {:?}", json_value);
                                        
                                        // Gotify WebSocket 可能返回包含 messages 数组的对象
                                        // 或者直接返回消息对象
                                        let message_to_emit = if json_value.get("messages").is_some() {
                                            // 如果是包含 messages 数组的格式，提取第一个消息
                                            if let Some(messages) = json_value.get("messages").and_then(|m| m.as_array()) {
                                                if let Some(first_msg) = messages.first() {
                                                    first_msg.clone()
                                                } else {
                                                    json_value
                                                }
                                            } else {
                                                json_value
                                            }
                                        } else {
                                            // 直接是消息对象
                                            json_value
                                        };
                                        
                                        eprintln!("发送到前端的数据: {:?}", message_to_emit);
                                        // 发送消息到前端
                                        let _ = app_handle.emit("gotify-message", &message_to_emit);
                                    } else {
                                        eprintln!("JSON 解析失败: {}", text);
                                    }
                                }
                                Ok(Message::Close(_)) => {
                                    eprintln!("WebSocket 连接已关闭");
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("WebSocket error: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to Gotify: {}", e);
                    }
                }
                
                // 连接断开后等待一段时间再重连
                let is_running = *running.lock().await;
                if !is_running {
                    break;
                }
                eprintln!("等待 5 秒后重连...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        Ok(())
    }

    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
    }
}


