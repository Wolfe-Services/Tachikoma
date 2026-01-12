// src/spec/impl_plan.rs

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A complete implementation plan for a spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPlan {
    /// Spec ID this plan is for
    pub spec_id: u32,
    /// Plan version
    pub version: String,
    /// Plan status
    pub status: PlanStatus,
    /// Overview/summary
    pub summary: String,
    /// Prerequisites before starting
    pub prerequisites: Vec<Prerequisite>,
    /// Ordered phases of implementation
    pub phases: Vec<ImplementationPhase>,
    /// Estimated total effort
    pub total_effort: EffortEstimate,
    /// Risk factors
    pub risks: Vec<Risk>,
    /// Verification/validation steps
    pub verification: VerificationPlan,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

/// Plan status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStatus {
    Draft,
    Ready,
    InProgress,
    Paused,
    Complete,
    Abandoned,
}

/// A prerequisite for implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prerequisite {
    /// Prerequisite description
    pub description: String,
    /// Type of prerequisite
    pub prereq_type: PrerequisiteType,
    /// Whether satisfied
    pub satisfied: bool,
    /// Reference (spec ID, URL, etc.)
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrerequisiteType {
    /// Another spec must be complete
    Spec,
    /// External dependency must exist
    Dependency,
    /// Knowledge/documentation needed
    Knowledge,
    /// Infrastructure/tooling required
    Infrastructure,
    /// Review/approval needed
    Approval,
}

/// A phase of implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPhase {
    /// Phase ID (within plan)
    pub id: String,
    /// Phase name
    pub name: String,
    /// Phase description
    pub description: String,
    /// Tasks in this phase
    pub tasks: Vec<Task>,
    /// Dependencies on other phases
    pub depends_on: Vec<String>,
    /// Estimated effort for this phase
    pub effort: EffortEstimate,
    /// Deliverables for this phase
    pub deliverables: Vec<String>,
    /// Phase status
    pub status: TaskStatus,
}

/// An individual task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task ID
    pub id: String,
    /// Task title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Task type
    pub task_type: TaskType,
    /// File paths involved
    pub files: Vec<String>,
    /// Subtasks
    pub subtasks: Vec<Subtask>,
    /// Dependencies on other tasks
    pub depends_on: Vec<String>,
    /// Effort estimate
    pub effort: EffortEstimate,
    /// Task status
    pub status: TaskStatus,
    /// Completion notes
    pub notes: Option<String>,
    /// AI context hints
    pub context_hints: Vec<String>,
}

/// Task types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// Write new code
    Implement,
    /// Modify existing code
    Refactor,
    /// Write tests
    Test,
    /// Write documentation
    Document,
    /// Review code
    Review,
    /// Research/investigation
    Research,
    /// Configuration/setup
    Configure,
    /// Integration work
    Integrate,
}

/// A subtask within a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    /// Subtask description
    pub description: String,
    /// Whether complete
    pub complete: bool,
    /// Optional file reference
    pub file: Option<String>,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Blocked,
    Complete,
    Skipped,
}

/// Effort estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortEstimate {
    /// Effort unit (hours, days, points)
    pub unit: EffortUnit,
    /// Minimum estimate
    pub min: f32,
    /// Expected estimate
    pub expected: f32,
    /// Maximum estimate
    pub max: f32,
    /// Confidence level
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffortUnit {
    Hours,
    Days,
    Points,
    Contexts, // AI context windows needed
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

/// Risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    /// Risk description
    pub description: String,
    /// Impact level
    pub impact: RiskLevel,
    /// Likelihood
    pub likelihood: RiskLevel,
    /// Mitigation strategy
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Verification plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPlan {
    /// Unit test requirements
    pub unit_tests: Vec<TestRequirement>,
    /// Integration test requirements
    pub integration_tests: Vec<TestRequirement>,
    /// Manual verification steps
    pub manual_checks: Vec<String>,
    /// Performance requirements
    pub performance: Option<PerformanceRequirement>,
    /// Documentation requirements
    pub documentation: Vec<String>,
}

/// Test requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequirement {
    /// Test description
    pub description: String,
    /// Target coverage
    pub coverage_target: Option<u8>,
    /// Test file path
    pub file: Option<String>,
    /// Whether implemented
    pub implemented: bool,
}

/// Performance requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirement {
    /// Metric name
    pub metric: String,
    /// Target value
    pub target: String,
    /// Current value (if measured)
    pub current: Option<String>,
}

/// Implementation plan builder
pub struct PlanBuilder {
    spec_id: u32,
    summary: String,
    prerequisites: Vec<Prerequisite>,
    phases: Vec<ImplementationPhase>,
    risks: Vec<Risk>,
}

