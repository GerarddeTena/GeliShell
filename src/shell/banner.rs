// src/shell/banner.rs

const GELI_ART: &str = r#"
                                                ████
                               ████████    ██████████████    ████████
                            ███████████████████      ███████████████████
                           █████       █████            █████       █████
                         ████            ██              ██            ████
                         ███             ██              ██             ███
                        ███              ██              ██      ███     ███
                        ███             ███              ███             ███
                       ███              ██                ██              ███
                       ███              ██                ██              ███
                      ███               ██                ██        ██     ███
                      ███              ███                ███       ██     ███
                      ███████          ██                  ██       ██████████
                     ███      ██████████████████████████████████████████    ███
                    ████               ██                  ██         ██    ████
                    ███                ██                  ██         ██     ███
                   ████                ██                  ██          █     ████
                   ███                 █                    █          ██     ███
                   ███                 █                    █          ██     ████
                  ████                 █                    █           ██     ███
                  ███                 ██                    ██          ███    ███
                 ████                 ██                    ██           ██    ████
            ████████                  ██                    ██           ██     ████████
        ███████  ██                   ██                    ██            ██     ██  ███████
     ███████     ██                   ██                     █            ██     ██     ███████
   █████         ██                  ███                     ██            ██     █         █████
 █████         ███                   ██                      ██            ██     ███         █████
████       ███████                   ██                      ██                   ███████       ████
███     ████   ███                   ██                      ██                   ███   ████     ███
███    ██       ███                  ██                      ██                  ███       ██    ███
███    ███        █████             ███                      ███             █████        ███    ███
████     ████           ██████████████                        ██████████████           ████     ████
 ███        ████                   ██████████████████████████████                   ████        ███
  ████          █████                                                          █████          ████
    ██████           █████████                                        █████████           ██████
      ███████                ████████████████████ ███████████    ██████                ███████
         ████████                          ██████████████                          ████████
              ███████████                                                  ███████████
                   ███████████████                                ███████████████
                         ██████████████████████████████████████████████████
                                         ██████████████████
"#;

// Códigos ANSI — purple y darkpink combinados
const PURPLE: &str = "\x1b[38;5;129m";
const DARKPINK: &str = "\x1b[38;5;198m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

pub fn print_banner(version: &str) {
    // Colorea el art alternando purple y darkpink por línea
    // para dar efecto degradado vertical
    let lines: Vec<&str> = GELI_ART.lines().collect();
    let total = lines.len();

    for (i, line) in lines.iter().enumerate() {
        // Primera mitad purple, segunda mitad darkpink
        let color = if i < total / 2 { PURPLE } else { DARKPINK };
        println!("{color}{line}{RESET}");
    }

    // Tagline centrada debajo del logo
    println!();
    println!("{BOLD}{PURPLE}   ██████╗ ███████╗██╗     ██╗{RESET}");
    println!("{BOLD}{DARKPINK}  ██╔════╝ ██╔════╝██║     ██║{RESET}");
    println!("{BOLD}{PURPLE}  ██║  ███╗█████╗  ██║     ██║{RESET}");
    println!("{BOLD}{DARKPINK}  ██║   ██║██╔══╝  ██║     ██║{RESET}");
    println!("{BOLD}{PURPLE}  ╚██████╔╝███████╗███████╗██║{RESET}");
    println!("{BOLD}{DARKPINK}   ╚═════╝ ╚══════╝╚══════╝╚═╝  v{version}{RESET}");
    println!();
    println!(
        "{PURPLE}  cross-platform shell · \
         {DARKPINK}subsystem translator · \
         {PURPLE}AI assistant{RESET}"
    );
    println!();
}
