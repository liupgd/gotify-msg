// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod gotify;

use tauri::{Manager, menu::{Menu, MenuItem}};
use tauri::tray::TrayIconBuilder;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    gotify_connection: Arc<Mutex<Option<Arc<gotify::GotifyConnection>>>>,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 创建系统托盘菜单
            let quit = MenuItem::with_id(app, "quit", "退出", true, Option::<&str>::None)?;
            let show = MenuItem::with_id(app, "show", "显示窗口", true, Option::<&str>::None)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            std::process::exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, _event| {
                    // 点击托盘图标显示窗口
                    if let Some(app) = tray.app_handle().get_webview_window("main") {
                        let _ = app.show();
                        let _ = app.set_focus();
                    }
                })
                .build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                // 窗口关闭时最小化到托盘
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            // 初始化应用状态
            app.manage(AppState {
                gotify_connection: Arc::new(Mutex::new(None)),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::save_config,
            commands::load_config,
            commands::start_gotify_connection,
            commands::stop_gotify_connection,
            commands::create_notification_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

