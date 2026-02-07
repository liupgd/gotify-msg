use crate::config::{self, AppConfig};
use crate::gotify::GotifyConnection;
use crate::AppState;
use tauri::{AppHandle, Manager, Emitter};
use std::sync::Arc;
use anyhow::Result;

#[tauri::command]
pub async fn save_config(
    server_url: String,
    token: String,
    timeout_seconds: u64,
) -> Result<(), String> {
     // 验证输入参数
     if timeout_seconds == 0 {
         return Err("超时时间必须大于 0".to_string());
     }

     if timeout_seconds > 300 {
         return Err("超时时间不能超过 300 秒".to_string());
     }
    println!("=== save_config 函数被调用 ===");
    eprintln!("参数: server_url={}, token={}, timeout_seconds={}", 
              server_url, 
              if token.is_empty() { "<empty>".to_string() } else { format!("{}...", &token[..token.len().min(4)]) },
              timeout_seconds);
    
    let config = AppConfig {
        server_url: server_url.clone(),
        token: token.clone(),
        timeout_seconds,
    };
    
    eprintln!("准备保存配置: server_url={}, timeout_seconds={}", config.server_url, config.timeout_seconds);
    
    match config::save_config(&config) {
        Ok(_) => {
            eprintln!("配置保存成功！");
            Ok(())
        }
        Err(e) => {
            eprintln!("配置保存失败: {}", e);
            Err(e.to_string())
        }
    }
}

fn play_notification_sound() {
    // 获取音频文件路径
    let audio_path = if cfg!(debug_assertions) {
        // 开发模式：使用项目目录
        std::path::PathBuf::from("../src/notification.wav")
    } else {
        // 生产模式：使用应用资源目录
        // 这里简化处理，实际应该使用 tauri::api::path 获取资源路径
        std::path::PathBuf::from("notification.wav")
    };

    let audio_path_str = audio_path.to_string_lossy();

    // 使用系统命令播放音频
    #[cfg(target_os = "linux")]
    {
        // Linux: 尝试使用 paplay (PulseAudio) 或 aplay (ALSA)
        let play_commands = [
            format!("paplay {}", audio_path_str),
            format!("aplay {}", audio_path_str),
            format!("ffplay -nodisp -autoexit {}", audio_path_str),
        ];

        for cmd in &play_commands {
            if let Ok(_) = std::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                eprintln!("✓ 正在播放提示音: {}", cmd);
                break;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 afplay
        let _ = std::process::Command::new("afplay")
            .arg(&audio_path_str)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 PowerShell
        let _ = std::process::Command::new("powershell")
            .args(&["-c", &format!("(New-Object Media.SoundPlayer '{}').PlaySync()", audio_path_str)])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
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
     // 验证输入参数
     if server_url.trim().is_empty() {
         return Err("服务器地址不能为空".to_string());
     }

     if token.trim().is_empty() {
         return Err("Token 不能为空".to_string());
     }

     if timeout_seconds == 0 {
         return Err("超时时间必须大于 0".to_string());
     }

     // 基本格式验证
     if !server_url.starts_with("http://") && 
        !server_url.starts_with("https://") && 
        !server_url.starts_with("ws://") && 
        !server_url.starts_with("wss://") {
         return Err("服务器地址必须以 http://, https://, ws:// 或 wss:// 开头".to_string());
     }
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
     // 验证输入参数
     if title.trim().is_empty() && message.trim().is_empty() {
         return Err("标题和消息不能同时为空".to_string());
     }

     if timeout_seconds == 0 {
         return Err("超时时间必须大于 0".to_string());
     }

     if timeout_seconds > 300 {
         return Err("超时时间不能超过 300 秒".to_string());
     }
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

    // 将数据编码到 URL 参数中，确保页面加载时就能获取
    let url = format!(
        "notification.html?title={}&message={}&priority={}&timeout={}",
        urlencoding::encode(&title),
        urlencoding::encode(&message),
        priority.unwrap_or(0),
        timeout_seconds
    );

    let _window = tauri::webview::WebviewWindowBuilder::new(
        &app_handle,
        window_label.clone(),
        tauri::WebviewUrl::App(url.into()),
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

    // 播放提示音
    play_notification_sound();

    Ok(())
}

