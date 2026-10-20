//! Desktop shell: the main window's life cycle, the tray icon for
//! performance mode and the process priority.
//!
//! Performance mode: minimizing or closing the window destroys it completely
//! (WebView2 and its processes end), only the Rust core keeps running the bot
//! and the overlay server. The tray icon brings the window back.

use std::sync::Arc;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WebviewWindow, WebviewWindowBuilder};

use crate::state::AppState;

const TRAY: &str = "main";
const MAIN: &str = "main";

fn state(app: &AppHandle) -> Option<Arc<AppState>> {
    app.try_state::<Arc<AppState>>().map(|s| s.inner().clone())
}

fn performance_mode(app: &AppHandle) -> bool {
    state(app).is_some_and(|s| s.config.read().settings.performance_mode)
}

/// German or English, like the app's own language setting.
fn de(app: &AppHandle) -> bool {
    let lang = state(app).map(|s| s.config.read().settings.language.clone()).unwrap_or_default();
    match lang.as_str() {
        "de" => true,
        "en" => false,
        _ => system_language_is_german(),
    }
}

#[cfg(windows)]
fn system_language_is_german() -> bool {
    use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;
    // Primary language id 0x07 = German.
    (unsafe { GetUserDefaultUILanguage() } & 0x3ff) == 0x07
}

#[cfg(not(windows))]
fn system_language_is_german() -> bool {
    std::env::var("LANG").is_ok_and(|l| l.starts_with("de"))
}

// ---------------------------------------------------------------------------
// Window
// ---------------------------------------------------------------------------

/// Shows the main window, creating it if it was closed (performance mode).
pub fn open_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        set_webview_visible(&w, true);
        return;
    }
    let Some(cfg) = app.config().app.windows.iter().find(|w| w.label == MAIN).cloned() else {
        return;
    };
    // WebView2 must not be created synchronously inside an event-loop callback
    // (tray click, second start): it would stay blank.
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if app.get_webview_window(MAIN).is_some() {
            return;
        }
        match WebviewWindowBuilder::from_config(&app, &cfg).and_then(|b| b.build()) {
            Ok(w) => {
                let _ = w.set_focus();
            }
            Err(e) => log::error!("could not open the window: {e}"),
        }
    });
}

/// Closes the whole interface; the bot keeps running in the tray.
pub fn to_tray(app: &AppHandle) {
    sync_tray(app, true);
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.destroy();
    }
}

/// Window events: minimizing pauses (or, in performance mode, closes) the UI.
pub fn on_window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    if window.label() != MAIN {
        return;
    }
    let app = window.app_handle();
    if let tauri::WindowEvent::Resized(_) = event {
        let minimized = window.is_minimized().unwrap_or(false);
        if minimized && performance_mode(app) {
            to_tray(app);
        } else if let Some(w) = app.get_webview_window(MAIN) {
            // A minimized window still renders unless WebView2 is told it's hidden.
            set_webview_visible(&w, !minimized);
        }
    }
}

/// Whether the app may quit when the last window closes.
pub fn keep_running_without_window(app: &AppHandle) -> bool {
    performance_mode(app)
}

#[cfg(windows)]
fn set_webview_visible(w: &WebviewWindow, visible: bool) {
    let _ = w.with_webview(move |wv| unsafe {
        // Hidden: Chromium stops rendering and throttles timers.
        let _ = wv.controller().SetIsVisible(visible);
    });
}

#[cfg(not(windows))]
fn set_webview_visible(_w: &WebviewWindow, _visible: bool) {}

// ---------------------------------------------------------------------------
// Tray
// ---------------------------------------------------------------------------

