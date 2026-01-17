#![cfg(test)]

use super::*;
use serial_test::serial;
use tempfile::TempDir;

/// Helper to create an isolated test environment with its own database
struct TestEnv {
    _temp_dir: TempDir,
}

impl TestEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("sessions");
        std::env::set_var("CS_DB_PATH", db_path.to_string_lossy().to_string());
        TestEnv { _temp_dir: temp_dir }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        std::env::remove_var("CS_DB_PATH");
    }
}

// ============================================================================
// UUID generation tests (no env var dependencies, can run in parallel)
// ============================================================================

#[test]
fn test_uuid5_deterministic() {
    let uuid1 = generate_uuid5("my-project+main");
    let uuid2 = generate_uuid5("my-project+main");
    assert_eq!(uuid1, uuid2);
}

#[test]
fn test_uuid5_different_inputs() {
    let uuid1 = generate_uuid5("my-project+main");
    let uuid2 = generate_uuid5("my-project+feature/auth");
    assert_ne!(uuid1, uuid2);
}

#[test]
fn test_uuid5_format() {
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
    let uuid = generate_uuid5("test");
    let chars: Vec<char> = uuid.chars().collect();
    assert_eq!(chars[14], '5', "UUID version should be 5");
}

#[test]
fn test_uuid5_known_value() {
    let uuid = generate_uuid5("claude-code-resumer+main");
    assert_eq!(uuid, "afe19c61-d53f-581c-985c-56e9daf4e63d");
}

#[test]
fn test_uuid5_special_characters() {
    let uuid1 = generate_uuid5("project+feature/auth");
    let uuid2 = generate_uuid5("project+fix/bug-123");
    let uuid3 = generate_uuid5("project+release@1.0");

    assert!(uuid1.len() == 36);
    assert!(uuid2.len() == 36);
    assert!(uuid3.len() == 36);

    assert_ne!(uuid1, uuid2);
    assert_ne!(uuid2, uuid3);
    assert_ne!(uuid1, uuid3);
}

#[test]
fn test_uuid5_empty_components() {
    let uuid = generate_uuid5("+");
    assert_eq!(uuid.len(), 36);
}

// ============================================================================
// UUID parsing tests (no env var dependencies)
// ============================================================================

#[test]
fn test_parse_uuid_valid() {
    let result = parse_uuid("6ba7b810-9dad-11d1-80b4-00c04fd430c8");
    assert!(result.is_some());
    let bytes = result.unwrap();
    assert_eq!(bytes[0], 0x6b);
    assert_eq!(bytes[1], 0xa7);
}

#[test]
fn test_parse_uuid_no_hyphens() {
    let result = parse_uuid("6ba7b8109dad11d180b400c04fd430c8");
    assert!(result.is_some());
}

#[test]
fn test_parse_uuid_invalid() {
    assert!(parse_uuid("6ba7b810").is_none());
    assert!(parse_uuid("zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz").is_none());
}

// ============================================================================
// Folder name test (no env var dependencies)
// ============================================================================

#[test]
fn test_get_folder_name() {
    let result = get_folder_name();
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

// ============================================================================
// Environment-dependent tests (must run serially)
// ============================================================================

#[test]
#[serial]
fn test_db_path_default() {
    std::env::remove_var("CS_DB_PATH");
    let path = get_db_path();
    assert!(path.to_string_lossy().contains(".cs"));
    assert!(path.to_string_lossy().ends_with("sessions"));
}

#[test]
#[serial]
fn test_db_path_override() {
    let custom_path = "/tmp/test-cs-sessions";
    std::env::set_var("CS_DB_PATH", custom_path);
    let path = get_db_path();
    assert_eq!(path.to_string_lossy(), custom_path);
    std::env::remove_var("CS_DB_PATH");
}

#[test]
#[serial]
fn test_get_namespace_default() {
    std::env::remove_var("CS_NAMESPACE");
    let ns = get_namespace();
    assert_eq!(ns, DEFAULT_NAMESPACE);
}

#[test]
#[serial]
fn test_get_namespace_custom() {
    let custom_ns = "12345678-1234-1234-1234-123456789012";
    std::env::set_var("CS_NAMESPACE", custom_ns);
    let ns = get_namespace();
    assert_eq!(ns[0], 0x12);
    std::env::remove_var("CS_NAMESPACE");
}

// ============================================================================
// Session database tests (use isolated temp dirs, must run serially)
// ============================================================================

#[test]
#[serial]
fn test_load_sessions_empty() {
    let _env = TestEnv::new();
    let sessions = load_sessions();
    assert!(sessions.is_empty());
}

#[test]
#[serial]
fn test_session_save_and_load() {
    let _env = TestEnv::new();
    let test_uuid = "test-uuid-12345678-1234-5678-1234-567812345678";

    save_session(test_uuid);

    let sessions = load_sessions();
    assert!(sessions.contains(test_uuid));
}

#[test]
#[serial]
fn test_session_save_multiple() {
    let _env = TestEnv::new();
    let uuid1 = "uuid-1111-1111-1111-111111111111";
    let uuid2 = "uuid-2222-2222-2222-222222222222";
    let uuid3 = "uuid-3333-3333-3333-333333333333";

    save_session(uuid1);
    save_session(uuid2);
    save_session(uuid3);

    let sessions = load_sessions();
    assert_eq!(sessions.len(), 3);
    assert!(sessions.contains(uuid1));
    assert!(sessions.contains(uuid2));
    assert!(sessions.contains(uuid3));
}

#[test]
#[serial]
fn test_session_remove() {
    let _env = TestEnv::new();
    let test_uuid = "test-remove-uuid-aaaa-bbbb-cccc-ddddeeeefffff";

    save_session(test_uuid);

    let sessions = load_sessions();
    assert!(sessions.contains(test_uuid), "Session should exist after save");

    remove_session(test_uuid);

    let sessions_after = load_sessions();
    assert!(!sessions_after.contains(test_uuid), "Session should be removed");
}

#[test]
#[serial]
fn test_session_remove_preserves_others() {
    let _env = TestEnv::new();
    let keep_uuid = "uuid-keep-1111-2222-333344445555";
    let remove_uuid = "uuid-remove-aaaa-bbbb-ccccddddeeee";

    save_session(keep_uuid);
    save_session(remove_uuid);

    remove_session(remove_uuid);

    let sessions = load_sessions();
    assert!(sessions.contains(keep_uuid), "Other session should remain");
    assert!(!sessions.contains(remove_uuid), "Removed session should be gone");
}

#[test]
#[serial]
fn test_session_remove_nonexistent() {
    let _env = TestEnv::new();
    remove_session("nonexistent-uuid");
    let sessions = load_sessions();
    assert!(sessions.is_empty());
}
