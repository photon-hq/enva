use std::env;
use directories::ProjectDirs;
use std::path::PathBuf;
use log::info;
use toml_edit::DocumentMut;
use which::which;

pub fn get_enva_executable_path() -> Option<PathBuf> {
    which("enva").ok()
}

pub fn get_token() -> Option<String> {
    if let Some(dirs) = ProjectDirs::from("codes", "photon", "enva") {
        let config_path = dirs.config_dir().join("config.toml");

        let text = std::fs::read_to_string(&config_path)
            .unwrap_or_else(|_| String::new());

        let doc = text.parse::<DocumentMut>().expect("You need to login first");

        return Some(doc["auth"]["gh_token"].as_str().unwrap().to_string());
    }

    None
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

    std::fs::write(&hook_path, content).expect("Failed to write hook file");
}