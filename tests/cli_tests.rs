use std::process::Command;

fn get_bin_path() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    if path.ends_with("deps") {
        path.pop(); // Remove deps
    }
    path.join("wa-ro")
}

#[test]
fn test_cli_help() {
    let output = Command::new(get_bin_path())
        .arg("--help")
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("READ-ONLY"));
    assert!(stdout.contains("watch"));
    assert!(stdout.contains("messages"));
    assert!(!stdout.contains("wa-ro send"));
}

#[test]
fn test_cli_auth_status() {
    let output = Command::new(get_bin_path())
        .args(["auth", "status"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Authenticated / Paired"));
    assert!(stdout.contains("STRICT READ-ONLY"));
}

#[test]
fn test_cli_auth_status_json() {
    let output = Command::new(get_bin_path())
        .args(["auth", "status", "--json"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let val: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output must be valid JSON");
    assert_eq!(val["is_authenticated"], true);
}

#[test]
fn test_cli_sync() {
    let output = Command::new(get_bin_path())
        .args(["sync"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Synchronizing local message store"));
    assert!(stdout.contains("Up-to-date"));
}

#[test]
fn test_cli_chats_list() {
    let output = Command::new(get_bin_path())
        .args(["chats", "list"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Futsal Terça 21h"));
    assert!(stdout.contains("Escola Básica 3º Ano"));
}

#[test]
fn test_cli_groups_list_json() {
    let output = Command::new(get_bin_path())
        .args(["groups", "list", "--json"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let val: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).expect("output must be valid JSON array");
    assert!(!val.is_empty());
    assert!(val.iter().any(|g| g["name"].as_str().unwrap().contains("Futsal")));
}

#[test]
fn test_cli_messages_search_offline() {
    let output = Command::new(get_bin_path())
        .args(["messages", "search", "coletes"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Miguel Rodrigues"));
    assert!(stdout.contains("Eu levo a bola e os coletes"));
}

#[test]
fn test_cli_messages_show() {
    let output = Command::new(get_bin_path())
        .args(["messages", "show", "MSG-ESC-001"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("MSG-ESC-001"));
    assert!(stdout.contains("Delegada Ana"));
    assert!(stdout.contains("reunião de pais"));
}

#[test]
fn test_cli_watch_group_json_success_criteria() {
    // Specifically test the success criterion:
    // `watch --group <name-or-jid> --json` works against fixtures
    let output = Command::new(get_bin_path())
        .args(["watch", "--group", "futsal", "--json"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 6, "Expected 6 JSON lines for futsal group");

    for line in lines {
        let msg: serde_json::Value = serde_json::from_str(line).expect("must parse JSON line");
        assert_eq!(msg["chat_jid"], "120363028471928374@g.us");
    }
}

#[test]
fn test_cli_watch_school_group_json() {
    let output = Command::new(get_bin_path())
        .args(["watch", "--group", "Escola Básica", "--json"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 5, "Expected 5 JSON lines for school group");

    for line in lines {
        let msg: serde_json::Value = serde_json::from_str(line).expect("must parse JSON line");
        assert_eq!(msg["chat_jid"], "120363098765432100@g.us");
    }
}
