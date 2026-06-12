fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("list");

    match cmd {
        "list" => list_ports(),
        "check" => {
            let port = args.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
            if port == 0 {
                eprintln!("Usage: lh check <port>");
                std::process::exit(1);
            }
            check_port(port);
        }
        "suggest" => {
            let start = args.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(3000);
            let end = args.get(3).and_then(|s| s.parse::<u16>().ok()).unwrap_or(3999);
            suggest_port(start, end);
        }
        "portmasters" => list_portmasters(),
        "projects" => list_project_ports(),
        _ => {
            eprintln!("Usage: lh [list|check <port>|suggest <start> <end>|portmasters|projects]");
            std::process::exit(1);
        }
    }
}

fn list_ports() {
    match lighthouse::scanner::scan_live_ports() {
        Ok(mut ports) => {
            println!("PORT   BIND            PROCESS");
            println!("-----  --------------  --------------------");
            ports.sort_by_key(|p| p.port);
            for p in ports {
                println!("{:<5}  {:<14}  {}", p.port, p.bind_address, p.process_name);
            }
        }
        Err(err) => {
            eprintln!("Failed to scan ports: {}", err);
            std::process::exit(1);
        }
    }
}

fn check_port(port: u16) {
    match lighthouse::scanner::check_single_port(port) {
        Ok(result) => {
            if result.in_use {
                println!("Port {} is in use", result.port);
                if let Some(proc_name) = result.process {
                    println!("Process: {}", proc_name);
                }
                if let Some(pid) = result.pid {
                    println!("PID: {}", pid);
                }
                if let Some(suggestion) = result.suggestion {
                    println!("Suggested free port: {}", suggestion);
                }
            } else {
                println!("Port {} is free", result.port);
            }
        }
        Err(err) => {
            eprintln!("Failed to check port {}: {}", port, err);
            std::process::exit(1);
        }
    }
}

fn suggest_port(start: u16, end: u16) {
    match lighthouse::resolver::suggest_port(start, end) {
        Ok(port) => println!("{}", port),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

fn list_portmasters() {
    match lighthouse::portmaster::discover_portmaster_files() {
        Ok(files) => {
            if files.is_empty() {
                println!("No PORTMASTER.md files found");
                return;
            }
            for file in files {
                println!("{}", file);
            }
        }
        Err(err) => {
            eprintln!("Failed to discover PORTMASTER.md files: {}", err);
            std::process::exit(1);
        }
    }
}

fn list_project_ports() {
    match lighthouse::projects::scan_project_ports() {
        Ok(mut ports) => {
            if ports.is_empty() {
                println!("No project port configs found");
                return;
            }
            ports.sort_by(|a, b| a.port.cmp(&b.port).then(a.project_name.cmp(&b.project_name)));
            println!("PORT   PROJECT                FILE");
            println!("-----  ---------------------  --------------------");
            for p in ports {
                println!("{:<5}  {:<21}  {}:{}", p.port, p.project_name, p.file_path, p.line_number);
            }
        }
        Err(err) => {
            eprintln!("Failed to scan project ports: {}", err);
            std::process::exit(1);
        }
    }
}