impl PlanBuilder {
    pub fn new(spec_id: u32) -> Self {
        Self {
            spec_id,
            summary: String::new(),
            prerequisites: Vec::new(),
            phases: Vec::new(),
            risks: Vec::new(),
        }
    }

    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    pub fn prerequisite(mut self, prereq: Prerequisite) -> Self {
        self.prerequisites.push(prereq);
        self
    }

    pub fn phase(mut self, phase: ImplementationPhase) -> Self {
        self.phases.push(phase);
        self
    }

    pub fn risk(mut self, risk: Risk) -> Self {
        self.risks.push(risk);
        self
    }

    pub fn build(self) -> ImplementationPlan {
        let total_effort = self.calculate_total_effort();

        ImplementationPlan {
            spec_id: self.spec_id,
            version: "1.0.0".to_string(),
            status: PlanStatus::Draft,
            summary: self.summary,
            prerequisites: self.prerequisites,
            phases: self.phases,
            total_effort,
            risks: self.risks,
            verification: VerificationPlan::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn calculate_total_effort(&self) -> EffortEstimate {
        let mut min = 0.0;
        let mut expected = 0.0;
        let mut max = 0.0;

        for phase in &self.phases {
            min += phase.effort.min;
            expected += phase.effort.expected;
            max += phase.effort.max;
        }

        EffortEstimate {
            unit: EffortUnit::Hours,
            min,
            expected,
            max,
            confidence: Confidence::Medium,
        }
    }
}

/// Parse implementation plan from markdown
pub struct PlanParser;

impl PlanParser {
    /// Extract implementation plan from spec content
    pub fn parse(content: &str, spec_id: u32) -> Option<ImplementationPlan> {
        let mut builder = PlanBuilder::new(spec_id);
        let mut current_phase: Option<ImplementationPhase> = None;
        let mut current_tasks: Vec<Task> = Vec::new();
        let mut in_impl_section = false;

        for line in content.lines() {
            // Detect Implementation Details section
            if line.starts_with("## Implementation") {
                in_impl_section = true;
                continue;
            }

            // Exit section on next ## heading
            if line.starts_with("## ") && in_impl_section && !line.contains("Implementation") {
                in_impl_section = false;
            }

            if !in_impl_section {
                continue;
            }

            // Parse phase headers (### Phase N: Name)
            if line.starts_with("### Phase") || line.starts_with("### Step") {
                // Save previous phase
                if let Some(mut phase) = current_phase.take() {
                    phase.tasks = std::mem::take(&mut current_tasks);
                    builder = builder.phase(phase);
                }

                // Start new phase
                let name = line.trim_start_matches('#').trim();
                current_phase = Some(ImplementationPhase {
                    id: format!("phase-{}", builder.phases.len() + 1),
                    name: name.to_string(),
                    description: String::new(),
                    tasks: Vec::new(),
                    depends_on: Vec::new(),
                    effort: EffortEstimate::default(),
                    deliverables: Vec::new(),
                    status: TaskStatus::Pending,
                });
            }

            // Parse task items (numbered or bulleted lists)
            if (line.starts_with("1.") || line.starts_with("- ") || line.starts_with("* "))
                && current_phase.is_some()
            {
                let task_text = line
                    .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == '-' || c == '*' || c == ' ')
                    .trim();

                if !task_text.is_empty() {
                    current_tasks.push(Task {
                        id: format!("task-{}", current_tasks.len() + 1),
                        title: task_text.to_string(),
                        description: String::new(),
                        task_type: Self::infer_task_type(task_text),
                        files: Vec::new(),
                        subtasks: Vec::new(),
                        depends_on: Vec::new(),
                        effort: EffortEstimate::default(),
                        status: TaskStatus::Pending,
                        notes: None,
                        context_hints: Vec::new(),
                    });
                }
            }
        }

        // Save final phase
        if let Some(mut phase) = current_phase.take() {
            phase.tasks = current_tasks;
            builder = builder.phase(phase);
        }

        // Only return plan if we found phases
        if builder.phases.is_empty() {
            None
        } else {
            Some(builder.build())
        }
    }

    /// Infer task type from description
    fn infer_task_type(description: &str) -> TaskType {
        let lower = description.to_lowercase();

        if lower.contains("test") {
            TaskType::Test
        } else if lower.contains("document") || lower.contains("readme") {
            TaskType::Document
        } else if lower.contains("refactor") || lower.contains("move") || lower.contains("rename") {
            TaskType::Refactor
        } else if lower.contains("research") || lower.contains("investigate") {
            TaskType::Research
        } else if lower.contains("configure") || lower.contains("setup") || lower.contains("install") {
            TaskType::Configure
        } else if lower.contains("integrate") || lower.contains("connect") {
            TaskType::Integrate
        } else if lower.contains("review") {
            TaskType::Review
        } else {
            TaskType::Implement
        }
    }
}

/// Render implementation plan to markdown
pub struct PlanRenderer;

impl PlanRenderer {
    pub fn render(plan: &ImplementationPlan) -> String {
        let mut output = String::new();

        output.push_str("## Implementation Plan\n\n");
        output.push_str(&format!("**Status**: {:?}\n", plan.status));
        output.push_str(&format!("**Estimated Effort**: {:.1}-{:.1} {:?}\n\n",
            plan.total_effort.min,
            plan.total_effort.max,
            plan.total_effort.unit
        ));

        output.push_str(&plan.summary);
        output.push_str("\n\n");

        // Prerequisites
        if !plan.prerequisites.is_empty() {
            output.push_str("### Prerequisites\n\n");
            for prereq in &plan.prerequisites {
                let check = if prereq.satisfied { "[x]" } else { "[ ]" };
                output.push_str(&format!("- {} {:?}: {}\n",
                    check, prereq.prereq_type, prereq.description
                ));
            }
            output.push_str("\n");
        }

        // Phases
        for (i, phase) in plan.phases.iter().enumerate() {
            output.push_str(&format!("### Phase {}: {}\n\n", i + 1, phase.name));

            if !phase.description.is_empty() {
                output.push_str(&phase.description);
                output.push_str("\n\n");
            }

            for task in &phase.tasks {
                let status = match task.status {
                    TaskStatus::Complete => "[x]",
                    TaskStatus::InProgress => "[~]",
                    _ => "[ ]",
                };
                output.push_str(&format!("- {} {}\n", status, task.title));

                for subtask in &task.subtasks {
                    let sub_status = if subtask.complete { "[x]" } else { "[ ]" };
                    output.push_str(&format!("  - {} {}\n", sub_status, subtask.description));
                }
            }

            if !phase.deliverables.is_empty() {
                output.push_str("\n**Deliverables**:\n");
                for d in &phase.deliverables {
                    output.push_str(&format!("- {}\n", d));
                }
            }

            output.push_str("\n");
        }

        // Risks
        if !plan.risks.is_empty() {
            output.push_str("### Risks\n\n");
            for risk in &plan.risks {
                output.push_str(&format!("- **{:?}/{:?}**: {}\n",
                    risk.impact, risk.likelihood, risk.description
                ));
                if let Some(mitigation) = &risk.mitigation {
                    output.push_str(&format!("  - Mitigation: {}\n", mitigation));
                }
            }
        }

        output
    }
}

impl Default for EffortEstimate {
    fn default() -> Self {
        Self {
            unit: EffortUnit::Hours,
            min: 1.0,
            expected: 2.0,
            max: 4.0,
            confidence: Confidence::Low,
        }
    }
}

impl Default for VerificationPlan {
    fn default() -> Self {
        Self {
            unit_tests: Vec::new(),
            integration_tests: Vec::new(),
            manual_checks: Vec::new(),
            performance: None,
            documentation: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_builder() {
        let plan = PlanBuilder::new(116)
            .summary("Test implementation")
            .phase(ImplementationPhase {
                id: "phase-1".to_string(),
                name: "Setup".to_string(),
                description: String::new(),
                tasks: vec![
                    Task {
                        id: "task-1".to_string(),
                        title: "Create module".to_string(),
                        description: String::new(),
                        task_type: TaskType::Implement,
                        files: vec!["src/new.rs".to_string()],
                        subtasks: Vec::new(),
                        depends_on: Vec::new(),
                        effort: EffortEstimate::default(),
                        status: TaskStatus::Pending,
                        notes: None,
                        context_hints: Vec::new(),
                    }
                ],
                depends_on: Vec::new(),
                effort: EffortEstimate::default(),
                deliverables: Vec::new(),
                status: TaskStatus::Pending,
            })
            .build();

        assert_eq!(plan.spec_id, 116);
        assert_eq!(plan.phases.len(), 1);
    }

    #[test]
    fn test_task_type_inference() {
        assert_eq!(PlanParser::infer_task_type("Write unit tests"), TaskType::Test);
        assert_eq!(PlanParser::infer_task_type("Refactor the parser"), TaskType::Refactor);
        assert_eq!(PlanParser::infer_task_type("Implement the feature"), TaskType::Implement);
    }

    #[test]
    fn test_plan_parsing_from_markdown() {
        let content = r#"# Test Spec

## Implementation Details

### Phase 1: Setup
1. Create basic structure
2. Add dependencies
3. Write initial tests

### Phase 2: Core Logic
- Implement parser
- Add validation
- Write documentation

## Other Section
Not parsed
"#;

        let plan = PlanParser::parse(content, 123).unwrap();
        assert_eq!(plan.spec_id, 123);
        assert_eq!(plan.phases.len(), 2);
        assert_eq!(plan.phases[0].name, "Phase 1: Setup");
        assert_eq!(plan.phases[0].tasks.len(), 3);
        assert_eq!(plan.phases[1].tasks.len(), 3);
    }

    #[test]
    fn test_plan_rendering() {
        let plan = PlanBuilder::new(124)
            .summary("Test plan summary")
            .phase(ImplementationPhase {
                id: "phase-1".to_string(),
                name: "Testing Phase".to_string(),
                description: String::new(),
                tasks: vec![
                    Task {
                        id: "task-1".to_string(),
                        title: "Write tests".to_string(),
                        description: String::new(),
                        task_type: TaskType::Test,
                        files: Vec::new(),
                        subtasks: vec![
                            Subtask {
                                description: "Unit tests".to_string(),
                                complete: true,
                                file: None,
                            },
                            Subtask {
                                description: "Integration tests".to_string(),
                                complete: false,
                                file: None,
                            }
                        ],
                        depends_on: Vec::new(),
                        effort: EffortEstimate::default(),
                        status: TaskStatus::Complete,
                        notes: None,
                        context_hints: Vec::new(),
                    }
                ],
                depends_on: Vec::new(),
                effort: EffortEstimate::default(),
                deliverables: vec!["Test suite".to_string()],
                status: TaskStatus::InProgress,
            })
            .build();

        let rendered = PlanRenderer::render(&plan);
        
        assert!(rendered.contains("## Implementation Plan"));
        assert!(rendered.contains("Test plan summary"));
        assert!(rendered.contains("### Phase 1: Testing Phase"));
        assert!(rendered.contains("- [x] Write tests"));
        assert!(rendered.contains("  - [x] Unit tests"));
        assert!(rendered.contains("  - [ ] Integration tests"));
        assert!(rendered.contains("**Deliverables**:"));
        assert!(rendered.contains("- Test suite"));
    }

    #[test]
    fn test_effort_calculation() {
        let phase1 = ImplementationPhase {
            id: "phase-1".to_string(),
            name: "Phase 1".to_string(),
            description: String::new(),
            tasks: Vec::new(),
            depends_on: Vec::new(),
            effort: EffortEstimate {
                unit: EffortUnit::Hours,
                min: 2.0,
                expected: 4.0,
                max: 8.0,
                confidence: Confidence::High,
            },
            deliverables: Vec::new(),
            status: TaskStatus::Pending,
        };

        let phase2 = ImplementationPhase {
            id: "phase-2".to_string(),
            name: "Phase 2".to_string(),
            description: String::new(),
            tasks: Vec::new(),
            depends_on: Vec::new(),
            effort: EffortEstimate {
                unit: EffortUnit::Hours,
                min: 1.0,
                expected: 2.0,
                max: 4.0,
                confidence: Confidence::Medium,
            },
            deliverables: Vec::new(),
            status: TaskStatus::Pending,
        };

        let plan = PlanBuilder::new(125)
            .phase(phase1)
            .phase(phase2)
            .build();

        assert_eq!(plan.total_effort.min, 3.0);
        assert_eq!(plan.total_effort.expected, 6.0);
        assert_eq!(plan.total_effort.max, 12.0);
    }

    #[test]
    fn test_prerequisite_types() {
        let prereq = Prerequisite {
            description: "Complete spec 120".to_string(),
            prereq_type: PrerequisiteType::Spec,
            satisfied: false,
            reference: Some("120".to_string()),
        };

        let plan = PlanBuilder::new(126)
            .prerequisite(prereq.clone())
            .build();

        assert_eq!(plan.prerequisites.len(), 1);
        assert_eq!(plan.prerequisites[0].prereq_type, PrerequisiteType::Spec);
        assert!(!plan.prerequisites[0].satisfied);
    }

    #[test]
    fn test_risk_assessment() {
        let risk = Risk {
            description: "Complex parsing logic".to_string(),
            impact: RiskLevel::High,
            likelihood: RiskLevel::Medium,
            mitigation: Some("Break into smaller tasks".to_string()),
        };

        let plan = PlanBuilder::new(127)
            .risk(risk.clone())
            .build();

        assert_eq!(plan.risks.len(), 1);
        assert_eq!(plan.risks[0].impact, RiskLevel::High);
        assert!(plan.risks[0].mitigation.is_some());
    }
}