use crate::{PortInfo, PortSource};
use regex::Regex;
use std::process::Command;

/// Scan all TCP listening ports on the system.
pub fn scan_live_ports() -> Result<Vec<PortInfo>, Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    return scan_macos();
    #[cfg(target_os = "linux")]
    return scan_linux();
    #[cfg(target_os = "windows")]
    return scan_windows();
}

#[cfg(target_os = "macos")]
fn scan_macos() -> Result<Vec<PortInfo>, Box<dyn std::error::Error>> {
    let output = Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-P", "-n", "-F", "pcRn"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_lsof_output(&stdout)
}

#[cfg(target_os = "linux")]
fn scan_linux() -> Result<Vec<PortInfo>, Box<dyn std::error::Error>> {
    let output = Command::new("ss")
        .args(["-tlnp", "--no-header"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_ss_output(&stdout)
}

#[cfg(target_os = "windows")]
fn scan_windows() -> Result<Vec<PortInfo>, Box<dyn std::error::Error>> {
    let output = Command::new("netstat")
        .args(["-ano", "-p", "TCP"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_netstat_output(&stdout)
}

/// Parse lsof -F output (macOS). Fields: p=PID, c=command, n=address:port.
/// Important: lsof's field output does not necessarily separate processes with blank lines,
/// so we flush accumulated socket entries whenever a new `p...` process record begins.
fn parse_lsof_output(output: &str) -> Result<Vec<PortInfo>, Box<dyn std::error::Error>> {
    let mut ports = Vec::new();
    let mut current_pid: Option<u32> = None;
    let mut current_cmd = String::new();
    let mut entries: Vec<(String, String)> = Vec::new();

    let flush_entries = |ports: &mut Vec<PortInfo>,
                         current_pid: Option<u32>,
                         current_cmd: &str,
                         entries: &mut Vec<(String, String)>| {
        for (addr, port_str) in entries.drain(..) {
            if let Ok(port) = port_str.parse::<u16>() {
                let bind_address = if addr == "*" || addr.is_empty() {
                    "0.0.0.0".to_string()
                } else {
                    addr
                };
                ports.push(PortInfo {
                    port,
                    protocol: "TCP".to_string(),
                    bind_address,
                    pid: current_pid,
                    process_name: current_cmd.to_string(),
                    service_name: None,
                    managed_by: None,
                    project: None,
                    source: PortSource::Live,
                });
            }
        }
    };

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        match line.as_bytes().first() {
            Some(b'p') => {
                if current_pid.is_some() || !entries.is_empty() {
                    flush_entries(&mut ports, current_pid, &current_cmd, &mut entries);
                }
                current_cmd.clear();
                if let Ok(pid) = line[1..].parse::<u32>() {
                    current_pid = Some(pid);
                } else {
                    current_pid = None;
                }
            }
            Some(b'c') => current_cmd = line[1..].to_string(),
            Some(b'n') => {
                let addr_port = &line[1..];
                if let Some(colon_pos) = addr_port.rfind(':') {
                    let addr = addr_port[..colon_pos]
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .to_string();
                    let port = addr_port[colon_pos + 1..].to_string();
                    entries.push((addr, port));
                }
            }
            _ => {}
        }
    }

    // Flush final process record.
    if current_pid.is_some() || !entries.is_empty() {
        flush_entries(&mut ports, current_pid, &current_cmd, &mut entries);
    }

    ports.sort_by(|a, b| a.port.cmp(&b.port).then(a.bind_address.cmp(&b.bind_address)));
    ports.dedup_by(|a, b| a.port == b.port && a.bind_address == b.bind_address && a.pid == b.pid);
    Ok(ports)
}

#[cfg(target_os = "linux")]
fn parse_ss_output(output: &str) -> Result<Vec<PortInfo>, Box<dyn std::error::Error>> {
    let mut ports = Vec::new();
    for line in output.lines() {
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 { continue; }

        let local = parts[3];
        let (addr, port_str) = parse_address_port(local);
        if let Ok(port) = port_str.parse::<u16>() {
            let process_info = parts.get(5).unwrap_or(&"-");
            let (pid, process_name) = parse_process_info(process_info);
            ports.push(PortInfo {
                port,
                protocol: "TCP".to_string(),
                bind_address: addr,
                pid,
                process_name,
                service_name: None,
                managed_by: None,
                project: None,
                source: PortSource::Live,
            });
        }
    }
    ports.sort_by(|a, b| a.port.cmp(&b.port).then(a.bind_address.cmp(&b.bind_address)));
    ports.dedup_by(|a, b| a.port == b.port && a.bind_address == b.bind_address && a.pid == b.pid);
    Ok(ports)
}

#[cfg(target_os = "windows")]
fn parse_netstat_output(output: &str) -> Result<Vec<PortInfo>, Box<dyn std::error::Error>> {
    let mut ports = Vec::new();
    let line_re = Regex::new(r"TCP\s+[\[\]]?([^\]\s:]+)[\]:]?:(\d+)\s+\S+\s+LISTENING\s+(\d+)").unwrap();
    for line in output.lines() {
        if let Some(caps) = line_re.captures(line) {
            if let Ok(port) = caps[2].parse::<u16>() {
                let addr = if caps[1].is_empty() { "0.0.0.0".to_string() } else { caps[1].to_string() };
                let pid = caps[3].parse::<u32>().ok();
                ports.push(PortInfo {
                    port,
                    protocol: "TCP".to_string(),
                    bind_address: addr,
                    pid,
                    process_name: String::new(),
                    service_name: None,
                    managed_by: None,
                    project: None,
                    source: PortSource::Live,
                });
            }
        }
    }
    ports.sort_by(|a, b| a.port.cmp(&b.port).then(a.bind_address.cmp(&b.bind_address)));
    ports.dedup_by(|a, b| a.port == b.port && a.bind_address == b.bind_address && a.pid == b.pid);
    Ok(ports)
}

fn parse_address_port(s: &str) -> (String, String) {
    if let Some(bracket_end) = s.rfind(']') {
        let addr = s[1..bracket_end].to_string();
        let port = s[bracket_end + 2..].to_string();
        return (addr, port);
    }
    if let Some(colon) = s.rfind(':') {
        return (s[..colon].to_string(), s[colon + 1..].to_string());
    }
    (s.to_string(), String::new())
}

fn parse_process_info(info: &str) -> (Option<u32>, String) {
    let pid_re = Regex::new(r"pid=(\d+)").ok();
    let pid = pid_re.as_ref()
        .and_then(|r| r.captures(info).and_then(|c| c[1].parse::<u32>().ok()));

    let name_re = Regex::new(r#""(\w+)""#).ok();
    let name = name_re.as_ref()
        .and_then(|r| r.captures(info).map(|c| c[1].to_string()))
        .unwrap_or_else(|| {
            info.split(&['(', ',', ')'])
                .find(|s| !s.is_empty() && *s != "users:")
                .unwrap_or(info)
                .to_string()
        });

    (pid, name)
}

pub fn check_single_port(port: u16) -> Result<crate::PortCheckResult, Box<dyn std::error::Error>> {
    let live_ports = scan_live_ports()?;
    let entry = live_ports.iter().find(|p| p.port == port);

    let suggestion = if entry.is_some() {
        let used: std::collections::HashSet<u16> = live_ports.iter().map(|p| p.port).collect();
        let mut found = None;
        for candidate in (port.saturating_add(1))..=3999 {
            if !used.contains(&candidate) {
                found = Some(candidate);
                break;
            }
        }
        found
    } else {
        None
    };

    Ok(crate::PortCheckResult {
        port,
        in_use: entry.is_some(),
        process: entry.map(|p| p.process_name.clone()),
        pid: entry.and_then(|p| p.pid),
        portmaster_entry: None,
        suggestion,
    })
}

pub fn scan_docker_ports() -> Result<Vec<(String, u16, String)>, Box<dyn std::error::Error>> {
    let output = Command::new("docker")
        .args(["ps", "--format", "{{.Names}}\t{{.Ports}}"])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 || parts[1].trim().is_empty() { continue; }
        let name = parts[0].trim().to_string();

        for mapping in parts[1].split(", ") {
            if let Some(arrow) = mapping.find("->") {
                let host_part = &mapping[..arrow];
                if let Some(colon) = host_part.rfind(':') {
                    let port_str = &host_part[colon + 1..];
                    if let Ok(host_port) = port_str.parse::<u16>() {
                        let bind = host_part[..colon]
                            .trim_start_matches('[')
                            .trim_end_matches(']');
                        let bind = if bind.is_empty() || bind == "::" {
                            "0.0.0.0".to_string()
                        } else {
                            bind.to_string()
                        };
                        results.push((name.clone(), host_port, bind));
                    }
                }
            }
        }
    }
    Ok(results)
}
