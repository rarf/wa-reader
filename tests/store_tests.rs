use wa_ro::store::fixture::FixtureStore;
use wa_ro::store::ReadOnlyStore;

#[test]
fn test_fixture_store_crud() {
    let store = FixtureStore::bundled().unwrap();

    let auth = store.auth_status().unwrap();
    assert!(auth.is_authenticated);

    let chats = store.list_chats(None, 10).unwrap();
    assert_eq!(chats.len(), 5);

    let groups = store.list_groups(None, 10).unwrap();
    assert_eq!(groups.len(), 3);

    let msgs = store.list_messages(None, 100).unwrap();
    assert_eq!(msgs.len(), 15);

    let search_results = store.search_messages("futsal", None, 10).unwrap();
    assert!(!search_results.is_empty());

    let get_res = store.get_message("MSG-FUT-001").unwrap();
    assert!(get_res.is_some());
    assert_eq!(get_res.unwrap().sender_name.unwrap(), "Carlos Silva");

    let not_found = store.get_message("DOES_NOT_EXIST").unwrap();
    assert!(not_found.is_none());
}

#[cfg(feature = "sqlite")]
#[test]
fn test_sqlite_store_operations() {
    use wa_ro::store::sqlite::SqliteStore;

    let temp_dir = std::env::temp_dir().join(format!("wa_ro_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("test.db");

    let sqlite_store = SqliteStore::open(&db_path).unwrap();
    let fixture_store = FixtureStore::bundled().unwrap();

    sqlite_store.seed_from_fixture(&fixture_store).unwrap();

    let auth = sqlite_store.auth_status().unwrap();
    assert!(auth.is_authenticated);

    let chats = sqlite_store.list_chats(None, 10).unwrap();
    assert_eq!(chats.len(), 5);

    let groups = sqlite_store.list_groups(None, 10).unwrap();
    assert_eq!(groups.len(), 3);

    let msgs = sqlite_store.list_messages(None, 100).unwrap();
    assert_eq!(msgs.len(), 15);

    let search = sqlite_store.search_messages("ATL", None, 10).unwrap();
    assert_eq!(search.len(), 1);
    assert!(search[0].text.contains("não há ATL"));

    let msg = sqlite_store.get_message("MSG-ESC-004").unwrap();
    assert!(msg.is_some());

    let _ = std::fs::remove_dir_all(&temp_dir);
}
