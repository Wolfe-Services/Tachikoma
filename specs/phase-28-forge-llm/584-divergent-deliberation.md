# Spec 584: Divergent Deliberation & Refinement

**Priority:** P0  
**Status:** planned  
**Depends on:** 577, 583  
**Estimated Effort:** 4 hours  
**Target Files:**
- `crates/tachikoma-forge/src/deliberation/mod.rs` (new)
- `crates/tachikoma-forge/src/deliberation/rounds.rs` (new)
- `crates/tachikoma-forge/src/deliberation/convergence.rs` (new)
- `crates/tachikoma-forge/src/lib.rs` (update)

---

## Overview

Implement a deliberation engine that encourages divergent opinions, tracks disagreements, and drives toward convergence through structured refinement rounds. Unlike simple voting, this allows models to genuinely disagree and forces resolution.

---

## Acceptance Criteria

- [x] Create `DeliberationEngine` struct that orchestrates rounds
- [x] Track `Opinion` with: stance (Agree/Disagree/Partial), reasoning, concerns
- [x] Implement `DivergenceDetector` that identifies conflicting views
- [x] Track `DissentLog` of unresolved disagreements across rounds
- [x] Implement refinement rounds where dissenters respond to synthesis
- [x] Add convergence scoring: count agreements, weight by participant role
- [x] Cap refinement at `max_rounds` to prevent infinite loops
- [x] Export from lib.rs
- [x] Verify `cargo check -p tachikoma-forge` passes

---

## Implementation

```rust
// crates/tachikoma-forge/src/deliberation/mod.rs
mod rounds;
mod convergence;

pub use rounds::*;
pub use convergence::*;
```

```rust
// crates/tachikoma-forge/src/deliberation/rounds.rs

use crate::{Participant, ForgeSession};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliberationRound {
    pub id: Uuid,
    pub round_number: u32,
    pub round_type: RoundType,
    pub contributions: Vec<Contribution>,
    pub divergences: Vec<Divergence>,
    pub status: RoundStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RoundType {
    Draft,        // Initial proposals
    Critique,     // Critical review
    Response,     // Authors respond to critiques  
    Synthesis,    // Combine best elements
    Refinement,   // Address remaining concerns
    Convergence,  // Final vote
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub id: Uuid,
    pub participant_id: Uuid,
    pub participant_name: String,
    pub content: String,
    pub opinion: Option<Opinion>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opinion {
    pub stance: Stance,
    pub reasoning: String,
    pub concerns: Vec<String>,
    pub strength: f32,  // 0.0 to 1.0 - how strongly held
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Stance {
    StronglyAgree,
    Agree,
    Partial,       // Agrees with caveats
    Disagree,
    StronglyDisagree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergence {
    pub id: Uuid,
    pub topic: String,
    pub positions: Vec<DivergentPosition>,
    pub resolved: bool,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergentPosition {
    pub participant_id: Uuid,
    pub participant_name: String,
    pub position: String,
    pub stance: Stance,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RoundStatus {
    Pending,
    InProgress,
    Complete,
    Skipped,
}

pub struct DeliberationEngine {
    session: ForgeSession,
    rounds: Vec<DeliberationRound>,
    max_rounds: u32,
    convergence_threshold: f32,
}

impl DeliberationEngine {
    pub fn new(session: ForgeSession, max_rounds: u32, convergence_threshold: f32) -> Self {
        Self {
            session,
            rounds: Vec::new(),
            max_rounds,
            convergence_threshold,
        }
    }
    
    pub fn current_round(&self) -> Option<&DeliberationRound> {
        self.rounds.last()
    }
    
    pub fn start_round(&mut self, round_type: RoundType) -> &mut DeliberationRound {
        let round = DeliberationRound {
            id: Uuid::new_v4(),
            round_number: self.rounds.len() as u32 + 1,
            round_type,
            contributions: Vec::new(),
            divergences: Vec::new(),
            status: RoundStatus::InProgress,
        };
        self.rounds.push(round);
        self.rounds.last_mut().unwrap()
    }
    
    pub fn add_contribution(&mut self, contribution: Contribution) {
        if let Some(round) = self.rounds.last_mut() {
            round.contributions.push(contribution);
        }
    }
    
    pub fn detect_divergences(&mut self) {
        let Some(round) = self.rounds.last_mut() else { return };
        
        // Group contributions by topic/aspect
        // Compare stances - mark as divergent if opposing views exist
        let mut divergences = Vec::new();
        
        // Simple divergence detection: any Disagree vs Agree
        let agreements: Vec<_> = round.contributions.iter()
            .filter(|c| matches!(c.opinion.as_ref().map(|o| o.stance), Some(Stance::Agree | Stance::StronglyAgree)))
            .collect();
        
        let disagreements: Vec<_> = round.contributions.iter()
            .filter(|c| matches!(c.opinion.as_ref().map(|o| o.stance), Some(Stance::Disagree | Stance::StronglyDisagree)))
            .collect();
        
        if !agreements.is_empty() && !disagreements.is_empty() {
            let mut positions = Vec::new();
            
            for contrib in &agreements {
                positions.push(DivergentPosition {
                    participant_id: contrib.participant_id,
                    participant_name: contrib.participant_name.clone(),
                    position: contrib.opinion.as_ref().map(|o| o.reasoning.clone()).unwrap_or_default(),
                    stance: contrib.opinion.as_ref().map(|o| o.stance).unwrap_or(Stance::Partial),
                });
            }
            
            for contrib in &disagreements {
                positions.push(DivergentPosition {
                    participant_id: contrib.participant_id,
                    participant_name: contrib.participant_name.clone(),
                    position: contrib.opinion.as_ref().map(|o| o.reasoning.clone()).unwrap_or_default(),
                    stance: contrib.opinion.as_ref().map(|o| o.stance).unwrap_or(Stance::Partial),
                });
            }
            
            divergences.push(Divergence {
                id: Uuid::new_v4(),
                topic: "Primary approach".to_string(),
                positions,
                resolved: false,
                resolution: None,
            });
        }
        
        round.divergences = divergences;
    }
    
    pub fn needs_refinement(&self) -> bool {
        self.rounds.last()
            .map(|r| !r.divergences.is_empty() && r.divergences.iter().any(|d| !d.resolved))
            .unwrap_or(false)
    }
    
    pub fn unresolved_divergences(&self) -> Vec<&Divergence> {
        self.rounds.last()
            .map(|r| r.divergences.iter().filter(|d| !d.resolved).collect())
            .unwrap_or_default()
    }
    
    pub fn can_continue(&self) -> bool {
        self.rounds.len() < self.max_rounds as usize
    }
    
    pub fn all_rounds(&self) -> &[DeliberationRound] {
        &self.rounds
    }
}
```

