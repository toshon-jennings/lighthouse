use crate::PortmasterEntry;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Discover all PORTMASTER.md files on the system.
/// Checks:
/// 1. ~/.config/agent-rules/PORTMASTER.md (global agent rules)
/// 2. Walks common project directories looking for PORTMASTER.md
pub fn discover_portmaster_files() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    // 1. Global agent rules location
    let global = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/agent-rules/PORTMASTER.md");
    if global.exists() {
        files.push(global.to_string_lossy().to_string());
    }

    // 2. Also check ~/.hermes/ and similar agent config dirs
    let hermes_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hermes");
    if hermes_dir.exists() {
        for entry in WalkDir::new(&hermes_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "PORTMASTER.md" {
                files.push(entry.path().to_string_lossy().to_string());
            }
        }
    }

    // 3. Walk home directory for project-level PORTMASTER.md files
    // Limit depth to avoid scanning the entire filesystem
    if let Some(home) = dirs::home_dir() {
        let scan_dirs = vec![
            home.join("projects"),
            home.join("code"),
            home.join("dev"),
            home.join("workspace"),
            home.join("src"),
            home.clone(), // home itself (for ~/PORTMASTER.md)
        ];

        for scan_dir in scan_dirs {
            if !scan_dir.exists() { continue; }
            for entry in WalkDir::new(&scan_dir)
                .max_depth(4)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_name() == "PORTMASTER.md" {
                    let path = entry.path().to_string_lossy().to_string();
                    if !files.contains(&path) {
                        files.push(path);
                    }
                }
            }
        }
    }

    Ok(files)
}

/// Load and parse all discovered PORTMASTER.md files.
/// Later files (project-level) override earlier ones (global) for the same port.
pub fn load_all_portmasters() -> Result<Vec<PortmasterEntry>, Box<dyn std::error::Error>> {
    let files = discover_portmaster_files()?;
    let mut all_entries = Vec::new();

    for file_path in &files {
        match parse_portmaster_file(file_path) {
            Ok(entries) => all_entries.extend(entries),
            Err(e) => eprintln!("Warning: failed to parse {}: {}", file_path, e),
        }
    }

    // Deduplicate: project-level (later) entries override global ones
    // We keep the last entry for each port
    let mut seen = std::collections::HashMap::new();
    for (i, entry) in all_entries.iter().enumerate() {
        seen.insert(entry.port, i);
    }

    let mut deduped: Vec<PortmasterEntry> = Vec::new();
    for (_, idx) in seen {
        deduped.push(all_entries[idx].clone());
    }
    deduped.sort_by_key(|e| e.port);

    Ok(deduped)
}

/// Parse a single PORTMASTER.md file.
/// Expects a markdown table with columns: Port | Service | Protocol | Bind | Managed By | Notes
pub fn parse_portmaster_file(path: &str) -> Result<Vec<PortmasterEntry>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut entries = Vec::new();

    // Find the table - look for lines with | characters
    let table_re = Regex::new(r"\|\s*(\d+)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|")?;

    for cap in table_re.captures_iter(&content) {
        if let Ok(port) = cap[1].trim().parse::<u16>() {
            entries.push(PortmasterEntry {
                port,
                service: cap[2].trim().to_string(),
                protocol: cap[3].trim().to_string(),
                bind: cap[4].trim().to_string(),
                managed_by: cap[5].trim().to_string(),
                notes: cap[6].trim().to_string(),
                source_file: path.to_string(),
            });
        }
    }

    // Also try simpler format: just "port: service" lines
    if entries.is_empty() {
        let simple_re = Regex::new(r"^(\d{2,5})\s*[:=]\s*(.+)$")?;
        for line in content.lines() {
            if let Some(cap) = simple_re.captures(line.trim()) {
                if let Ok(port) = cap[1].parse::<u16>() {
                    entries.push(PortmasterEntry {
                        port,
                        service: cap[2].trim().to_string(),
                        protocol: "TCP".to_string(),
                        bind: "*".to_string(),
                        managed_by: "unknown".to_string(),
                        notes: String::new(),
                        source_file: path.to_string(),
                    });
                }
            }
        }
    }

    Ok(entries)
}

/// Parse port allocation ranges from PORTMASTER.md content.
/// Looks for lines like "3000 - 3999 | App servers"
pub fn parse_allocation_ranges(content: &str) -> Vec<(u16, u16, String)> {
    let mut ranges = Vec::new();
    let re = Regex::new(r"(\d{2,5})\s*-\s*(\d{2,5})\s*\|\s*(.+)").unwrap();

    for cap in re.captures_iter(content) {
        if let (Ok(start), Ok(end)) = (cap[1].parse::<u16>(), cap[2].parse::<u16>()) {
            ranges.push((start, end, cap[3].trim().to_string()));
        }
    }

    ranges
}
