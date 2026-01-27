use crate::config::{self, AppConfig};
use crate::gotify::GotifyConnection;
use crate::AppState;
use tauri::{AppHandle, Manager, Emitter};
use std::sync::Arc;

#[tauri::command]
pub async fn save_config(
    server_url: String,
    token: String,
    timeout_seconds: u64,
) -> Result<(), String> {
    let config = AppConfig {
        server_url,
        token,
        timeout_seconds,
    };
    config::save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_config() -> Result<AppConfig, String> {
    config::load_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_gotify_connection(
    app_handle: AppHandle,
    server_url: String,
    token: String,
    timeout_seconds: u64,
) -> Result<(), String> {
    // 先停止现有连接
    stop_gotify_connection(app_handle.clone()).await?;

    let config = AppConfig {
        server_url,
        token,
        timeout_seconds,
    };

    // 保存配置
    config::save_config(&config).map_err(|e| e.to_string())?;

    let connection = Arc::new(GotifyConnection::new(app_handle.clone(), config.clone()));
    
    // 启动连接
    connection.start().await.map_err(|e| e.to_string())?;

    // 保存连接到应用状态
    let state = app_handle.state::<AppState>();
    let mut conn = state.gotify_connection.lock().await;
    *conn = Some(connection);

    Ok(())
}

#[tauri::command]
pub async fn stop_gotify_connection(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<AppState>();
    let mut conn = state.gotify_connection.lock().await;
    if let Some(connection) = conn.take() {
        connection.stop().await;
    }
    Ok(())
}

#[tauri::command]
pub async fn create_notification_window(
    app_handle: AppHandle,
    title: String,
    message: String,
    priority: Option<i32>,
    timeout_seconds: u64,
) -> Result<(), String> {
    // 获取主窗口位置和大小来计算通知窗口位置
    let main_window = app_handle.get_webview_window("main");
    let (screen_width, screen_height) = if let Some(window) = main_window {
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let size = monitor.size();
            (size.width as i32, size.height as i32)
        } else {
            (1920, 1080) // 默认值
        }
    } else {
        (1920, 1080)
    };

    let window_width = 350;
    let window_height = 150;
    let x = screen_width - window_width - 20;
    let y = screen_height - window_height - 20;

    let window_label = format!("notification-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs());

    let _window = tauri::webview::WebviewWindowBuilder::new(
        &app_handle,
        window_label.clone(),
        tauri::WebviewUrl::App("notification.html".into()),
    )
    .title("")
    .inner_size(window_width as f64, window_height as f64)
    .position(x as f64, y as f64)
    .resizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .build()
    .map_err(|e| e.to_string())?;

    // 等待窗口加载完成后发送消息数据
    let app_handle_clone = app_handle.clone();
    let window_label_clone = window_label.clone();
    let notification_data = serde_json::json!({
        "title": title,
        "message": message,
        "priority": priority,
        "timeout": timeout_seconds,
    });
    
    // 使用延迟确保窗口已加载
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if let Some(w) = app_handle_clone.get_webview_window(&window_label_clone) {
            let _ = w.emit("notification-data", &notification_data);
        }
    });

    Ok(())
}

