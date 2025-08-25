use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use plist::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use fs_extra::dir::{self, CopyOptions};

#[derive(Debug, Clone, ValueEnum)]
enum DefaultLocation {
    Resources,
    MacOS,
    Contents,
}

/// A macOS app generator that packages an executable into an .app bundle
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the executable file to package
    #[arg(short, long)]
    executable: String,

    /// Name of the app (without .app extension)
    #[arg(short, long)]
    name: String,

    /// Optional icon file path (.icns format)
    #[arg(short, long)]
    icon: Option<String>,

    /// Optional app version
    #[arg(short = 'v', long, default_value = "1.0.0")]
    app_version: String,

    /// Optional bundle identifier
    #[arg(short, long, default_value = "com.example.app")]
    bundle_id: String,

    /// Output directory (app will be created as {output}/{name}.app)
    #[arg(short, long, default_value = ".")]
    output: String,
    
    /// Additional files or directories to include in the app bundle
    /// Format: source_path:target_location
    /// Example: --additional-file data.txt:Resources/data.txt
    /// Example: --additional-file config:Contents/Resources/config
    #[arg(short = 'a', long = "additional-file", value_name = "SOURCE:TARGET")]
    additional_files: Vec<String>,
    
    /// Override the default location for additional files
    /// This option will set the base directory inside the app bundle
    /// where additional files will be copied if no specific target is provided
    #[arg(short = 'd', long = "default-location", value_enum, default_value = "resources")]
    default_location: DefaultLocation,
    
    /// Show terminal window when the application runs
    /// By default, the terminal window is hidden
    #[arg(short = 't', long = "show-terminal", default_value_t = false)]
    show_terminal: bool,
    
    /// Enable single instance mode to ensure only one instance of the app runs per user
    /// This adds code to prevent multiple instances of the application from running simultaneously
    #[arg(short = 's', long = "single-instance", default_value_t = false)]
    single_instance: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Create the app bundle structure
    let app_path = create_app_structure(&args)?;
    
    // Copy the executable
    copy_executable(&args, &app_path)?;
    
    // Create Info.plist
    create_info_plist(&args, &app_path)?;
    
    // Copy icon if provided
    if let Some(icon_path) = &args.icon {
        copy_icon(icon_path, &app_path)?;
    }
    
    // Copy additional files if specified
    if !args.additional_files.is_empty() {
        copy_additional_files(&args, &app_path)?;
    }
    
    println!("Successfully created app bundle at: {}", app_path.display());
    
    Ok(())
}

/// Creates the basic app bundle directory structure
fn create_app_structure(args: &Args) -> Result<PathBuf> {
    let app_path = Path::new(&args.output)
        .join(format!("{}.app", args.name));
    let contents_path = app_path.join("Contents");
    let macos_path = contents_path.join("MacOS");
    let resources_path = contents_path.join("Resources");
    
    // Remove existing app if it exists
    if app_path.exists() {
        fs::remove_dir_all(&app_path)
            .context("Failed to remove existing app bundle")?;
    }
    
    // Create directory structure
    fs::create_dir_all(&macos_path)
        .context("Failed to create MacOS directory")?;
    fs::create_dir_all(&resources_path)
        .context("Failed to create Resources directory")?;
    
    Ok(app_path)
}

