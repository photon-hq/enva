use directories::ProjectDirs;
use git2::Repository;
use log::{error, info};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;
use which::which;

pub fn get_enva_executable_path() -> Option<PathBuf> {
    which("enva").ok()
}

pub fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("codes", "photon", "enva").map(|dirs| dirs.config_dir().join("config.toml"))
}

pub fn read_config() -> DocumentMut {
    let config_path = get_config_path().expect("Failed to get config path");
    let text = std::fs::read_to_string(&config_path).unwrap_or_else(|_| String::new());
    text.parse::<DocumentMut>().unwrap_or_default()
}

pub fn write_config(doc: DocumentMut) {
    let config_path = get_config_path().expect("Failed to get config path");
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create config directory");
    }
    std::fs::write(&config_path, doc.to_string()).expect("Failed to write config file");
}

pub fn get_token() -> Option<String> {
    let doc = read_config();
    doc.get("auth")
        .and_then(|item| item.get("gh_token"))
        .and_then(|item| item.as_str())
        .map(|s| s.to_string())
}

pub fn write_git_hook(hook_name: &str, hook_content: &str) {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let hook_path = current_dir.join(".git").join("hooks").join(hook_name);

    info!("Writing hook to: {}", hook_path.display());

    let mut content = std::fs::read_to_string(&hook_path).unwrap_or_else(|_| String::new());

    if !content.contains("#!/bin/sh") {
        content = format!("#!/bin/sh\n{}", content);
    }

    if !content.contains(hook_content) {
        content.push_str(format!("\n{}", hook_content).as_str());
    }

    fs::write(&hook_path, content).expect("Failed to write hook file");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)
            .expect("Failed to read hook file metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).expect("Failed to set hook file permissions");
    }
}

pub fn get_repo_url() -> String {
    let repo = Repository::open(".").expect("Failed to open git repository");

    // Try to get the remote from the current branch's upstream
    if let Ok(head) = repo.head()
        && let Some(branch_name) = head.shorthand()
        && let upstream_remote_key = format!("branch.{}.remote", branch_name)
        && let Ok(config) = repo.config()
        && let Ok(remote_name) = config.get_string(&upstream_remote_key)
        && let Ok(remote) = repo.find_remote(&remote_name)
        && let Some(url) = remote.url()
    {
        info!(
            "Using branch '{}' upstream remote '{}' with URL: {}",
            branch_name, remote_name, url
        );
        return url.to_string();
    }

    // Try common remote names
    for name in ["origin", "main", "upstream"] {
        if let Ok(remote) = repo.find_remote(name)
            && let Some(url) = remote.url()
        {
            info!("Found remote '{}' with URL: {}", name, url);
            return url.to_string();
        }
    }

    // Get the first available remote
    let remotes = repo.remotes().expect("Failed to get list of remotes");
    if let Some(first_remote_name) = remotes.get(0)
        && let Ok(remote) = repo.find_remote(first_remote_name)
        && let Some(url) = remote.url()
    {
        info!(
            "Using first available remote '{}' with URL: {}",
            first_remote_name, url
        );
        return url.to_string();
    }

    panic!("No git remote found. Please add a remote with 'git remote add <name> <url>'");
}

pub async fn check_ownership() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let git_path = current_dir.join(".git");

    if Path::new(&git_path).exists() {
        info!(".git folder found at: {}", git_path.display());

        let repo_url = get_repo_url();

        info!("Remote URL: {}", repo_url);

        if !enva_shared::check_ownership(&get_token().expect("You need to login first"), &repo_url)
            .await
            .unwrap_or(false)
        {
            panic!("You does not have ownership of the repository");
        }
    } else {
        error!(
            ".git folder not found in current directory: {}",
            current_dir.display()
        );
        panic!("Please run this command within a git repository");
    }
}

pub fn read_env_file() -> HashMap<String, String> {
    let mut env_files = HashMap::new();

    let current_dir = env::current_dir().expect("Failed to get current directory");

    let entries = fs::read_dir(&current_dir).expect("Failed to read current directory");

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file()
            && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            && file_name.starts_with(".env")
        {
            info!("Reading env file: {}", path.display());

            if let Ok(content) = fs::read_to_string(&path) {
                env_files.insert(file_name.to_string(), content);
            }
        }
    }

    env_files
}
