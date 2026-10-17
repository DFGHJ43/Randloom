//! Headless snapshots for the full TUI frame.
//!
//! State is constructed directly (fields + results + status) so these tests
//! do not depend on the RNG or the clock. Regenerate snapshots with
//! `INSTA_UPDATE=always cargo test --offline --test render_snapshots`.

use randloom::app::AppState;
use randloom::config::{DistType, FieldId, UniformMode};
use randloom::fmt::NumberStyle;
use randloom::ui;

fn uniform_state() -> AppState {
    let mut state = AppState::new();
    state.set_field_text(FieldId::Count, "8");
    state.set_field_text(FieldId::Seed, "42");
    state.set_field_text(FieldId::Min, "0");
    state.set_field_text(FieldId::Max, "100");
    state.dist = DistType::Uniform;
    state.results = vec![58.0, 56.0, 56.0, 50.0, 60.0, 42.0, 17.0, 88.0];
    state.status = "Gen 8 uniform numbers. Seed 42.".to_string();
    state
}

fn uniform_continuous_state() -> AppState {
    let mut state = AppState::new();
    state.set_field_text(FieldId::Count, "8");
    state.set_field_text(FieldId::Seed, "42");
    state.set_field_text(FieldId::Min, "-2.5");
    state.set_field_text(FieldId::Max, "9.5");
    state.params.uniform_mode = UniformMode::Continuous;
    state.dist = DistType::Uniform;
    state.result_style = NumberStyle::Decimal6;
    state.results = vec![
        0.841471,
        -1.234_567_89e-7,
        0.000001,
        9.499999,
        -2.499999,
        3.25,
        7.0,
        0.0,
    ];
    state.status = "Gen 8 continuous uniform numbers. Seed 42.".to_string();
    state
}

fn normal_state() -> AppState {
    let mut state = AppState::new();
    state.set_field_text(FieldId::Count, "8");
    state.set_field_text(FieldId::Seed, "42");
    state.set_field_text(FieldId::Mean, "0.0");
    state.set_field_text(FieldId::StdDev, "1.5");
    state.dist = DistType::Normal;
    state.result_style = NumberStyle::Decimal6;
    state.results = vec![-1.5, -0.75, 0.0, 0.25, 0.5, 1.0, 1.25, 2.5];
    state.status = "Gen 8 normal numbers. Seed 42.".to_string();
    state
}

fn bernoulli_state() -> AppState {
    let mut state = AppState::new();
    state.set_field_text(FieldId::Count, "8");
    state.set_field_text(FieldId::Seed, "42");
    state.set_field_text(FieldId::Prob, "0.35");
    state.dist = DistType::Bernoulli;
    state.results = vec![1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0];
    state.status = "Gen 8 bernoulli numbers. Seed 42.".to_string();
    state
}

fn poisson_state() -> AppState {
    let mut state = AppState::new();
    state.set_field_text(FieldId::Count, "8");
    state.set_field_text(FieldId::Seed, "42");
    state.set_field_text(FieldId::Lambda, "3.0");
    state.dist = DistType::Poisson;
    state.results = vec![2.0, 4.0, 3.0, 1.0, 5.0, 2.0, 3.0, 3.0];
    state.status = "Gen 8 poisson numbers. Seed 42.".to_string();
    state
}

fn state_for(dist: DistType) -> AppState {
    match dist {
        DistType::Uniform => uniform_state(),
        DistType::Normal => normal_state(),
        DistType::Bernoulli => bernoulli_state(),
        DistType::Poisson => poisson_state(),
    }
}

#[test]
fn snapshot_80x24_unicode_all_distributions() {
    for dist in DistType::ALL {
        insta::assert_snapshot!(
            format!("80x24_{}_unicode", dist.name()),
            ui::render_to_string(&state_for(dist), 80, 24, true)
        );
    }
}

#[test]
fn snapshot_60x22_unicode_all_distributions() {
    for dist in DistType::ALL {
        insta::assert_snapshot!(
            format!("60x22_{}_unicode", dist.name()),
            ui::render_to_string(&state_for(dist), 60, 22, true)
        );
    }
}

#[test]
fn snapshot_80x24_ascii_all_distributions() {
    for dist in DistType::ALL {
        insta::assert_snapshot!(
            format!("80x24_{}_ascii", dist.name()),
            ui::render_to_string(&state_for(dist), 80, 24, false)
        );
    }
}

