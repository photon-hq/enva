use directories::ProjectDirs;
use toml_edit::DocumentMut;

pub fn get_token() -> Option<String> {
    if let Some(dirs) = ProjectDirs::from("codes", "photon", "enva") {
        let config_path = dirs.config_dir().join("config.toml");

        let text = std::fs::read_to_string(&config_path)
            .unwrap_or_else(|_| String::new());

        let doc = text.parse::<DocumentMut>().expect("You need to login first");
        
        return Some(doc["auth"]["gh_token"].as_str().unwrap().to_string());
    }
    
    return None;
}