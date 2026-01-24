use sha1::{Digest, Sha1};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, exit};

// Unix-specific import for exec()
#[cfg(unix)]
use std::os::unix::process::CommandExt;

/// Claude CLI boolean flags (no value required)
const CLAUDE_BOOL_FLAGS: &[&str] = &[
    "--allow-dangerously-skip-permissions",
    "--chrome",
    "-c", "--continue",
    "--dangerously-skip-permissions",
    "--disable-slash-commands",
    "--fork-session",
    "--ide",
    "--include-partial-messages",
    "--mcp-debug",
    "--no-chrome",
    "--no-session-persistence",
    "-p", "--print",
    "--replay-user-messages",
    "--strict-mcp-config",
    "--verbose",
];

/// Claude CLI options that take a value
const CLAUDE_VALUE_FLAGS: &[&str] = &[
    "--add-dir",
    "--agent",
    "--agents",
    "--allowed-tools", "--allowedTools",
    "--append-system-prompt",
    "--betas",
    "-d", "--debug",
    "--disallowed-tools", "--disallowedTools",
    "--fallback-model",
    "--file",
    "--input-format",
    "--json-schema",
    "--max-budget-usd",
    "--mcp-config",
    "--model",
    "--output-format",
    "--permission-mode",
    "--plugin-dir",
    "--setting-sources",
    "--settings",
    "--system-prompt",
    "--tools",
];

/// Claude CLI subcommands (bypass session logic entirely)
const CLAUDE_SUBCOMMANDS: &[&str] = &[
    "doctor", "install", "mcp", "plugin", "setup-token", "update",
];

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

/// Get the user's home directory (cross-platform)
fn get_home_dir() -> Option<PathBuf> {
    // Try HOME first (Unix, and sometimes set on Windows)
    if let Ok(home) = env::var("HOME") {
        return Some(PathBuf::from(home));
    }
    // Try USERPROFILE (Windows)
    if let Ok(profile) = env::var("USERPROFILE") {
        return Some(PathBuf::from(profile));
    }
    // Fallback: try to construct from HOMEDRIVE + HOMEPATH (Windows)
    if let (Ok(drive), Ok(path)) = (env::var("HOMEDRIVE"), env::var("HOMEPATH")) {
        return Some(PathBuf::from(format!("{}{}", drive, path)));
    }
    None
}

/// Get the path to the sessions database file (~/.cs/sessions)
/// Can be overridden with CS_DB_PATH environment variable (useful for testing)
fn get_db_path() -> PathBuf {
    if let Ok(custom_path) = env::var("CS_DB_PATH") {
        return PathBuf::from(custom_path);
    }
    let home = get_home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".cs").join("sessions")
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
    hasher.update(namespace);
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

/// Get the binary name for current platform
fn get_binary_name() -> Option<&'static str> {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return Some("cs-windows-x64.exe");
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    return Some("cs-windows-arm64.exe");

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return Some("cs-macos-arm64");
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return Some("cs-macos-intel");

    #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
    return Some("cs-linux-x64");
    #[cfg(all(target_os = "linux", target_arch = "aarch64", target_env = "gnu"))]
    return Some("cs-linux-arm64");
    #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "musl"))]
    return Some("cs-linux-x64-musl");
    #[cfg(all(target_os = "linux", target_arch = "x86", target_env = "musl"))]
    return Some("cs-linux-i686-musl");

    #[cfg(all(target_os = "freebsd", target_arch = "x86_64"))]
    return Some("cs-freebsd-x64");

    #[cfg(all(target_os = "android", target_arch = "aarch64"))]
    return Some("cs-android-arm64");
    #[cfg(all(target_os = "android", target_arch = "arm"))]
    return Some("cs-android-arm32");
    #[cfg(all(target_os = "android", target_arch = "x86_64"))]
    return Some("cs-android-x64");

    #[allow(unreachable_code)]
    None
}

/// Get the path to the current executable
fn get_current_exe_path() -> Result<PathBuf, String> {
    env::current_exe().map_err(|e| format!("Failed to get current executable path: {}", e))
}

