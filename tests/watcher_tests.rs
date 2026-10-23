use wa_ro::models::{Message, WatchFilter};
use wa_ro::store::fixture::FixtureStore;
use wa_ro::watcher::{Watcher, WatcherOptions};

#[test]
fn test_watch_filter_group_name_futsal() {
    let store = FixtureStore::bundled().unwrap();
    let filter = WatchFilter {
        group: Some("futsal".to_string()),
        ..Default::default()
    };
    let options = WatcherOptions {
        filter,
        json: true,
        follow: false,
        interval_ms: 100,
    };

    let watcher = Watcher::new(&store, options);
    let mut output = Vec::new();
    let count = watcher.run(&mut output).unwrap();

    assert_eq!(count, 6, "Expected 6 messages for futsal group");
    let output_str = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    assert_eq!(lines.len(), 6);

    for line in lines {
        let msg: Message = serde_json::from_str(line).expect("Each line must be valid JSON Message");
        assert_eq!(msg.chat_jid, "120363028471928374@g.us");
    }
}

#[test]
fn test_watch_filter_group_jid() {
    let store = FixtureStore::bundled().unwrap();
    let filter = WatchFilter {
        group: Some("120363098765432100@g.us".to_string()),
        ..Default::default()
    };
    let options = WatcherOptions {
        filter,
        json: true,
        follow: false,
        interval_ms: 100,
    };

    let watcher = Watcher::new(&store, options);
    let mut output = Vec::new();
    let count = watcher.run(&mut output).unwrap();

    assert_eq!(count, 5, "Expected 5 messages for school group");
    let output_str = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    assert_eq!(lines.len(), 5);

    for line in lines {
        let msg: Message = serde_json::from_str(line).expect("Each line must be valid JSON Message");
        assert!(msg.chat_name.unwrap().contains("Escola"));
    }
}

#[test]
fn test_watch_filter_query_keyword() {
    let store = FixtureStore::bundled().unwrap();
    let filter = WatchFilter {
        query: Some("reunião".to_string()),
        ..Default::default()
    };
    let options = WatcherOptions {
        filter,
        json: false,
        follow: false,
        interval_ms: 100,
    };

    let watcher = Watcher::new(&store, options);
    let mut output = Vec::new();
    let count = watcher.run(&mut output).unwrap();

    assert_eq!(count, 1);
    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("Aviso: a reunião de pais foi remarcada"));
}

#[test]
fn test_watch_limit() {
    let store = FixtureStore::bundled().unwrap();
    let filter = WatchFilter {
        group: Some("futsal".to_string()),
        limit: Some(2),
        ..Default::default()
    };
    let options = WatcherOptions {
        filter,
        json: true,
        follow: false,
        interval_ms: 100,
    };

    let watcher = Watcher::new(&store, options);
    let mut output = Vec::new();
    let count = watcher.run(&mut output).unwrap();

    assert_eq!(count, 2);
    let output_str = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();
    assert_eq!(lines.len(), 2);
}
