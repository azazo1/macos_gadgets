use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlistDict {
    #[serde(rename = "Label")]
    pub label: String,
    #[serde(rename = "ProgramArguments")]
    pub program_arguments: Vec<String>,
    #[serde(rename = "RunAtLoad")]
    pub run_at_load: Option<bool>,
    #[serde(rename = "StandardOutPath")]
    pub stdout_path: Option<String>,
    #[serde(rename = "StandardErrorPath")]
    pub stderr_path: Option<String>,
    #[serde(rename = "KeepAlive")]
    pub keep_alive: Option<bool>,
    #[serde(rename = "StartInterval")]
    pub start_interval: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct LaunchAgent {
    pub label: String,
    pub path: PathBuf,
    pub pid: Option<i32>,
    pub last_exit_status: Option<i32>,
    pub program: String,
    pub arguments: Vec<String>,
}

pub fn get_user_agents() -> Result<Vec<LaunchAgent>> {
    // Get all agents for the current user
    let output = Command::new("launchctl")
        .args(["list"])
        .output()
        .context("Failed to execute launchctl list")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("launchctl list failed"));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut agents = Vec::new();

    // Parse the output to get the status of each service
    let mut statuses = std::collections::HashMap::new();
    
    for line in output_str.lines().skip(1) { // Skip the header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let pid_str = parts[0].trim();
            let status_str = parts[1].trim();
            let label = parts[2..].join(" ").trim().to_string();
            
            let pid = if pid_str == "-" {
                None
            } else {
                pid_str.parse::<i32>().ok()
            };
            
            let status = if status_str == "-" {
                None
            } else {
                status_str.parse::<i32>().ok()
            };
            
            statuses.insert(label, (pid, status));
        }
    }

    // Now check the LaunchAgents directory for plist files
    if let Some(home_dir) = dirs::home_dir() {
        let launch_agents_dir = home_dir.join("Library/LaunchAgents");
        
        if launch_agents_dir.exists() {
            for entry in fs::read_dir(launch_agents_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("plist") {
                    if let Ok(plist_content) = fs::read_to_string(&path) {
                        let plist_parsed = plist::from_bytes::<PlistDict>(plist_content.as_bytes());
                        if let Ok(dict) = plist_parsed {
                            let label = dict.label.clone();
                            let (pid, status) = statuses.get(&label).copied().unwrap_or((None, None));
                            
                            let program = dict.program_arguments.first().cloned().unwrap_or_default();
                            let arguments = if dict.program_arguments.len() > 1 {
                                dict.program_arguments[1..].to_vec()
                            } else {
                                Vec::new()
                            };
                            
                            agents.push(LaunchAgent {
                                label,
                                path,
                                pid,
                                last_exit_status: status,
                                program,
                                arguments,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(agents)
}

pub fn create_launch_agent(
    plist_path: &Path,
    domain: &str,
    program: &str,
    arguments: &[String],
    keep_alive: bool,
    stdout_path: &str,
    stderr_path: &str,
    use_interval: bool,
    interval: u32,
) -> Result<()> {
    // Create program arguments array
    let mut program_arguments = vec![program.to_string()];
    program_arguments.extend(arguments.iter().cloned());

    // Create the plist dictionary
    let dict = PlistDict {
        label: domain.to_string(),
        program_arguments,
        run_at_load: Some(true),
        stdout_path: Some(stdout_path.to_string()),
        stderr_path: Some(stderr_path.to_string()),
        keep_alive: Some(keep_alive),
        start_interval: if use_interval { Some(interval) } else { None },
    };

    // XML Header
    let mut plist_xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    plist_xml.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
    plist_xml.push_str("<plist version=\"1.0\">\n<dict>\n");
    
    // Add Label
    plist_xml.push_str(&format!("\t<key>Label</key>\n\t<string>{}</string>\n", dict.label));
    
    // Add ProgramArguments
    plist_xml.push_str("\t<key>ProgramArguments</key>\n\t<array>\n");
    for arg in &dict.program_arguments {
        plist_xml.push_str(&format!("\t\t<string>{}</string>\n", arg));
    }
    plist_xml.push_str("\t</array>\n");
    
    // Add RunAtLoad
    plist_xml.push_str("\t<key>RunAtLoad</key>\n\t<true/>\n");
    
    // Add StandardOutPath
    if let Some(path) = &dict.stdout_path {
        plist_xml.push_str(&format!("\t<key>StandardOutPath</key>\n\t<string>{}</string>\n", path));
    }
    
    // Add StandardErrorPath
    if let Some(path) = &dict.stderr_path {
        plist_xml.push_str(&format!("\t<key>StandardErrorPath</key>\n\t<string>{}</string>\n", path));
    }
    
    // Add KeepAlive
    if let Some(keep_alive) = dict.keep_alive {
        if keep_alive {
            plist_xml.push_str("\t<key>KeepAlive</key>\n\t<true/>\n");
        }
    }
    
    // Add StartInterval
    if let Some(interval) = dict.start_interval {
        plist_xml.push_str(&format!("\t<key>StartInterval</key>\n\t<integer>{}</integer>\n", interval));
    }
    
    // Close the dict and plist
    plist_xml.push_str("</dict>\n</plist>");
    
    // Add KeepAlive if needed
    let plist_xml = if let Some(keep_alive) = dict.keep_alive {
        if keep_alive {
            plist_xml.replace("</dict>", "<key>KeepAlive</key>\n\t<true/>\n</dict>")
        } else {
            plist_xml
        }
    } else {
        plist_xml
    };
    
    // Add StartInterval if needed
    let plist_xml = if let Some(interval) = dict.start_interval {
        plist_xml.replace("</dict>", &format!("<key>StartInterval</key>\n\t<integer>{}</integer>\n</dict>", interval))
    } else {
        plist_xml
    };
    
    // Write the plist file
    fs::write(plist_path, plist_xml)?;
    
    Ok(())
}
