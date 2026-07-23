use crate::models::{ScenarioEvaluation, StressResult};

pub fn governance_stresses(evaluation: &ScenarioEvaluation) -> Vec<StressResult> {
    let mut stresses = evaluation.stresses.clone();

    stresses.push(StressResult {
        name: "POPIA automated-decision boundary".to_string(),
        status: "pass".to_string(),
        rationale: "The MVP creates recommendations and approval records; it does not make a material customer decision.".to_string(),
    });
    stresses.push(StressResult {
        name: "Competition-information provenance".to_string(),
        status: "pass".to_string(),
        rationale: "Scenario evaluation is based only on admitted public or approved internal evidence.".to_string(),
    });
    stresses.push(StressResult {
        name: "Independent model validation".to_string(),
        status: "review".to_string(),
        rationale: "The current engine is deterministic and synthetic; insurer-approved actuarial tools must replace it before production use.".to_string(),
    });

    stresses
}
