mod app;
mod ascii;
mod cli;
mod error;
mod layout;
mod lexer;
mod parser;
mod svg;
mod types;
mod validator;

use cli::{Cli, Commands};
use std::process;

fn main() {
    let cli = Cli::parse_args();

    let result = match cli.command {
        Commands::Compile { input, output } => app::App::compile(input, output),
        Commands::Preview { input } => match app::App::preview(input) {
            Ok(ascii) => {
                println!("{}", ascii);
                Ok(())
            }
            Err(e) => Err(e),
        },
        Commands::Validate { input } => app::App::validate(input),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        process::exit(e.exit_code());
    }
}
