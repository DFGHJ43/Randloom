//! Parameter parsing, validation, field buffers and visibility.

use randloom::config::{
    DistType, FIELD_BUF_LEN, FIELD_COUNT, FieldBuf, FieldId, LAMBDA_MAX, Params, initial_fields,
    parse_f64, parse_i32, parse_u64,
};

fn fields_with(id: FieldId, text: &str) -> [FieldBuf; FIELD_COUNT] {
    let mut fields = initial_fields();
    fields[id as usize] = FieldBuf::new(text);
    fields
}

#[test]
fn initial_fields_match_c_defaults() {
    let fields = initial_fields();
    let expected = ["1", "0", "0", "100", "0.0", "1.0", "0.5", "1.0"];
    for (field, expected) in fields.iter().zip(expected) {
        assert_eq!(field.text, expected);
        assert_eq!(field.cursor, expected.len());
    }

    let params = Params::from_fields(&fields);
    assert_eq!(params.count, 1);
    assert_eq!(params.seed, 0);
    assert_eq!(params.min, 0);
    assert_eq!(params.max, 100);
    assert_eq!(params.mean, 0.0);
    assert_eq!(params.stddev, 1.0);
    assert_eq!(params.prob, 0.5);
    assert_eq!(params.lambda, 1.0);
}

#[test]
fn parse_i32_handles_invalid_and_overflow() {
    assert_eq!(parse_i32("42"), 42);
    assert_eq!(parse_i32("-7"), -7);
    assert_eq!(parse_i32("+7"), 7);
    assert_eq!(parse_i32(""), 0);
    assert_eq!(parse_i32("-"), 0);
    assert_eq!(parse_i32("."), 0);
    assert_eq!(parse_i32("12x"), 0);
    assert_eq!(parse_i32("  8  "), 8);
    assert_eq!(parse_i32("999999999999999999999"), i32::MAX);
    assert_eq!(parse_i32("-999999999999999999999"), i32::MIN);
}

#[test]
fn parse_u64_saturates_on_overflow() {
    assert_eq!(parse_u64("42"), 42);
    assert_eq!(parse_u64(""), 0);
    assert_eq!(parse_u64("18446744073709551615"), u64::MAX);
    assert_eq!(parse_u64("18446744073709551616"), u64::MAX);
    assert_eq!(parse_u64("12x"), 0);
}

#[test]
fn params_apply_c_clamps() {
    let params = Params::from_fields(&fields_with(FieldId::Count, "0"));
    assert_eq!(params.count, 1);

    let params = Params::from_fields(&fields_with(FieldId::Count, "1001"));
    assert_eq!(params.count, 1000);

    let params = Params::from_fields(&fields_with(FieldId::Count, "-5"));
    assert_eq!(params.count, 1);

    let params = Params::from_fields(&fields_with(FieldId::Prob, "-1"));
    assert_eq!(params.prob, 0.0);

    let params = Params::from_fields(&fields_with(FieldId::Prob, "2"));
    assert_eq!(params.prob, 1.0);

    let params = Params::from_fields(&fields_with(FieldId::StdDev, "0"));
    assert_eq!(params.stddev, 1.0);

    let params = Params::from_fields(&fields_with(FieldId::StdDev, "-3"));
    assert_eq!(params.stddev, 1.0);

    let params = Params::from_fields(&fields_with(FieldId::Lambda, "0"));
    assert_eq!(params.lambda, 1.0);

    let params = Params::from_fields(&fields_with(FieldId::Lambda, "2000000000"));
    assert_eq!(params.lambda, LAMBDA_MAX);

    let params = Params::from_fields(&fields_with(FieldId::Mean, "abc"));
    assert_eq!(params.mean, 0.0);

    let params = Params::from_fields(&fields_with(FieldId::Seed, "18446744073709551616"));
    assert_eq!(params.seed, u64::MAX);
}

#[test]
fn parse_f64_rejects_invalid_and_non_finite() {
    assert_eq!(parse_f64("1.25"), 1.25);
    assert_eq!(parse_f64(""), 0.0);
    assert_eq!(parse_f64("."), 0.0);
    assert_eq!(parse_f64("abc"), 0.0);
}

#[test]
fn field_visibility_matches_distribution() {
    for dist in DistType::ALL {
        assert!(FieldId::Count.visible(dist));
        assert!(FieldId::Seed.visible(dist));
    }

    let visible = |dist| {
        FieldId::ALL
            .into_iter()
            .filter(|field| field.visible(dist))
            .collect::<Vec<_>>()
    };

    assert_eq!(
        visible(DistType::Uniform),
        vec![FieldId::Count, FieldId::Seed, FieldId::Min, FieldId::Max]
    );
    assert_eq!(
        visible(DistType::Normal),
        vec![
            FieldId::Count,
            FieldId::Seed,
            FieldId::Mean,
            FieldId::StdDev
        ]
    );
    assert_eq!(
        visible(DistType::Bernoulli),
        vec![FieldId::Count, FieldId::Seed, FieldId::Prob]
    );
    assert_eq!(
        visible(DistType::Poisson),
        vec![FieldId::Count, FieldId::Seed, FieldId::Lambda]
    );
}

#[test]
fn field_accepts_only_the_c_character_set() {
    for field in FieldId::ALL {
        assert!(field.accepts('5'));
    }
    assert!(FieldId::Count.accepts('-'));
    assert!(FieldId::Count.accepts('.'));
    assert!(!FieldId::Seed.accepts('-'));
    assert!(!FieldId::Seed.accepts('.'));
    assert!(!FieldId::Seed.accepts('a'));
}

#[test]
fn field_buffer_editing_and_cursor() {
    let mut field = FieldBuf::new("12");
    field.move_left();
    field.insert('9');
    assert_eq!(field.text, "192");
    assert_eq!(field.cursor, 2);

    field.backspace();
    assert_eq!(field.text, "12");
    assert_eq!(field.cursor, 1);

    field.move_left();
    field.move_left();
    field.insert('7');
    assert_eq!(field.text, "712");
    assert_eq!(field.cursor, 1);

    field.move_right();
    assert_eq!(field.cursor, 2);
}

#[test]
fn field_buffer_enforces_c_length_and_ascii() {
    let mut field = FieldBuf::new("");
    for _ in 0..100 {
        field.insert('9');
    }
    assert_eq!(field.len(), FIELD_BUF_LEN - 1);
    assert_eq!(field.cursor, FIELD_BUF_LEN - 1);

    let mut field = FieldBuf::new("");
    field.insert('é');
    assert!(field.is_empty());
}
