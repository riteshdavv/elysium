use std::{fs::{OpenOptions}, io::Write, thread, time::Duration};
use chrono::Utc;
use device_query::{DeviceQuery, DeviceState};

pub fn start_logger() {
    thread::spawn(move || {
        let device_state = DeviceState::new();
        let mut last_mouse = device_state.get_mouse();
        let mut last_keys = device_state.get_keys();

        loop {
            let mouse = device_state.get_mouse();
            let keys = device_state.get_keys();

            let now = Utc::now().to_rfc3339();
            let log_path = "logs/user-activity.json";

            let mut log = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .expect("Failed to open log file");

            if mouse != last_mouse {
                let _ = writeln!(
                    log,
                    r#"{{"type": "mousemove", "timestamp": "{}"}}"#,
                    now
                );
                last_mouse = mouse;
            }

            if keys != last_keys && !keys.is_empty() {
                let _ = writeln!(
                    log,
                    r#"{{"type": "keydown", "timestamp": "{}", "keys": {:?}}}"#,
                    now, keys
                );
                last_keys = keys;
            }

            thread::sleep(Duration::from_millis(300));
        }
    });
}