/// Perform self-update by downloading latest release from GitHub
fn self_update() -> Result<(), String> {
    let binary_name = get_binary_name()
        .ok_or_else(|| "Unsupported platform for auto-update".to_string())?;

    let download_url = format!(
        "https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/{}",
        binary_name
    );

    let current_exe = get_current_exe_path()?;
    let current_version = env!("CS_VERSION");

    println!("cs self-update");
    println!("──────────────────────────────────────────────");
    println!("Current version: {}", current_version);
    println!("Binary: {}", binary_name);
    println!("Downloading from: {}", download_url);
    println!();

    // Create temp file path
    let temp_path = current_exe.with_extension("new");

    // Download using platform-appropriate method
    #[cfg(windows)]
    let download_result = download_windows(&download_url, &temp_path);
    #[cfg(not(windows))]
    let download_result = download_unix(&download_url, &temp_path);

    download_result?;

    // Verify the download succeeded and file exists
    if !temp_path.exists() {
        return Err("Download failed: file not created".to_string());
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    // Replace the current binary
    // On Windows, we can't replace a running executable directly
    #[cfg(windows)]
    {
        let backup_path = current_exe.with_extension("old");
        // Remove old backup if exists
        let _ = fs::remove_file(&backup_path);
        // Rename current to backup
        fs::rename(&current_exe, &backup_path)
            .map_err(|e| format!("Failed to backup current binary: {}", e))?;
        // Rename new to current
        fs::rename(&temp_path, &current_exe)
            .map_err(|e| format!("Failed to install new binary: {}", e))?;
        // Remove backup
        let _ = fs::remove_file(&backup_path);
    }

    #[cfg(not(windows))]
    {
        fs::rename(&temp_path, &current_exe)
            .map_err(|e| format!("Failed to replace binary: {}", e))?;
    }

    println!("✓ Successfully updated!");
    println!();
    println!("Run 'cs --version' to verify the new version.");

    Ok(())
}

/// Download file using curl or wget (Unix)
#[cfg(not(windows))]
fn download_unix(url: &str, dest: &std::path::Path) -> Result<(), String> {
    // Try curl first
    let curl_result = Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(dest)
        .arg(url)
        .output();

    if let Ok(output) = curl_result {
        if output.status.success() {
            return Ok(());
        }
    }

    // Fall back to wget
    let wget_result = Command::new("wget")
        .args(["-q", "-O"])
        .arg(dest)
        .arg(url)
        .output();

    match wget_result {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(format!(
            "Download failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )),
        Err(_) => Err("Neither curl nor wget available for download".to_string()),
    }
}

/// Download file using PowerShell (Windows)
#[cfg(windows)]
fn download_windows(url: &str, dest: &std::path::Path) -> Result<(), String> {
    let dest_str = dest.to_string_lossy();
    let ps_command = format!(
        "Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
        url, dest_str
    );

    let result = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_command])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

    if result.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Download failed: {}",
            String::from_utf8_lossy(&result.stderr)
        ))
    }
}

