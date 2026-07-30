mod commands;
mod audio;
mod detector;
mod whisper;
mod translation;

use commands::SessionManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(SessionManager::new())
        .setup(|app| {
            let tray = tauri::tray::TrayIconBuilder::new()
                .menu(
                    &tauri::menu::MenuBuilder::new(app)
                        .item(&tauri::menu::MenuItemBuilder::with_id("toggle", "Toggle Sidebar").build(app)?)
                        .item(&tauri::menu::MenuItemBuilder::with_id("quit", "Quit").build(app)?)
                        .build()?,
                )
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle" => {
                        if let Some(window) = app.get_webview_window("main") {
                            if let Ok(visible) = window.is_visible() {
                                if visible {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                }
                            }
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_voxscribe_session,
            commands::stop_voxscribe_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VoxScribe");
}
