//! Scheduled competitive-position check.
//!
//! Runs the same monitor the API exposes, writes a Markdown report, and exits
//! non-zero when our position has moved against us by more than the configured
//! threshold. That exit code is the point: a scheduled job fails when we are
//! being left behind, in the same way a test suite fails when code breaks.
//!
//! Usage:
//!   monitor [--report <path>] [--trigger <name>] [--no-signals] [--allow-breach]

use std::{env, fmt::Write as _, process::ExitCode};

use marketshield::{
    build_state,
    error::AppError,
    models::{ComparisonMatrix, MonitorRunRequest, MonitorRunResult},
    repository::{Repository, now_rfc3339},
    services::comparison,
};

struct Options {
    report_path: Option<String>,
    trigger: String,
    raise_signals: bool,
    allow_breach: bool,
}

fn parse_options() -> Result<Options, String> {
    let mut options = Options {
        report_path: None,
        trigger: "scheduled".to_string(),
        raise_signals: true,
        allow_breach: false,
    };

    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--report" => {
                options.report_path = Some(
                    args.next()
                        .ok_or_else(|| "--report requires a path".to_string())?,
                );
            }
            "--trigger" => {
                options.trigger = args
                    .next()
                    .ok_or_else(|| "--trigger requires a name".to_string())?;
            }
            "--no-signals" => options.raise_signals = false,
            "--allow-breach" => options.allow_breach = true,
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    Ok(options)
}

