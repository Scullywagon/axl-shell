use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use super::super::history::History;

#[test]
fn new_history_reads_history_file() {
    let home = temp_home_dir();
    let history_path = home.join(".axl_history");
    fs::write(&history_path, "first command\nsecond command\n").unwrap();

    let mut history = History::new(Some(home.clone())).unwrap();

    assert!(history.at_end());
    assert_eq!(history.get_next(), Some(&"second command".to_owned()));
    assert_eq!(history.get_next(), Some(&"first command".to_owned()));
    assert_eq!(history.get_prev(), Some(&"second command".to_owned()));

    let _ = fs::remove_dir_all(home);
}

#[test]
fn new_history_creates_history_file() {
    let home = temp_home_dir();
    let history_path = home.join(".axl_history");

    assert!(!history_path.exists());

    let history = History::new(Some(home.clone())).unwrap();

    assert!(history_path.exists());
    assert!(history.at_end());

    let _ = fs::remove_dir_all(home);
}

#[test]
fn add_pushes_to_history_file() {
    let home = temp_home_dir();
    let history_path = home.join(".axl_history");

    let mut history = History::new(Some(home.clone())).unwrap();
    history.add("echo hello");

    assert_eq!(history.get_next(), Some(&"echo hello".to_owned()));
    assert_eq!(fs::read_to_string(history_path).unwrap(), "echo hello\n");
    let _ = fs::remove_dir_all(home);
}


fn temp_home_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "axl-shell-history-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
