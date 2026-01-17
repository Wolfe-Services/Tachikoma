use crate::{Participant, ForgeSession};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliberationRound {
    pub id: Uuid,
    pub round_number: u32,
    pub round_type: DeliberationRoundType,
    pub contributions: Vec<Contribution>,
    pub divergences: Vec<Divergence>,
    pub status: RoundStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DeliberationRoundType {
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
    
    pub fn start_round(&mut self, round_type: DeliberationRoundType) -> &mut DeliberationRound {
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

/// Specialized implementation to handle dissent tracking across rounds
pub struct DivergenceDetector {
    dissent_log: crate::DissentLog,
}

impl DivergenceDetector {
    pub fn new(session_id: tachikoma_common_core::ForgeSessionId) -> Self {
        Self {
            dissent_log: crate::DissentLog::new(session_id),
        }
    }
    
    pub fn analyze_round(&mut self, round: &DeliberationRound) -> Vec<String> {
        let mut new_conflicts = Vec::new();
        
        for divergence in &round.divergences {
            if !divergence.resolved {
                let conflict_id = format!("round_{}_topic_{}", round.round_number, divergence.topic.replace(' ', "_"));
                
                let dissent = crate::Dissent {
                    id: conflict_id.clone(),
                    description: format!(
                        "Unresolved disagreement in round {} on '{}': {} participants with conflicting stances",
                        round.round_number,
                        divergence.topic,
                        divergence.positions.len()
                    ),
                    timestamp: chrono::Utc::now(),
                };
                
                // Only add if it's a new conflict
                if !self.dissent_log.dissents.iter().any(|d| d.id == conflict_id) {
                    self.dissent_log.dissents.push(dissent);
                    new_conflicts.push(conflict_id);
                }
            }
        }
        
        new_conflicts
    }
    
    pub fn get_dissent_log(&self) -> &crate::DissentLog {
        &self.dissent_log
    }
}