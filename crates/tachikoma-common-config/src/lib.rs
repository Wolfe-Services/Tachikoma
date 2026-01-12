//! Configuration types for Tachikoma.
//!
//! This crate provides the configuration types used by Tachikoma
//! for `.tachikoma/config.yaml` files.

pub mod types;
pub mod loader;
pub mod env;

#[cfg(test)]
mod integration_test;

pub use types::*;
pub use loader::*;
pub use env::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_sensible_values() {
        let config = TachikomaConfig::default();
        
        // Check backend defaults
        assert_eq!(config.backend.brain, "claude");
        assert_eq!(config.backend.think_tank, "o3");
        assert!(config.backend.api_keys.is_empty());
        assert!(config.backend.endpoints.is_empty());
        
        // Check loop config defaults
        assert_eq!(config.loop_config.max_iterations, 100);
        assert_eq!(config.loop_config.redline_threshold, 0.75);
        assert_eq!(config.loop_config.iteration_delay_ms, 1000);
        assert_eq!(config.loop_config.stop_on.len(), 3);
        assert!(config.loop_config.stop_on.contains(&StopCondition::Redline));
        assert!(config.loop_config.stop_on.contains(&StopCondition::TestFailStreak(3)));
        assert!(config.loop_config.stop_on.contains(&StopCondition::NoProgress(5)));
        
        // Check policy defaults
        assert!(config.policies.deploy_requires_tests);
        assert!(config.policies.attended_by_default);
        assert!(!config.policies.auto_commit);
        assert!(!config.policies.auto_push);
        assert!(config.policies.require_spec);
        
        // Check forge defaults
        assert_eq!(config.forge.participants.len(), 2);
        assert!(config.forge.participants.contains(&"claude".to_string()));
        assert!(config.forge.participants.contains(&"gemini".to_string()));
        assert_eq!(config.forge.oracle, "o3");
        assert_eq!(config.forge.max_rounds, 5);
        assert_eq!(config.forge.convergence_threshold, 0.9);
    }

    #[test]
    fn test_config_serializes_to_yaml() {
        let config = TachikomaConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        
        // Check that key sections are present
        assert!(yaml.contains("backend:"));
        assert!(yaml.contains("loop_config:"));
        assert!(yaml.contains("policies:"));
        assert!(yaml.contains("forge:"));
        
        // Check that defaults are serialized
        assert!(yaml.contains("brain: claude"));
        assert!(yaml.contains("think_tank: o3"));
        assert!(yaml.contains("max_iterations: 100"));
    }

    #[test]
    fn test_partial_configs_merge_with_defaults() {
        // Test that we can deserialize partial configs
        let partial_yaml = r#"
backend:
  brain: gpt-4
loop_config:
  max_iterations: 50
"#;
        
        let config: TachikomaConfig = serde_yaml::from_str(partial_yaml).unwrap();
        
        // Check that specified values override defaults
        assert_eq!(config.backend.brain, "gpt-4");
        assert_eq!(config.loop_config.max_iterations, 50);
        
        // Check that unspecified values use defaults
        assert_eq!(config.backend.think_tank, "o3");
        assert_eq!(config.loop_config.redline_threshold, 0.75);
        assert!(config.policies.deploy_requires_tests);
    }

    #[test]
    fn test_stop_condition_enum_parses_correctly() {
        // Test StopCondition serialization/deserialization
        let conditions = vec![
            StopCondition::Redline,
            StopCondition::TestFailStreak(5),
            StopCondition::NoProgress(10),
            StopCondition::ErrorRate(25),
            StopCondition::ManualStop,
            StopCondition::AllComplete,
        ];
        
        for condition in conditions {
            let yaml = serde_yaml::to_string(&condition).unwrap();
            let parsed: StopCondition = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(condition, parsed);
        }
        
        // Test that snake_case is used
        let test_fail = StopCondition::TestFailStreak(3);
        let yaml = serde_yaml::to_string(&test_fail).unwrap();
        assert!(yaml.contains("test_fail_streak"));
    }

    #[test]
    fn test_all_types_implement_required_traits() {
        // Test Debug, Clone, and PartialEq where applicable
        let config = TachikomaConfig::default();
        let cloned = config.clone();
        
        // Should be able to format debug output
        assert!(!format!("{:?}", config).is_empty());
        assert!(!format!("{:?}", cloned).is_empty());
        
        // Test StopCondition PartialEq
        assert_eq!(StopCondition::Redline, StopCondition::Redline);
        assert_ne!(StopCondition::Redline, StopCondition::ManualStop);
        assert_eq!(StopCondition::TestFailStreak(3), StopCondition::TestFailStreak(3));
        assert_ne!(StopCondition::TestFailStreak(3), StopCondition::TestFailStreak(5));
    }
}