use std::{env, fs, io::{self, Write}, os::unix::fs::FileTypeExt, path::PathBuf, time::SystemTime};
use chrono::{DateTime, Local};
use clap::Parser;

struct SSHSocket {
    name: String,
    timestamp: Option<SystemTime>,
} 

fn find_sockets() -> Vec<SSHSocket> {
    let mut paths: Vec<PathBuf> = vec![PathBuf::from("/tmp")];
    let mut found: Vec<SSHSocket> = vec![];

    while paths.len() > 0 {
        paths.pop().map(|curr| {
            if curr.is_dir() {
                // find all contained paths
                fs::read_dir(&curr).map_or_else(|e| {
                    // check if it's a permission error
                    if e.raw_os_error() != Some(13) {
                        eprintln!("Error reading directory {}: {}", curr.display(), e)
                    }
                }, |dir_entries| dir_entries.for_each(|entry| entry.map_or_else(
                    |e| {
                        // check if it's a permission error
                        if e.raw_os_error() != Some(13) {
                            eprintln!("Error reading directory {}: {}", curr.display(), e)
                        }
                    },
                    |entry| paths.push(entry.path())
                )));
            } if fs::metadata(&curr).map_or(false, |meta| meta.file_type().is_socket()) {
                if curr.file_name().map_or(false, |n| {
                    let string = String::from(n.to_string_lossy());
                    string.starts_with("agent.")
                }) {
                    found.push(SSHSocket {
                        name: curr.to_string_lossy().to_string(),
                        timestamp: curr.metadata().ok().and_then(|meta| meta.modified().ok()),
                    });
                }
            }  
        });
    }

    // Sort sockets by timestamp (newest first)
    found.sort_by(|a, b| {
        b.timestamp.cmp(&a.timestamp)
    });

    found
}

fn display_table(found: &[SSHSocket]) {
    let current_var = env::var("SSH_AUTH_SOCK").unwrap_or_default();
    
    if found.is_empty() {
        eprintln!("No SSH agent sockets found.");
    } else {
        eprintln!("┌─────┬─────────┬─────────────────────┬─────────────────────────────────────────┐");
        eprintln!("│ Idx │ Status  │ Modified            │ Socket Path                             │");
        eprintln!("├─────┼─────────┼─────────────────────┼─────────────────────────────────────────┤");
        
        for (idx, sock) in found.iter().enumerate() {
            let status = if sock.name == current_var { "CURRENT" } else { "       " };
            let timestamp = sock.timestamp
                .map(|ts| DateTime::<Local>::from(ts).format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            eprintln!("│ {:>3} │ {} │ {:>19} │ {:<39} │", 
                idx + 1, status, timestamp, sock.name);
        }
        
        eprintln!("└─────┴─────────┴─────────────────────┴─────────────────────────────────────────┘");
    }
}

fn clear_screen() {
    eprint!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

fn main() -> io::Result<()> {
    #[derive(Parser, Debug)]
    #[command(about = "Select or print SSH_AUTH_SOCK from available agent sockets. Use with eval \"$(refresh-auth-sock -r)\" or source <(refresh-auth-sock -r)\"")]
    struct Args {
        /// Pick the most recent auth sock and print export statement
        #[arg(short = 'r', long = "recent")]
        recent: bool,
    }

    let args = Args::parse();

    if args.recent {
        let found = find_sockets();
        if found.is_empty() {
            // keep behavior similar to interactive mode
            eprintln!("No SSH agent sockets found.");
            return Ok(());
        }

        let selected_socket = &found[0];
        // env::set_var("SSH_AUTH_SOCK", &selected_socket.name);
        println!("export SSH_AUTH_SOCK={}", selected_socket.name);
        return Ok(());
    }

    let stdin = io::stdin();
    let mut first_run = true;
    
    loop {
        if !first_run {
            clear_screen();
        }
        first_run = false;
        
        let found = find_sockets();
        display_table(&found);
        
        if found.is_empty() {
            return Ok(());
        }
        
        eprint!("\nEnter socket number (1-{}) or 'q' to quit: ", found.len());
        io::stdout().flush()?;
        
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();
        
        if input == "q" || input == "Q" {
            break;
        }
        
        match input.parse::<usize>() {
            Ok(idx) if idx > 0 && idx <= found.len() => {
                let selected_socket = &found[idx - 1];
                unsafe {
                    env::set_var("SSH_AUTH_SOCK", &selected_socket.name);
                }
            },
            _ => {
                eprintln!("Invalid selection. Please enter a number between 1 and {} or 'q' to quit.", found.len());
                eprint!("Press Enter to continue...");
                io::stdout().flush()?;
                let mut _dummy = String::new();
                stdin.read_line(&mut _dummy)?;
            }
        }
    }

    println!("export SSH_AUTH_SOCK={}", env::var("SSH_AUTH_SOCK").unwrap_or_default());

    Ok(())
}
