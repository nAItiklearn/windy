#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use serde::Serialize;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, RunEvent, State, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use windy_core::action::{self, Action, Modifier};
use windy_core::clipboard::{Monitor, NewItem, Outcome};
use windy_core::history::History;
use windy_core::search::search;
use windy_core::settings::{PinTo, PopupPosition, Settings};
use windy_core::storage::Storage;
use windy_core::types::{kind, Item};

struct Core {
    history: History,
    storage: Storage,
    settings: Settings,
    monitor: Monitor,
    last_token: u64,
    settings_path: PathBuf,
}

type Shared = Mutex<Core>;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn lock(state: &Shared) -> Result<std::sync::MutexGuard<'_, Core>, String> {
    state
        .lock()
        .map_err(|_| "internal state unavailable".to_string())
}

/// Only our own two windows may call commands.
fn guard(window: &WebviewWindow) -> Result<(), String> {
    match window.label() {
        "popup" | "settings" => Ok(()),
        _ => Err("forbidden".into()),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemView {
    id: i64,
    title: String,
    pin: Option<String>,
    application: Option<String>,
    copies: u32,
    kind: &'static str,
}

fn view(it: &Item) -> ItemView {
    let has_text = it.content(kind::TEXT).is_some();
    let kind = if it.content(kind::FILES).is_some() {
        "files"
    } else if !has_text && kind::IMAGES.iter().any(|k| it.content(k).is_some()) {
        "image"
    } else {
        "text"
    };
    ItemView {
        id: it.id,
        title: it.title.clone(),
        pin: it.pin.map(String::from),
        application: it.application.clone(),
        copies: it.number_of_copies,
        kind,
    }
}

#[tauri::command]
fn list_items(
    window: WebviewWindow,
    state: State<'_, Shared>,
    query: String,
) -> Result<Vec<ItemView>, String> {
    guard(&window)?;
    let core = lock(&state)?;
    let order = core.history.ordered(&core.settings);
    let (pinned, rest): (Vec<&Item>, Vec<&Item>) = order.into_iter().partition(|i| i.pin.is_some());
    let run = |group: Vec<&Item>| -> Vec<ItemView> {
        let titles: Vec<&str> = group.iter().map(|i| i.title.as_str()).collect();
        search(&query, core.settings.search_mode, &titles)
            .into_iter()
            .map(|i| view(group[i]))
            .collect()
    };
    let (pinned, rest) = (run(pinned), run(rest));
    Ok(match core.settings.pin_to {
        PinTo::Top => pinned.into_iter().chain(rest).collect(),
        PinTo::Bottom => rest.into_iter().chain(pinned).collect(),
    })
}

#[tauri::command]
fn get_image(
    window: WebviewWindow,
    state: State<'_, Shared>,
    id: i64,
) -> Result<Option<String>, String> {
    guard(&window)?;
    let core = lock(&state)?;
    let Some(item) = core.history.get(id) else {
        return Ok(None);
    };
    Ok(kind::IMAGES.iter().find_map(|k| item.content(k)).map(|c| {
        format!(
            "data:{};base64,{}",
            c.kind,
            base64::engine::general_purpose::STANDARD.encode(&c.data)
        )
    }))
}

fn parse_modifier(s: &str) -> Modifier {
    match s {
        "primary" => Modifier::Primary,
        "alt" => Modifier::Alt,
        "altShift" => Modifier::AltShift,
        "primaryShift" => Modifier::PrimaryShift,
        _ => Modifier::None,
    }
}

#[tauri::command]
fn select_item(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, Shared>,
    id: i64,
    modifier: String,
) -> Result<(), String> {
    guard(&window)?;
    let (contents, paste) = {
        let core = lock(&state)?;
        let Some(item) = core.history.get(id) else {
            return Ok(());
        };
        let s = &core.settings;
        match action::resolve(
            s.paste_by_default,
            s.remove_formatting_by_default,
            parse_modifier(&modifier),
        ) {
            Action::Unknown => return Ok(()),
            Action::Copy => (item.contents.clone(), false),
            Action::CopyWithoutFormatting => (action::strip_formatting(&item.contents), false),
            Action::Paste => (item.contents.clone(), true),
            Action::PasteWithoutFormatting => (action::strip_formatting(&item.contents), true),
        }
    };
    windy_platform::write_contents(&contents).map_err(|e| e.to_string())?;
    {
        let mut core = lock(&state)?;
        let settings = core.settings.clone();
        let item = NewItem {
            application: None,
            contents,
            from_windy: true,
        };
        core.history.add(item, now_ms(), &settings);
        persist(&mut core);
        core.last_token = windy_platform::change_token();
    }
    if let Some(popup) = app.get_webview_window("popup") {
        let _ = popup.hide();
    }
    let _ = app.emit("history-changed", ());
    if paste {
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_millis(150));
            if let Err(e) = windy_platform::send_paste() {
                eprintln!("paste failed: {e}");
            }
        });
    }
    Ok(())
}

