mod handlers;

use clap::{Args, Parser, Subcommand};
#[derive(Parser, Debug)]
#[command(
    name = "enva",
    about = "Sharing env files within GitHub organizations",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Login(LoginArgs),
    Active
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct LoginArgs {
    #[arg(long, help = "GitHub personal access token")]
    token: Option<String>,

    #[arg(long, help = "Use Github cli token")]
    gh: bool
}

fn main() {
    env_logger::init();
    
    let cli = Cli::parse();

    match cli.command {
        Command::Login(args) => handlers::login(args),
        Command::Active => {}
    }
}
