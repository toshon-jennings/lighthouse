use crate::ProjectPort;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Scan project directories for port configurations.
pub fn scan_project_ports() -> Result<Vec<ProjectPort>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let scan_dirs = vec![
        home.join("projects"),
        home.join("code"),
        home.join("dev"),
        home.join("workspace"),
        home.join("src"),
    ];

    for scan_dir in &scan_dirs {
        if !scan_dir.exists() {
            continue;
        }
        for entry in WalkDir::new(scan_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                match name {
                    ".env" | ".env.local" | ".env.development" | ".env.production" => {
                        results.extend(scan_env_file(path));
                    }
                    "docker-compose.yml" | "docker-compose.yaml" => {
                        results.extend(scan_docker_compose(path));
                    }
                    "package.json" => {
                        results.extend(scan_package_json(path));
                    }
                    "vite.config.js" | "vite.config.ts" => {
                        results.extend(scan_vite_config(path));
                    }
                    "next.config.js" | "next.config.ts" | "next.config.mjs" => {
                        results.extend(scan_next_config(path));
                    }
                    "Cargo.toml" => {
                        results.extend(scan_cargo_toml(path));
                    }
                    _ => {}
                }
            }
        }
    }

    results.sort_by(|a, b| a.port.cmp(&b.port).then(a.file_path.cmp(&b.file_path)));
    results.dedup_by(|a, b| {
        a.port == b.port && a.file_path == b.file_path && a.line_number == b.line_number
    });
    Ok(results)
}

fn scan_env_file(path: &Path) -> Vec<ProjectPort> {
    let mut results = Vec::new();
    let project_dir = path.parent().unwrap_or(path);
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let port_re = Regex::new(r"(?i)^([A-Z_]*PORT[A-Z_]*)\s*=\s*(\d{2,5})").unwrap();

    if let Ok(content) = fs::read_to_string(path) {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(cap) = port_re.captures(line.trim()) {
                if let Ok(port) = cap[2].parse::<u16>() {
                    results.push(ProjectPort {
                        port,
                        project_path: project_dir.to_string_lossy().to_string(),
                        project_name: project_name.clone(),
                        file_path: path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        line_content: line.trim().to_string(),
                        config_key: cap[1].to_string(),
                    });
                }
            }
        }
    }
    results
}

fn scan_docker_compose(path: &Path) -> Vec<ProjectPort> {
    let mut results = Vec::new();
    let project_dir = path.parent().unwrap_or(path);
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Match port mappings like 3000:3000 or "8080:80"
    let port_re = Regex::new(r#"(\d{2,5}):\d{2,5}"#).unwrap();

    if let Ok(content) = fs::read_to_string(path) {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(cap) = port_re.captures(line) {
                if let Ok(port) = cap[1].parse::<u16>() {
                    results.push(ProjectPort {
                        port,
                        project_path: project_dir.to_string_lossy().to_string(),
                        project_name: project_name.clone(),
                        file_path: path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        line_content: line.trim().to_string(),
                        config_key: "ports".to_string(),
                    });
                }
            }
        }
    }
    results
}

fn scan_package_json(path: &Path) -> Vec<ProjectPort> {
    let mut results = Vec::new();
    let project_dir = path.parent().unwrap_or(path);
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let port_re = Regex::new(r#"(?i)(?:"PORT"\s*=|port\s*[=:\s]+|-p\s+|--port\s+)(\d{2,5})"#).unwrap();

    if let Ok(content) = fs::read_to_string(path) {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(cap) = port_re.captures(line) {
                if let Ok(port) = cap[1].parse::<u16>() {
                    results.push(ProjectPort {
                        port,
                        project_path: project_dir.to_string_lossy().to_string(),
                        project_name: project_name.clone(),
                        file_path: path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        line_content: line.trim().to_string(),
                        config_key: "scripts".to_string(),
                    });
                }
            }
        }
    }
    results
}

fn scan_vite_config(path: &Path) -> Vec<ProjectPort> {
    let mut results = Vec::new();
    let project_dir = path.parent().unwrap_or(path);
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let port_re = Regex::new(r"(?i)port\s*:\s*(\d{2,5})").unwrap();

    if let Ok(content) = fs::read_to_string(path) {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(cap) = port_re.captures(line) {
                if let Ok(port) = cap[1].parse::<u16>() {
                    results.push(ProjectPort {
                        port,
                        project_path: project_dir.to_string_lossy().to_string(),
                        project_name: project_name.clone(),
                        file_path: path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        line_content: line.trim().to_string(),
                        config_key: "server.port".to_string(),
                    });
                }
            }
        }
    }
    results
}

fn scan_next_config(path: &Path) -> Vec<ProjectPort> {
    let mut results = Vec::new();
    let project_dir = path.parent().unwrap_or(path);
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Match PORT=3000, PORT: 3000, PORT = '3000', etc.
    let port_re = Regex::new(r#"(?i)(?:PORT|port)\s*[:=]\s*['"]?(\d{2,5})['"]?"#).unwrap();

    if let Ok(content) = fs::read_to_string(path) {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(cap) = port_re.captures(line) {
                if let Ok(port) = cap[1].parse::<u16>() {
                    results.push(ProjectPort {
                        port,
                        project_path: project_dir.to_string_lossy().to_string(),
                        project_name: project_name.clone(),
                        file_path: path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        line_content: line.trim().to_string(),
                        config_key: "env.PORT".to_string(),
                    });
                }
            }
        }
    }
    results
}

fn scan_cargo_toml(path: &Path) -> Vec<ProjectPort> {
    let mut results = Vec::new();
    let project_dir = path.parent().unwrap_or(path);
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let port_re = Regex::new(r#"(?i)(?:port|PORT)\s*=\s*['"]?(\d{2,5})['"]?"#).unwrap();

    if let Ok(content) = fs::read_to_string(path) {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(cap) = port_re.captures(line) {
                if let Ok(port) = cap[1].parse::<u16>() {
                    results.push(ProjectPort {
                        port,
                        project_path: project_dir.to_string_lossy().to_string(),
                        project_name: project_name.clone(),
                        file_path: path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        line_content: line.trim().to_string(),
                        config_key: "port".to_string(),
                    });
                }
            }
        }
    }
    results
}