fn persist(core: &mut Core) {
    if let Err(e) = core.storage.apply(&mut core.history) {
        eprintln!("saving history failed: {e}");
    }
}

#[tauri::command]
fn toggle_pin(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, Shared>,
    id: i64,
) -> Result<(), String> {
    guard(&window)?;
    {
        let mut core = lock(&state)?;
        if !core.history.unpin(id) {
            let rand = now_ms() as u64;
            core.history.pin(id, rand, &['p']);
        }
        persist(&mut core);
    }
    let _ = app.emit("history-changed", ());
    Ok(())
}

#[tauri::command]
fn delete_item(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, Shared>,
    id: i64,
) -> Result<(), String> {
    guard(&window)?;
    {
        let mut core = lock(&state)?;
        core.history.delete(id);
        persist(&mut core);
    }
    let _ = app.emit("history-changed", ());
    Ok(())
}

#[tauri::command]
fn clear_history(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, Shared>,
    all: bool,
) -> Result<(), String> {
    guard(&window)?;
    let clear_system = {
        let mut core = lock(&state)?;
        if all {
            core.history.clear_all();
        } else {
            core.history.clear();
        }
        persist(&mut core);
        core.settings.clear_system_clipboard
    };
    if clear_system {
        let _ = windy_platform::clear_clipboard();
        if let Ok(mut core) = lock(&state) {
            core.last_token = windy_platform::change_token();
        }
    }
    if let Some(popup) = app.get_webview_window("popup") {
        let _ = popup.hide();
    }
    let _ = app.emit("history-changed", ());
    Ok(())
}

#[tauri::command]
fn get_settings(window: WebviewWindow, state: State<'_, Shared>) -> Result<Settings, String> {
    guard(&window)?;
    Ok(lock(&state)?.settings.clone())
}

#[tauri::command]
fn set_settings(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, Shared>,
    mut settings: Settings,
) -> Result<(), String> {
    guard(&window)?;
    settings.clamp();
    let hotkey_changed = {
        let mut core = lock(&state)?;
        let changed = core.settings.popup_hotkey != settings.popup_hotkey;
        core.monitor.ignore_events = settings.ignore_events;
        core.settings = settings;
        let s = core.settings.clone();
        core.history.enforce_limit(&s);
        persist(&mut core);
        match serde_json::to_string_pretty(&core.settings) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&core.settings_path, json) {
                    eprintln!("saving settings failed: {e}");
                }
            }
            Err(e) => eprintln!("encoding settings failed: {e}"),
        }
        changed.then(|| core.settings.popup_hotkey.clone())
    };
    if let Some(hk) = hotkey_changed {
        register_hotkey(&app, &hk)?;
    }
    let _ = app.emit("history-changed", ());
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Info {
    version: &'static str,
    os: &'static str,
}

#[tauri::command]
fn get_info(window: WebviewWindow) -> Result<Info, String> {
    guard(&window)?;
    Ok(Info {
        version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
    })
}

#[tauri::command]
fn hide_popup(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    guard(&window)?;
    if let Some(popup) = app.get_webview_window("popup") {
        let _ = popup.hide();
    }
    Ok(())
}

#[tauri::command]
fn open_settings_window(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    guard(&window)?;
    open_settings(&app);
    Ok(())
}

#[tauri::command]
fn quit_app(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    guard(&window)?;
    app.exit(0);
    Ok(())
}

fn open_settings(app: &AppHandle) {
    if let Some(popup) = app.get_webview_window("popup") {
        let _ = popup.hide();
    }
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let built = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Windy Settings")
        .inner_size(560.0, 680.0)
        .build();
    if let Err(e) = built {
        eprintln!("opening settings failed: {e}");
    }
}

