mod ai;
mod bot;
mod commands;
mod config;
mod error;
mod secrets;
mod state;
mod storage;
mod twitch;

use std::sync::Arc;

use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Several crates enable different rustls backends; pick one explicitly so
    // the WebSocket client can build its TLS config.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }));
    }

    builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("tao", log::LevelFilter::Warn)
                .level_for("tungstenite", log::LevelFilter::Warn)
                .max_file_size(2_000_000)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
                .build(),
        )
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;

            let config_dir = app.path().app_config_dir()?;
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&config_dir)?;
            std::fs::create_dir_all(&data_dir)?;
            let state = Arc::new(AppState::new(config_dir, data_dir));

            let (autostart, channel) = {
                let cfg = state.config.read();
                (
                    cfg.settings.start_bot_on_launch && cfg.settings.wizard_completed,
                    cfg.settings.active_channel_id.clone(),
                )
            };
            app.manage(state.clone());

            if autostart
                && let Some(channel) = channel {
                    let handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        if let Err(e) = state.bot.start(handle, state.clone(), channel, false).await {
                            log::warn!("autostart failed: {e}");
                        }
                    });
                }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_bootstrap,
            commands::save_config,
            commands::get_key_status,
            commands::set_ai_key,
            commands::ai_test,
            commands::ai_list_models,
            commands::detect_claude_cli,
            commands::twitch_login_start,
            commands::twitch_login_cancel,
            commands::twitch_remove_account,
            commands::bot_start,
            commands::bot_stop,
            commands::bot_running,
            commands::bot_control,
            commands::get_modlog,
            commands::get_usage,
            commands::clear_research_cache,
            commands::get_emote_urls,
            commands::twitch_api,
            commands::open_data_dir,
        ])
        .build(tauri::generate_context!())
        .expect("error while building HighBazar")
        .run(|app, event| {
            // Stop the bot cleanly on exit so usage numbers and the list of known
            // chatters are written to disk instead of being lost.
            if let tauri::RunEvent::Exit = event
                && let Some(state) = app.try_state::<Arc<AppState>>()
            {
                let state = state.inner().clone();
                let handle = app.clone();
                tauri::async_runtime::block_on(async move {
                    state.bot.stop(&handle).await;
                    state.usage.lock().save_if_dirty();
                });
            }
        });
}
