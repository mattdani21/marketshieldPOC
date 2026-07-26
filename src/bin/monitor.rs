//! Scheduled competitive-position check.
//!
//! Runs the same monitor the API exposes, writes a Markdown report, and exits
//! non-zero when our position has moved against us by more than the configured
//! threshold. That exit code is the point: a scheduled job fails when we are
//! being left behind, in the same way a test suite fails when code breaks.
//!
//! Usage:
//!   monitor [--report <path>] [--html <path>] [--trigger <name>]
//!           [--no-signals] [--allow-breach]

use std::{env, process::ExitCode};

use marketshield::{
    build_state,
    error::AppError,
    models::{MonitorRunRequest, MonitorRunResult},
    services::report,
};

struct Options {
    report_path: Option<String>,
    html_path: Option<String>,
    trigger: String,
    raise_signals: bool,
    allow_breach: bool,
}

fn parse_options() -> Result<Options, String> {
    let mut options = Options {
        report_path: None,
        html_path: None,
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
            "--html" => {
                options.html_path = Some(
                    args.next()
                        .ok_or_else(|| "--html requires a path".to_string())?,
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

    let data = report::gather(&state.repository, &result).await?;

    let markdown = report::markdown(&data);
    print!("{markdown}");

    if let Some(path) = &options.report_path {
        write(path, &markdown).await?;
    }
    if let Some(path) = &options.html_path {
        write(path, &report::html(&data)).await?;
    }

    Ok(result)
}

async fn write(path: &str, contents: &str) -> Result<(), AppError> {
    tokio::fs::write(path, contents)
        .await
        .map_err(|error| AppError::Internal(format!("failed to write {path}: {error}")))
}
