use crate::LoginArgs;
use crate::utils::{get_enva_executable_path, get_token, write_git_hook};
use directories::ProjectDirs;
use git2::Repository;
use log::{error, info};
use std::env;
use std::path::Path;
use std::process::Command;
use toml_edit::{DocumentMut, value};

pub(crate) fn login(args: LoginArgs) {
    let mut token = args.token.unwrap_or_default();
    if args.gh {
        let output = Command::new("gh")
            .args(["auth", "token"])
            .output()
            .expect("Failed to get token from Github cli");

        token = String::from_utf8(output.stdout)
            .expect("Github CLI returned invalid UTF-8")
            .trim()
            .to_string();
    }

    if token.is_empty() {
        error!("No token provided. Please provide a token or use the --gh flag.");
        return;
    }

    info!("Token received successfully: {}", token);

    if let Some(dirs) = ProjectDirs::from("codes", "photon", "enva") {
        let config_path = dirs.config_dir().join("config.toml");

        info!("Config path: {}", config_path.display());

        let text = std::fs::read_to_string(&config_path).unwrap_or_else(|_| String::new());

        let mut doc = text.parse::<DocumentMut>().unwrap_or(DocumentMut::new());

        doc["auth"]["gh_token"] = value(token);

        std::fs::create_dir_all(dirs.config_dir()).expect("Failed to create config directory");
        std::fs::write(&config_path, doc.to_string()).expect("Failed to write config file");

        info!("Token updated successfully");
    } else {
        error!("Failed to get config path")
    }
}

pub async fn active() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let git_path = current_dir.join(".git");

    if Path::new(&git_path).exists() {
        info!(".git folder found at: {}", git_path.display());

        let repo = Repository::open(".").expect("Failed to open git repository");
        let remote = repo
            .find_remote("origin")
            .expect("Failed to find remote origin");
        let repo_url = remote.url().expect("Failed to get remote URL");

        info!("Remote URL: {}", repo_url);

        if !shared::check_ownership(&get_token().expect("You need to login first"), repo_url)
            .await
            .unwrap_or(false)
        {
            panic!("You does not have ownership of the repository");
        }

        let enva_path = get_enva_executable_path().expect("Failed to get enva executable path");

        info!("Executing enva binary at: {}", enva_path.display());

        write_git_hook("post-commit", &format!("{} commit", enva_path.display()));
        write_git_hook("post-merge", &format!("{} fetch", enva_path.display()));
        write_git_hook("post-checkout", &format!("{} fetch", enva_path.display()));
    } else {
        error!(
            ".git folder not found in current directory: {}",
            current_dir.display()
        );
        panic!("Please run this command within a git repository");
    }
}
