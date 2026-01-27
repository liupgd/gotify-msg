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

                // 确保 URL 格式正确
                let base_url = config.server_url.trim_end_matches('/');
                let ws_url = if base_url.starts_with("https://") {
                    base_url.replace("https://", "wss://") + "/stream?token=" + &config.token
                } else if base_url.starts_with("http://") {
                    base_url.replace("http://", "ws://") + "/stream?token=" + &config.token
                } else {
                    format!("wss://{}/stream?token={}", base_url, config.token)
                };
                
                match connect_async(&ws_url).await {
                    Ok((ws_stream, _)) => {
                        let (mut _write, mut read) = ws_stream.split();
                        
                        while let Some(message) = read.next().await {
                            let is_running = *running.lock().await;
                            if !is_running {
                                break;
                            }

                            match message {
                                Ok(Message::Text(text)) => {
                                    if let Ok(json_value) = serde_json::from_str::<Value>(&text) {
                                        // 发送消息到前端
                                        let _ = app_handle.emit("gotify-message", &json_value);
                                    }
                                }
                                Ok(Message::Close(_)) => {
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
                        // 等待一段时间后重连
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
    }
}

