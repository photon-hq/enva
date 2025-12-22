use directories::ProjectDirs;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml_edit::DocumentMut;
use toml_edit::de::from_document;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
struct Database {
    #[serde(default)]
    commits: HashMap<String, Commit>,
}

#[derive(Deserialize, Serialize)]
struct Commit {
    branch: String,
    env_files_paths: HashMap<String, String>, // original file name: local file path
}

pub fn save(
    repo_url: &str,
    branch: &str,
    commit_id: &str,
    env_files: &HashMap<String, String>,
) -> Result<(), String> {
    if let Some((owner, repo_name)) = shared::parse_github_repo(repo_url) {
        let id = format!("{}/{}/{}", owner, repo_name, commit_id);

        let mut env_files_paths: HashMap<String, String> = HashMap::new();

        for (key, value) in env_files {
            let file_id = Uuid::new_v4().to_string();

            save_file(&file_id, value)?;

            env_files_paths.insert(key.clone(), file_id);
        }

        let commit = Commit {
            branch: branch.to_string(),
            env_files_paths,
        };

        if let Some(dirs) = ProjectDirs::from("codes", "photon", "enva") {
            let db_path = dirs.config_dir().join("db.toml");
            info!("DB path: {}", db_path.display());

            std::fs::create_dir_all(dirs.config_dir()).expect("Failed to create config directory");

            let text = std::fs::read_to_string(&db_path).unwrap_or_else(|_| String::new());

            let doc = text.parse::<DocumentMut>().map_err(|e| e.to_string())?;

            let mut db: Database = from_document(doc.clone()).map_err(|e| e.to_string())?;

            db.commits.insert(id.clone(), commit);

            let updated_toml = toml::to_string(&db).map_err(|e| e.to_string())?;
            std::fs::write(&db_path, updated_toml).map_err(|e| e.to_string())?;

            return Ok(());
        }
    }

    Err(format!("Failed to parse repo URL: {}", repo_url))
}

fn save_file(file_id: &str, content: &str) -> Result<(), String> {
    if let Some(dirs) = ProjectDirs::from("codes", "photon", "enva") {
        let envs_dir = dirs.config_dir().join("envs");
        info!("Env dir: {}", envs_dir.display());

        std::fs::create_dir_all(&envs_dir).expect("Failed to create env directory");

        let env_file_path = envs_dir.join(file_id);
        std::fs::write(&env_file_path, content).expect("Failed to write env file");

        return Ok(());
    }

    Err(format!("Failed to save file: {}", file_id))
}

pub fn read(
    repo_url: &str,
    commit_id: &str,
) -> Result<HashMap<String, String>, String> {
    if let Some((owner, repo_name)) = shared::parse_github_repo(repo_url) {
        let id = format!("{}/{}/{}", owner, repo_name, commit_id);

        if let Some(dirs) = ProjectDirs::from("codes", "photon", "enva") {
            let db_path = dirs.config_dir().join("db.toml");

            let text = std::fs::read_to_string(&db_path).unwrap_or_else(|_| String::new());

            let doc = text.parse::<DocumentMut>().map_err(|e| e.to_string())?;

            let db: Database = from_document(doc.clone()).map_err(|e| e.to_string())?;

            let env_files = db
                .commits
                .get(&id)
                .ok_or_else(|| format!("Commit {} not found", id))?
                .env_files_paths
                .iter()
                .map(|(name, file_id)| {
                    // Return Result<(String, String), String>
                    read_file(file_id).map(|content| (name.clone(), content))
                })
                .collect::<Result<HashMap<String, String>, String>>()?;

            return Ok(env_files);
        }
    }

    Err(format!("Failed to parse repo URL: {}", repo_url))
}

fn read_file(file_id: &str) -> Result<String, String> {
    if let Some(dirs) = ProjectDirs::from("codes", "photon", "enva") {
        let envs_dir = dirs.config_dir().join("envs");

        let env_file_path = envs_dir.join(file_id);
        return Ok(std::fs::read_to_string(&env_file_path)
            .map_err(|e| e.to_string())?
            .to_string());
    }

    Err(format!("Failed to read file: {}", file_id))
}
