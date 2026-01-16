use sha1::{Digest, Sha1};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, exit};

/// Default UUID v5 namespace (DNS namespace from RFC 4122)
const DEFAULT_NAMESPACE: [u8; 16] = [
    0x6b, 0xa7, 0xb8, 0x10,
    0x9d, 0xad, 0x11, 0xd1,
    0x80, 0xb4, 0x00, 0xc0,
    0x4f, 0xd4, 0x30, 0xc8,
];

/// Parse a UUID string (e.g., "6ba7b810-9dad-11d1-80b4-00c04fd430c8") into bytes
fn parse_uuid(uuid_str: &str) -> Option<[u8; 16]> {
    let hex: String = uuid_str.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex.len() != 32 {
        return None;
    }

    let mut bytes = [0u8; 16];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let hex_pair = std::str::from_utf8(chunk).ok()?;
        bytes[i] = u8::from_str_radix(hex_pair, 16).ok()?;
    }
    Some(bytes)
}

/// Get namespace from CS_NAMESPACE env var or use default
fn get_namespace() -> [u8; 16] {
    env::var("CS_NAMESPACE")
        .ok()
        .and_then(|s| parse_uuid(&s))
        .unwrap_or(DEFAULT_NAMESPACE)
}

/// Get the path to the sessions database file (~/.cs/sessions)
fn get_db_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cs").join("sessions")
}

/// Load existing session UUIDs from database
fn load_sessions() -> HashSet<String> {
    let db_path = get_db_path();
    let mut sessions = HashSet::new();

    if let Ok(file) = File::open(&db_path) {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let uuid = line.trim().to_string();
            if !uuid.is_empty() {
                sessions.insert(uuid);
            }
        }
    }

    sessions
}

/// Save a new session UUID to the database
fn save_session(uuid: &str) {
    let db_path = get_db_path();

    // Create directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Append UUID to file
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&db_path)
    {
        let _ = writeln!(file, "{}", uuid);
    }
}

/// Remove a session UUID from the database
fn remove_session(uuid: &str) {
    let db_path = get_db_path();
    if let Ok(content) = fs::read_to_string(&db_path) {
        let filtered: Vec<&str> = content
            .lines()
            .filter(|line| line.trim() != uuid)
            .collect();
        // Write back with newline at end if there are entries
        let new_content = if filtered.is_empty() {
            String::new()
        } else {
            filtered.join("\n") + "\n"
        };
        let _ = fs::write(&db_path, new_content);
    }
}

/// List all sessions in database
fn list_sessions() {
    let sessions = load_sessions();
    if sessions.is_empty() {
        println!("No sessions in database.");
    } else {
        println!("Sessions ({}):", sessions.len());
        for uuid in &sessions {
            println!("  {}", uuid);
        }
    }
}

/// Clear entire session database
fn clear_sessions() {
    let db_path = get_db_path();
    match fs::remove_file(&db_path) {
        Ok(_) => println!("Session database cleared."),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("Session database already empty.");
        }
        Err(e) => {
            eprintln!("Error clearing database: {}", e);
        }
    }
}

