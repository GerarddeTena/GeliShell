use geli_shell::{Reporter, StderrReporter, shell::{
    builtins::{BuiltinRegistry, BuiltinResult},
    executor::{Executor, ExecutionConfig},
    guard::default_guard,
    reporter::SilentReporter,
    translator::{self, Subsystem, TranslationPipeline},
}, parser::{lexer::Lexer, parser::Parser}, Guard};
use std::sync::Arc;
use std::io::{self, Write};
use tokio::signal;

#[tokio::main]
async fn main() {
    let reporter  = StderrReporter::new();

    // ── Carga el mapa de comandos ─────────────────────────────
    let result = match translator::load() {
        Ok(r)  => r,
        Err(e) => { reporter.error(&e.to_string()); std::process::exit(1); }
    };
    result.report(&reporter);

    // ── Inicializa el sistema ─────────────────────────────────
    let map       = Arc::new(result.map);
    let subsystem = Subsystem::detect(&reporter);
    let pipeline  = TranslationPipeline::new(Arc::clone(&map), subsystem.clone());
    let executor  = Executor::new(subsystem);
    let guard     = default_guard();
    let config    = ExecutionConfig::minimal();
    let mut builtins = BuiltinRegistry::new();

    reporter.info(&format!("GeliShell — subsystem: {}", executor_subsystem_str()));

    // ── REPL ──────────────────────────────────────────────────
    loop {
        print!("geli> ");
        io::stdout().flush().ok();

        let mut input = String::new();

        // Ctrl+D — cierra la shell limpiamente
        if io::stdin().read_line(&mut input).is_err()
            || input.is_empty()
        {
            println!();
            reporter.info("goodbye");
            break;
        }

        let input = input.trim();
        if input.is_empty() { continue; }

        // ── Historial ─────────────────────────────────────────
        builtins.push_history(input.to_owned());

        // ── Lexer ─────────────────────────────────────────────
        let tokens = match Lexer::new(input).tokenize() {
            Ok(t)  => t,
            Err(e) => { reporter.error(&e.to_string()); continue; }
        };

        // ── Parser ────────────────────────────────────────────
        let ast = match Parser::new(tokens).parse() {
            Ok(a)  => a,
            Err(e) => { reporter.error(&e.to_string()); continue; }
        };

        // ── Builtins ──────────────────────────────────────────
        match builtins.try_execute(&ast, &reporter) {
            BuiltinResult::Handled       => continue,
            BuiltinResult::Exit(code)    => std::process::exit(code),
            BuiltinResult::NotABuiltin   => {}
        }

        // ── Guard ─────────────────────────────────────────────
        if let Err(e) = guard.check(&ast) {
            reporter.error(&e.to_string());
            continue;
        }

        // ── Pipeline → String nativo ──────────────────────────
        let command = match pipeline.run(&ast, &SilentReporter::new()) {
            Ok(c)  => c,
            Err(e) => { reporter.error(&e.to_string()); continue; }
        };

        // ── Executor con Ctrl+C ───────────────────────────────
        tokio::select! {
            result = executor.run(&command, &config, &reporter) => {
                match result {
                    Ok(res) if !res.success() => {
                        reporter.warn(&format!(
                            "exit code: {}", res.exit_code
                        ));
                    }
                    Err(e) => reporter.error(&e.to_string()),
                    Ok(_)  => {}
                }
            }
            _ = signal::ctrl_c() => {
                println!();
                reporter.warn("^C — command cancelled");
            }
        }
    }
}

fn executor_subsystem_str() -> &'static str {
    #[cfg(target_os = "windows")]
    return "powershell";
    
    #[cfg(not(target_os = "windows"))]
    return "bash";
}