#[test]
fn focused_field_and_status_styles_are_set() {
    let state = normal_state();
    let layout = ui::Layout::compute(80, 24);
    let frame = ui::render(&state, &layout, &ui::GlyphSet::new(true));

    // Count is focused and lives at (lx + 11, y + 6) = (12, 9). The whole
    // value area, including its padding, is reverse video; brackets are not.
    let value = frame.get(12, 9).expect("count value cell");
    assert_eq!(value.ch, '8');
    assert!(value.style.reverse);
    assert!(frame.get(21, 9).expect("count padding cell").style.reverse);
    assert!(!frame.get(22, 9).expect("closing bracket").style.reverse);

    // Seed is visible but not focused.
    let seed = frame.get(12, 10).expect("seed value cell");
    assert_eq!(seed.ch, '4');
    assert!(!seed.style.reverse);

    // The status bar is bold, including its padding; the right border is not.
    let status = frame.get(layout.lx, layout.status_y).expect("status cell");
    assert_eq!(status.ch, 'G');
    assert!(status.style.bold);
    assert!(
        frame
            .get(layout.cols as i32 - 2, layout.status_y)
            .expect("status padding")
            .style
            .bold
    );
    assert!(
        !frame
            .get(layout.cols as i32 - 1, layout.status_y)
            .expect("right border")
            .style
            .bold
    );
}

#[test]
fn snapshot_focused_seed_field() {
    let mut state = normal_state();
    state.focus = FieldId::Seed;
    state.status = "Seed field focused.".to_string();
    insta::assert_snapshot!(
        "80x24_focused_seed",
        ui::render_to_string(&state, 80, 24, true)
    );
}

#[test]
fn snapshot_empty_results() {
    let mut state = AppState::new();
    state.results.clear();
    state.status.clear();
    insta::assert_snapshot!(
        "80x24_empty_results",
        ui::render_to_string(&state, 80, 24, true)
    );
}

#[test]
fn snapshot_1000_results_scrolled() {
    let mut state = normal_state();
    state.results = (0..1000).map(|value| value as f64).collect();
    state.result_scroll = 990;
    state.status = "Gen 1000 normal numbers. Seed 42.".to_string();
    insta::assert_snapshot!(
        "80x24_1000_results_scrolled",
        ui::render_to_string(&state, 80, 24, true)
    );
}

#[test]
fn snapshot_uniform_continuous_layouts() {
    let state = uniform_continuous_state();
    insta::assert_snapshot!(
        "80x24_uniform_continuous_unicode",
        ui::render_to_string(&state, 80, 24, true)
    );
    insta::assert_snapshot!(
        "60x22_uniform_continuous_unicode",
        ui::render_to_string(&state, 60, 22, true)
    );
    insta::assert_snapshot!(
        "80x24_uniform_continuous_ascii",
        ui::render_to_string(&state, 80, 24, false)
    );
}

#[test]
fn uniform_mode_checkbox_tracks_the_mode() {
    let mut state = uniform_state();
    let layout = ui::Layout::compute(80, 24);
    let row = |frame: &ui::Frame, y: i32| -> String {
        (0..layout.lw)
            .map(|x| frame.get(layout.lx + x, y).expect("control row cell").ch)
            .collect()
    };

    let frame = ui::render(&state, &layout, &ui::GlyphSet::new(true));
    assert!(row(&frame, 8).contains("[ ] Continuous (C)"));

    state.params.uniform_mode = UniformMode::Continuous;
    let frame = ui::render(&state, &layout, &ui::GlyphSet::new(true));
    assert!(row(&frame, 8).contains("[x] Continuous (C)"));

    // Other distributions do not show the uniform mode row.
    state.dist = DistType::Normal;
    let frame = ui::render(&state, &layout, &ui::GlyphSet::new(true));
    assert_eq!(row(&frame, 8).trim(), "");
}

#[test]
fn uniform_mode_hint_is_only_drawn_when_it_fits() {
    // Left-panel widths 17, 18 and 19 cannot hold the 18-character
    // "[x] Continuous (C)" row, so the hint must be omitted rather than
    // clipped. 80x24 has a 22-column panel and shows the hint.
    for (cols, rows) in [
        (60, 22),
        (67, 24),
        (68, 24),
        (70, 24),
        (72, 24),
        (74, 24),
        (80, 24),
    ] {
        let layout = ui::Layout::compute(cols, rows);
        let mut state = uniform_state();
        state.params.uniform_mode = UniformMode::Continuous;
        let frame = ui::render(&state, &layout, &ui::GlyphSet::new(true));
        let row: String = (0..layout.lw)
            .map(|x| frame.get(layout.lx + x, 8).expect("mode row cell").ch)
            .collect();
        let row = row.trim_end();

        assert!(row.contains("Continuous"), "{cols}x{rows}: {row:?}");
        assert!(
            !row.contains('(') || row.ends_with("(C)"),
            "{cols}x{rows}: clipped hint {row:?}"
        );
        if layout.lw >= 20 {
            assert!(row.ends_with("Continuous (C)"), "{cols}x{rows}: {row:?}");
        }
    }
}

#[test]
fn snapshot_too_small_overlay() {
    let message = "Terminal too small: 40x18. Need at least 60x22.";
    insta::assert_snapshot!(
        "too_small_overlay",
        ui::too_small(40, 18, message).to_string_trimmed()
    );
}
