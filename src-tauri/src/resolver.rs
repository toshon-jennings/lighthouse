use crate::{scanner, Conflict, PortInfo, PortSource, PortmasterEntry, ProjectPort};

/// Detect conflicts between live ports, PORTMASTER entries, and project configs.
pub fn detect_conflicts(
    live_ports: &[PortInfo],
    pm_entries: &[PortmasterEntry],
    project_ports: &[ProjectPort],
) -> Vec<Conflict> {
    let mut conflicts = Vec::new();

    // 1. Check for live ports that conflict with each other
    for (i, port_a) in live_ports.iter().enumerate() {
        for port_b in live_ports.iter().skip(i + 1) {
            if port_a.port == port_b.port && ports_overlap(&port_a.bind_address, &port_b.bind_address) {
                let suggestion = suggest_port_for_conflict(port_a.port, pm_entries, live_ports);
                conflicts.push(Conflict {
                    port: port_a.port,
                    bind_a: port_a.bind_address.clone(),
                    bind_b: port_b.bind_address.clone(),
                    process_a: port_a.process_name.clone(),
                    process_b: port_b.process_name.clone(),
                    pid_a: port_a.pid,
                    pid_b: port_b.pid,
                    suggestion,
                    explanation: format!(
                        "Both '{}' and '{}' are listening on port {}",
                        port_a.process_name, port_b.process_name, port_a.port
                    ),
                });
            }
        }
    }

    // 2. Check for project configs that conflict with live ports
    for pp in project_ports {
        for lp in live_ports {
            if pp.port == pp.port
                && pp.project_path != get_project_path_from_process(&lp.process_name)
                && ports_overlap(&lp.bind_address, "0.0.0.0")
            {
                let suggestion = suggest_port_for_conflict(pp.port, pm_entries, live_ports);
                conflicts.push(Conflict {
                    port: pp.port,
                    bind_a: lp.bind_address.clone(),
                    bind_b: "0.0.0.0".to_string(),
                    process_a: lp.process_name.clone(),
                    process_b: format!("{} (config)", pp.project_name),
                    pid_a: lp.pid,
                    pid_b: None,
                    suggestion,
                    explanation: format!(
                        "Project '{}' wants port {} but '{}' is already using it",
                        pp.project_name, pp.port, lp.process_name
                    ),
                });
            }
        }
    }

    // 3. Check for project config conflicts with other project configs
    for (i, pp_a) in project_ports.iter().enumerate() {
        for pp_b in project_ports.iter().skip(i + 1) {
            if pp_a.port == pp_b.port && pp_a.project_path != pp_b.project_path {
                let suggestion = suggest_port_for_conflict(pp_a.port, pm_entries, live_ports);
                conflicts.push(Conflict {
                    port: pp_a.port,
                    bind_a: "configured".to_string(),
                    bind_b: "configured".to_string(),
                    process_a: pp_a.project_name.clone(),
                    process_b: pp_b.project_name.clone(),
                    pid_a: None,
                    pid_b: None,
                    suggestion,
                    explanation: format!(
                        "Both '{}' and '{}' are configured to use port {}",
                        pp_a.project_name, pp_b.project_name, pp_a.port
                    ),
                });
            }
        }
    }

    conflicts
}