fn position_popup(app: &AppHandle, win: &WebviewWindow, mode: PopupPosition) {
    let Ok(size) = win.outer_size() else { return };
    let (w, h) = (i64::from(size.width), i64::from(size.height));
    let cursor = app.cursor_position().ok();
    let monitor = cursor
        .and_then(|c| app.monitor_from_point(c.x, c.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    let Some(mon) = monitor else { return };
    let (mx, my) = (i64::from(mon.position().x), i64::from(mon.position().y));
    let (mw, mh) = (i64::from(mon.size().width), i64::from(mon.size().height));
    let (x, y) = match (mode, cursor) {
        (PopupPosition::Cursor, Some(c)) => (c.x as i64, c.y as i64),
        _ => (mx + (mw - w) / 2, my + (mh - h) / 2),
    };
    let x = x.clamp(mx, (mx + mw - w).max(mx));
    let y = y.clamp(my, (my + mh - h).max(my));
    let _ = win.set_position(PhysicalPosition::new(x as i32, y as i32));
}

fn toggle_popup(app: &AppHandle) {
    let Some(w) = app.get_webview_window("popup") else {
        return;
    };
    if w.is_visible().unwrap_or(false) {
        let _ = w.hide();
        return;
    }
    let mode = app
        .try_state::<Shared>()
        .and_then(|s| s.lock().ok().map(|c| c.settings.popup_position))
        .unwrap_or(PopupPosition::Cursor);
    position_popup(app, &w, mode);
    let _ = w.show();
    let _ = w.set_focus();
    let _ = w.emit("popup-shown", ());
}

fn register_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    let gs = app.global_shortcut();
    gs.unregister_all().map_err(|e| e.to_string())?;
    gs.register(hotkey)
        .map_err(|e| format!("cannot register {hotkey}: {e}"))
}

fn start_watcher(app: AppHandle) {
    std::thread::spawn(move || loop {
        let interval = {
            let state = app.state::<Shared>();
            let Ok(core) = state.lock() else { return };
            u64::from(core.settings.clipboard_check_interval_ms)
        };
        std::thread::sleep(Duration::from_millis(interval));

        let token = windy_platform::change_token();
        {
            let state = app.state::<Shared>();
            let Ok(core) = state.lock() else { return };
            if token == core.last_token {
                continue;
            }
        }
        let event = windy_platform::read_event();
        if event.items.is_empty() {
            continue;
        }
        let state = app.state::<Shared>();
        let Ok(mut core) = state.lock() else { return };
        core.last_token = token;
        let settings = core.settings.clone();
        if let Outcome::Store(item) = core.monitor.process(&event, &settings) {
            core.history.add(item, now_ms(), &settings);
            persist(&mut core);
            drop(core);
            let _ = app.emit("history-changed", ());
        }
    });
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Windy", true, None::<&str>)?;
    let pause = CheckMenuItem::with_id(app, "pause", "Pause recording", true, false, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &pause, &settings, &sep, &quit])?;
    let pause_item = pause.clone();
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Windy")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open" => toggle_popup(app),
            "settings" => open_settings(app),
            "quit" => app.exit(0),
            "pause" => {
                let state = app.state::<Shared>();
                let guard = state.lock();
                if let Ok(mut core) = guard {
                    core.monitor.ignore_events = !core.monitor.ignore_events;
                    let _ = pause_item.set_checked(core.monitor.ignore_events);
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_popup(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn build_popup(app: &AppHandle) -> tauri::Result<()> {
    let popup = WebviewWindowBuilder::new(app, "popup", WebviewUrl::App("popup.html".into()))
        .title("Windy")
        .inner_size(450.0, 560.0)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .shadow(true)
        .build()?;
    let handle = popup.clone();
    popup.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            let _ = handle.hide();
        }
    });
    Ok(())
}

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            toggle_popup(app)
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_popup(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let dir = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let settings_path = dir.join("settings.json");
            let settings = std::fs::read_to_string(&settings_path)
                .map(|s| Settings::from_json(&s))
                .unwrap_or_default();
            let storage = Storage::open(&dir.join("history.sqlite"))?;
            let mut history = History::from_items(storage.load()?);
            history.enforce_limit(&settings);
            let monitor = Monitor::new(&settings);
            let hotkey = settings.popup_hotkey.clone();
            let mut core = Core {
                history,
                storage,
                settings,
                monitor,
                last_token: windy_platform::change_token(),
                settings_path,
            };
            persist(&mut core);
            app.manage(Mutex::new(core));

            build_popup(app.handle())?;
            build_tray(app.handle())?;
            if let Err(e) = register_hotkey(app.handle(), &hotkey) {
                eprintln!("{e}");
            }
            start_watcher(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_items,
            get_image,
            select_item,
            toggle_pin,
            delete_item,
            clear_history,
            get_settings,
            set_settings,
            get_info,
            hide_popup,
            open_settings_window,
            quit_app
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Windy");

    app.run(|app, event| match event {
        RunEvent::ExitRequested { api, code, .. } if code.is_none() => api.prevent_exit(),
        RunEvent::Exit => {
            if let Some(state) = app.try_state::<Shared>() {
                let guard = state.lock();
                if let Ok(mut core) = guard {
                    if core.settings.clear_on_quit {
                        core.history.clear();
                        persist(&mut core);
                    }
                }
            }
        }
        _ => {}
    });
}
