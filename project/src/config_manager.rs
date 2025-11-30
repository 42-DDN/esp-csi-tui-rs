// --- File: src/config_manager.rs ---
// --- Purpose: Handles File I/O for saving/loading templates ---

use std::fs;
use std::path::Path;
use crate::layout_tree::TilingManager;

const TEMPLATE_DIR: &str = "templates";

/// Ensures the template directory exists
pub fn init() -> std::io::Result<()> {
    if !Path::new(TEMPLATE_DIR).exists() {
        fs::create_dir(TEMPLATE_DIR)?;
    }
    Ok(())
}

/// Saves the current layout tree to a JSON file
pub fn save_template(name: &str, manager: &TilingManager) -> std::io::Result<()> {
    init()?; // Ensure dir exists
    let json = serde_json::to_string_pretty(manager)?;

    // CHANGED: Use .json extension
    let filename = format!("{}/{}.json", TEMPLATE_DIR, name);
    fs::write(filename, json)?;
    Ok(())
}

/// Loads a layout tree from a JSON file
pub fn load_template(filename: &str) -> std::io::Result<TilingManager> {
    let path = format!("{}/{}", TEMPLATE_DIR, filename);
    let content = fs::read_to_string(path)?;
    let manager: TilingManager = serde_json::from_str(&content)?;
    Ok(manager)
}

/// Lists all available .json files
pub fn list_templates() -> std::io::Result<Vec<String>> {
    init()?;
    let mut files = Vec::new();
    for entry in fs::read_dir(TEMPLATE_DIR)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            // CHANGED: Look for .json files
            if ext == "json" {
                if let Some(name) = path.file_name() {
                    files.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    files.sort();
    Ok(files)
}