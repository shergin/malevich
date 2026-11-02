use super::{MarkKind, Orientation, Rule};

#[test]
#[should_panic(expected = "finite position")]
fn non_finite_rules_panic() {
    Rule::h(f64::NAN);
}

#[test]
fn rules_carry_their_orientation() {
    assert_eq!(Rule::h(2.0).orientation, Orientation::Horizontal(2.0));
    assert_eq!(Rule::v(3.0).orientation, Orientation::Vertical(3.0));
}

#[test]
fn spans_carry_both_bounds_in_either_order_and_must_be_finite() {
    assert_eq!(
        Rule::v_span(3.0, 1.0).orientation,
        Orientation::VerticalSpan(3.0, 1.0)
    );
    assert_eq!(
        Rule::h_span(-1.0, 2.0).orientation,
        Orientation::HorizontalSpan(-1.0, 2.0)
    );
    let bad = Rule {
        orientation: Orientation::VerticalSpan(0.0, f64::INFINITY),
        color: None,
        label: None,
        dash: super::Dash::Solid,
    };
    assert!(bad.validate().is_err());
}

#[test]
#[should_panic(expected = "finite bounds")]
fn non_finite_spans_panic() {
    Rule::h_span(f64::NAN, 1.0);
}
