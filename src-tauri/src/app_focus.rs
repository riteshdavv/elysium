use chrono::{DateTime, Local};
use serde::Serialize;
use std::{fs::OpenOptions, io::Write, thread, time::Duration};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};
use windows::Win32::Foundation::{HWND};
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};

#[derive(Serialize)]
struct AppFocusLog {
    timestamp: DateTime<Local>,
    application: String,
    window_title: String,
    duration_seconds: u64,
    category: String,
    intent: String,
    url: Option<String>,
}

fn get_active_window_info() -> Option<(String, String)> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0 == 0 {
            return None;
        }

        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title);
        let window_title = String::from_utf16_lossy(&title[..len as usize]);

        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32ProcessID == pid {
                    let name = String::from_utf16_lossy(&entry.szExeFile);
                    return Some((name.trim_matches(char::from(0)).to_string(), window_title));
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        None
    }
}

fn categorize_app<'a>(app: &'a str, window_title: &'a str) -> (&'a str, &'a str){
    match app.to_lowercase().as_str() {
        "code.exe" => ("coding", "development"),
        "brave.exe" | "chrome.exe" | "msedge.exe" => {
            if window_title.contains("TUF+") {
                ("browsing", "DSA_learning")
            } else {
                ("browsing", "research")
            }
        }
        "explorer.exe" => ("navigation", "file_explorer"),
        "cmd.exe" | "powershell.exe" | "windowsterminal.exe" => ("terminal", "dev_shell"),
        _ => ("other", "unknown")
    }
}

pub fn start_app_focus_logger() {
    let mut last_window: Option<(String, String)> = None;
    let mut last_timestamp = Local::now();

    thread::spawn(move || loop {
        if let Some((app, title)) = get_active_window_info() {
            if last_window
                .as_ref()
                .map(|(last_app, last_title)| last_app != &app || last_title != &title)
                .unwrap_or(true)
            {
                let now = Local::now();
                let duration = now.signed_duration_since(last_timestamp).num_seconds() as u64;

                if let Some((ref last_app, ref last_title)) = last_window {
                    let (category, intent) = categorize_app(last_app, last_title);

                    let log = AppFocusLog {
                        timestamp: last_timestamp,
                        application: last_app.clone(),
                        window_title: last_title.clone(),
                        duration_seconds: duration,
                        category: category.to_string(),
                        intent: intent.to_string(),
                        url: None,
                    };

                    let log_path = "logs/user-activity.json";
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(log_path)
                        .expect("Failed to open log file");

                    if let Ok(json) = serde_json::to_string(&log) {
                        writeln!(file, "{}", json).ok();
                    }
                }

                last_timestamp = now;
                last_window = Some((app, title));
            }
        }

        thread::sleep(Duration::from_secs(5));
    });
}
