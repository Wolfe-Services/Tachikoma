//! Plugin discovery and loading
//!
//! Handles finding and loading plugins from the `.tachikoma/plugins/` directory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{PluginError, PluginManifest, PluginType, Result};

/// Plugin discovery and loading
pub struct PluginLoader {
    plugins_dir: PathBuf,
    manifests: HashMap<String, PluginManifest>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new(project_root: &Path) -> Self {
        Self {
            plugins_dir: project_root.join(".tachikoma/plugins"),
            manifests: HashMap::new(),
        }
    }
    
    /// Discover and load all plugin manifests
    pub fn discover(&mut self) -> Result<()> {
        self.manifests.clear();
        
        // Load agents
        self.discover_type(&self.plugins_dir.join("agents"), PluginType::Agent)?;
        
        // Load trackers
        self.discover_type(&self.plugins_dir.join("trackers"), PluginType::Tracker)?;
        
        // Load templates (templates don't have manifests, just directories)
        // Templates are handled separately by the TemplateEngine
        
        Ok(())
    }
    
    /// Discover plugins of a specific type
    fn discover_type(&mut self, dir: &Path, expected_type: PluginType) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let manifest_path = path.join("plugin.yaml");
                
                if manifest_path.exists() {
                    let content = std::fs::read_to_string(&manifest_path)?;
                    let manifest: PluginManifest = serde_yaml::from_str(&content)?;
                    
                    // Validate type
                    if manifest.plugin_type != expected_type {
                        return Err(PluginError::InvalidManifest(format!(
                            "Plugin {} is type {:?} but found in {:?} directory",
                            manifest.name, manifest.plugin_type, expected_type
                        )));
                    }
                    
                    self.manifests.insert(manifest.name.clone(), manifest);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a plugin manifest by name
    pub fn get_manifest(&self, name: &str) -> Option<&PluginManifest> {
        self.manifests.get(name)
    }
    
    /// List all plugin manifests
    pub fn list_manifests(&self) -> Vec<&PluginManifest> {
        self.manifests.values().collect()
    }
    
    /// List plugins of a specific type
    pub fn list_by_type(&self, plugin_type: PluginType) -> Vec<&PluginManifest> {
        self.manifests
            .values()
            .filter(|m| m.plugin_type == plugin_type)
            .collect()
    }
    
    /// Validate all loaded plugins
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();
        
        for manifest in self.manifests.values() {
            if let Err(errors) = manifest.check_requirements() {
                for err in errors {
                    warnings.push(format!("{}: requirement not met - {}", manifest.name, err));
                }
            }
        }
        
        Ok(warnings)
    }
    
    /// Get the plugin directory path
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }
    
    /// Get path to a specific plugin
    pub fn plugin_path(&self, name: &str, plugin_type: PluginType) -> PathBuf {
        let type_dir = match plugin_type {
            PluginType::Agent => "agents",
            PluginType::Tracker => "trackers",
            PluginType::Template => "templates",
        };
        
        self.plugins_dir.join(type_dir).join(name)
    }
    
    /// List available template sets
    pub fn list_template_sets(&self) -> Result<Vec<String>> {
        let templates_dir = self.plugins_dir.join("templates");
        
        if !templates_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut sets = Vec::new();
        
        for entry in std::fs::read_dir(&templates_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    sets.push(name.to_string());
                }
            }
        }
        
        Ok(sets)
    }
    
    /// Initialize plugin directory structure
    pub fn init_plugin_dir(&self) -> Result<()> {
        std::fs::create_dir_all(self.plugins_dir.join("agents"))?;
        std::fs::create_dir_all(self.plugins_dir.join("trackers"))?;
        std::fs::create_dir_all(self.plugins_dir.join("templates/default"))?;
        
        // Create default templates
        let default_system = include_str!("../templates/system-prompt.hbs.default");
        let default_task = include_str!("../templates/task-prompt.hbs.default");
        
        let system_path = self.plugins_dir.join("templates/default/system-prompt.hbs");
        let task_path = self.plugins_dir.join("templates/default/task-prompt.hbs");
        
        if !system_path.exists() {
            std::fs::write(&system_path, default_system)?;
        }
        
        if !task_path.exists() {
            std::fs::write(&task_path, default_task)?;
        }
        
        Ok(())
    }
}
