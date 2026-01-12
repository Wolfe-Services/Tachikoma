# Spec 124: Progress Calculation

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 124
- **Status**: Planned
- **Dependencies**: 121-spec-metadata, 123-checkbox-tracking
- **Estimated Context**: ~9%

## Objective

Implement comprehensive progress calculation for specs, phases, and the overall project. Progress is computed from multiple signals including checkbox completion, spec status, implementation coverage, and test results. The system provides accurate, real-time progress metrics for project tracking.

## Acceptance Criteria

- [ ] Spec-level progress is calculated accurately
- [ ] Phase-level aggregation works correctly
- [ ] Project-level rollup is computed
- [ ] Multiple progress metrics are supported
- [ ] Weighted progress calculations work
- [ ] Progress history is tracked
- [ ] Real-time progress updates are efficient
- [ ] Progress visualization data is generated

## Implementation Details

### Progress Calculation System

```rust
// src/spec/progress.rs

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::spec::metadata::{SpecMetadata, SpecStatus};
use crate::spec::checkbox::{CheckboxTracker, CheckboxStats};

/// Progress metrics for a single spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecProgress {
    /// Spec ID
    pub spec_id: u32,
    /// Overall completion percentage (0-100)
    pub overall: u8,
    /// Component breakdowns
    pub components: ProgressComponents,
    /// Weighted score (considers importance)
    pub weighted_score: f32,
    /// Trend (improving, stable, declining)
    pub trend: ProgressTrend,
    /// Last calculated timestamp
    pub calculated_at: DateTime<Utc>,
}

/// Progress broken down by component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressComponents {
    /// Acceptance criteria completion
    pub acceptance_criteria: ComponentProgress,
    /// Implementation progress
    pub implementation: ComponentProgress,
    /// Testing progress
    pub testing: ComponentProgress,
    /// Documentation progress
    pub documentation: ComponentProgress,
    /// Overall status weight
    pub status_weight: f32,
}

/// Progress for a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentProgress {
    /// Completion percentage
    pub percentage: u8,
    /// Total items
    pub total: u32,
    /// Completed items
    pub completed: u32,
    /// Weight in overall calculation
    pub weight: f32,
}

/// Progress trend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressTrend {
    Improving,
    Stable,
    Declining,
    New,
}

/// Phase-level progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseProgress {
    /// Phase number
    pub phase: u32,
    /// Phase name
    pub name: String,
    /// Overall completion percentage
    pub overall: u8,
    /// Spec count statistics
    pub spec_counts: SpecCounts,
    /// Average spec progress
    pub avg_spec_progress: f32,
    /// Individual spec progress
    pub specs: Vec<SpecProgress>,
    /// Blocking issues count
    pub blockers: u32,
    /// Estimated completion date
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Spec count statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecCounts {
    pub total: u32,
    pub complete: u32,
    pub in_progress: u32,
    pub planned: u32,
    pub blocked: u32,
}

/// Project-level progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectProgress {
    /// Overall completion percentage
    pub overall: u8,
    /// Phase progress breakdown
    pub phases: Vec<PhaseProgress>,
    /// Total spec counts
    pub total_specs: SpecCounts,
    /// Velocity (specs completed per week)
    pub velocity: f32,
    /// Burn rate (effort spent vs remaining)
    pub burn_rate: BurnRate,
    /// Key milestones
    pub milestones: Vec<Milestone>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

/// Burn rate metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnRate {
    /// Completed effort
    pub completed: f32,
    /// Remaining effort
    pub remaining: f32,
    /// Projected total
    pub projected_total: f32,
    /// On track status
    pub on_track: bool,
}

/// Progress milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub target_date: Option<DateTime<Utc>>,
    pub completion_percentage: u8,
    pub specs: Vec<u32>,
}

/// Progress calculation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressConfig {
    /// Weight for acceptance criteria
    pub acceptance_weight: f32,
    /// Weight for implementation
    pub implementation_weight: f32,
    /// Weight for testing
    pub testing_weight: f32,
    /// Weight for documentation
    pub documentation_weight: f32,
    /// Status-based weights
    pub status_weights: HashMap<SpecStatus, f32>,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        let mut status_weights = HashMap::new();
        status_weights.insert(SpecStatus::Complete, 1.0);
        status_weights.insert(SpecStatus::Review, 0.9);
        status_weights.insert(SpecStatus::InProgress, 0.5);
        status_weights.insert(SpecStatus::Planned, 0.0);
        status_weights.insert(SpecStatus::Draft, 0.0);
        status_weights.insert(SpecStatus::Blocked, 0.0);
        status_weights.insert(SpecStatus::Deprecated, 0.0);

        Self {
            acceptance_weight: 0.4,
            implementation_weight: 0.3,
            testing_weight: 0.2,
            documentation_weight: 0.1,
            status_weights,
        }
    }
}

/// Progress calculator
pub struct ProgressCalculator {
    config: ProgressConfig,
    history: Vec<ProgressSnapshot>,
}

/// Historical progress snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSnapshot {
    pub timestamp: DateTime<Utc>,
    pub project_progress: u8,
    pub phase_progress: HashMap<u32, u8>,
}

impl ProgressCalculator {
    pub fn new(config: ProgressConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
        }
    }

    /// Calculate progress for a single spec
    pub fn calculate_spec_progress(
        &self,
        metadata: &SpecMetadata,
        checkbox_stats: &CheckboxStats,
    ) -> SpecProgress {
        // Calculate component progress
        let acceptance = self.calc_acceptance_progress(checkbox_stats);
        let implementation = self.calc_implementation_progress(metadata);
        let testing = self.calc_testing_progress(metadata, checkbox_stats);
        let documentation = self.calc_documentation_progress(metadata);

        // Get status weight
        let status_weight = self.config.status_weights
            .get(&metadata.status)
            .copied()
            .unwrap_or(0.0);

        // Calculate overall
        let weighted_sum =
            acceptance.percentage as f32 * self.config.acceptance_weight +
            implementation.percentage as f32 * self.config.implementation_weight +
            testing.percentage as f32 * self.config.testing_weight +
            documentation.percentage as f32 * self.config.documentation_weight;

        let overall = (weighted_sum * status_weight / 100.0 * 100.0) as u8;

        // Determine trend (would need history for accurate trend)
        let trend = if metadata.status == SpecStatus::Complete {
            ProgressTrend::Stable
        } else if overall > 50 {
            ProgressTrend::Improving
        } else {
            ProgressTrend::New
        };

        SpecProgress {
            spec_id: metadata.id,
            overall,
            components: ProgressComponents {
                acceptance_criteria: acceptance,
                implementation,
                testing,
                documentation,
                status_weight,
            },
            weighted_score: weighted_sum * status_weight / 100.0,
            trend,
            calculated_at: Utc::now(),
        }
    }

    /// Calculate acceptance criteria progress
    fn calc_acceptance_progress(&self, stats: &CheckboxStats) -> ComponentProgress {
        ComponentProgress {
            percentage: stats.percentage,
            total: stats.total as u32,
            completed: stats.checked as u32,
            weight: self.config.acceptance_weight,
        }
    }

    /// Calculate implementation progress
    fn calc_implementation_progress(&self, metadata: &SpecMetadata) -> ComponentProgress {
        let has_impl = metadata.implementation_status.has_implementation;
        let has_code = metadata.implementation_status.has_code;

        let percentage = match (has_impl, has_code) {
            (true, true) => 100,
            (true, false) => 50,
            (false, true) => 30,
            (false, false) => 0,
        };

        ComponentProgress {
            percentage,
            total: 2,
            completed: (has_impl as u32) + (has_code as u32),
            weight: self.config.implementation_weight,
        }
    }

    /// Calculate testing progress
    fn calc_testing_progress(
        &self,
        metadata: &SpecMetadata,
        checkbox_stats: &CheckboxStats,
    ) -> ComponentProgress {
        let has_tests = metadata.implementation_status.has_tests;

        // Count test-related checkboxes
        let test_checkboxes = checkbox_stats.by_section
            .get("Testing Requirements")
            .or_else(|| checkbox_stats.by_section.get("Tests"))
            .copied()
            .unwrap_or((0, 0));

        let percentage = if test_checkboxes.0 > 0 {
            ((test_checkboxes.1 as f32 / test_checkboxes.0 as f32) * 100.0) as u8
        } else if has_tests {
            50
        } else {
            0
        };

        ComponentProgress {
            percentage,
            total: test_checkboxes.0 as u32,
            completed: test_checkboxes.1 as u32,
            weight: self.config.testing_weight,
        }
    }

    /// Calculate documentation progress
    fn calc_documentation_progress(&self, metadata: &SpecMetadata) -> ComponentProgress {
        // Check for documentation indicators
        let has_docs = metadata.section_count > 3 && metadata.word_count > 200;

        ComponentProgress {
            percentage: if has_docs { 100 } else { 0 },
            total: 1,
            completed: if has_docs { 1 } else { 0 },
            weight: self.config.documentation_weight,
        }
    }

    /// Calculate phase progress
    pub fn calculate_phase_progress(
        &self,
        phase: u32,
        name: &str,
        specs: Vec<(SpecMetadata, CheckboxStats)>,
    ) -> PhaseProgress {
        let mut spec_progress = Vec::new();
        let mut counts = SpecCounts {
            total: specs.len() as u32,
            complete: 0,
            in_progress: 0,
            planned: 0,
            blocked: 0,
        };

        let mut total_progress: f32 = 0.0;

        for (metadata, checkbox_stats) in &specs {
            let progress = self.calculate_spec_progress(metadata, checkbox_stats);
            total_progress += progress.overall as f32;
            spec_progress.push(progress);

            match metadata.status {
                SpecStatus::Complete => counts.complete += 1,
                SpecStatus::InProgress | SpecStatus::Review => counts.in_progress += 1,
                SpecStatus::Blocked => counts.blocked += 1,
                _ => counts.planned += 1,
            }
        }

        let avg_progress = if !specs.is_empty() {
            total_progress / specs.len() as f32
        } else {
            0.0
        };

        // Calculate overall based on completion status
        let overall = if counts.total > 0 {
            ((counts.complete as f32 / counts.total as f32) * 100.0) as u8
        } else {
            0
        };

        PhaseProgress {
            phase,
            name: name.to_string(),
            overall,
            spec_counts: counts,
            avg_spec_progress: avg_progress,
            specs: spec_progress,
            blockers: counts.blocked,
            estimated_completion: None, // Would need velocity calculation
        }
    }

    /// Calculate project-wide progress
    pub fn calculate_project_progress(
        &mut self,
        phases: Vec<PhaseProgress>,
    ) -> ProjectProgress {
        let mut total_specs = SpecCounts {
            total: 0,
            complete: 0,
            in_progress: 0,
            planned: 0,
            blocked: 0,
        };

        let mut weighted_progress: f32 = 0.0;

        for phase in &phases {
            total_specs.total += phase.spec_counts.total;
            total_specs.complete += phase.spec_counts.complete;
            total_specs.in_progress += phase.spec_counts.in_progress;
            total_specs.planned += phase.spec_counts.planned;
            total_specs.blocked += phase.spec_counts.blocked;

            weighted_progress += phase.overall as f32 * phase.spec_counts.total as f32;
        }

        let overall = if total_specs.total > 0 {
            (weighted_progress / total_specs.total as f32) as u8
        } else {
            0
        };

        // Calculate velocity from history
        let velocity = self.calculate_velocity();

        // Calculate burn rate
        let burn_rate = BurnRate {
            completed: total_specs.complete as f32,
            remaining: (total_specs.total - total_specs.complete) as f32,
            projected_total: total_specs.total as f32,
            on_track: velocity > 0.5,
        };

        // Record snapshot
        let mut phase_map = HashMap::new();
        for phase in &phases {
            phase_map.insert(phase.phase, phase.overall);
        }

        self.history.push(ProgressSnapshot {
            timestamp: Utc::now(),
            project_progress: overall,
            phase_progress: phase_map,
        });

        ProjectProgress {
            overall,
            phases,
            total_specs,
            velocity,
            burn_rate,
            milestones: Vec::new(), // Would be configured separately
            updated_at: Utc::now(),
        }
    }

    /// Calculate velocity (specs per week)
    fn calculate_velocity(&self) -> f32 {
        if self.history.len() < 2 {
            return 0.0;
        }

        let recent: Vec<_> = self.history.iter().rev().take(7).collect();

        if recent.len() < 2 {
            return 0.0;
        }

        let first = recent.last().unwrap();
        let last = recent.first().unwrap();

        let progress_diff = last.project_progress as f32 - first.project_progress as f32;
        let time_diff = last.timestamp.signed_duration_since(first.timestamp);
        let weeks = time_diff.num_days() as f32 / 7.0;

        if weeks > 0.0 {
            progress_diff / weeks
        } else {
            0.0
        }
    }

    /// Generate progress report
    pub fn generate_report(&self, project: &ProjectProgress) -> String {
        let mut report = String::new();

        report.push_str("# Progress Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", project.updated_at.format("%Y-%m-%d %H:%M UTC")));

        // Overall progress
        report.push_str("## Overall Progress\n\n");
        report.push_str(&self.render_progress_bar(project.overall));
        report.push_str(&format!(" **{}%**\n\n", project.overall));

        // Spec counts
        report.push_str("| Status | Count |\n|--------|-------|\n");
        report.push_str(&format!("| Total | {} |\n", project.total_specs.total));
        report.push_str(&format!("| Complete | {} |\n", project.total_specs.complete));
        report.push_str(&format!("| In Progress | {} |\n", project.total_specs.in_progress));
        report.push_str(&format!("| Planned | {} |\n", project.total_specs.planned));
        report.push_str(&format!("| Blocked | {} |\n\n", project.total_specs.blocked));

        // Phase breakdown
        report.push_str("## Phase Breakdown\n\n");
        for phase in &project.phases {
            report.push_str(&format!("### Phase {}: {}\n\n", phase.phase, phase.name));
            report.push_str(&self.render_progress_bar(phase.overall));
            report.push_str(&format!(" {}%\n", phase.overall));
            report.push_str(&format!("- Specs: {}/{} complete\n",
                phase.spec_counts.complete, phase.spec_counts.total
            ));
            if phase.blockers > 0 {
                report.push_str(&format!("- Blockers: {}\n", phase.blockers));
            }
            report.push_str("\n");
        }

        // Metrics
        report.push_str("## Metrics\n\n");
        report.push_str(&format!("- Velocity: {:.1} progress/week\n", project.velocity));
        report.push_str(&format!("- Burn Rate: {:.1}/{:.1}\n",
            project.burn_rate.completed, project.burn_rate.remaining
        ));
        report.push_str(&format!("- On Track: {}\n",
            if project.burn_rate.on_track { "Yes" } else { "No" }
        ));

        report
    }

    /// Render ASCII progress bar
    fn render_progress_bar(&self, percentage: u8) -> String {
        let filled = percentage as usize / 5;
        let empty = 20 - filled;
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }
}

impl Default for ProgressCalculator {
    fn default() -> Self {
        Self::new(ProgressConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_calculation() {
        let calc = ProgressCalculator::default();

        let mut metadata = SpecMetadata::default();
        metadata.id = 1;
        metadata.status = SpecStatus::InProgress;
        metadata.implementation_status.has_implementation = true;
        metadata.implementation_status.has_code = true;

        let stats = CheckboxStats {
            total: 10,
            checked: 7,
            percentage: 70,
            by_section: HashMap::new(),
        };

        let progress = calc.calculate_spec_progress(&metadata, &stats);
        assert!(progress.overall > 0);
    }

    #[test]
    fn test_spec_counts_aggregation() {
        let counts = SpecCounts {
            total: 20,
            complete: 10,
            in_progress: 5,
            planned: 3,
            blocked: 2,
        };

        assert_eq!(counts.total, counts.complete + counts.in_progress + counts.planned + counts.blocked);
    }

    #[test]
    fn test_progress_bar_rendering() {
        let calc = ProgressCalculator::default();

        let bar = calc.render_progress_bar(50);
        assert!(bar.contains("██████████"));
        assert!(bar.contains("░░░░░░░░░░"));
    }
}

// Add default implementation for SpecMetadata for testing
impl Default for SpecMetadata {
    fn default() -> Self {
        use crate::spec::metadata::*;

        Self {
            id: 0,
            title: String::new(),
            phase: 0,
            phase_name: String::new(),
            status: SpecStatus::Planned,
            path: std::path::PathBuf::new(),
            dependencies: Vec::new(),
            dependents: Vec::new(),
            dependency_depth: 0,
            estimated_context: ContextEstimate {
                display: "~10%".to_string(),
                percentage: 10,
                confidence: Confidence::Medium,
            },
            acceptance_criteria: AcceptanceCriteriaStats {
                total: 0,
                completed: 0,
                percentage: 0,
                by_section: HashMap::new(),
            },
            implementation_status: ImplementationStatus {
                has_implementation: false,
                has_code: false,
                has_tests: false,
                languages: Vec::new(),
            },
            created_at: None,
            modified_at: None,
            age_days: None,
            staleness_score: 0,
            complexity: 1,
            word_count: 0,
            code_block_count: 0,
            section_count: 0,
            custom: HashMap::new(),
            schema_version: 1,
            extracted_at: Utc::now(),
        }
    }
}
```

## Testing Requirements

- [ ] Unit tests for spec progress calculation
- [ ] Tests for component weight calculations
- [ ] Tests for phase aggregation
- [ ] Tests for project rollup
- [ ] Tests for velocity calculation
- [ ] Tests for trend detection
- [ ] Integration tests with real specs
- [ ] Tests for edge cases (empty phases, etc.)

## Related Specs

- **121-spec-metadata.md**: Metadata source for calculations
- **123-checkbox-tracking.md**: Checkbox state input
- **119-readme-autogen.md**: Progress display in READMEs