#[tokio::main]
async fn main() -> ExitCode {
    let options = match parse_options() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    match run(&options).await {
        Ok(result) => {
            if result.breached && !options.allow_breach {
                eprintln!(
                    "competitive position breached a monitoring threshold ({} finding(s))",
                    result.findings.len()
                );
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            eprintln!("monitor failed: {error}");
            ExitCode::from(2)
        }
    }
}

async fn run(options: &Options) -> Result<MonitorRunResult, AppError> {
    let database_url = env::var("MARKETSHIELD_DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/marketshield.db".to_string());

    let state = build_state(&database_url).await?;
    let result = marketshield::services::monitor::run(
        &state.repository,
        MonitorRunRequest {
            trigger: Some(options.trigger.clone()),
            raise_signals: options.raise_signals,
        },
    )
    .await?;

    let report = build_report(&state.repository, &result).await?;
    print!("{report}");

    if let Some(path) = &options.report_path {
        tokio::fs::write(path, &report)
            .await
            .map_err(|error| AppError::Internal(format!("failed to write report: {error}")))?;
    }

    Ok(result)
}

async fn build_report(
    repository: &Repository,
    result: &MonitorRunResult,
) -> Result<String, AppError> {
    let mut report = String::new();

    writeln!(report, "# MarketShield competitive position report").ok();
    writeln!(report).ok();
    writeln!(
        report,
        "Run `{}` · trigger `{}` · {}",
        result.run_id, result.trigger, result.started_at
    )
    .ok();
    writeln!(report).ok();
    writeln!(
        report,
        "> Demonstration data. Every competitor value in this report is synthetic."
    )
    .ok();
    writeln!(report).ok();

    writeln!(
        report,
        "**Status:** {}",
        if result.breached {
            "position moved against us beyond threshold"
        } else {
            "no threshold breached"
        }
    )
    .ok();
    writeln!(report).ok();

    writeln!(report, "## Position by product line").ok();
    writeln!(report).ok();
    writeln!(
        report,
        "| Product line | Position | Change | Rank | Leader |"
    )
    .ok();
    writeln!(report, "| --- | ---: | ---: | :---: | --- |").ok();
    for line in &result.product_lines {
        let change = line
            .score_change
            .map(|value| format!("{value:+.1}"))
            .unwrap_or_else(|| "baseline".to_string());
        writeln!(
            report,
            "| {} | {:.1} | {} | {} of {} | {} |",
            line.product_line_name,
            line.position_score,
            change,
            line.rank,
            line.provider_count,
            line.leader_name
        )
        .ok();
    }
    writeln!(report).ok();

    if result.findings.is_empty() {
        writeln!(report, "## Findings").ok();
        writeln!(report).ok();
        writeln!(
            report,
            "No competitive drift detected since the previous run."
        )
        .ok();
        writeln!(report).ok();
    } else {
        writeln!(report, "## Findings").ok();
        writeln!(report).ok();
        for finding in &result.findings {
            writeln!(
                report,
                "- **[{}] {}** — {}",
                finding.severity, finding.headline, finding.detail
            )
            .ok();
        }
        writeln!(report).ok();
    }

    if !result.cases_opened.is_empty() {
        writeln!(report, "## Cases opened by the monitor").ok();
        writeln!(report).ok();
        for case_id in &result.cases_opened {
            writeln!(report, "- `{case_id}`").ok();
        }
        writeln!(report).ok();
    }

    // The comparison itself: what we offer against each competitor today.
    let competitors = repository.list_competitors().await?;
    for line in &result.product_lines {
        let product_line = repository.get_product_line(&line.product_line_id).await?;
        let features = repository
            .list_feature_definitions(&line.product_line_id)
            .await?;
        let observations = repository
            .list_feature_observations(&line.product_line_id)
            .await?;
        let matrix = comparison::build_matrix(
            product_line,
            &competitors,
            &features,
            &observations,
            None,
            now_rfc3339()?,
        )?;

        write_matrix(&mut report, &matrix);
    }

    Ok(report)
}

fn write_matrix(report: &mut String, matrix: &ComparisonMatrix) {
    writeln!(
        report,
        "## {} — feature comparison",
        matrix.product_line.name
    )
    .ok();
    writeln!(report).ok();

    // Column order follows the standings so the leader reads first.
    let columns: Vec<&str> = matrix
        .standings
        .iter()
        .map(|standing| standing.competitor_name.as_str())
        .collect();

    write!(report, "| Dimension | Unit | Better |").ok();
    for column in &columns {
        write!(report, " {column} |").ok();
    }
    writeln!(report).ok();
    write!(report, "| --- | --- | --- |").ok();
    for _ in &columns {
        write!(report, " ---: |").ok();
    }
    writeln!(report).ok();

    for feature in &matrix.features {
        let better = if feature.direction == "lower_is_better" {
            "lower"
        } else {
            "higher"
        };
        write!(
            report,
            "| {} | {} | {} |",
            feature.name, feature.unit, better
        )
        .ok();
        for column in &columns {
            let cell = feature
                .providers
                .iter()
                .find(|provider| provider.competitor_name == *column)
                .map(|provider| {
                    // Mark the winning value so the losing rows are obvious.
                    if provider.is_best {
                        format!("**{}**", provider.value)
                    } else {
                        provider.value.to_string()
                    }
                })
                .unwrap_or_else(|| "—".to_string());
            write!(report, " {cell} |").ok();
        }
        writeln!(report).ok();
    }

    write!(report, "| **Weighted position** | | |").ok();
    for column in &columns {
        let score = matrix
            .standings
            .iter()
            .find(|standing| standing.competitor_name == *column)
            .map(|standing| format!("**{:.1}**", standing.position_score))
            .unwrap_or_else(|| "—".to_string());
        write!(report, " {score} |").ok();
    }
    writeln!(report).ok();
    writeln!(report).ok();

    if !matrix.our_position.largest_gaps.is_empty() {
        writeln!(report, "Largest weighted deficits:").ok();
        writeln!(report).ok();
        for gap in &matrix.our_position.largest_gaps {
            writeln!(
                report,
                "- **{}** — we are at {} {} against {} {} at {}.",
                gap.feature_name,
                gap.our_value,
                gap.unit,
                gap.best_value,
                gap.unit,
                gap.best_provider
            )
            .ok();
        }
        writeln!(report).ok();
    }

    if !matrix.recent_moves.is_empty() {
        writeln!(report, "Recorded competitor moves:").ok();
        writeln!(report).ok();
        for competitor_move in matrix.recent_moves.iter().take(10) {
            writeln!(
                report,
                "- {} · {} moved {} from {} to {} ({}) — source: {}",
                &competitor_move.changed_at[..10],
                competitor_move.competitor_name,
                competitor_move.feature_name,
                competitor_move.previous_value,
                competitor_move.current_value,
                if competitor_move.favourable_to_them {
                    "in their favour"
                } else {
                    "against them"
                },
                competitor_move.source_reference,
            )
            .ok();
        }
        writeln!(report).ok();
    }
}
