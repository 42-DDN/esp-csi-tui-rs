// --- File: src/config_manager.rs ---
// --- Purpose: Handles File I/O for saving/loading templates and managing defaults ---

use std::fs;
use std::path::Path;
use crate::layout_tree::TilingManager;

// Points to "project/templates/" (Sibling to src/)
// This relies on the application being run from the project root (standard cargo behavior)
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
    init()?;
    let json = serde_json::to_string_pretty(manager)?;
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

/// Lists all available .json files with their default status
/// Returns: Vec<(filename, is_default)>
pub fn list_templates() -> std::io::Result<Vec<(String, bool)>> {
    init()?;
    let mut files = Vec::new();
    for entry in fs::read_dir(TEMPLATE_DIR)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "json" {
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy().to_string();
                    // Peek inside to see if it's default
                    let is_default = is_template_default(&name_str).unwrap_or(false);
                    files.push((name_str, is_default));
                }
            }
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

/// Helper to peek at JSON without full parsing if possible, or just load it
fn is_template_default(filename: &str) -> std::io::Result<bool> {
    let tm = load_template(filename)?;
    Ok(tm.is_default)
}

/// Iterates through all templates to find the one marked default
pub fn load_startup_template() -> Option<TilingManager> {
    if let Ok(files) = list_templates() {
        for (filename, is_default) in files {
            if is_default {
                if let Ok(tm) = load_template(&filename) {
                    return Some(tm);
                }
            }
        }
    }
    None
}

/// Sets the given template as default, unsetting others
pub fn set_default_template(target_filename: &str) -> std::io::Result<()> {
    let files = list_templates()?;

    for (filename, is_default) in files {
        if filename == target_filename {
            // Set this one to true
            let mut tm = load_template(&filename)?;
            if !tm.is_default {
                tm.is_default = true;
                save_template(&filename.replace(".json", ""), &tm)?;
            }
        } else if is_default {
            // Unset previous default
            let mut tm = load_template(&filename)?;
            tm.is_default = false;
            save_template(&filename.replace(".json", ""), &tm)?;
        }
    }
    Ok(())
}