use std::process::Command;
use directories::ProjectDirs;
use log::{error, info};
use crate::{LoginArgs};
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

        let text = std::fs::read_to_string(&config_path)
            .unwrap_or_else(|_| String::new());

        let mut doc = text.parse::<DocumentMut>().unwrap_or(DocumentMut::new());

        doc["auth"]["gh_token"] = value(token);

        std::fs::create_dir_all(dirs.config_dir()).expect("Failed to create config directory");
        std::fs::write(&config_path, doc.to_string()).expect("Failed to write config file");

        info!("Token updated successfully");
    } else {
        error!("Failed to get config path")
    }
}