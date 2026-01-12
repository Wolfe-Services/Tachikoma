//! Template loading from various sources.

use std::path::{Path, PathBuf};

use crate::templates::{Template, TemplateFile, TemplateManifest, manifest::TemplateError};

/// Source of a template
#[derive(Debug, Clone)]
pub enum TemplateSource {
    /// Built-in template
    Builtin(String),
    /// Local directory
    Local(PathBuf),
    /// Git repository
    Git { url: String, ref_: Option<String> },
    /// Registry template
    Registry { name: String, version: Option<String> },
}

/// Template loader
pub struct TemplateLoader {
    cache_dir: PathBuf,
}

impl TemplateLoader {
    pub fn new() -> Result<Self, TemplateError> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("tachikoma")
            .join("templates");

        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// Load a template from any source
    pub async fn load(&self, source: &TemplateSource) -> Result<Template, TemplateError> {
        match source {
            TemplateSource::Builtin(name) => self.load_builtin(name),
            TemplateSource::Local(path) => self.load_local(path).await,
            TemplateSource::Git { url, ref_ } => self.load_git(url, ref_.as_deref()).await,
            TemplateSource::Registry { name, version } => {
                self.load_registry(name, version.as_deref()).await
            }
        }
    }

    /// Load a built-in template
    fn load_builtin(&self, name: &str) -> Result<Template, TemplateError> {
        crate::templates::BuiltinTemplates::get(name)
            .ok_or_else(|| TemplateError::NotFound(name.to_string()))
    }

    /// Load a template from local directory
    async fn load_local(&self, path: &Path) -> Result<Template, TemplateError> {
        // Load manifest
        let manifest_path = path.join("template.toml");
        let manifest = if manifest_path.exists() {
            TemplateManifest::load(&manifest_path)?
        } else {
            // Create default manifest
            TemplateManifest {
                template: crate::templates::manifest::TemplateMetadata {
                    name: path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "custom".to_string()),
                    description: "Custom template".to_string(),
                    version: "0.1.0".to_string(),
                    authors: vec![],
                    tags: vec![],
                    min_tachikoma_version: None,
                },
                variables: vec![],
                files: Default::default(),
                hooks: Default::default(),
            }
        };

        // Load files
        let files = self.load_files(path, &manifest).await?;

        Ok(Template {
            name: manifest.template.name.clone(),
            description: manifest.template.description.clone(),
            source: TemplateSource::Local(path.to_path_buf()),
            manifest,
            files,
        })
    }

    /// Load a template from Git repository
    async fn load_git(&self, url: &str, ref_: Option<&str>) -> Result<Template, TemplateError> {
        // Create cache key from URL
        let cache_key = url
            .replace("://", "_")
            .replace('/', "_")
            .replace('.', "_");

        let cache_path = self.cache_dir.join(&cache_key);

        // Clone or update repository
        if cache_path.exists() {
            // Update existing clone
            let status = tokio::process::Command::new("git")
                .args(["pull"])
                .current_dir(&cache_path)
                .output()
                .await
                .map_err(|e| TemplateError::Io(e))?;

            if !status.status.success() {
                // If pull fails, remove and re-clone
                std::fs::remove_dir_all(&cache_path)?;
            }
        }

        if !cache_path.exists() {
            // Clone repository
            let mut args = vec!["clone", "--depth", "1"];

            if let Some(r) = ref_ {
                args.extend(["--branch", r]);
            }

            args.extend([url, cache_path.to_str().unwrap()]);

            let status = tokio::process::Command::new("git")
                .args(&args)
                .output()
                .await
                .map_err(|e| TemplateError::Io(e))?;

            if !status.status.success() {
                return Err(TemplateError::NotFound(format!(
                    "Failed to clone: {}",
                    String::from_utf8_lossy(&status.stderr)
                )));
            }
        }

        self.load_local(&cache_path).await
    }

    /// Load a template from registry
    async fn load_registry(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<Template, TemplateError> {
        // Registry URL would be configurable
        let registry_url = "https://registry.tachikoma.dev/templates";

        let url = match version {
            Some(v) => format!("{}/{}/{}", registry_url, name, v),
            None => format!("{}/{}/latest", registry_url, name),
        };

        // Fetch template info
        let response = reqwest::get(&url)
            .await
            .map_err(|e| TemplateError::NotFound(e.to_string()))?;

        if !response.status().is_success() {
            return Err(TemplateError::NotFound(format!(
                "Template '{}' not found in registry",
                name
            )));
        }

        #[derive(serde::Deserialize)]
        struct RegistryEntry {
            git_url: String,
            git_ref: Option<String>,
        }

        let entry: RegistryEntry = response
            .json()
            .await
            .map_err(|e| TemplateError::NotFound(e.to_string()))?;

        self.load_git(&entry.git_url, entry.git_ref.as_deref()).await
    }

    /// Load files from a template directory
    async fn load_files(
        &self,
        path: &Path,
        manifest: &TemplateManifest,
    ) -> Result<Vec<TemplateFile>, TemplateError> {
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                // Skip hidden files and template.toml
                !name.starts_with('.') && name != "template.toml"
            })
        {
            let entry = entry.map_err(|e| TemplateError::Io(e.into()))?;

            if !entry.file_type().is_file() {
                continue;
            }

            let relative_path = entry.path().strip_prefix(path).unwrap();

            // Check include/exclude patterns
            let path_str = relative_path.to_string_lossy();

            let excluded = manifest.files.exclude.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(&path_str))
                    .unwrap_or(false)
            });

            if excluded {
                continue;
            }

            // Check if should be processed
            let no_process = manifest.files.no_process.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(&path_str))
                    .unwrap_or(false)
            });

            // Read content
            let content = std::fs::read_to_string(entry.path())?;

            // Check if executable
            #[cfg(unix)]
            let executable = {
                use std::os::unix::fs::PermissionsExt;
                entry.metadata().map_err(|e| TemplateError::Io(e))?.permissions().mode() & 0o111 != 0
            };

            #[cfg(not(unix))]
            let executable = false;

            files.push(TemplateFile {
                path: relative_path.to_path_buf(),
                content,
                process: !no_process,
                executable,
            });
        }

        Ok(files)
    }

    /// List available templates from all sources
    pub async fn list_all(&self) -> Vec<TemplateInfo> {
        let mut templates = Vec::new();

        // Built-in templates
        templates.extend(crate::templates::BuiltinTemplates::list());

        // Cached templates
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(template) = self.load_local(&entry.path()).await {
                    templates.push(TemplateInfo {
                        name: template.name,
                        description: template.description,
                        source: "cached".to_string(),
                    });
                }
            }
        }

        templates
    }
}

impl Default for TemplateLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create template loader")
    }
}

/// Summary information about a template
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub source: String,
}