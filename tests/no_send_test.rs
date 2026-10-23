use clap::CommandFactory;
use wa_ro::cli::Cli;

#[test]
fn test_no_send_subcommand_in_cli_definition() {
    let cmd = Cli::command();
    // Verify that "send" subcommand does not exist
    assert!(
        cmd.find_subcommand("send").is_none(),
        "CRITICAL INVARIANT VIOLATION: 'send' subcommand found in CLI command definition!"
    );

    // Verify subcommands list only contains read-only verbs
    let subcommands: Vec<&str> = cmd.get_subcommands().map(|s| s.get_name()).collect();
    assert!(subcommands.contains(&"auth"));
    assert!(subcommands.contains(&"sync"));
    assert!(subcommands.contains(&"messages"));
    assert!(subcommands.contains(&"chats"));
    assert!(subcommands.contains(&"groups"));
    assert!(subcommands.contains(&"watch"));
    assert!(!subcommands.contains(&"send"));
    assert!(!subcommands.contains(&"post"));
    assert!(!subcommands.contains(&"write"));
}

#[test]
fn test_help_text_does_not_contain_send_command() {
    let mut cmd = Cli::command();
    let mut help_bytes = Vec::new();
    cmd.write_long_help(&mut help_bytes).unwrap();
    let help_text = String::from_utf8(help_bytes).unwrap();

    assert!(help_text.contains("READ-ONLY"));
    assert!(help_text.contains("CRITICAL SAFETY INVARIANT"));
    // Ensure no command line example or usage advertises sending messages
    assert!(!help_text.contains("wa-ro send"));
}

#[test]
fn test_parsing_send_fails() {
    use clap::Parser;
    let result = Cli::try_parse_from(["wa-ro", "send", "Hello"]);
    assert!(
        result.is_err(),
        "Executing 'wa-ro send' must fail parsing because send is absent"
    );
}
