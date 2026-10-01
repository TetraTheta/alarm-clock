use super::*;

#[test]
fn alarm_is_due_once_per_day_at_its_configured_time() {
  assert!(is_due(3_600, 20260930, 20261001, 3_600));
  assert!(!is_due(3_600, 20261001, 20261001, 3_600));
  assert!(!is_due(3_600, 20260930, 20261001, 3_601));
}
