//! Test fixture loading and management.

use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Global fixture cache
static FIXTURE_CACHE: OnceLock<HashMap<PathBuf, String>> = OnceLock::new();

/// Get the fixtures directory path
pub fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".into());

    // Try multiple possible locations
    let candidates = [
        PathBuf::from(&manifest_dir).join("tests/fixtures"),
        PathBuf::from(&manifest_dir).join("../tests/fixtures"),
        PathBuf::from(&manifest_dir).join("../../tests/fixtures"),
        PathBuf::from("tests/fixtures"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    // Default to first candidate
    candidates[0].clone()
}

/// Load a fixture as a string
pub fn load_fixture(path: impl AsRef<Path>) -> String {
    let full_path = fixtures_dir().join(path.as_ref());
    
    // Check cache first
    let cache = FIXTURE_CACHE.get_or_init(|| HashMap::new());
    if let Some(cached) = cache.get(&full_path) {
        return cached.clone();
    }
    
    let content = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {:?}: {}", full_path, e));
    
    // Cache the result for performance
    let mut cache = HashMap::new();
    if let Some(existing_cache) = FIXTURE_CACHE.get() {
        cache.clone_from(existing_cache);
    }
    cache.insert(full_path, content.clone());
    
    content
}

/// Load a fixture as bytes
pub fn load_fixture_bytes(path: impl AsRef<Path>) -> Vec<u8> {
    let full_path = fixtures_dir().join(path.as_ref());
    std::fs::read(&full_path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {:?}: {}", full_path, e))
}

/// Load a JSON fixture and deserialize
pub fn load_json_fixture<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    let content = load_fixture(path);
    serde_json::from_str(&content).expect("Failed to parse JSON fixture")
}

/// Load a YAML fixture and deserialize
pub fn load_yaml_fixture<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    let content = load_fixture(path);
    serde_yaml::from_str(&content).expect("Failed to parse YAML fixture")
}

/// Fixture with template variables
pub struct TemplatedFixture {
    template: String,
    variables: HashMap<String, String>,
}

impl TemplatedFixture {
    /// Load a template fixture
    pub fn load(path: impl AsRef<Path>) -> Self {
        Self {
            template: load_fixture(path),
            variables: HashMap::new(),
        }
    }

    /// Set a variable value
    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Set multiple variables
    pub fn vars(mut self, vars: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        for (k, v) in vars {
            self.variables.insert(k.into(), v.into());
        }
        self
    }

    /// Render the template with substitutions
    pub fn render(&self) -> String {
        let mut result = self.template.clone();
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Render and parse as JSON
    pub fn render_json<T: DeserializeOwned>(&self) -> T {
        let rendered = self.render();
        serde_json::from_str(&rendered).expect("Failed to parse rendered template as JSON")
    }
}

/// Fixture set for parameterized tests
pub struct FixtureSet<T> {
    fixtures: Vec<(String, T)>,
}

impl<T: DeserializeOwned> FixtureSet<T> {
    /// Load all fixtures from a directory
    pub fn from_dir(dir: impl AsRef<Path>) -> Self {
        let full_dir = fixtures_dir().join(dir.as_ref());
        let mut fixtures = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&full_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    let name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let content = std::fs::read_to_string(&path)
                        .expect("Failed to read fixture");
                    let data: T = serde_json::from_str(&content)
                        .expect("Failed to parse fixture");
                    fixtures.push((name, data));
                }
            }
        }

        Self { fixtures }
    }

    /// Get iterator over fixtures
    pub fn iter(&self) -> impl Iterator<Item = &(String, T)> {
        self.fixtures.iter()
    }

    /// Get fixture by name
    pub fn get(&self, name: &str) -> Option<&T> {
        self.fixtures.iter()
            .find(|(n, _)| n == name)
            .map(|(_, data)| data)
    }
}

/// Macro for inline fixture definitions
#[macro_export]
macro_rules! fixture {
    // JSON fixture
    (json: $($json:tt)+) => {
        serde_json::json!($($json)+)
    };

    // YAML fixture (string)
    (yaml: $yaml:expr) => {
        serde_yaml::from_str::<serde_yaml::Value>($yaml).expect("Invalid YAML")
    };

    // File fixture
    (file: $path:expr) => {
        $crate::fixtures::load_fixture($path)
    };
}

pub mod common;

// Re-export for backward compatibility
use serde::{Deserialize, Serialize};

/// Legacy test fixture struct for compatibility
#[deprecated(note = "Use the new fixture functions instead")]
pub struct TestFixture {
    name: String,
    data_dir: Option<PathBuf>,
}

#[allow(deprecated)]
impl TestFixture {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            data_dir: None,
        }
    }

    pub fn with_data_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.data_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn load_json<T>(&self, filename: &str) -> Result<T, Box<dyn std::error::Error>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let path = self.get_data_path(filename);
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read fixture file {}: {}", path.display(), e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from {}: {}", path.display(), e).into())
    }

    pub fn save_json<T>(&self, filename: &str, data: &T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Serialize,
    {
        let path = self.get_data_path(filename);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(data)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn load_text(&self, filename: &str) -> Result<String, std::io::Error> {
        let path = self.get_data_path(filename);
        std::fs::read_to_string(path)
    }

    pub fn save_text(&self, filename: &str, content: &str) -> Result<(), std::io::Error> {
        let path = self.get_data_path(filename);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }

    fn get_data_path(&self, filename: &str) -> PathBuf {
        match &self.data_dir {
            Some(dir) => dir.join(filename),
            None => PathBuf::from("tests/fixtures").join(&self.name).join(filename),
        }
    }
}

/// Common test data for API testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTestData {
    pub endpoint: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<serde_json::Value>,
    pub expected_status: u16,
    pub expected_response: Option<serde_json::Value>,
}

/// Common test data for file operations
#[derive(Debug, Clone)]
pub struct FileTestData {
    pub name: String,
    pub content: String,
    pub encoding: String,
}

impl Default for FileTestData {
    fn default() -> Self {
        Self {
            name: "test-file.txt".to_string(),
            content: "Hello, test world!".to_string(),
            encoding: "utf-8".to_string(),
        }
    }
}

/// Generate sample test data
pub fn sample_api_data() -> ApiTestData {
    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    
    ApiTestData {
        endpoint: "/api/test".to_string(),
        method: "POST".to_string(),
        headers,
        body: Some(serde_json::json!({"test": "data"})),
        expected_status: 200,
        expected_response: Some(serde_json::json!({"success": true})),
    }
}

/// Generate sample file data
pub fn sample_file_data() -> FileTestData {
    FileTestData::default()
}