//! Tool and environment detection.

use std::path::PathBuf;
use std::process::Command;
use std::collections::HashMap;
use crate::env::ApiKeys;

/// Tool detection results.
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub available: bool,
}

/// System environment status.
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub tools: HashMap<String, ToolInfo>,
    pub api_keys: HashMap<String, bool>,
    pub os: String,
    pub arch: String,
}

/// Environment detection utilities.
pub struct Detection;

impl Detection {
    /// Detect all relevant tools and environment.
    pub async fn scan() -> SystemStatus {
        let mut tools = HashMap::new();
        
        // Core development tools
        tools.insert("rust".to_string(), Self::detect_rust().await);
        tools.insert("node".to_string(), Self::detect_node().await);
        tools.insert("npm".to_string(), Self::detect_npm().await);
        
        // Version control
        tools.insert("jj".to_string(), Self::detect_jj().await);
        tools.insert("git".to_string(), Self::detect_git().await);
        
        // Utilities
        tools.insert("ripgrep".to_string(), Self::detect_ripgrep().await);
        tools.insert("fd".to_string(), Self::detect_fd().await);
        
        // API keys
        let mut api_keys = HashMap::new();
        api_keys.insert("anthropic".to_string(), ApiKeys::anthropic().is_some());
        api_keys.insert("openai".to_string(), ApiKeys::openai().is_some());
        api_keys.insert("google".to_string(), ApiKeys::google().is_some());
        
        SystemStatus {
            tools,
            api_keys,
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }
    
    /// Detect a specific tool.
    pub async fn detect_tool(name: &str) -> ToolInfo {
        match name {
            "rust" | "rustc" => Self::detect_rust().await,
            "node" | "nodejs" => Self::detect_node().await,
            "npm" => Self::detect_npm().await,
            "jj" => Self::detect_jj().await,
            "git" => Self::detect_git().await,
            "ripgrep" | "rg" => Self::detect_ripgrep().await,
            "fd" => Self::detect_fd().await,
            _ => ToolInfo {
                name: name.to_string(),
                version: None,
                path: None,
                available: false,
            }
        }
    }
    
    async fn detect_rust() -> ToolInfo {
        Self::detect_with_version_arg("rustc", &["--version"]).await
    }
    
    async fn detect_node() -> ToolInfo {
        Self::detect_with_version_arg("node", &["--version"]).await
    }
    
    async fn detect_npm() -> ToolInfo {
        Self::detect_with_version_arg("npm", &["--version"]).await
    }
    
    async fn detect_jj() -> ToolInfo {
        Self::detect_with_version_arg("jj", &["--version"]).await
    }
    
    async fn detect_git() -> ToolInfo {
        Self::detect_with_version_arg("git", &["--version"]).await
    }
    
    async fn detect_ripgrep() -> ToolInfo {
        Self::detect_with_version_arg("rg", &["--version"]).await
    }
    
    async fn detect_fd() -> ToolInfo {
        Self::detect_with_version_arg("fd", &["--version"]).await
    }
    
    async fn detect_with_version_arg(command: &str, args: &[&str]) -> ToolInfo {
        let mut tool_info = ToolInfo {
            name: command.to_string(),
            version: None,
            path: None,
            available: false,
        };
        
        // Try to get the path first
        if let Ok(output) = Command::new("which").arg(command).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    tool_info.path = Some(PathBuf::from(&path_str));
                }
            }
        }
        
        // Try to get version
        if let Ok(output) = Command::new(command).args(args).output() {
            if output.status.success() {
                tool_info.available = true;
                let output_str = String::from_utf8_lossy(&output.stdout);
                tool_info.version = Self::extract_version(&output_str);
            }
        }
        
        tool_info
    }
    
    fn extract_version(output: &str) -> Option<String> {
        // Extract version from common patterns
        for line in output.lines() {
            let line = line.trim();
            
            // Pattern: "tool 1.2.3" or "tool version 1.2.3"
            let words: Vec<&str> = line.split_whitespace().collect();
            for word in &words {
                // Look for semantic version pattern
                if Self::looks_like_version(word) {
                    return Some(word.to_string());
                }
            }
            
            // Pattern: "version 1.2.3" at start of line
            if line.to_lowercase().starts_with("version ") {
                if let Some(version) = line.split_whitespace().nth(1) {
                    if Self::looks_like_version(version) {
                        return Some(version.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    fn looks_like_version(s: &str) -> bool {
        // Check if string looks like a version number
        let chars: Vec<char> = s.chars().collect();
        
        // Must start with a digit
        if chars.is_empty() || !chars[0].is_ascii_digit() {
            return false;
        }
        
        // Must contain at least one dot
        if !s.contains('.') {
            return false;
        }
        
        // Should be mostly digits and dots
        let valid_chars = chars.iter().all(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || c.is_ascii_alphabetic());
        
        valid_chars && s.len() >= 3 // Minimum "1.0"
    }
    
    /// Check if essential tools are available.
    pub async fn check_requirements() -> Vec<String> {
        let mut missing = Vec::new();
        
        let rust = Self::detect_rust().await;
        if !rust.available {
            missing.push("Rust toolchain (install from https://rustup.rs/)".to_string());
        }
        
        let node = Self::detect_node().await;
        if !node.available {
            missing.push("Node.js (install from https://nodejs.org/)".to_string());
        }
        
        // Version control (at least one)
        let jj = Self::detect_jj().await;
        let git = Self::detect_git().await;
        if !jj.available && !git.available {
            missing.push("Version control system (install jj or git)".to_string());
        }
        
        // API keys (at least one)
        if ApiKeys::anthropic().is_none() && ApiKeys::openai().is_none() && ApiKeys::google().is_none() {
            missing.push("API key (set ANTHROPIC_API_KEY, OPENAI_API_KEY, or GOOGLE_API_KEY)".to_string());
        }
        
        missing
    }
    
    /// Get installation recommendations for missing tools.
    pub fn install_recommendations() -> HashMap<String, Vec<String>> {
        let mut recommendations = HashMap::new();
        
        let os = std::env::consts::OS;
        
        // Rust
        recommendations.insert("rust".to_string(), vec![
            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".to_string(),
            "Or visit: https://rustup.rs/".to_string(),
        ]);
        
        // Node.js
        let node_install = match os {
            "macos" => vec!["brew install node".to_string()],
            "linux" => vec!["sudo apt install nodejs npm".to_string(), "Or use your package manager".to_string()],
            _ => vec!["Visit: https://nodejs.org/".to_string()],
        };
        recommendations.insert("node".to_string(), node_install);
        
        // jj
        let jj_install = match os {
            "macos" => vec!["brew install jj".to_string()],
            "linux" => vec!["cargo install jj-cli".to_string()],
            _ => vec!["cargo install jj-cli".to_string()],
        };
        recommendations.insert("jj".to_string(), jj_install);
        
        // git
        let git_install = match os {
            "macos" => vec!["brew install git".to_string()],
            "linux" => vec!["sudo apt install git".to_string()],
            _ => vec!["Visit: https://git-scm.com/downloads".to_string()],
        };
        recommendations.insert("git".to_string(), git_install);
        
        // ripgrep
        let rg_install = match os {
            "macos" => vec!["brew install ripgrep".to_string()],
            "linux" => vec!["sudo apt install ripgrep".to_string()],
            _ => vec!["cargo install ripgrep".to_string()],
        };
        recommendations.insert("ripgrep".to_string(), rg_install);
        
        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_detection() {
        // Test various version patterns
        assert!(Detection::looks_like_version("1.2.3"));
        assert!(Detection::looks_like_version("1.0"));
        assert!(Detection::looks_like_version("2.1.4-beta"));
        assert!(!Detection::looks_like_version("abc"));
        assert!(!Detection::looks_like_version(""));
        assert!(!Detection::looks_like_version("1"));
        
        // Test version extraction
        assert_eq!(Detection::extract_version("rustc 1.75.0"), Some("1.75.0".to_string()));
        assert_eq!(Detection::extract_version("node v20.10.0"), Some("v20.10.0".to_string()));
        assert_eq!(Detection::extract_version("jj 0.12.0"), Some("0.12.0".to_string()));
        assert_eq!(Detection::extract_version("version 1.0.0"), Some("1.0.0".to_string()));
    }
    
    #[tokio::test]
    async fn test_tool_detection_doesnt_panic() {
        // These might fail on systems without tools, but shouldn't panic
        let _rust = Detection::detect_rust().await;
        let _node = Detection::detect_node().await;
        let _git = Detection::detect_git().await;
        let _jj = Detection::detect_jj().await;
    }
    
    #[tokio::test]
    async fn test_full_system_scan() {
        let status = Detection::scan().await;
        
        // Should have detected OS and arch
        assert!(!status.os.is_empty());
        assert!(!status.arch.is_empty());
        
        // Should have attempted to detect core tools
        assert!(status.tools.contains_key("rust"));
        assert!(status.tools.contains_key("node"));
        assert!(status.tools.contains_key("git"));
        
        // Should have checked for API keys
        assert!(status.api_keys.contains_key("anthropic"));
        assert!(status.api_keys.contains_key("openai"));
        assert!(status.api_keys.contains_key("google"));
    }
    
    #[test]
    fn test_install_recommendations() {
        let recommendations = Detection::install_recommendations();
        
        // Should have recommendations for core tools
        assert!(recommendations.contains_key("rust"));
        assert!(recommendations.contains_key("node"));
        assert!(recommendations.contains_key("git"));
        
        // Recommendations shouldn't be empty
        for (tool, commands) in &recommendations {
            assert!(!commands.is_empty(), "No recommendations for {}", tool);
        }
    }
}