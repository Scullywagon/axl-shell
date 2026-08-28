use crate::spoke::input::Input;

#[test]
fn new_starts_empty_with_no_draft() {
    let input = Input::new();

    assert!(input.is_empty());
    assert_eq!(input.get_cursor(), 0);
}

#[test]
fn push_inserts_at_cursor_and_advances() {
    let mut input = Input::new();

    input.push('L');

    assert!(!input.is_empty());
    assert_eq!(input.get_cursor(), 1);
}

#[test]
fn push_string_inserts_at_cursor_and_advances_by_string_length() {
    let mut input = Input::new();

    input.push_string("word").unwrap();

    assert!(!input.is_empty());
    assert_eq!(input.get_cursor(), 4);
}

#[test]
fn backspace_removes_character_before_cursor() {
    let mut input = Input::new();

    input.push_string("word").unwrap();
    input.backspace();

    assert_eq!(input.get_active(), "wor".to_string());
}

#[test]
fn delete_removes_character_at_cursor() {
    let mut input = Input::new();

    input.push_string("word").unwrap();
    input.cursor_left();
    input.cursor_left();
    input.delete();

    assert_eq!(input.get_active(), "wod".to_string());
}

#[test]
fn delete_is_noop_at_end_of_buffer() {
    let mut input = Input::new();

    input.push_string("word").unwrap();
    input.delete();

    assert_eq!(input.get_active(), "word".to_string());
}

#[test]
fn cursor_left_moves_only_when_possible() {
    let mut input = Input::new();

    input.push_string("w").unwrap();
    input.cursor_left();
    input.cursor_left();

    assert_eq!(input.get_cursor(), 0);
}

#[test]
fn cursor_right_moves_only_when_possible() {
    let mut input = Input::new();

    input.push_string("w").unwrap();
    input.cursor_right();

    assert_eq!(input.get_cursor(), 1);
}

#[test]
fn draft_round_trip_saves_and_restores_active_buffer() {
    let mut input = Input::new();

    input.push_string("original").unwrap();
    input.save_active_as_draft();

    input.set_active("edited".to_string());
    assert_eq!(input.get_active(), "edited".to_string());
    assert!(input.has_draft());

    input.draft_as_active();

    assert_eq!(input.get_active(), "original".to_string());
    assert!(!input.has_draft());
}
