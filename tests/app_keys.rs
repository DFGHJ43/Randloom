//! App state machine tests: keys, tab order, editing, generation, scrolling.

use randloom::app::AppState;
use randloom::config::{DistType, FieldId};
use randloom::dist::uniform;
use randloom::input::{Flow, Key};
use randloom::rng::Rng;

const VISIBLE_ROWS: usize = 4;

fn press(state: &mut AppState, key: Key) -> Flow {
    state.handle_key(key, VISIBLE_ROWS)
}

#[test]
fn generate_uses_the_configured_seed_and_count() {
    let mut state = AppState::new();
    state.set_field_text(FieldId::Count, "5");
    state.set_field_text(FieldId::Seed, "42");
    state.set_field_text(FieldId::Min, "0");
    state.set_field_text(FieldId::Max, "100");

    let flow = press(&mut state, Key::Char('g'));
    assert_eq!(flow, Flow::Continue);
    assert_eq!(state.results.len(), 5);
    assert_eq!(state.result_scroll, 0);
    assert!(state.status.contains("Seed 42"));
    assert!(!state.status.contains("(auto)"));

    let mut rng = Rng::from_seed(42);
    let expected: Vec<f64> = (0..5)
        .map(|_| uniform::sample(&mut rng, 0, 100) as f64)
        .collect();
    assert_eq!(state.results, expected);
}

#[test]
fn auto_seed_is_reported_in_status() {
    let mut state = AppState::new();
    state.set_field_text(FieldId::Count, "3");
    state.set_field_text(FieldId::Seed, "0");
    press(&mut state, Key::Char('G'));

    assert_eq!(state.results.len(), 3);
    assert!(state.status.contains("(auto)"));
}

#[test]
fn distribution_switches_reset_focus_and_set_status() {
    let mut state = AppState::new();
    state.focus = FieldId::Max;
    press(&mut state, Key::Char('n'));

    assert_eq!(state.dist, DistType::Normal);
    assert_eq!(state.focus, FieldId::Count);
    assert_eq!(state.status, "Switched to Normal.");
}

#[test]
fn tab_order_skips_invisible_fields_for_uniform() {
    let mut state = AppState::new();
    assert_eq!(state.focus, FieldId::Count);

    press(&mut state, Key::Tab);
    assert_eq!(state.focus, FieldId::Seed);
    press(&mut state, Key::Tab);
    assert_eq!(state.focus, FieldId::Min);
    press(&mut state, Key::Tab);
    assert_eq!(state.focus, FieldId::Max);
    press(&mut state, Key::Tab);
    assert_eq!(state.focus, FieldId::Count);
}

#[test]
fn tab_order_skips_invisible_fields_for_normal() {
    let mut state = AppState::new();
    press(&mut state, Key::Char('n'));

    press(&mut state, Key::Tab);
    assert_eq!(state.focus, FieldId::Seed);
    press(&mut state, Key::Tab);
    assert_eq!(state.focus, FieldId::Mean);
    press(&mut state, Key::Tab);
    assert_eq!(state.focus, FieldId::StdDev);
    press(&mut state, Key::Tab);
    assert_eq!(state.focus, FieldId::Count);
}

#[test]
fn seed_field_accepts_digits_only() {
    let mut state = AppState::new();
    state.set_field_text(FieldId::Seed, "");
    state.focus = FieldId::Seed;

    press(&mut state, Key::Char('a'));
    press(&mut state, Key::Char('-'));
    press(&mut state, Key::Char('.'));
    assert_eq!(state.fields[FieldId::Seed as usize].text, "");

    press(&mut state, Key::Char('1'));
    press(&mut state, Key::Char('2'));
    assert_eq!(state.fields[FieldId::Seed as usize].text, "12");
    assert_eq!(state.params.seed, 12);
}

#[test]
fn editing_and_backspace_update_the_buffer_and_params() {
    let mut state = AppState::new();
    state.set_field_text(FieldId::Count, "");
    state.focus = FieldId::Count;

    press(&mut state, Key::Char('5'));
    press(&mut state, Key::Char('0'));
    assert_eq!(state.fields[FieldId::Count as usize].text, "50");

    press(&mut state, Key::Backspace);
    assert_eq!(state.fields[FieldId::Count as usize].text, "5");

    press(&mut state, Key::Left);
    assert_eq!(state.fields[FieldId::Count as usize].cursor, 0);
    press(&mut state, Key::Char('1'));
    assert_eq!(state.fields[FieldId::Count as usize].text, "15");
}

#[test]
fn down_and_up_clamp_scrolling() {
    let mut state = AppState::new();
    state.results = (0..10).map(|value| value as f64).collect();

    for _ in 0..20 {
        press(&mut state, Key::Down);
    }
    assert_eq!(state.result_scroll, 6);

    for _ in 0..20 {
        press(&mut state, Key::Up);
    }
    assert_eq!(state.result_scroll, 0);
}

#[test]
fn quit_keys_stop_the_app() {
    for key in [Key::Char('q'), Key::Char('Q'), Key::Esc, Key::CtrlC] {
        let mut state = AppState::new();
        assert_eq!(press(&mut state, key), Flow::Quit);
        assert!(!state.running);
    }
}

#[test]
fn export_without_results_only_updates_status() {
    let mut state = AppState::new();
    press(&mut state, Key::Char('e'));
    assert_eq!(state.status, "Nothing to export!");
}

#[test]
fn enter_is_a_no_op() {
    let mut state = AppState::new();
    let before = state.status.clone();
    assert_eq!(press(&mut state, Key::Enter), Flow::Continue);
    assert_eq!(state.status, before);
}
