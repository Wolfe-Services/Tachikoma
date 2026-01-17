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

/// Calculate convergence with participant role weighting
pub fn calculate_convergence_weighted(
    rounds: &[DeliberationRound],
    threshold: f32,
    participant_weights: &std::collections::HashMap<uuid::Uuid, f32>,
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
    
    let mut weighted_agree = 0.0f32;
    let mut weighted_disagree = 0.0f32;
    let mut weighted_partial = 0.0f32;
    let mut total_weight = 0.0f32;
    
    let mut agree_count = 0u32;
    let mut disagree_count = 0u32; 
    let mut partial_count = 0u32;
    let mut blocking_concerns = Vec::new();
    
    for contrib in &round.contributions {
        let weight = participant_weights.get(&contrib.participant_id).copied().unwrap_or(1.0);
        total_weight += weight;
        
        match contrib.opinion.as_ref().map(|o| o.stance) {
            Some(Stance::StronglyAgree | Stance::Agree) => {
                weighted_agree += weight;
                agree_count += 1;
            }
            Some(Stance::StronglyDisagree | Stance::Disagree) => {
                weighted_disagree += weight;
                disagree_count += 1;
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
            Some(Stance::Partial) => {
                weighted_partial += weight;
                partial_count += 1;
            }
            None => {}
        }
    }
    
    let score = if total_weight > 0.0 {
        (weighted_agree + weighted_partial * 0.5) / total_weight
    } else {
        0.0
    };
    
    ConvergenceScore {
        score,
        agreement_count: agree_count,
        disagreement_count: disagree_count,
        partial_count,
        is_converged: score >= threshold && weighted_disagree == 0.0,
        blocking_concerns,
    }
}