use crossterm::{
    execute,
    style::{Color, SetBackgroundColor, SetForegroundColor},
};
use geli_shell::shell::{
    config::{history_store::PersistentCommandHistory, ShellConfig, VisualConfig},
    reporter::Reporter,
    translator::{CommandMap, Subsystem},
};
use std::collections::BTreeSet;
use std::io::Write;

pub fn render_prompt(subsystem: &Subsystem, visual: &VisualConfig) -> String {
    let cwd = match std::env::current_dir() {
        Err(_) => "?".to_owned(),
        Ok(path) => {
            let normalized = path.to_string_lossy().replace('\\', "/");

            let home_var = if cfg!(target_os = "windows") {
                std::env::var("USERPROFILE")
            } else {
                std::env::var("HOME")
            };

            match home_var {
                Ok(home) => {
                    let home_normalized = home.replace('\\', "/");
                    if normalized.starts_with(&home_normalized) {
                        let stripped = &normalized[home_normalized.len()..];
                        if stripped.is_empty() {
                            "~".to_owned()
                        } else {
                            format!("~{stripped}")
                        }
                    } else {
                        normalized
                    }
                }
                Err(_) => normalized,
            }
        }
    };

    let path_color = visual.prompt_path_ansi256;
    let sub_color = visual.prompt_subsystem_ansi256;
    let name_color = visual.prompt_name_ansi256;
    let dim_color = visual.prompt_dim_ansi256;

    let icon = "󰊠";
    let sep = "";
    let prompt_char = "❯";

    let segment_1 = format!(
        "\x1b[38;5;{name_color}m[ {icon} GeliShell ]\x1b[0m"
    );

    let segment_2 = format!(
        "\x1b[38;5;{dim_color}m{sep} \x1b[38;5;{sub_color}m_{}_\x1b[38;5;{dim_color}m\x1b[0m",
        subsystem.as_str().to_uppercase()
    );

    let segment_3 = format!(
        "\x1b[38;5;{path_color}m{cwd}\x1b[0m"
    );

    let prompt = format!(
        "\x1b[1m\x1b[38;5;{name_color}m{prompt_char}\x1b[0m "
    );

    format!("{segment_1} {segment_2} {segment_3} {prompt}")
}

pub fn build_completion_pool(map: &CommandMap, config: &ShellConfig) -> Vec<String> {
    let mut set = BTreeSet::new();

    for builtin in [
        "cd",
        "clear",
        "exit",
        "export",
        "history",
        "source",
        "unset",
        "g",
        "gerisabet",
    ] {
        set.insert(builtin.to_owned());
    }

    for cmd in map.iter() {
        set.insert(cmd.name.clone());

        if let Some(entry) = &cmd.translate.bash {
            if !entry.exact.trim().is_empty() {
                set.insert(entry.exact.clone());
            }
            for suggestion in &entry.suggestions {
                if !suggestion.trim().is_empty() {
                    set.insert(suggestion.clone());
                }
            }
        }
        if let Some(entry) = &cmd.translate.powershell {
            if !entry.exact.trim().is_empty() {
                set.insert(entry.exact.clone());
            }
            for suggestion in &entry.suggestions {
                if !suggestion.trim().is_empty() {
                    set.insert(suggestion.clone());
                }
            }
        }
        if let Some(entry) = &cmd.translate.cmd {
            if !entry.exact.trim().is_empty() {
                set.insert(entry.exact.clone());
            }
            for suggestion in &entry.suggestions {
                if !suggestion.trim().is_empty() {
                    set.insert(suggestion.clone());
                }
            }
        }
    }

    for custom in &config.customization.custom_commands {
        if !custom.name.trim().is_empty() {
            set.insert(custom.name.trim().to_owned());
        }
    }

    set.into_iter().collect()
}

pub fn expand_custom_command(input: &str, config: &ShellConfig) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let Some(head) = parts.next() else {
        return trimmed.to_owned();
    };

    let Some(custom) = config
        .customization
        .custom_commands
        .iter()
        .find(|entry| entry.name.trim() == head && !entry.template.trim().is_empty())
    else {
        return trimmed.to_owned();
    };

    let tail = parts.next().unwrap_or("").trim();
    if tail.is_empty() {
        custom.template.trim().to_owned()
    } else {
        format!("{} {}", custom.template.trim(), tail)
    }
}

pub async fn append_history_or_warn(
    command_history: &mut PersistentCommandHistory,
    input: &str,
    reporter: &dyn Reporter,
) {
    if let Err(error) = command_history.append_async(input).await {
        reporter.warn(&format!("history append failed: {error}"));
    }
}

pub fn apply_visual_settings(config: &ShellConfig, reporter: &dyn Reporter) {
    let mut out = std::io::stdout();
    if let Err(error) = execute!(
        out,
        SetForegroundColor(Color::AnsiValue(config.visual.terminal_foreground_ansi256)),
        SetBackgroundColor(Color::AnsiValue(config.visual.terminal_background_ansi256)),
    ) {
        reporter.warn(&format!("visual apply failed: {error}"));
    }

    if let Err(error) = write!(out, "\x1b]50;{}\x07", config.visual.font_family) {
        reporter.warn(&format!("font apply failed: {error}"));
    }
    if let Err(error) = out.flush() {
        reporter.warn(&format!("visual flush failed: {error}"));
    }
}

pub fn strip_wrapping_quotes(input: &str) -> &str {
    if input.len() < 2 {
        return input;
    }

    let bytes = input.as_bytes();
    let starts_with_double = bytes.first() == Some(&b'"');
    let ends_with_double = bytes.last() == Some(&b'"');
    let starts_with_single = bytes.first() == Some(&b'\'');
    let ends_with_single = bytes.last() == Some(&b'\'');

    if (starts_with_double && ends_with_double) || (starts_with_single && ends_with_single) {
        &input[1..input.len() - 1]
    } else {
        input
    }
}
