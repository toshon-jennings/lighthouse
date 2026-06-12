use crate::FixPreview;
use regex::Regex;
use std::fs;

/// Preview what changing a port in a config file would look like.
pub fn preview_fix(file_path: &str, line_number: usize) -> Result<Vec<FixPreview>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;

    let lines: Vec<&str> = content.lines().collect();
    if line_number == 0 || line_number > lines.len() {
        return Err(format!(
            "Line {} is out of range (file has {} lines)",
            line_number,
            lines.len()
        ));
    }

    let old_line = lines[line_number - 1];
    let mut previews = Vec::new();

    // Find all port numbers in the line and suggest replacements
    let port_re = Regex::new(r"\b(\d{2,5})\b").unwrap();
    for cap in port_re.captures_iter(old_line) {
        if let Ok(old_port) = cap[1].parse::<u16>() {
            // Only suggest for ports in valid range
            if old_port >= 1024 {
                let new_port = old_port + 1; // Simple suggestion: next port
                let new_line = old_line.replace(&cap[1], &new_port.to_string());
                if new_line != old_line {
                    previews.push(FixPreview {
                        file_path: file_path.to_string(),
                        line_number,
                        old_line: old_line.to_string(),
                        new_line,
                        old_port,
                        new_port,
                    });
                }
            }
        }
    }

    if previews.is_empty() {
        // If no specific port found, return a generic preview
        previews.push(FixPreview {
            file_path: file_path.to_string(),
            line_number,
            old_line: old_line.to_string(),
            new_line: old_line.to_string(),
            old_port: 0,
            new_port: 0,
        });
    }

    Ok(previews)
}

/// Apply a port fix to a config file.
pub fn apply_fix(file_path: &str, line_number: usize, new_port: u16) -> Result<String, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;

    let lines: Vec<&str> = content.lines().collect();
    if line_number == 0 || line_number > lines.len() {
        return Err(format!(
            "Line {} is out of range (file has {} lines)",
            line_number,
            lines.len()
        ));
    }

    let old_line = lines[line_number - 1];

    // Find the port number in the line and replace it
    let port_re = Regex::new(r"\b(\d{2,5})\b").unwrap();
    let mut new_line = old_line.to_string();
    let mut replaced = false;

    for cap in port_re.captures_iter(old_line) {
        if let Ok(port) = cap[1].parse::<u16>() {
            if port >= 1024 && port <= 65535 {
                new_line = new_line.replacen(&cap[1], &new_port.to_string(), 1);
                replaced = true;
                break;
            }
        }
    }

    if !replaced {
        return Err(format!(
            "No port number found on line {} of {}",
            line_number, file_path
        ));
    }

    // Reconstruct the file
    let mut new_content = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i + 1 == line_number {
            new_content.push_str(&new_line);
        } else {
            new_content.push_str(line);
        }
        if i < lines.len() - 1 {
            new_content.push('\n');
        }
    }

    fs::write(file_path, &new_content)
        .map_err(|e| format!("Failed to write {}: {}", file_path, e))?;

    Ok(format!(
        "Updated {} line {}: {} -> {}",
        file_path, line_number, old_line.trim(), new_line.trim()
    ))
}

/// Find the file and line where a port is configured for a given project.
pub fn find_port_config(
    project_path: &str,
    port: u16,
) -> Option<(String, usize, String)> {
    let port_str = port.to_string();

    // Search common config files
    let config_files = vec![
        ".env",
        ".env.local",
        ".env.development",
        "docker-compose.yml",
        "docker-compose.yaml",
        "package.json",
        "vite.config.js",
        "vite.config.ts",
        "next.config.js",
        "next.config.ts",
    ];

    for config_name in &config_files {
        let config_path = format!("{}/{}", project_path, config_name);
        if let Ok(content) = fs::read_to_string(&config_path) {
            for (line_num, line) in content.lines().enumerate() {
                if line.contains(&port_str) {
                    return Some((config_path.clone(), line_num + 1, line.to_string()));
                }
            }
        }
    }

    None
}
