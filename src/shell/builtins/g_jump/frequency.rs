use crate::t;
use std::time::{SystemTime, UNIX_EPOCH};

/// Timestamp Unix actual en segundos
pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Factor de decay según tiempo desde la última visita
///
/// < 1 hora   → ×4.0   (muy reciente — altísima prioridad)
/// < 1 día    → ×2.0   (hoy)
/// < 1 semana → ×1.0   (esta semana)
/// > 1 semana → ×0.5   (antiguo)
pub fn decay_factor(last_visit_secs: u64) -> f64 {
    let elapsed = now_secs().saturating_sub(last_visit_secs);

    const HOUR: u64 = 3_600;
    const DAY: u64 = 86_400;
    const WEEK: u64 = 604_800;

    if elapsed < HOUR {
        4.0
    } else if elapsed < DAY {
        2.0
    } else if elapsed < WEEK {
        1.0
    } else {
        0.5
    }
}

/// Score de frecency para una entrada
/// score = visitas × decay + bonus_case
pub fn frecency_score(visits: u32, last_visit_secs: u64, case_bonus: f64) -> f64 {
    (visits as f64) * decay_factor(last_visit_secs) + case_bonus
}

/// Formatea el tiempo transcurrido para display
pub fn elapsed_display(last_visit_secs: u64) -> String {
    let elapsed = now_secs().saturating_sub(last_visit_secs);

    const MINUTE: u64 = 60;
    const HOUR: u64 = 3_600;
    const DAY: u64 = 86_400;

    if elapsed < MINUTE {
        t!("builtin.g_jump.elapsed_just_now")
    } else if elapsed < HOUR {
        t!(
            "builtin.g_jump.elapsed_minutes_ago",
            minutes = elapsed / MINUTE
        )
    } else if elapsed < DAY {
        t!("builtin.g_jump.elapsed_hours_ago", hours = elapsed / HOUR)
    } else if elapsed < DAY * 2 {
        t!("builtin.g_jump.elapsed_yesterday")
    } else {
        t!("builtin.g_jump.elapsed_days_ago", days = elapsed / DAY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decay_recent_is_highest() {
        let now = now_secs();
        assert!(decay_factor(now) > decay_factor(now - 3_601));
        assert!(decay_factor(now - 3_601) > decay_factor(now - 86_401));
        assert!(decay_factor(now - 86_401) > decay_factor(now - 604_801));
    }

    #[test]
    fn frecency_score_increases_with_visits() {
        let now = now_secs();
        let low = frecency_score(1, now, 0.0);
        let high = frecency_score(10, now, 0.0);
        assert!(high > low);
    }

    #[test]
    fn case_bonus_affects_score() {
        let now = now_secs();
        let without = frecency_score(5, now, 0.0);
        let with_b = frecency_score(5, now, 50.0);
        assert!(with_b > without);
    }

    #[test]
    fn elapsed_display_formats_correctly() {
        let now = now_secs();
        assert_eq!(elapsed_display(now), "just now");
        assert_eq!(elapsed_display(now - 3_601), "1h ago");
        assert_eq!(elapsed_display(now - 86_400), "yesterday");
        assert_eq!(elapsed_display(now - 172_801), "2d ago");
    }
}
