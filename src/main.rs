use std::{env, fs, io::{self, Write}, os::unix::fs::FileTypeExt, path::PathBuf, time::SystemTime};
use chrono::{DateTime, Local};

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
        println!("No SSH agent sockets found.");
    } else {
        println!("┌─────┬─────────┬─────────────────────┬─────────────────────────────────────────┐");
        println!("│ Idx │ Status  │ Modified            │ Socket Path                             │");
        println!("├─────┼─────────┼─────────────────────┼─────────────────────────────────────────┤");
        
        for (idx, sock) in found.iter().enumerate() {
            let status = if sock.name == current_var { "CURRENT" } else { "       " };
            let timestamp = sock.timestamp
                .map(|ts| DateTime::<Local>::from(ts).format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            println!("│ {:>3} │ {} │ {:>19} │ {:<39} │", 
                idx + 1, status, timestamp, sock.name);
        }
        
        println!("└─────┴─────────┴─────────────────────┴─────────────────────────────────────────┘");
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

fn main() -> io::Result<()> {
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
        
        print!("\nEnter socket number (1-{}) or 'q' to quit: ", found.len());
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
                println!("Invalid selection. Please enter a number between 1 and {} or 'q' to quit.", found.len());
                print!("Press Enter to continue...");
                io::stdout().flush()?;
                let mut _dummy = String::new();
                stdin.read_line(&mut _dummy)?;
            }
        }
    }

    Ok(())
}
