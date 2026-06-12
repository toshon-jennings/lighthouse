use crate::{PortInfo, PortSource};
use regex::Regex;
use std::process::Command;

/// Scan all TCP listening ports on the system.
/// Uses `lsof` on macOS and `ss` on Linux.
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
    // Use lsof to get all TCP listeners with PID and process name
    let output = Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-P", "-n", "-F", "pcRnTSTDevice"])
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

/// Parse lsof -F output (macOS)
/// The -F flag outputs fields prefixed with identifiers:
/// p = PID, c = command name, R = PPID, n = node name (address:port)
fn parse_lsof_output(output: &str) -> Result<Vec<PortInfo>, Box<dyn std::error::Error>> {
    let mut ports = Vec::new();
    let mut current_pid: Option<u32> = None;
    let mut current_cmd = String::new();
    let mut current_addr = String::new();
    let mut current_port_str = String::new();

    for line in output.lines() {
        if line.is_empty() {
            // Flush current entry
            if let Some(pid) = current_pid {
                if !current_port_str.is_empty() {
                    if let Ok(port) = current_port_str.parse::<u16>() {
                        let bind_address = if current_addr == "*" || current_addr.is_empty() {
                            "0.0.0.0".to_string()
                        } else {
                            current_addr.clone()
                        };
                        ports.push(PortInfo {
                            port,
                            protocol: "TCP".to_string(),
                            bind_address,
                            pid: Some(pid),
                            process_name: current_cmd.clone(),
                            service_name: None,
                            managed_by: None,
                            project: None,
                            source: PortSource::Live,
                        });
                    }
                }
            }
            current_pid = None;
            current_cmd.clear();
            current_addr.clear();
            current_port_str.clear();
            continue;
        }

        match &line[..1] {
            "p" => {
                if let Ok(pid) = line[1..].parse::<u32>() {
                    current_pid = Some(pid);
                }
            }
            "c" => current_cmd = line[1..].to_string(),
            "n" => {
                // Format: address:port or *:port or [::1]:port
                let addr_port = &line[1..];
                if let Some(colon_pos) = addr_port.rfind(':') {
                    current_addr = addr_port[..colon_pos]
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .to_string();
                    current_port_str = addr_port[colon_pos + 1..].to_string();
                }
            }
            _ => {} // ignore other fields
        }
    }

    // Deduplicate: same port + bind + PID
    ports.sort_by_key(|p| (p.port, p.bind_address.clone(), p.pid));
    ports.dedup_by_key(|p| (p.port, p.bind_address.clone(), p.pid));

    Ok(ports)
}

#[cfg(target_os = "fn parse_ss_output(output: &str) -> Result<Vec<PortInfo>, Box<dyn std::error::Error>> {
    let mut ports = Vec::new();
    let re = Regex::new(r"(\S+)\s+\d+\s+\d+\s+(\S+):(\d+)\s+\S+\s+(.+)?")?;

    for line in output.lines() {
        if line.is_empty() { continue; }
        // ss -tlnp output: State  Recv-Q  Send-Q  Local Address:Port  Peer Address:Port  Process
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 { continue; }

        let local = parts[3];
        let (addr, port_str) = parse_address_port(local);
        if let Ok(port) = port_str.parse::<u16>() {
            // Extract PID/process from last field if present
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

    ports.sort_by_key(|p| (p.port, p.bind_address.clone(), p.pid));
    ports.dedup_by_key(|p| (p.port, p.bind_address.clone(), p.pid));
    Ok(ports)
}

#[cfg(target_os = "windows")]
fn parse_netstat_output(output: &str) -> Result<Vec<PortInfo>, Box<dyn std::error::Error>> {
    let mut ports = Vec::new();
    let re = Regex::new(r"TCP\s+\[?([^\]]*)\]?:(\d+)\s+\S+\s+LISTENING\s+(\d+)")?;

    for line in output.lines() {
        if let Some(caps) = re.captures(line) {
            if let Ok(port) = caps[2].parse::<u16>() {
                let addr = if caps[1].is_empty() { "0.0.0.0".to_string() } else { caps[1].to_string() };
                let pid = caps[3].parse::<u32>().ok();

                ports.push(PortInfo {
                    port,
                    protocol: "TCP".to_string(),
                    bind_address: addr,
                    pid,
                    process_name: String::new(), // Would need tasklist lookup
                    service_name: None,
                    managed_by: None,
                    project: None,
                    source: PortSource::Live,
                });
            }
        }
    }

    ports.sort_by_key(|p| (p.port, p.bind_address.clone(), p.pid));
    ports.dedup_by_key(|p| (p.port, p.bind_address.clone(), p.pid));
    Ok(ports)
}

fn parse_address_port(s: &str) -> (String, String) {
    // Handle [::1]:3000 format (IPv6)
    if let Some(bracket_end) = s.rfind(']') {
        let addr = s[1..bracket_end].to_string();
        let port = s[bracket_end + 2..].to_string(); // skip ']:'
        return (addr, port);
    }
    // Handle *:3000 or 127.0.0.1:3000
    if let Some(colon) = s.rfind(':') {
        let addr = s[..colon].to_string();
        let port = s[colon + 1..].to_string();
        return (addr, port);
    }
    (s.to_string(), String::new())
}

fn parse_process_info(info: &str) -> (Option<u32>, String) {
    // ss format: "users:((\"node\",pid=1234,fd=5))" or just "-"
    let re = Regex::new(r"pid=(\d+)").ok();
    let pid = re.as_ref().and_then(|r| {
        r.captures(info).and_then(|c| c[1].parse::<u32>().ok())
    });

    let name_re = Regex::new(r#""(\w+)""#).ok();
    let name = name_re.as_ref().and_then(|r| {
        r.captures(info).map(|c| c[1].to_string())
    }).unwrap_or_else(|| info.to_string());

    (pid, name)
}

/// Check if a specific port is currently in use
pub fn check_single_port(port: u16) -> Result<crate::PortCheckResult, Box<dyn std::error::Error>> {
    let live_ports = scan_live_ports()?;
    let entry = live_ports.iter().find(|p| p.port == port);

    Ok(crate::PortCheckResult {
        port,
        in_use: entry.is_some(),
        process: entry.map(|p| p.process_name.clone()),
        pid: entry.and_then(|p| p.pid),
        portmaster_entry: None,
        suggestion: None,
    })
}

/// Scan Docker containers for port mappings
pub fn scan_docker_ports() -> Result<Vec<(String, u16, String)>, Box<dyn std::error::Error>> {
    let output = Command::new("docker")
        .args(["ps", "--format", "{{.Names}}\t{{.Ports}}"])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new()); // Docker not running or not installed
    }

    let mut results = Vec::new();
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 { continue; }
        let name = parts[0].trim().to_string();
        let ports_str = parts[1].trim();

        if ports_str.is_empty() { continue; }

        // Parse port mappings like: 0.0.0.0:5055->5055/tcp, [::]:5055->5055/tcp
        for mapping in ports_str.split(", ") {
            let mapping = mapping.trim();
            if mapping.is_empty() { continue; }

            // Extract host port: everything before '->'
            if let Some(arrow_pos) = mapping.find("->") {
                let host_part = &mapping[..arrow_pos];
                // host_part looks like 0.0.0.0:5055 or [::]:5055
                if let Some(colon_pos) = host_part.rfind(':') {
                    let port_str = &host_part[colon_pos + 1..];
                    if let Ok(host_port) = port_str.parse::<u16>() {
                        // Determine bind address
                        let bind = &host_part[..colon_pos]
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
