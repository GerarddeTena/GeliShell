use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use toml::Value;

static LOCALE_EN: &str = include_str!("../../../locales/en.toml");
static LOCALE_ES: &str = include_str!("../../../locales/es.toml");

static TRANSLATIONS: OnceLock<RwLock<Translations>> = OnceLock::new();

struct Translations {
    active: HashMap<String, String>,
    fallback: HashMap<String, String>,
}

fn global() -> &'static RwLock<Translations> {
    TRANSLATIONS.get_or_init(|| {
        let fallback = parse_locale(LOCALE_EN);
        RwLock::new(Translations {
            active: fallback.clone(),
            fallback,
        })
    })
}

pub fn init_i18n(lang_code: &str) {
    let locale = load_locale(lang_code);
    let fallback = parse_locale(LOCALE_EN);
    let translations = Translations {
        active: locale.clone(),
        fallback,
    };
    let lock = TRANSLATIONS.get_or_init(|| RwLock::new(translations));
    if let Ok(mut guard) = lock.write() {
        guard.active = locale;
    }
}

pub fn t(key: &str) -> String {
    let lock = global();
    if let Ok(guard) = lock.read() {
        if let Some(val) = guard.active.get(key) {
            return val.clone();
        }
        if let Some(val) = guard.fallback.get(key) {
            return val.clone();
        }
    }
    key.to_owned()
}

pub fn t_with(key: &str, params: &[(&str, &str)]) -> String {
    let mut text = t(key);
    for (name, value) in params {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

pub fn detect_language(config_lang: &str) -> String {
    if let Ok(lang) = std::env::var("GELISHELL_LANG") {
        let lang = lang.trim().to_ascii_lowercase();
        if !lang.is_empty() {
            return normalize_lang_code(&lang);
        }
    }
    if !config_lang.is_empty() {
        return normalize_lang_code(config_lang);
    }
    #[cfg(not(target_os = "windows"))]
    if let Ok(lang) = std::env::var("LANG") {
        let base = lang.split('_').next().unwrap_or("").to_ascii_lowercase();
        if !base.is_empty() && is_supported(&base) {
            return base;
        }
    }
    "en".to_owned()
}

pub fn supported_languages() -> &'static [&'static str] {
    &["en", "es"]
}

fn normalize_lang_code(lang: &str) -> String {
    let base = lang
        .split(['-', '_'])
        .next()
        .unwrap_or("en")
        .to_ascii_lowercase();
    if is_supported(&base) {
        base
    } else {
        "en".to_owned()
    }
}

fn is_supported(lang: &str) -> bool {
    supported_languages().contains(&lang)
}

fn load_locale(lang_code: &str) -> HashMap<String, String> {
    match normalize_lang_code(lang_code).as_str() {
        "es" => parse_locale(LOCALE_ES),
        _ => parse_locale(LOCALE_EN),
    }
}

fn parse_locale(raw: &str) -> HashMap<String, String> {
    match raw.parse::<Value>() {
        Ok(value) => {
            let mut map = HashMap::new();
            flatten_value(&value, String::new(), &mut map);
            map
        }
        Err(_) => HashMap::new(),
    }
}

fn flatten_value(value: &Value, prefix: String, out: &mut HashMap<String, String>) {
    match value {
        Value::Table(table) => {
            for (key, val) in table {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten_value(val, full_key, out);
            }
        }
        Value::String(s) => {
            out.insert(prefix, s.clone());
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_nested_toml_to_dotted_keys() {
        let toml = r#"
[error]
cd_failed = "cd: {path}: {error}"

[builtin.source]
missing_arg = "source: missing file argument"
"#;
        let map = parse_locale(toml);
        assert_eq!(
            map.get("error.cd_failed").map(String::as_str),
            Some("cd: {path}: {error}")
        );
        assert_eq!(
            map.get("builtin.source.missing_arg").map(String::as_str),
            Some("source: missing file argument")
        );
    }

    #[test]
    fn t_with_interpolates_params() {
        let result = t_with(
            "builtin.cd.error",
            &[("path", "/foo"), ("error", "not found")],
        );
        assert!(result.contains("/foo"));
        assert!(result.contains("not found"));
    }

    #[test]
    fn normalize_lang_strips_region() {
        assert_eq!(normalize_lang_code("es-ES"), "es");
        assert_eq!(normalize_lang_code("en-US"), "en");
        assert_eq!(normalize_lang_code("es"), "es");
        assert_eq!(normalize_lang_code("fr"), "en");
    }

    #[test]
    fn en_locale_parses_without_error() {
        let map = parse_locale(LOCALE_EN);
        assert!(!map.is_empty(), "en.toml must parse to non-empty map");
        assert!(map.contains_key("builtin.cd.error"));
        assert!(map.contains_key("repl.goodbye"));
        assert!(map.contains_key("pipeline.canonical_match"));
    }

    #[test]
    fn es_locale_parses_without_error() {
        let map = parse_locale(LOCALE_ES);
        assert!(!map.is_empty(), "es.toml must parse to non-empty map");
        assert!(map.contains_key("builtin.cd.error"));
    }

    #[test]
    fn t_returns_key_when_not_found() {
        let result = t("this.key.does.not.exist.xyz");
        assert_eq!(result, "this.key.does.not.exist.xyz");
    }
}
