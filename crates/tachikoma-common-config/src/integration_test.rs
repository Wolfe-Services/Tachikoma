use std::env;
use tempfile::tempdir;
use std::fs;

use super::*;

#[test]
fn test_full_config_loading_workflow() {
    // Set up environment variables
    env::set_var("TEST_BRAIN_MODEL", "custom-brain");
    env::set_var("TEST_MAX_ITERATIONS", "150");
    
    // Create a temporary directory structure
    let dir = tempdir().unwrap();
    let tachikoma_dir = dir.path().join(".tachikoma");
    fs::create_dir_all(&tachikoma_dir).unwrap();
    
    // Create a config file with env vars and partial configuration
    let config_content = r#"
backend:
  brain: ${TEST_BRAIN_MODEL}
  think_tank: o3-mini
  api_keys:
    claude: sk-claude-123
    openai: sk-openai-456
    
loop_config:
  max_iterations: ${TEST_MAX_ITERATIONS}
  redline_threshold: ${REDLINE:-0.85}
  
policies:
  auto_commit: true
  deploy_requires_tests: false
  
# Forge config will use defaults since not specified
"#;
    
    fs::write(tachikoma_dir.join("config.yaml"), config_content).unwrap();
    
    // Load the config
    let loader = ConfigLoader::new(dir.path());
    let config = loader.load().unwrap();
    
    // Verify environment variable expansion worked
    assert_eq!(config.backend.brain, "custom-brain");
    assert_eq!(config.loop_config.max_iterations, 150);
    assert_eq!(config.loop_config.redline_threshold, 0.85); // default value
    
    // Verify partial config merges with defaults
    assert_eq!(config.backend.think_tank, "o3-mini"); // overridden
    assert_eq!(config.loop_config.iteration_delay_ms, 1000); // default
    assert_eq!(config.policies.auto_commit, true); // overridden
    assert_eq!(config.policies.deploy_requires_tests, false); // overridden
    assert_eq!(config.policies.require_spec, true); // default
    
    // Verify forge config uses all defaults
    assert_eq!(config.forge.max_rounds, 5);
    assert_eq!(config.forge.oracle, "o3");
    assert_eq!(config.forge.convergence_threshold, 0.9);
    assert_eq!(config.forge.participants, vec!["claude", "gemini"]);
    
    // Verify API keys were loaded
    assert_eq!(config.backend.api_keys.get("claude"), Some(&"sk-claude-123".to_string()));
    assert_eq!(config.backend.api_keys.get("openai"), Some(&"sk-openai-456".to_string()));
    
    // Test saving the config back
    let mut modified_config = config.clone();
    modified_config.backend.brain = "saved-brain".to_string();
    
    loader.save(&modified_config).unwrap();
    
    // Verify saved config can be loaded
    let reloaded = loader.load().unwrap();
    assert_eq!(reloaded.backend.brain, "saved-brain");
    
    // Clean up environment variables
    env::remove_var("TEST_BRAIN_MODEL");
    env::remove_var("TEST_MAX_ITERATIONS");
}