fn print_help() {
    eprintln!("cs - Claude Code Session Manager");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    cs              Start/resume session (folder+branch or folder-only)");
    eprintln!("    cs --force      Force create new session (ignore database)");
    eprintln!("    cs --reset      Remove session from database and create new");
    eprintln!("    cs --resume     Resume using Claude's picker (fallback if not found)");
    eprintln!("    cs --list       List all sessions in database");
    eprintln!("    cs --clear      Clear entire session database");
    eprintln!("    cs --dry-run    Show session info without launching Claude");
    eprintln!("    cs upgrade      Update cs to the latest version");
    eprintln!("    cs --help       Show this help message");
    eprintln!("    cs --version    Show version");
    eprintln!();
    eprintln!("SHORT FLAGS:");
    eprintln!("    -f              Same as --force");
    eprintln!("    -R              Same as --resume");
    eprintln!("    -l              Same as --list");
    eprintln!("    -n              Same as --dry-run");
    eprintln!("    -h              Same as --help");
    eprintln!("    -v              Same as --version");
    eprintln!("    -U              Same as upgrade");
    eprintln!();
    eprintln!("CLAUDE CODE OPTIONS:");
    eprintln!("    All Claude Code CLI options are passed through:");
    eprintln!("    --chrome, --model <m>, --verbose, -c, -p, etc.");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("    cs --chrome              # Enable Chrome integration");
    eprintln!("    cs --model opus          # Use opus model");
    eprintln!("    cs -f --verbose          # Force new session + verbose mode");
    eprintln!("    cs doctor                # Run claude doctor (bypass session)");
    eprintln!();
    eprintln!("SESSION FORMAT:");
    eprintln!("    Git repo:    <folder>+<branch> -> deterministic UUID v5");
    eprintln!("    Non-git:     <folder> -> deterministic UUID v5 (folder-only)");
    eprintln!("    Example: my-project+feature/auth -> 4b513bfa-8c71-512b-...");
    eprintln!("    Example: my-folder -> a1b2c3d4-e5f6-5789-...");
    eprintln!();
    eprintln!("TROUBLESHOOTING:");
    eprintln!("    If you see \"No conversation found\" error:");
    eprintln!("        cs --resume  # Use Claude's picker to find/select session");
    eprintln!("        cs --reset   # Clears stale entry and creates fresh session");
    eprintln!();
    eprintln!("ENVIRONMENT VARIABLES:");
    eprintln!("    CS_NAMESPACE    Custom UUID v5 namespace (default: DNS namespace)");
    eprintln!("                    Example: export CS_NAMESPACE=\"your-custom-uuid-here\"");
    eprintln!();
    eprintln!("FILES:");
    eprintln!("    ~/.cs/sessions  Session database (one UUID per line)");
    eprintln!("                    (Windows: %USERPROFILE%\\.cs\\sessions)");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Track mode flags
    let mut dry_run = false;
    let mut force_create = false;
    let mut reset_mode = false;
    let mut resume_mode = false;
    let mut passthrough_args: Vec<String> = Vec::new();

    // Check for Claude subcommands first - pass entire command through (bypass session logic)
    if args.len() > 1 && CLAUDE_SUBCOMMANDS.contains(&args[1].as_str()) {
        let claude_args: Vec<String> = args[1..].to_vec();
        launch_claude_owned(claude_args);
    }

    // Parse arguments with index-based loop to handle value flags
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            // cs-specific flags (early exit)
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--version" | "-v" => {
                println!("cs {}", env!("CS_VERSION"));
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
            "upgrade" | "-U" => {
                match self_update() {
                    Ok(_) => return,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        exit(1);
                    }
                }
            }

            // cs-specific mode flags
            "--dry-run" | "-n" => {
                dry_run = true;
            }
            "--force" | "-f" => {
                force_create = true;
            }
            "--reset" => {
                reset_mode = true;
            }
            "--resume" | "-R" => {
                resume_mode = true;
            }

            // Blocked flags (conflict with cs session management)
            "--session-id" => {
                eprintln!("Error: '--session-id' conflicts with cs session management");
                eprintln!("cs automatically manages session IDs based on folder+branch");
                exit(1);
            }

            // Check for Claude boolean flags
            _ if CLAUDE_BOOL_FLAGS.contains(&arg.as_str()) => {
                passthrough_args.push(arg.clone());
            }

            // Check for Claude value flags
            _ if CLAUDE_VALUE_FLAGS.contains(&arg.as_str()) => {
                passthrough_args.push(arg.clone());
                i += 1;
                if i < args.len() {
                    passthrough_args.push(args[i].clone());
                } else {
                    eprintln!("Error: '{}' requires a value", arg);
                    exit(1);
                }
            }

            // Handle --flag=value syntax
            _ if arg.contains('=') => {
                let key = arg.split('=').next().unwrap();
                if CLAUDE_VALUE_FLAGS.contains(&key) || CLAUDE_BOOL_FLAGS.contains(&key) {
                    passthrough_args.push(arg.clone());
                } else {
                    eprintln!("Unknown argument: {}", arg);
                    eprintln!("Run 'cs --help' for cs options");
                    eprintln!("Run 'claude --help' for Claude options");
                    exit(1);
                }
            }

            // Positional argument (prompt) - pass through to Claude
            _ if !arg.starts_with('-') => {
                passthrough_args.push(arg.clone());
            }

            // Unknown flag
            _ => {
                eprintln!("Unknown argument: {}", arg);
                eprintln!("Run 'cs --help' for cs options");
                eprintln!("Run 'claude --help' for Claude options");
                exit(1);
            }
        }
        i += 1;
    }

    // Get folder name
    let folder_name = match get_folder_name() {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    // Get git branch (optional - fall back to folder-only if not in a git repo)
    let (session_name, is_git_repo) = match get_git_branch() {
        Ok(branch_name) => (format!("{}+{}", folder_name, branch_name), true),
        Err(_) => (folder_name.clone(), false),
    };
    let session_uuid = generate_uuid5(&session_name);

    // Handle reset mode: remove existing entry from database
    if reset_mode {
        remove_session(&session_uuid);
    }

    // Load session database (after potential reset)
    let sessions = load_sessions();
    let session_exists = sessions.contains(&session_uuid);

    // Determine effective status for display
    let status_display = if resume_mode {
        "resume-with-picker"
    } else if force_create || reset_mode {
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
    if !is_git_repo {
        println!("│ Note:    Not a git repo (folder-only mode)");
    }
    println!("└─────────────────────────────────────────────");
    println!();

    // Check for dry-run
    if dry_run {
        if !passthrough_args.is_empty() {
            println!("Passthrough args: {:?}", passthrough_args);
        }
        return;
    }

    // Determine which arguments to use
    let mut claude_args: Vec<String> = if resume_mode {
        println!("Resuming session (with picker fallback)...");
        vec!["--resume".to_string(), session_uuid.clone()]
    } else if force_create || reset_mode || !session_exists {
        if !session_exists {
            save_session(&session_uuid);
        }
        println!("Creating session...");
        vec!["--session-id".to_string(), session_uuid.clone()]
    } else {
        println!("Resuming session...");
        vec!["-r".to_string(), session_uuid.clone()]
    };

    // Append passthrough args
    claude_args.extend(passthrough_args);

    // Launch claude (platform-specific)
    launch_claude_owned(claude_args);
}

/// Check if claude CLI is installed
fn check_claude_installed() -> bool {
    #[cfg(windows)]
    let check_cmd = Command::new("where").arg("claude").output();
    #[cfg(not(windows))]
    let check_cmd = Command::new("which").arg("claude").output();

    match check_cmd {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Print error message when Claude CLI is not found
fn print_claude_not_found_error() {
    eprintln!("Error: Claude CLI not found in PATH");
    eprintln!();
    eprintln!("Claude Code CLI must be installed to use cs.");
    eprintln!();
    eprintln!("Install Claude Code:");
    eprintln!("  npm install -g @anthropic-ai/claude-code");
    eprintln!();
    eprintln!("Or visit: https://docs.anthropic.com/en/docs/claude-code");
}

/// Launch claude with the given arguments (Unix version - replaces current process)
#[cfg(unix)]
#[allow(dead_code)]
fn launch_claude(args: &[&str]) -> ! {
    // Check if claude exists before replacing the process
    if !check_claude_installed() {
        print_claude_not_found_error();
        exit(127);
    }

    let err = Command::new("claude").args(args).exec();

    // If we get here, the exec call failed
    if err.kind() == std::io::ErrorKind::NotFound {
        print_claude_not_found_error();
        exit(127);
    }

    eprintln!("Error launching claude: {}", err);
    exit(1);
}

/// Launch claude with owned String arguments (Unix version)
/// Uses exec() to replace the current process - args are passed as array, not shell string
#[cfg(unix)]
fn launch_claude_owned(args: Vec<String>) -> ! {
    // Check if claude exists before replacing the process
    if !check_claude_installed() {
        print_claude_not_found_error();
        exit(127);
    }

    let err = Command::new("claude").args(&args).exec();

    // If we get here, the exec call failed
    if err.kind() == std::io::ErrorKind::NotFound {
        print_claude_not_found_error();
        exit(127);
    }

    eprintln!("Error launching claude: {}", err);
    exit(1);
}

/// Launch claude with the given arguments (Windows version - spawns child process)
#[cfg(windows)]
#[allow(dead_code)]
fn launch_claude(args: &[&str]) -> ! {
    match Command::new("claude").args(args).spawn() {
        Ok(mut child) => {
            match child.wait() {
                Ok(status) => exit(status.code().unwrap_or(0)),
                Err(e) => {
                    eprintln!("Error waiting for claude: {}", e);
                    exit(1);
                }
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                print_claude_not_found_error();
                exit(127);
            }
            eprintln!("Error launching claude: {}", e);
            exit(1);
        }
    }
}

/// Launch claude with owned String arguments (Windows version)
#[cfg(windows)]
fn launch_claude_owned(args: Vec<String>) -> ! {
    match Command::new("claude").args(&args).spawn() {
        Ok(mut child) => {
            match child.wait() {
                Ok(status) => exit(status.code().unwrap_or(0)),
                Err(e) => {
                    eprintln!("Error waiting for claude: {}", e);
                    exit(1);
                }
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                print_claude_not_found_error();
                exit(127);
            }
            eprintln!("Error launching claude: {}", e);
            exit(1);
        }
    }
}

#[cfg(test)]
mod tests;
