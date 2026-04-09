// src/shell/banner.rs

use std::io::Write;

const GELI_ART: &str = r#"
                                         ███████ █████████
                                ██████████     ██        ███████████
                             ██████      ███    ██      ██       ████████
                         ████      ███     ██   ██     ██     ████       ██
                        ██████████    ██    █   ██     █    ███  ████████ ██
                       ███       █████████████  █████████████████        ████
                      ██      ████     ███   █████    ██  ██████████        ██
                     ██      ██          ██  █   █    █ ███        ███       █
                     █      ██            █████  ██████████          ███    ███
                    ████████           ███     ███         ██         ██   ██ ██
                    █     ███        ██         ██           ██     ████████  ██
                   ██    ██ ██████████          ██            ██   ██   ██     ██
                   █    ██         ██           ██             █  ██     ██    ██
                  ██    ██    █    █ ██         ██ ████████████████      ██     ██
                 ██    ██   ██    ██   █████  ██████         █████       ███    ██
                 ██    ██ ██      ██       ████  █              ██       ███    ███
                ██    ██    █     ██             █             ███         ██    ██
                █    ███         ██              ██      █     ███        ███   ████
               ██    ███         ██        █     ██ ██ ██      ███         ███   ███
               ██   ███          ██   █ ██       ██  ███        ███       █ ██    ███
              ██     ██          ██    ██ █      ██              ██        ███    ███
              █    ████          ██      █ █     ██             ███          ██    ██
             ██    ███           ██               █             ███        ████  █████
             ██    ███          ███               █              ██         ███ █████
              ████████          ██               ██              ██          ████████
               ██████         ████              █ █              ███        ████████
                   ██       ██████              ████            ████         ████
                   ██     █ ██ ███              ████             █████     ████
                   ██   █████████              ██ ██         ██ ████ ██████ ██
                    ███████ █████         █ ██ ██ █       ███ █   █████ █████
                     ██████ ████      ██████████ ████ █  ██ ██ █████████████
                       ███████ ███ ████████████ █████████████ ██ ███
                                  █████ █ ███ ███ █████████ ██████
                                     █████████        ████████
"#;

// Códigos ANSI — purple y darkpink combinados
const PURPLE: &str = "\x1b[38;5;129m";
const DARKPINK: &str = "\x1b[38;5;198m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

/// Imprime el banner de inicio en el writer dado.
/// Acepta cualquier `dyn Write` (stdout, buffer de test, etc.)
/// para no acoplar la función a stdout directamente.
pub fn print_banner(version: &str, out: &mut dyn Write) {
    // Colorea el art alternando purple y darkpink por línea
    // para dar efecto degradado vertical
    let lines: Vec<&str> = GELI_ART.lines().collect();
    let total = lines.len();

    for (i, line) in lines.iter().enumerate() {
        let color = if i < total / 2 { PURPLE } else { DARKPINK };
        let _ = writeln!(out, "{color}{line}{RESET}");
    }

    let _ = writeln!(out);
    let _ = writeln!(out, "{BOLD}{PURPLE}   ██████╗ ███████╗██╗     ██╗{RESET}");
    let _ = writeln!(out, "{BOLD}{DARKPINK}  ██╔════╝ ██╔════╝██║     ██║{RESET}");
    let _ = writeln!(out, "{BOLD}{PURPLE}  ██║  ███╗█████╗  ██║     ██║{RESET}");
    let _ = writeln!(out, "{BOLD}{DARKPINK}  ██║   ██║██╔══╝  ██║     ██║{RESET}");
    let _ = writeln!(out, "{BOLD}{PURPLE}  ╚██████╔╝███████╗███████╗██║{RESET}");
    let _ = writeln!(
        out,
        "{BOLD}{DARKPINK}   ╚═════╝ ╚══════╝╚══════╝╚═╝  v{version}{RESET}"
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "{PURPLE}  {}{RESET}",
        crate::shell::i18n::t("banner.tagline")
    );
    let _ = writeln!(out);
}