/// Merge live ports, PORTMASTER entries, and project configs into a unified view.
pub fn merge_ports(
    live_ports: &[PortInfo],
    pm_entries: &[PortmasterEntry],
    project_ports: &[ProjectPort],
) -> Vec<PortInfo> {
    let mut merged: Vec<PortInfo> = Vec::new();
    let docker_ports = scanner::scan_docker_ports().unwrap_or_default();

    // Start with live ports, enriching with PORTMASTER and project data
    for lp in live_ports {
        let mut info = lp.clone();

        // Enrich from PORTMASTER
        if let Some(pm) = pm_entries.iter().find(|p| p.port == lp.port) {
            info.service_name = Some(pm.service.clone());
            info.managed_by = Some(pm.managed_by.clone());
        }

        // Check if it is a docker container
        for (name, port, _) in &docker_ports {
            if *port == lp.port {
                info.managed_by = Some(format!("Docker ({})", name));
                info.source = PortSource::Docker;
                break;
            }
        }

        // Enrich from project configs
        if let Some(pp) = project_ports.iter().find(|p| p.port == lp.port) {
            info.project = Some(pp.project_name.clone());
        }

        merged.push(info);
    }

    // Add PORTMASTER entries that are not currently live
    for pm in pm_entries {
        if !merged.iter().any(|p| p.port == pm.port) {
            merged.push(PortInfo {
                port: pm.port,
                protocol: pm.protocol.clone(),
                bind_address: pm.bind.clone(),
                pid: None,
                process_name: String::new(),
                service_name: Some(pm.service.clone()),
                managed_by: Some(pm.managed_by.clone()),
                project: None,
                source: PortSource::Portmaster,
            });
        }
    }

    // Add project configs that are not currently live
    for pp in project_ports {
        if !merged.iter().any(|p| p.port == pp.port) {
            merged.push(PortInfo {
                port: pp.port,
                protocol: "TCP".to_string(),
                bind_address: "*".to_string(),
                pid: None,
                process_name: String::new(),
                service_name: None,
                managed_by: None,
                project: Some(pp.project_name.clone()),
                source: PortSource::ProjectConfig,
            });
        }
    }

    merged.sort_by_key(|p| p.port);
    merged
}

/// Suggest a free port, preferring the same allocation range.
pub fn suggest_port(range_start: u16, range_end: u16) -> Result<u16, String> {
    let live_ports = scanner::scan_live_ports()
        .map_err(|e| format!("Failed to scan ports: {}", e))?;
    let live_set: std::collections::HashSet<u16> = live_ports.iter().map(|p| p.port).collect();

    for port in range_start..=range_end {
        if !live_set.contains(&port) {
            return Ok(port);
        }
    }

    Err(format!(
        "No free ports in range {}-{}",
        range_start, range_end
    ))
}

fn suggest_port_for_conflict(
    conflict_port: u16,
    _pm_entries: &[PortmasterEntry],
    live_ports: &[PortInfo],
) -> Option<u16> {
    let live_set: std::collections::HashSet<u16> = live_ports.iter().map(|p| p.port).collect();

    // Try nearby ports first (+1, -1, +2, -2, etc.)
    for offset in 1..100 {
        if conflict_port + offset <= 65535 && !live_set.contains(&(conflict_port + offset)) {
            return Some(conflict_port + offset);
        }
        if conflict_port >= offset && !live_set.contains(&(conflict_port - offset)) {
            return Some(conflict_port - offset);
        }
    }
    None
}

fn ports_overlap(addr_a: &str, addr_b: &str) -> bool {
    // 0.0.0.0 overlaps with everything
    if addr_a == "0.0.0.0" || addr_b == "0.0.0.0" {
        return true;
    }
    // Same address
    if addr_a == addr_b {
        return true;
    }
    // 127.0.0.1 overlaps with ::1 (localhost)
    let is_local_a = addr_a == "127.0.0.1" || addr_a == "::1" || addr_a == "localhost";
    let is_local_b = addr_b == "127.0.0.1" || addr_b == "::1" || addr_b == "localhost";
    is_local_a && is_local_b
}

fn get_project_path_from_process(process_name: &str) -> String {
    match process_name {
        "node" | "node.exe" => String::new(),
        "next-server" | "next-dev" => String::new(),
        _ => String::new(),
    }
}

/// Detect the allocation range for a given port based on PORTMASTER.md ranges.
pub fn detect_range_for_port(port: u16) -> Option<(u16, u16)> {
    // Default allocation ranges (mirroring PORTMASTER.md conventions)
    let ranges: Vec<(u16, u16)> = vec![
        (1024, 2999),
        (3000, 3999),
        (4000, 4999),
        (5000, 5999),
        (6000, 7999),
        (8000, 8999),
        (9000, 9999),
    ];

    for (start, end) in ranges {
        if port >= start && port <= end {
            return Some((start, end));
        }
    }
    None
}
