pub mod scanner;
pub mod portmaster;
pub mod projects;
pub mod resolver;
pub mod config_editor;
pub mod monitor;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub port: u16,
    pub protocol: String,
    pub bind_address: String,
    pub pid: Option<u32>,
    pub process_name: String,
    pub service_name: Option<String>,
    pub managed_by: Option<String>,
    pub project: Option<String>,
    pub source: PortSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortSource {
    Live,
    Portmaster,
    ProjectConfig,
    Docker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub port: u16,
    pub bind_a: String,
    pub bind_b: String,
    pub process_a: String,
    pub process_b: String,
    pub pid_a: Option<u32>,
    pub pid_b: Option<u32>,
    pub suggestion: Option<u16>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortmasterEntry {
    pub port: u16,
    pub service: String,
    pub protocol: String,
    pub bind: String,
    pub managed_by: String,
    pub notes: String,
    pub source_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPort {
    pub port: u16,
    pub project_path: String,
    pub project_name: String,
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub config_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPreview {
    pub file_path: String,
    pub line_number: usize,
    pub old_line: String,
    pub new_line: String,
    pub old_port: u16,
    pub new_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortCheckResult {
    pub port: u16,
    pub in_use: bool,
    pub process: Option<String>,
    pub pid: Option<u32>,
    pub portmaster_entry: Option<PortmasterEntry>,
    pub suggestion: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub ports: Vec<PortInfo>,
    pub conflicts: Vec<Conflict>,
    pub portmaster_entries: Vec<PortmasterEntry>,
    pub project_ports: Vec<ProjectPort>,
    pub last_scan: String,
    pub status: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            ports: Vec::new(),
            conflicts: Vec::new(),
            portmaster_entries: Vec::new(),
            project_ports: Vec::new(),
            last_scan: "Never".to_string(),
            status: "ok".to_string(),
        }
    }
}

pub struct SharedState {
    pub state: Arc<Mutex<AppState>>,
}

#[tauri::command]
async fn scan_ports(
    state: tauri::State<'_, SharedState>,
) -> Result<AppState, String> {
    let mut app_state = state.state.lock().await;

    let live_ports = scanner::scan_live_ports().map_err(|e| e.to_string())?;
    let pm_entries = portmaster::load_all_portmasters().map_err(|e| e.to_string())?;
    let project_ports = projects::scan_project_ports().map_err(|e| e.to_string())?;
    let conflicts = resolver::detect_conflicts(&live_ports, &pm_entries, &project_ports);
    let merged = resolver::merge_ports(&live_ports, &pm_entries, &project_ports);

    app_state.ports = merged;
    app_state.conflicts = conflicts;
    app_state.portmaster_entries = pm_entries;
    app_state.project_ports = project_ports;
    app_state.last_scan = chrono::Local::now().format("%H:%M:%S").to_string();
    app_state.status = if app_state.conflicts.is_empty() {
        "ok".to_string()
    } else {
        "warning".to_string()
    };

    Ok(app_state.clone())
}

#[tauri::command]
async fn check_port(port: u16) -> Result<PortCheckResult, String> {
    scanner::check_single_port(port).map_err(|e| e.to_string())
}

#[tauri::command]
async fn suggest_port(range_start: u16, range_end: u16) -> Result<u16, String> {
    resolver::suggest_port(range_start, range_end).map_err(|e| e.to_string())
}

#[tauri::command]
async fn preview_fix(project_port_id: String) -> Result<Vec<FixPreview>, String> {
    let parts: Vec<&str> = project_port_id.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("Invalid project_port_id format".to_string());
    }
    let file_path = parts[0].to_string();
    let line_num: usize = parts[1].parse().map_err(|_| "Invalid line number")?;
    config_editor::preview_fix(&file_path, line_num).map_err(|e| e.to_string())
}

#[tauri::command]
async fn apply_fix(
    file_path: String,
    line_number: usize,
    new_port: u16,
) -> Result<String, String> {
    config_editor::apply_fix(&file_path, line_number, new_port).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_portmaster_files() -> Result<Vec<String>, String> {
    portmaster::discover_portmaster_files().map_err(|e| e.to_string())
}

#[tauri::command]
async fn kill_process(pid: u32) -> Result<String, String> {
    #[cfg(unix)]
    {
        use std::process::Command;
        let output = Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()
            .map_err(|e| format!("Failed to kill PID {}: {}", pid, e))?;
        if output.status.success() {
            Ok(format!("Killed process {}", pid))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to kill PID {}: {}", pid, stderr))
        }
    }
    #[cfg(not(unix))]
    {
        Err("Process killing is only supported on Unix systems".to_string())
    }
}

#[tauri::command]
async fn open_config_file(file_path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&file_path)
        .spawn()
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&file_path)
        .spawn()
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", &file_path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn run() {
    let shared_state = SharedState {
        state: Arc::new(Mutex::new(AppState::default())),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(shared_state)
        .invoke_handler(tauri::generate_handler![
            scan_ports,
            check_port,
            suggest_port,
            preview_fix,
            apply_fix,
            get_portmaster_files,
            kill_process,
            open_config_file,
        ])
        .setup(|app| {
            let state = app.state::<SharedState>();
            monitor::start_monitoring(state.state.clone(), app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Lighthouse");
}
