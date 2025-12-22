use crate::{endpoints, LoginArgs};
use crate::utils::{check_ownership, get_enva_executable_path, get_repo_url, read_env_file, write_git_hook};
use directories::ProjectDirs;
use log::{error, info};
use std::process::Command;
use git2::Repository;
use toml_edit::{DocumentMut, value};
use shared::models::CommitRequest;

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
    check_ownership().await;
    
    let enva_path = get_enva_executable_path().expect("Failed to get enva executable path");

    info!("Executing enva binary at: {}", enva_path.display());

    write_git_hook("post-commit", &format!("{} commit", enva_path.display()));
    write_git_hook("post-merge", &format!("{} fetch", enva_path.display()));
    write_git_hook("post-checkout", &format!("{} fetch", enva_path.display()));
}

pub async fn commit() {
    check_ownership().await;

    let repo = Repository::open(".").expect("Failed to open git repository");

    let repo_url = get_repo_url();

    let head = repo.head().expect("Failed to get HEAD reference");
    let commit = head.peel_to_commit().expect("Failed to get commit from HEAD");
    let commit_id = commit.id().to_string();

    info!("Latest commit: {}", commit_id);

    endpoints::call_commit(CommitRequest {
        repo_url,
        branch: head.shorthand().expect("Failed to get current branch").to_string(),
        commit_id: commit_id.clone(),
        env_files: read_env_file(),
    }).await.expect("Failed to commit");

    info!("Commit pushed: {}", commit_id);
}