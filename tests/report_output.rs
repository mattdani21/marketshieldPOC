//! Tests for the reports non-technical readers actually receive.
//!
//! The HTML report is meant to survive being emailed or dropped on a shared
//! drive, and to be read by people who will not see this repository. Those two
//! facts are what these tests hold in place.

use marketshield::{
    build_state,
    models::MonitorRunRequest,
    services::{monitor, report},
};

async fn rendered() -> (String, String) {
    let state = build_state("sqlite::memory:")
        .await
        .expect("state should build");

    let run = monitor::run(
        &state.repository,
        MonitorRunRequest {
            trigger: Some("test".to_string()),
            raise_signals: true,
        },
    )
    .await
    .expect("monitor should run");

    let data = report::gather(&state.repository, &run)
        .await
        .expect("report data should gather");

    (report::markdown(&data), report::html(&data))
}

/// A single file with no external requests. If this fails, the report stops
/// rendering the moment it leaves the machine that made it.
#[tokio::test]
async fn the_html_report_is_entirely_self_contained() {
    let (_, html) = rendered().await;

    for pattern in ["http://", "https://", "<script", "<img", "@import", "url("] {
        assert!(
            !html.contains(pattern),
            "the HTML report must not contain {pattern:?} — it has to render offline"
        );
    }
}

/// Synthetic data leaving the building without its label is the main way this
/// report could do harm.
#[tokio::test]
async fn both_reports_declare_that_the_data_is_synthetic() {
    let (markdown, html) = rendered().await;

    assert!(markdown.to_lowercase().contains("synthetic"));
    assert!(html.to_lowercase().contains("synthetic"));
    assert!(
        html.contains("Fictional demonstration"),
        "the banner must be present in the HTML"
    );
}

/// The limits are the first thing a product actuary will ask about, so they
/// ship with the report rather than in a conversation.
#[tokio::test]
async fn the_html_report_states_its_own_limits() {
    let (_, html) = rendered().await;

    assert!(html.contains("What this report does not tell you"));
    assert!(html.contains("relative, not absolute"));
    assert!(html.contains("validated actuarial model"));
    assert!(html.contains("weights are a judgement"));
}

#[tokio::test]
async fn both_reports_lead_with_the_plain_language_answer() {
    let (markdown, html) = rendered().await;

    for (name, output) in [("markdown", &markdown), ("html", &html)] {
        assert!(
            output.contains("What this says"),
            "{name} should lead with the plain-language summary"
        );
        assert!(
            output.contains("Ridgeline Wealth"),
            "{name} should name the leading competitor"
        );
        assert!(
            output.contains("Effective annual cost at R500k"),
            "{name} should name the largest deficit"
        );
    }
}

/// Every competitor number in the report has to be traceable to a source.
#[tokio::test]
async fn the_html_report_carries_sources_for_competitor_values() {
    let (_, html) = rendered().await;

    assert!(html.contains("Public EAC disclosure, May 2026"));
    assert!(html.contains("Recorded competitor moves"));
}

#[tokio::test]
async fn units_are_rendered_for_people_not_for_the_schema() {
    let (markdown, html) = rendered().await;

    for (name, output) in [("markdown", &markdown), ("html", &html)] {
        assert!(
            !output.contains("zar_millions"),
            "{name} leaks a storage unit into a human-facing report"
        );
    }
}
