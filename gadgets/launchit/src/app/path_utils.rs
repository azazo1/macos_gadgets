use std::path::PathBuf;
use glob::Pattern;

// Find path suggestions based on current input
pub fn find_path_suggestions(current_path: &str) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    // If empty, start with root or home
    if current_path.is_empty() || current_path == "/" {
        suggestions.push("/".to_string());
        if let Some(home) = dirs::home_dir() {
            if let Some(home_str) = home.to_str() {
                suggestions.push(home_str.to_string());
            }
        }
        return suggestions;
    }
    
    // Determine the directory to search in and the pattern to match against
    let (dir_to_search, pattern_to_match) = if current_path.ends_with('/') {
        // If path ends with /, we're looking for contents inside a directory
        (PathBuf::from(current_path), "*".to_string())
    } else {
        // Otherwise we're looking for completions of the current path component
        let path = PathBuf::from(current_path);
        if let Some(parent) = path.parent() {
            if let Some(filename) = path.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    (PathBuf::from(parent), format!("{}*", filename_str))
                } else {
                    return suggestions;
                }
            } else {
                return suggestions;
            }
        } else {
            // If no parent, we're at the root
            (PathBuf::from("/"), format!("{}*", current_path))
        }
    };
    
    // Now search for matching files/directories
    if let Some(dir_str) = dir_to_search.to_str() {
        if let Ok(pattern) = Pattern::new(&pattern_to_match) {
            if let Ok(entries) = std::fs::read_dir(dir_to_search) {
                for entry in entries.flatten() {
                    if let Ok(filename) = entry.file_name().into_string() {
                        if pattern.matches(&filename) {
                            let path = entry.path();
                            if let Some(path_str) = path.to_str() {
                                let display_path = if path.is_dir() {
                                    format!("{}/", path_str)
                                } else {
                                    path_str.to_string()
                                };
                                suggestions.push(display_path);
                            }
                        }
                    }
                }
            }
        }
    }
    
    suggestions
}