/// Copies the executable to the app bundle
fn copy_executable(args: &Args, app_path: &Path) -> Result<()> {
    let source_path = Path::new(&args.executable);
    let executable_name = source_path.file_name().unwrap();
    let macos_dir = app_path.join("Contents").join("MacOS");
    let target_path = macos_dir.join(executable_name);
    
    // Check if executable exists and is executable
    if !source_path.exists() {
        anyhow::bail!("Executable file not found: {}", args.executable);
    }
    
    // Copy the executable
    fs::copy(source_path, &target_path)
        .context("Failed to copy executable to app bundle")?;
    
    // Set executable permissions
    Command::new("chmod")
        .args(&["+x", &target_path.to_string_lossy()])
        .output()
        .context("Failed to set executable permissions")?;
    
    // If single instance mode is enabled, create a wrapper script
    if args.single_instance {
        // Rename the original executable
        let original_exec_path = macos_dir.join(format!("{}_original", executable_name.to_string_lossy()));
        fs::rename(&target_path, &original_exec_path)
            .context("Failed to rename original executable")?;
        
        // Create wrapper script for single instance check
        let wrapper_script = format!(r#"#!/bin/bash

# Single instance implementation
APP_NAME="{app_name}"
LOCK_FILE="/tmp/${{APP_NAME}}.lock"
PID_FILE="/tmp/${{APP_NAME}}.pid"

# Check if another instance is running
if [ -f "$LOCK_FILE" ]; then
    EXISTING_PID=$(cat "$PID_FILE" 2>/dev/null)
    if [ ! -z "$EXISTING_PID" ]; then
        if ps -p $EXISTING_PID > /dev/null; then
            # Another instance is running, activate it
            osascript -e 'tell application "{app_name}" to activate' 2>/dev/null
            exit 0
        fi
    fi
fi

# Create lock file and store PID
echo $$ > "$PID_FILE"
touch "$LOCK_FILE"

# Run the original executable
SCRIPT_DIR="$( cd "$( dirname "${{BASH_SOURCE[0]}}" )" && pwd )"
exec "$SCRIPT_DIR/{original_name}_original" "$@"

# Clean up lock on exit
trap 'rm -f "$LOCK_FILE" "$PID_FILE"' EXIT
"#, app_name = args.name, original_name = executable_name.to_string_lossy());
        
        fs::write(&target_path, wrapper_script)
            .context("Failed to write single instance wrapper script")?;
        
        Command::new("chmod")
            .args(&["+x", &target_path.to_string_lossy()])
            .output()
            .context("Failed to set permissions on wrapper script")?;
    }
    
    Ok(())
}

/// Creates the Info.plist file for the app bundle
fn create_info_plist(args: &Args, app_path: &Path) -> Result<()> {
    let executable_name = Path::new(&args.executable)
        .file_name()
        .unwrap()
        .to_string_lossy();
    
    // Create initial plist entries
    let mut plist_entries = vec![
        ("CFBundleName".to_string(), Value::String(args.name.clone())),
        ("CFBundleDisplayName".to_string(), Value::String(args.name.clone())),
        ("CFBundleIdentifier".to_string(), Value::String(args.bundle_id.clone())),
        ("CFBundleVersion".to_string(), Value::String(args.app_version.clone())),
        ("CFBundleShortVersionString".to_string(), Value::String(args.app_version.clone())),
        ("CFBundleExecutable".to_string(), Value::String(executable_name.to_string())),
        ("CFBundleIconFile".to_string(), Value::String(if args.icon.is_some() {
            Path::new(args.icon.as_ref().unwrap())
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        } else {
            "".to_string()
        })),
        ("CFBundlePackageType".to_string(), Value::String("APPL".to_string())),
        ("LSMinimumSystemVersion".to_string(), Value::String("10.10.0".to_string())),
        ("LSUIElement".to_string(), Value::Boolean(!args.show_terminal)), // Controls terminal window visibility
        ("NSHighResolutionCapable".to_string(), Value::Boolean(true)),
    ];
    
    // Add single instance configuration if enabled
    if args.single_instance {
        plist_entries.push(("JVFApplicationLaunchOnlyIfForeground".to_string(), Value::Boolean(false)));
        plist_entries.push(("JVFApplicationActivateOnLaunch".to_string(), Value::Boolean(true)));
        plist_entries.push(("JVFApplicationSingleInstanceModeEnabled".to_string(), Value::Boolean(true)));
    }
    
    let plist_data = plist::Dictionary::from_iter(plist_entries);
    
    let plist_path = app_path.join("Contents").join("Info.plist");
    
    let file = std::fs::File::create(plist_path)
        .context("Failed to create Info.plist file")?;
    
    plist::to_writer_xml(file, &Value::Dictionary(plist_data))
        .context("Failed to write Info.plist content")?;
    
    Ok(())
}

/// Copies the icon file to the app bundle
fn copy_icon(icon_path: &str, app_path: &Path) -> Result<()> {
    let source_path = Path::new(icon_path);
    
    // Check if the icon exists and is a .icns file
    if !source_path.exists() {
        anyhow::bail!("Icon file not found: {}", icon_path);
    }
    
    if source_path.extension().unwrap_or_default() != "icns" {
        println!("Warning: Icon should be in .icns format for best results");
    }
    
    let target_path = app_path
        .join("Contents")
        .join("Resources")
        .join(source_path.file_name().unwrap());
    
    // Copy the icon
    fs::copy(source_path, target_path)
        .context("Failed to copy icon to app bundle")?;
    
    Ok(())
}

/// Copy additional files or directories to the app bundle
fn copy_additional_files(args: &Args, app_path: &Path) -> Result<()> {
    for file_entry in &args.additional_files {
        // Split the entry by colon to get source and target paths
        let parts: Vec<&str> = file_entry.split(':').collect();
        let source_path = parts[0];
        
        // Determine the target path
        let target_path = if parts.len() > 1 && !parts[1].is_empty() {
            // User provided specific target path
            app_path.join("Contents").join(parts[1])
        } else {
            // Use default location
            match args.default_location {
                DefaultLocation::Resources => app_path.join("Contents").join("Resources"),
                DefaultLocation::MacOS => app_path.join("Contents").join("MacOS"),
                DefaultLocation::Contents => app_path.join("Contents"),
            }.join(Path::new(source_path).file_name().unwrap())
        };
        
        let source = Path::new(source_path);
        
        if !source.exists() {
            anyhow::bail!("Source file or directory not found: {}", source_path);
        }
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {}", parent.display()))?;
        }
        
        // Copy file or directory
        if source.is_file() {
            // Copy file
            fs::copy(source, &target_path)
                .context(format!("Failed to copy file {} to {}", source_path, target_path.display()))?;
            
            // Check if source is executable, and if so, set permissions on target
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = fs::metadata(source)
                    .context(format!("Failed to get metadata for {}", source_path))?;
                let permissions = metadata.permissions();
                
                if permissions.mode() & 0o111 != 0 {
                    Command::new("chmod")
                        .args(&["+x", &target_path.to_string_lossy()])
                        .output()
                        .context(format!("Failed to set executable permissions on {}", target_path.display()))?;
                }
            }
        } else {
            // Copy directory recursively
            let options = CopyOptions::new()
                .overwrite(true)
                .copy_inside(true);
            
            dir::copy(source, target_path.parent().unwrap(), &options)
                .context(format!("Failed to copy directory {} to {}", source_path, target_path.parent().unwrap().display()))?;
        }
        
        println!("Added {} to {}", source_path, target_path.display());
    }
    
    Ok(())
}