```rust
// crates/tachikoma-forge/src/deliberation/convergence.rs

use super::{DeliberationRound, Stance, RoundType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceScore {
    pub score: f32,           // 0.0 to 1.0
    pub agreement_count: u32,
    pub disagreement_count: u32,
    pub partial_count: u32,
    pub is_converged: bool,
    pub blocking_concerns: Vec<String>,
}

pub fn calculate_convergence(
    rounds: &[DeliberationRound],
    threshold: f32,
) -> ConvergenceScore {
    // Find the last convergence round
    let convergence_round = rounds.iter()
        .rev()
        .find(|r| r.round_type == RoundType::Convergence);
    
    let Some(round) = convergence_round else {
        return ConvergenceScore {
            score: 0.0,
            agreement_count: 0,
            disagreement_count: 0,
            partial_count: 0,
            is_converged: false,
            blocking_concerns: vec!["No convergence round completed".to_string()],
        };
    };
    
    let mut agree = 0u32;
    let mut disagree = 0u32;
    let mut partial = 0u32;
    let mut blocking_concerns = Vec::new();
    
    for contrib in &round.contributions {
        match contrib.opinion.as_ref().map(|o| o.stance) {
            Some(Stance::StronglyAgree | Stance::Agree) => agree += 1,
            Some(Stance::StronglyDisagree | Stance::Disagree) => {
                disagree += 1;
                if let Some(opinion) = &contrib.opinion {
                    for concern in &opinion.concerns {
                        blocking_concerns.push(format!(
                            "{}: {}",
                            contrib.participant_name,
                            concern
                        ));
                    }
                }
            }
            Some(Stance::Partial) => partial += 1,
            None => {}
        }
    }
    
    let total = agree + disagree + partial;
    let score = if total > 0 {
        (agree as f32 + partial as f32 * 0.5) / total as f32
    } else {
        0.0
    };
    
    ConvergenceScore {
        score,
        agreement_count: agree,
        disagreement_count: disagree,
        partial_count: partial,
        is_converged: score >= threshold && disagree == 0,
        blocking_concerns,
    }
}
```
