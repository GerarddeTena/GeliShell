#[path = "cli/gerisabet.rs"]
mod gerisabet_cli;

mod handlers {
    #[path = "assistant.rs"]
    pub mod assistant;
}

use geli_shell::shell::reporter::StderrReporter;
use gerisabet_cli::{handle_gerisabet_args, print_gerisabet_help};

#[tokio::main]
async fn main() {
    let reporter = StderrReporter::new();
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_gerisabet_help();
        std::process::exit(0);
    }

    handle_gerisabet_args(&args[1..], &reporter).await;
}
