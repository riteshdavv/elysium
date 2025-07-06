// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logger;
mod app_focus;

fn main() {
    logger::start_logger();
    app_focus::start_app_focus_logger();

    // keep main thread alive
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

