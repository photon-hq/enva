mod encryption;
mod endpoints;
mod handlers;
mod utils;

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
    Active(ActiveArgs),
    #[command(hide = true)]
    Commit,
    #[command(hide = true)]
    Fetch,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct LoginArgs {
    #[arg(long, help = "GitHub personal access token")]
    token: Option<String>,

    #[arg(long, help = "Use Github cli token")]
    gh: bool,
}

#[derive(Args, Debug)]
struct ActiveArgs {
    #[arg(long, short, help = "Set password for encryption")]
    password: Option<String>,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Login(args) => handlers::login(args),
        Command::Active(args) => handlers::active(args).await,
        Command::Commit => handlers::commit().await,
        Command::Fetch => handlers::fetch().await,
    }
}
