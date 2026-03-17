/// Resultado del matching con su score de case
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub path: String,
    pub case_bonus: f64,
}

/// Estrategia de matching — case insensitive
/// pero prioriza coincidencia exacta de case
pub fn match_pattern(path: &str, pattern: &str) -> Option<MatchResult> {
    // Toma el último componente del path para el matching principal
    // "/home/gerard/projects/geliShell" → "geliShell"
    let basename = path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path);

    // ── Exact case match en basename ──────────────────────────
    if basename.contains(pattern) {
        return Some(MatchResult {
            path: path.to_owned(),
            case_bonus: 50.0, // prioridad máxima
        });
    }

    // ── Case insensitive en basename ──────────────────────────
    let pattern_lower = pattern.to_lowercase();
    let basename_lower = basename.to_lowercase();

    if basename_lower.contains(&pattern_lower) {
        return Some(MatchResult {
            path: path.to_owned(),
            case_bonus: 0.0,
        });
    }

    // ── Fuzzy en basename — todos los chars del patrón
    //    aparecen en orden en el basename ─────────────────────
    if fuzzy_match(&basename_lower, &pattern_lower) {
        return Some(MatchResult {
            path: path.to_owned(),
            case_bonus: -10.0, // penalización leve por fuzzy
        });
    }

    // ── Fallback: case insensitive en path completo ───────────
    if path.to_lowercase().contains(&pattern_lower) {
        return Some(MatchResult {
            path: path.to_owned(),
            case_bonus: -5.0,
        });
    }

    None
}

/// Fuzzy match — todos los caracteres del patrón
/// aparecen en orden en el texto
fn fuzzy_match(text: &str, pattern: &str) -> bool {
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    let mut pattern_chars = pattern_lower.chars();
    let mut current = pattern_chars.next();

    for ch in text_lower.chars() {
        match current {
            None => return true,
            Some(p) if ch == p => {
                current = pattern_chars.next();
            }
            _ => {}
        }
    }
    current.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_case_gets_max_bonus() {
        let result = match_pattern("/home/gerard/projects/geliShell", "geliShell");
        assert!(result.is_some());
        assert_eq!(result.unwrap().case_bonus, 50.0);
    }

    #[test]
    fn case_insensitive_gets_zero_bonus() {
        let result = match_pattern("/home/gerard/projects/geliShell", "gelishell");
        assert!(result.is_some());
        assert_eq!(result.unwrap().case_bonus, 0.0);
    }

    #[test]
    fn fuzzy_match_gets_penalty() {
        let result = match_pattern("/home/gerard/projects/geliShell", "gls");
        assert!(result.is_some());
        assert_eq!(result.unwrap().case_bonus, -10.0);
    }

    #[test]
    fn no_match_returns_none() {
        let result = match_pattern("/home/gerard/projects/geliShell", "xxxxxxx");
        assert!(result.is_none());
    }

    #[test]
    fn matches_in_full_path_when_basename_misses() {
        let result = match_pattern("/home/gerard/projects/geliShell", "gerard");
        assert!(result.is_some());
    }

    #[test]
    fn fuzzy_match_fn_works() {
        assert!(fuzzy_match("geliShell", "gls"));
        assert!(fuzzy_match("projects", "pjs"));
        assert!(!fuzzy_match("abc", "xyz"));
    }
}
