use octocrab::Octocrab;
use log::info;
use std::path::PathBuf;
use directories::ProjectDirs;

pub mod models;

pub fn get_config_dir() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("ENVA_CONFIG_PATH") {
        return Some(PathBuf::from(path));
    }

    ProjectDirs::from("codes", "photon", "enva").map(|dirs| dirs.config_dir().to_path_buf())
}

pub fn parse_github_repo(url: &str) -> Option<(String, String)> {
    // SSH form: git@github.com:org/repo.git
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        let parts: Vec<_> = rest.trim_end_matches(".git").split('/').collect();
        if parts.len() == 2 {
            return Some((parts[0].into(), parts[1].into()));
        }
    }

    // HTTPS / SSH URL form
    let url = url::Url::parse(url).ok()?;
    if url.host_str()? != "github.com" {
        return None;
    }

    let mut segments = url.path_segments()?;
    let owner = segments.next()?;
    let repo = segments.next()?.trim_end_matches(".git");

    Some((owner.to_string(), repo.to_string()))
}

fn build_octocrab(token: &str) -> octocrab::Result<Octocrab> {
    Octocrab::builder().personal_token(token).build()
}
pub async fn check_ownership(token: &str, repo_url: &str) -> Result<bool, String> {
    let octocrab = build_octocrab(token).map_err(|e| e.to_string())?;

    info!("Octocrab built successfully");

    let user = octocrab.current().user().await.map_err(|e| e.to_string())?;
    let username = user.login;

    if let Some((owner, repo_name)) = parse_github_repo(repo_url) {
        let repo = octocrab
            .repos(&owner, &repo_name)
            .get()
            .await
            .map_err(|e| e.to_string())?;

        if let Some(perms) = repo.permissions {
            // Allow if user has write permission
            if perms.push {
                info!("User {} has write permission to {}/{}", username, owner, repo_name);
                return Ok(true);
            }

            // Otherwise, check if user is org member with at least read permission
            if !perms.pull {
                return Err("You don't have read permissions on this repo".to_string());
            }

            if let Some(owner_info) = repo.owner {
                if owner_info.r#type != "Organization" {
                    return Err("This repo is not owned by an organization".to_string());
                }

                let is_member = octocrab
                    .orgs(&owner_info.login)
                    .check_membership(&username)
                    .await
                    .unwrap_or(false); // Returns false if not a member or error

                if !is_member {
                    return Err(format!(
                        "User {} is not a member of organization {}",
                        username, owner
                    ));
                }

                info!("User {} verified as org member with read permission to {}/{}", username, owner, repo_name);
            }
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_config_dir_env_var() {
        let temp_path = "/tmp/enva_config_test";
        unsafe { env::set_var("ENVA_CONFIG_PATH", temp_path); }
        assert_eq!(get_config_dir(), Some(PathBuf::from(temp_path)));
        unsafe { env::remove_var("ENVA_CONFIG_PATH"); }
    }

    #[test]
    fn test_get_config_dir_default() {
        unsafe { env::remove_var("ENVA_CONFIG_PATH"); }
        let config_dir = get_config_dir();
        assert!(config_dir.is_some());
    }
}