/// Creates or removes the tray icon to match performance mode.
pub fn sync_tray(app: &AppHandle, on: bool) {
    let exists = app.tray_by_id(TRAY).is_some();
    if on && !exists {
        let mut b = TrayIconBuilder::with_id(TRAY)
            .tooltip("HighBazar")
            .show_menu_on_left_click(false)
            .on_menu_event(|app, e| match e.id().as_ref() {
                "open" => open_window(app),
                "bot" => toggle_bot(app.clone()),
                "quit" => app.exit(0),
                _ => {}
            })
            .on_tray_icon_event(|tray, e| {
                if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                    open_window(tray.app_handle());
                }
            });
        if let Some(icon) = app.default_window_icon() {
            b = b.icon(icon.clone());
        }
        if let Err(e) = b.build(app) {
            log::warn!("tray icon: {e}");
            return;
        }
        refresh_tray(app.clone());
    } else if !on && exists {
        let _ = app.remove_tray_by_id(TRAY);
    }
}

/// Updates the tray tooltip and menu with the bot's state.
pub fn refresh_tray(app: AppHandle) {
    if app.tray_by_id(TRAY).is_none() {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let Some(st) = state(&app) else { return };
        let running = st.bot.current().await;
        let de = de(&app);
        let (tip, bot_label) = match &running {
            Some((id, sandbox)) => {
                let name = st.config.read().channel(id).map(|c| c.name.clone()).unwrap_or_default();
                let mode = match (*sandbox, de) {
                    (true, true) => " (Testmodus)",
                    (true, false) => " (test mode)",
                    _ => "",
                };
                (
                    if de { format!("HighBazar – Bot läuft: {name}{mode}") } else { format!("HighBazar – bot running: {name}{mode}") },
                    if de { "Bot stoppen" } else { "Stop bot" },
                )
            }
            None => (
                if de { "HighBazar – Bot gestoppt".to_string() } else { "HighBazar – bot stopped".to_string() },
                if de { "Bot starten" } else { "Start bot" },
            ),
        };
        let menu = (|| -> tauri::Result<Menu<tauri::Wry>> {
            Menu::with_items(
                &app,
                &[
                    &MenuItem::with_id(&app, "open", if de { "HighBazar öffnen" } else { "Open HighBazar" }, true, None::<&str>)?,
                    &MenuItem::with_id(&app, "bot", bot_label, true, None::<&str>)?,
                    &PredefinedMenuItem::separator(&app)?,
                    &MenuItem::with_id(&app, "quit", if de { "Beenden" } else { "Quit" }, true, None::<&str>)?,
                ],
            )
        })();
        if let Some(tray) = app.tray_by_id(TRAY) {
            let _ = tray.set_tooltip(Some(tip));
            if let Ok(menu) = menu {
                let _ = tray.set_menu(Some(menu));
            }
        }
    });
}

fn toggle_bot(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let Some(st) = state(&app) else { return };
        if st.bot.current().await.is_some() {
            st.bot.stop(&app).await;
        } else {
            let channel = st.config.read().settings.active_channel_id.clone();
            if let Some(channel) = channel
                && let Err(e) = st.bot.start(app.clone(), st.clone(), channel, false).await
            {
                log::warn!("start from tray failed: {e}");
                // The window explains what is missing.
                open_window(&app);
            }
        }
        refresh_tray(app);
    });
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/// Applies settings that live outside the web UI.
pub fn apply_settings(app: &AppHandle, st: &Arc<AppState>) {
    let s = st.config.read().settings.clone();
    set_low_priority(s.low_priority);
    sync_tray(app, s.performance_mode);
    crate::overlay::sync(st);
    st.overlay.settings(&s);
}

/// "Below normal" lets games win every contest for the CPU. Child processes
/// started afterwards (WebView2) inherit it.
pub fn set_low_priority(on: bool) {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Threading::{BELOW_NORMAL_PRIORITY_CLASS, GetCurrentProcess, NORMAL_PRIORITY_CLASS, SetPriorityClass};
        SetPriorityClass(GetCurrentProcess(), if on { BELOW_NORMAL_PRIORITY_CLASS } else { NORMAL_PRIORITY_CLASS });
    }
    #[cfg(not(windows))]
    let _ = on;
}
