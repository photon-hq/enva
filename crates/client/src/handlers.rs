use crate::{endpoints, ActiveArgs, LoginArgs};
use crate::utils::{
    check_ownership, get_enva_executable_path, get_repo_url, read_config, read_env_file,
    write_config, write_git_hook,
};
use log::{error, info};
use std::process::Command;
use git2::Repository;
use toml_edit::value;
use enva_shared::models::{CommitRequest, FetchRequest};
use crate::encryption::{decrypt_string, encrypt_string, save_pwd};

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

    let mut doc = read_config();

    doc["auth"]["gh_token"] = value(token);

    write_config(doc);

    info!("Token updated successfully");
}

pub async fn active(args: ActiveArgs) {
    check_ownership().await;

    if let Some(password) = args.password {
        info!("Saving password to keychain");

        save_pwd(&get_repo_url(), &password);

        info!("Password saved successfully");

        let (owner, repo_name) = enva_shared::parse_github_repo(&get_repo_url()).expect("Invalid repo URL");

        info!("Setting encrypted flag to true in config");

        let mut doc = read_config();
        doc[&format!("{owner}:{repo_name}")]["encrypted"] = value(true);
        write_config(doc);
    }
    
    let enva_path = get_enva_executable_path().expect("Failed to get enva executable path");

    info!("Executing enva binary at: {}", enva_path.display());

    write_git_hook("post-commit", &format!("{} commit", enva_path.display()));
    write_git_hook("post-merge", &format!("{} fetch", enva_path.display()));
    write_git_hook("post-checkout", &format!("{} fetch", enva_path.display()));

    fetch().await;
}

pub async fn commit() {
    check_ownership().await;

    let repo = Repository::open(".").expect("Failed to open git repository");

    let repo_url = get_repo_url();

    let head = repo.head().expect("Failed to get HEAD reference");
    let commit = head.peel_to_commit().expect("Failed to get commit from HEAD");
    let commit_id = commit.id().to_string();

    info!("Latest commit: {}", commit_id);

    let (owner, repo_name) = enva_shared::parse_github_repo(&repo_url).expect("Invalid repo URL");

    let doc = read_config();

    let encrypted = doc[&format!("{owner}:{repo_name}")]["encrypted"].as_bool().unwrap_or(false);

    let env_files = match encrypted {
        false => read_env_file(),
        true => read_env_file().into_iter().map(|(k, v)| (k, encrypt_string(&repo_url, &v))).collect()
    };

    let res = endpoints::call_commit(CommitRequest {
        repo_url,
        branch: head
            .shorthand()
            .expect("Failed to get current branch")
            .to_string(),
        commit_id: commit_id.clone(),
        env_files,
    })
    .await
    .expect("Failed to commit");

    if !res.success {
        panic!("Failed to commit: {}", res.error.unwrap_or_default());
    }

    info!("Commit pushed: {}", commit_id);
}

pub async fn fetch() {
    check_ownership().await;

    let repo = Repository::open(".").expect("Failed to open git repository");

    let repo_url = get_repo_url();

    let (owner, repo_name) = enva_shared::parse_github_repo(&repo_url).expect("Failed to parse GitHub repo URL");

    let head = repo.head().expect("Failed to get HEAD reference");
    let commit = head.peel_to_commit().expect("Failed to get commit from HEAD");
    let commit_id = commit.id().to_string();

    info!("Latest commit: {}", commit_id);

    let res = endpoints::call_fetch(FetchRequest {
        repo_url: repo_url.clone(),
        commit_id,
    }).await.expect("Failed to fetch");

    if !res.success {
        panic!("Failed to fetch: {}", res.error.unwrap_or_default());
    }

    info!("Env files fetched successfully");

    let doc = read_config();
    let encrypted = doc[&format!("{owner}:{repo_name}")]["encrypted"].as_bool().unwrap_or(false);

    for (file_path, content) in res.env_files.unwrap_or_default() {
        std::fs::write(file_path, match encrypted {
            false => content,
            true => decrypt_string(&repo_url, &content)
        }).expect("Failed to write env file");
    }
}