/// Generate a deterministic UUID v5 from a name using the configured namespace
fn generate_uuid5(name: &str) -> String {
    let namespace = get_namespace();
    let mut hasher = Sha1::new();
    hasher.update(&namespace);
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
    eprintln!("    cs --force      Force create new session (ignore database)");
    eprintln!("    cs --reset      Remove session from database and create new");
    eprintln!("    cs --list       List all sessions in database");
    eprintln!("    cs --clear      Clear entire session database");
    eprintln!("    cs --dry-run    Show session info without launching Claude");
    eprintln!("    cs --help       Show this help message");
    eprintln!("    cs --version    Show version");
    eprintln!();
    eprintln!("SHORT FLAGS:");
    eprintln!("    -f              Same as --force");
    eprintln!("    -l              Same as --list");
    eprintln!("    -n              Same as --dry-run");
    eprintln!("    -h              Same as --help");
    eprintln!("    -v              Same as --version");
    eprintln!();
    eprintln!("SESSION FORMAT:");
    eprintln!("    <folder>+<branch> -> deterministic UUID v5");
    eprintln!("    Example: my-project+feature/auth -> 4b513bfa-8c71-512b-...");
    eprintln!();
    eprintln!("TROUBLESHOOTING:");
    eprintln!("    If you see \"No conversation found\" error:");
    eprintln!("        cs --reset   # Clears stale entry and creates fresh session");
    eprintln!();
    eprintln!("ENVIRONMENT VARIABLES:");
    eprintln!("    CS_NAMESPACE    Custom UUID v5 namespace (default: DNS namespace)");
    eprintln!("                    Example: export CS_NAMESPACE=\"your-custom-uuid-here\"");
    eprintln!();
    eprintln!("FILES:");
    eprintln!("    ~/.cs/sessions  Session database (one UUID per line)");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Track mode flags
    let mut dry_run = false;
    let mut force_create = false;
    let mut reset_mode = false;

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
            "--list" | "-l" => {
                list_sessions();
                return;
            }
            "--clear" => {
                clear_sessions();
                return;
            }
            "--dry-run" | "-n" => {
                dry_run = true;
            }
            "--force" | "-f" => {
                force_create = true;
            }
            "--reset" => {
                reset_mode = true;
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

    // Handle reset mode: remove existing entry from database
    if reset_mode {
        remove_session(&session_uuid);
    }

    // Load session database (after potential reset)
    let sessions = load_sessions();
    let session_exists = sessions.contains(&session_uuid);

    // Determine effective status for display
    let status_display = if force_create || reset_mode {
        "force-create"
    } else if session_exists {
        "exists"
    } else {
        "new"
    };

    // Print info
    println!("┌─────────────────────────────────────────────");
    println!("│ Session: {}", session_name);
    println!("│ UUID:    {}", session_uuid);
    println!("│ Status:  {}", status_display);
    println!("└─────────────────────────────────────────────");
    println!();

    // Check for dry-run
    if dry_run {
        return;
    }

    // Launch claude with appropriate flag
    let err = if force_create || reset_mode || !session_exists {
        // Save new session to database before launching
        if !session_exists {
            save_session(&session_uuid);
        }
        println!("Creating session...");
        Command::new("claude")
            .args(["--session-id", &session_uuid])
            .exec()
    } else {
        println!("Resuming session...");
        Command::new("claude")
            .args(["-r", &session_uuid])
            .exec()
    };

    // If we get here, exec failed
    eprintln!("Error launching claude: {}", err);
    exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid5_deterministic() {
        // Same input should always produce same output
        let uuid1 = generate_uuid5("my-project+main");
        let uuid2 = generate_uuid5("my-project+main");
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn test_uuid5_different_inputs() {
        // Different inputs should produce different outputs
        let uuid1 = generate_uuid5("my-project+main");
        let uuid2 = generate_uuid5("my-project+feature/auth");
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_uuid5_format() {
        // UUID should be in standard format: 8-4-4-4-12
        let uuid = generate_uuid5("test");
        let parts: Vec<&str> = uuid.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    #[test]
    fn test_uuid5_version_bits() {
        // UUID v5 should have version 5 in the 13th character (index 14 with hyphens)
        let uuid = generate_uuid5("test");
        let chars: Vec<char> = uuid.chars().collect();
        // Position 14 is the version nibble (after 8-4- = 12 chars + 2 hyphens)
        assert_eq!(chars[14], '5', "UUID version should be 5");
    }

    #[test]
    fn test_uuid5_known_value() {
        // Test against a known session name
        let uuid = generate_uuid5("claude-code-resumer+main");
        // This should always produce the same UUID
        assert_eq!(uuid, "afe19c61-d53f-581c-985c-56e9daf4e63d");
    }

    #[test]
    fn test_uuid5_special_characters() {
        // Should handle special characters in branch names
        let uuid1 = generate_uuid5("project+feature/auth");
        let uuid2 = generate_uuid5("project+fix/bug-123");
        let uuid3 = generate_uuid5("project+release@1.0");

        // All should be valid UUIDs
        assert!(uuid1.len() == 36);
        assert!(uuid2.len() == 36);
        assert!(uuid3.len() == 36);

        // All should be different
        assert_ne!(uuid1, uuid2);
        assert_ne!(uuid2, uuid3);
        assert_ne!(uuid1, uuid3);
    }

    #[test]
    fn test_uuid5_empty_components() {
        // Edge case: empty strings should still produce valid UUID
        let uuid = generate_uuid5("+");
        assert_eq!(uuid.len(), 36);
    }

    #[test]
    fn test_get_folder_name() {
        // Should return Ok with some folder name (we're in a valid directory)
        let result = get_folder_name();
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_db_path() {
        let path = get_db_path();
        assert!(path.to_string_lossy().contains(".cs"));
        assert!(path.to_string_lossy().ends_with("sessions"));
    }

    #[test]
    fn test_load_sessions() {
        // Should return a HashSet (may or may not be empty depending on system state)
        let sessions = load_sessions();
        // Just verify it returns a valid HashSet without crashing
        let _ = sessions.len();
    }

    #[test]
    fn test_session_save_and_load() {
        // Create a temp file for testing
        let test_uuid = "test-uuid-12345678-1234-5678-1234-567812345678";

        // Save session
        save_session(test_uuid);

        // Load and verify
        let sessions = load_sessions();
        assert!(sessions.contains(test_uuid));

        // Cleanup: remove the test entry (optional, leaves in db)
    }

    #[test]
    fn test_session_remove() {
        // Test removing a session from the database
        let test_uuid = "test-remove-uuid-aaaa-bbbb-cccc-ddddeeeefffff";

        // Save session first
        save_session(test_uuid);

        // Verify it exists
        let sessions = load_sessions();
        assert!(sessions.contains(test_uuid));

        // Remove it
        remove_session(test_uuid);

        // Verify it's gone
        let sessions_after = load_sessions();
        assert!(!sessions_after.contains(test_uuid));
    }

    #[test]
    fn test_parse_uuid_valid() {
        // Standard UUID format with hyphens
        let result = parse_uuid("6ba7b810-9dad-11d1-80b4-00c04fd430c8");
        assert!(result.is_some());
        let bytes = result.unwrap();
        assert_eq!(bytes[0], 0x6b);
        assert_eq!(bytes[1], 0xa7);
    }

    #[test]
    fn test_parse_uuid_no_hyphens() {
        // UUID without hyphens should also work
        let result = parse_uuid("6ba7b8109dad11d180b400c04fd430c8");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_uuid_invalid() {
        // Too short
        assert!(parse_uuid("6ba7b810").is_none());
        // Invalid characters
        assert!(parse_uuid("zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz").is_none());
    }

    #[test]
    fn test_get_namespace_default() {
        // Without CS_NAMESPACE set, should return default
        env::remove_var("CS_NAMESPACE");
        let ns = get_namespace();
        assert_eq!(ns, DEFAULT_NAMESPACE);
    }
}
