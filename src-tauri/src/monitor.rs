use tauri::Emitter;
use crate::{resolver, scanner};
use std::sync::Arc;
use std::time::Duration;
use tauri::async_runtime::spawn;
use tokio::sync::Mutex;

/// Start the background monitoring loop.
/// Polls for port changes and emits events to the frontend.
pub fn start_monitoring(
    state: Arc<Mutex<crate::AppState>>,
    app_handle: tauri::AppHandle,
) {
    spawn(async move {
        let mut previous_conflicts: Vec<crate::Conflict> = Vec::new();

        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            let live_ports = match scanner::scan_live_ports() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let project_ports = match crate::projects::scan_project_ports() {
                Ok(p) => p,
                Err(_) => Vec::new(),
            };

            let pm_entries = match crate::portmaster::load_all_portmasters() {
                Ok(e) => e,
                Err(_) => Vec::new(),
            };

            let conflicts = resolver::detect_conflicts(&live_ports, &pm_entries, &project_ports);
            let merged = resolver::merge_ports(&live_ports, &pm_entries, &project_ports);

            // Check for new conflicts
            let new_conflicts: Vec<_> = conflicts
                .iter()
                .filter(|c| !previous_conflicts.iter().any(|p| p.port == c.port))
                .collect();

            let mut app_state = state.lock().await;
            app_state.ports = merged;
            app_state.conflicts = conflicts.clone();
            app_state.last_scan = chrono::Local::now().format("%H:%M:%S").to_string();
            app_state.status = if conflicts.is_empty() {
                "ok".to_string()
            } else {
                "warning".to_string()
            };
            drop(app_state);

            // Emit event to frontend
            for conflict in &new_conflicts {
                let _ = app_handle.emit(
                    "conflict-detected",
                    serde_json::json!({
                        "port": conflict.port,
                        "explanation": &conflict.explanation,
                        "suggestion": conflict.suggestion,
                    }),
                );
            }

            previous_conflicts = conflicts;
        }
    });
}
