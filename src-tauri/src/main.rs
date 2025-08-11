#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::menu::{CheckMenuItem, MenuBuilder, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri::WebviewWindowBuilder;
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tauri_plugin_dialog::DialogExt;

mod css_template;

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
enum EventId {
    Lock = 0,
    Settings = 1,
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u32)
    }
}

impl From<u32> for EventId {
    fn from(val: u32) -> Self {
        match val {
            0 => EventId::Lock,
            1 => EventId::Settings,
            _ => panic!("Unknown event ID: {}", val),
        }
    }
}

#[tauri::command]
fn open_chat(webview_window: tauri::WebviewWindow, stream_id: &str) -> Result<(), tauri::Error> {
    webview_window.eval( 
        format!(
            "window.location.replace('https://www.youtube.com/live_chat?is_popout=1&hl=en&persist_hl=1&v={}')",
            stream_id
        )
        .as_str(),
    )
}

#[tauri::command]
fn get_app_settings(app_handle: tauri::AppHandle) -> AppState {
    let state = app_handle.state::<Mutex<AppState>>();
    let state = state.lock().unwrap();
    state.clone()
}

#[tauri::command]
fn toggle_lock_window(app_handle: tauri::AppHandle, is_locked: bool) -> Result<(), String> {
    let window = app_handle.get_webview_window("main").ok_or("Window not found")?;
    let state = app_handle.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();
    
    if is_locked {
        set_window_locked(&window).map_err(|e| e.to_string())?;
    } else {
        set_window_unlocked(&window).map_err(|e| e.to_string())?;
    }
    
    state.is_locked = is_locked;
    save_app_state(&app_handle, &state).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn toggle_full_chat(app_handle: tauri::AppHandle, is_full_chat: bool) -> Result<(), String> {
    let window = app_handle.get_webview_window("main").ok_or("Window not found")?;
    let state = app_handle.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();
    
    if is_full_chat {
        set_all_chat(&window).map_err(|e| e.to_string())?;
    } else {
        set_top_chat(&window).map_err(|e| e.to_string())?; 
    }
    
    state.is_full_chat = is_full_chat;
    save_app_state(&app_handle, &state).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn adjust_font_size(app_handle: tauri::AppHandle, increase: bool) -> Result<f32, String> {
    let window = app_handle.get_webview_window("main").ok_or("Window not found")?;
    let state = app_handle.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();
    
    if increase {
        state.font_size += 0.05;
    } else {
        state.font_size -= 0.05;
        if state.font_size < 0.1 {
            state.font_size = 0.1;
        }
    }
    
    set_font_size(&window, state.font_size).map_err(|e| e.to_string())?;
    save_app_state(&app_handle, &state).map_err(|e| e.to_string())?;
    
    Ok(state.font_size)
}

#[tauri::command]
fn set_font_size_value(app_handle: tauri::AppHandle, font_size: f32) -> Result<f32, String> {
    let window = app_handle.get_webview_window("main").ok_or("Window not found")?;
    let state = app_handle.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();
    
    let new_font_size = font_size.clamp(0.1, 5.0);
    state.font_size = new_font_size;
    
    set_font_size(&window, state.font_size).map_err(|e| e.to_string())?;
    save_app_state(&app_handle, &state).map_err(|e| e.to_string())?;
    
    Ok(state.font_size)
}

#[tauri::command]
fn open_settings_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    let settings_window = app_handle.get_webview_window("settings");
    
    if let Some(window) = settings_window {
        window.set_focus().map_err(|e| e.to_string())?;
    } else {
        create_settings_window(app_handle).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
fn set_message_font_name(app_handle: tauri::AppHandle, font_name: Option<String>) -> Result<(), String> {
    let state = app_handle.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();

    // Basic validation/sanitization could be added here if needed
    state.message_font = font_name.clone();

    save_app_state(&app_handle, &state).map_err(|e| e.to_string())?;
    refresh_page(&app_handle).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn set_author_font_name(app_handle: tauri::AppHandle, font_name: Option<String>) -> Result<(), String> {
    let state = app_handle.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();

    // Basic validation/sanitization could be added here if needed
    state.author_font = font_name.clone();

    save_app_state(&app_handle, &state).map_err(|e| e.to_string())?;
    refresh_page(&app_handle).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn set_background_color(app_handle: tauri::AppHandle, color: Option<String>) -> Result<(), String> {
    let state = app_handle.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();

    state.background_color = color;

    save_app_state(&app_handle, &state).map_err(|e| e.to_string())?;
    refresh_page(&app_handle).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn set_message_color(app_handle: tauri::AppHandle, color: Option<String>) -> Result<(), String> {
    let state = app_handle.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();

    state.message_color = color;

    save_app_state(&app_handle, &state).map_err(|e| e.to_string())?;
    refresh_page(&app_handle).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn set_author_color(app_handle: tauri::AppHandle, color: Option<String>) -> Result<(), String> {
    let state = app_handle.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();

    state.author_color = color;

    save_app_state(&app_handle, &state).map_err(|e| e.to_string())?;
    refresh_page(&app_handle).map_err(|e| e.to_string())?;

    Ok(())
}

fn refresh_page(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let window = app_handle.get_webview_window("main").ok_or("Window not found")?;
    window.eval("location.reload();").map_err(|e| e.to_string())?;
    Ok(())
}

 
#[derive(Serialize, Deserialize, Clone)]
struct AppState {
    #[serde(skip)]
    is_locked: bool,
    is_full_chat: bool,
    font_size: f32,
    message_font: Option<String>,
    author_font: Option<String>,
    background_color: Option<String>,
    message_color: Option<String>,
    author_color: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_locked: false,
            is_full_chat: true,
            font_size: 1.0,
            message_font: None, // Default to Imprima (handled in CSS injection)
            author_font: None, // Default author font
            background_color: None, // Default to transparent (rgba(0,0,0,0))
            message_color: None, // Default to white (handled in CSS injection)
            author_color: None, // Default to #cccccc (handled in CSS injection)
        }
    }
}

fn create_settings_window(app_handle: tauri::AppHandle) -> Result<(), tauri::Error> {
    WebviewWindowBuilder::new(
        &app_handle,
        "settings", // window label
        tauri::WebviewUrl::App("settings.html".into())
    )
    .title("YouTube Chat Overlay Settings")
    .inner_size(500.0, 800.0)
    .resizable(true)
    .build()?;
    
    Ok(())
}

fn get_app_state_path(app: &tauri::AppHandle) -> PathBuf {  
    let mut path = app.path().app_config_dir().unwrap();
    path.push("app_state.json");
    path
}

fn save_app_state(app: &tauri::AppHandle, state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let state_path = get_app_state_path(app);
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    fs::write(state_path, json)?;
    Ok(())
}

fn load_app_state(app: &tauri::AppHandle) -> AppState {
    let state_path = get_app_state_path(app);
    match fs::read_to_string(state_path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => AppState::default(),
    }
}

fn set_top_chat(window: &tauri::WebviewWindow) -> Result<(), tauri::Error> {
    window.eval(r#"
  [...document.querySelectorAll('#chat-messages #menu a')].filter(x => x.innerText.toLowerCase().includes('top chat'))[0].click()
  "#)
}

fn set_all_chat(window: &tauri::WebviewWindow) -> Result<(), tauri::Error> {
    window.eval(r#"
  [...document.querySelectorAll('#chat-messages #menu a')].filter(x => x.innerText.toLowerCase().includes('live chat'))[0].click()
  "#)
}

fn set_font_size(window: &tauri::WebviewWindow, font_size: f32) -> Result<(), tauri::Error> {
    window.eval(format!(r#"
    document.querySelector('#contents').style.zoom = {}
    document.querySelector('#item-scroller').scrollBy(0, document.querySelector('#item-scroller').scrollHeight)
    "#, font_size).as_str())
}


fn set_window_unlocked(window: &tauri::WebviewWindow) -> Result<(), tauri::Error> {
    window.set_ignore_cursor_events(false)?;
    window.set_decorations(true)?;
    window.set_always_on_top(false)?;
    window.set_shadow(true)?;
    Ok(())
}

fn set_window_locked(window: &tauri::WebviewWindow) -> Result<(), tauri::Error> {
    window.set_ignore_cursor_events(true)?;
    window.set_decorations(false)?;
    window.set_always_on_top(true)?;
    window.set_shadow(false)?;
    Ok(())
}

fn main() {
    if let Err(e) = css_template::init_templates() {
        eprintln!("Failed to initialize templates: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            
            let initial_state = load_app_state(app.handle());
            app.manage(Mutex::new(initial_state));
            
            if let Err(e) = set_window_unlocked(&window) {
                eprintln!("Failed to initialize window in unlocked state: {}", e);
            }

            app.remove_tray_by_id("main");
            let handle = app.handle();
            let state = app.state::<Mutex<AppState>>();
            let state = state.lock().unwrap();

            let menu = MenuBuilder::new(handle)
            .items(&[
              &CheckMenuItem::with_id(handle, EventId::Lock.to_string(), "Lock window", true, state.is_locked, None::<&str>)?,
              &MenuItem::with_id(handle, EventId::Settings.to_string(), "Open Settings", true, None::<&str>)?,
            ])
            .separator()
            .quit()
            .build()?;

            app.on_menu_event(move |app, event| {
                let state = app.state::<Mutex<AppState>>();
                let mut state = state.lock().unwrap();
                let mut state_changed = false;

                let event_id = EventId::from(event.id().0.parse::<u32>().unwrap());
                
                match event_id {
                    EventId::Settings => {
                        if let Err(e) = open_settings_window(app.app_handle().clone()) {
                            eprintln!("Failed to open settings window: {}", e);
                        }
                    },
                    EventId::Lock => {
                    if state.is_locked {
                        if let Err(e) = set_window_unlocked(&window) {
                            eprintln!("Failed to unlock window: {}", e);
                        }
                    } else if let Err(e) = set_window_locked(&window) {
                        eprintln!("Failed to lock window: {}", e);
                    }
                    state.is_locked = !state.is_locked;
                        state_changed = true;
                    }
                }

                if state_changed {
                    if let Err(e) = save_app_state(app, &state) {
                        eprintln!("Failed to save app state: {}", e);
                    }
                }
            });

            TrayIconBuilder::new()
                .icon(handle.default_window_icon().ok_or(tauri::Error::WindowNotFound)?.clone())
                .menu(&menu)
                .build(app)?;
            Ok(())
        })
        .on_page_load(|window, _| {
          let webview_window = window.get_webview_window("main").unwrap();
          let state_mutex = window.app_handle().state::<Mutex<AppState>>();
          let state = state_mutex.lock().unwrap();

            let message_font_to_use = state.message_font.clone().unwrap_or_else(|| "Imprima".to_string());
            let author_font_to_use = state.author_font.clone().unwrap_or_else(|| "Changa One".to_string());
            let background_color = state.background_color.clone().unwrap_or_else(|| "rgba(0,0,0,0)".to_string());
            let message_color = state.message_color.clone().unwrap_or_else(|| "#ffffff".to_string());
            let author_color = state.author_color.clone().unwrap_or_else(|| "#cccccc".to_string());

            let message_font_css_family = format!("\"{}\"", message_font_to_use);
            let author_font_css_family = format!("\"{}\"", author_font_to_use);

            let message_font_import_url = format!("https://fonts.googleapis.com/css2?family={}", message_font_to_use.replace(" ", "+"));
            let author_font_import_url = format!("https://fonts.googleapis.com/css2?family={}", author_font_to_use.replace(" ", "+"));
            if window.url().unwrap().to_string().contains("youtube.com/live_chat") {
                println!("Injecting CSS with message font: {}, author font: {}, background color: {}, message color: {}, author color: {}",
                    &message_font_to_use, &author_font_to_use, &background_color, &message_color, &author_color);
                
                let css_injection_script = match css_template::generate_css_injection_script(
                    &author_font_import_url,
                    &message_font_import_url,
                    &background_color,
                    &message_font_css_family,
                    &author_color,
                    &author_font_css_family,
                    &message_color,
                ) {
                    Ok(script) => script,
                    Err(e) => {
                        eprintln!("Failed to generate CSS injection script: {}", e);
                        String::new()
                    }
                };
                
                if let Err(e) = webview_window.eval(&css_injection_script) {
                    // show messagebox to user
                    let handle = window.app_handle();
                    handle.dialog().message(e.to_string()).title("Failed to inject CSS").blocking_show();
                    eprintln!("Failed to inject CSS: {}", e);
                }

            if state.is_full_chat {
                let _ = set_all_chat(&webview_window);
            } else {
                let _ = set_top_chat(&webview_window);
            }
            let _ = set_font_size(&webview_window, state.font_size);
          } 

        })
        .invoke_handler(tauri::generate_handler![
            open_chat,
            get_app_settings,
            set_message_font_name,
            set_author_font_name,
            toggle_lock_window,
            toggle_full_chat,
            adjust_font_size,
            set_font_size_value,
            open_settings_window,
            set_background_color,
            set_message_color,
            set_author_color
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
