use sha1::{Digest, Sha1};
use std::env;
use std::os::unix::process::CommandExt;
use std::process::{Command, exit};

/// UUID v5 namespace (DNS namespace from RFC 4122)
const DNS_NAMESPACE: [u8; 16] = [
    0x6b, 0xa7, 0xb8, 0x10,
    0x9d, 0xad, 0x11, 0xd1,
    0x80, 0xb4, 0x00, 0xc0,
    0x4f, 0xd4, 0x30, 0xc8,
];

/// Generate a deterministic UUID v5 from a name
fn generate_uuid5(name: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(&DNS_NAMESPACE);
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();

    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50; // Version 5
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 10xx

    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]),
        u16::from_be_bytes([bytes[8], bytes[9]]),
        u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]])
    )
}

/// Get current git branch name
fn get_git_branch() -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| "Failed to execute git command")?;

    if !output.status.success() {
        return Err("Not a git repository or no branch found".to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get current folder name
fn get_folder_name() -> Result<String, String> {
    env::current_dir()
        .map_err(|_| "Failed to get current directory")?
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to get folder name".to_string())
}

fn print_help() {
    eprintln!("cs - Claude Code Session Manager");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    cs              Start/resume session for current folder+branch");
    eprintln!("    cs --dry-run    Show session info without launching claude");
    eprintln!("    cs --help       Show this help message");
    eprintln!("    cs --version    Show version");
    eprintln!();
    eprintln!("Session name format: <folder>+<branch>");
    eprintln!("Example: my-project+feature/auth -> UUID: 4b513bfa-8c71-512b-...");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Handle flags
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--version" | "-v" => {
                println!("cs {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--dry-run" | "-n" => {
                // Continue but don't launch claude
            }
            arg => {
                eprintln!("Unknown argument: {}", arg);
                eprintln!("Run 'cs --help' for usage");
                exit(1);
            }
        }
    }

    // Get folder name
    let folder_name = match get_folder_name() {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    // Get git branch
    let branch_name = match get_git_branch() {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    // Create session name and UUID
    let session_name = format!("{}+{}", folder_name, branch_name);
    let session_uuid = generate_uuid5(&session_name);

    // Print info
    println!("┌─────────────────────────────────────────────");
    println!("│ Session: {}", session_name);
    println!("│ UUID:    {}", session_uuid);
    println!("└─────────────────────────────────────────────");
    println!();

    // Check for dry-run
    if args.len() > 1 && (args[1] == "--dry-run" || args[1] == "-n") {
        return;
    }

    // Launch claude - use Unix exec() syscall to replace current process
    let err = Command::new("claude")
        .args(["--session-id", &session_uuid])
        .exec();

    // If we get here, exec failed
    eprintln!("Error launching claude: {}", err);
    exit(1);
}
