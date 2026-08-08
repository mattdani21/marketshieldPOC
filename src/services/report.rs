//! Competitive position reporting.
//!
//! The monitor's findings are only useful if the people who noticed the problem
//! can read them. This module renders a run as Markdown (for engineers and CI
//! summaries) and as a single self-contained HTML file (for product actuaries,
//! committees and anyone who will never run `cargo`).
//!
//! The HTML has no external requests of any kind, so it survives being emailed,
//! dropped on a shared drive, or served as a static file.

use std::fmt::Write as _;

use crate::{
    error::AppError,
    models::{
        ComparisonMatrix, MonitorRunResult, PositionSnapshot, ProductLineMonitorResult, unit_label,
    },
    repository::{Repository, now_rfc3339},
    services::comparison,
};

pub struct LineReport {
    pub line: ProductLineMonitorResult,
    pub matrix: ComparisonMatrix,
    pub history: Vec<PositionSnapshot>,
}

pub struct ReportData {
    pub run: MonitorRunResult,
    pub lines: Vec<LineReport>,
}

/// Collects everything both renderers need, so neither reaches for the database.
pub async fn gather(
    repository: &Repository,
    run: &MonitorRunResult,
) -> Result<ReportData, AppError> {
    let competitors = repository.list_competitors().await?;
    let mut lines = Vec::new();

    for line in &run.product_lines {
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
        let history = repository
            .list_position_snapshots(&line.product_line_id)
            .await?;

        lines.push(LineReport {
            line: line.clone(),
            matrix,
            history,
        });
    }

    Ok(ReportData {
        run: run.clone(),
        lines,
    })
}

// ---------------------------------------------------------------------------
// Shared phrasing
// ---------------------------------------------------------------------------

/// One plain-language sentence per product line. This is the sentence a sponsor
/// reads first, so it states the position, the direction of travel, and the
/// single largest reason.
fn plain_summary(report: &LineReport) -> String {
    let position = &report.matrix.our_position;
    let direction = match report.line.score_change {
        Some(change) if change <= -0.05 => {
            format!("down {:.1} points since the previous check", change.abs())
        }
        Some(change) if change >= 0.05 => {
            format!("up {change:.1} points since the previous check")
        }
        Some(_) => "unchanged since the previous check".to_string(),
        None => "measured for the first time".to_string(),
    };

    let reason = position
        .largest_gaps
        .first()
        .map(|gap| {
            format!(
                " The largest single reason is {}, where we are at {} against {} at {}.",
                gap.feature_name,
                value_with_unit(gap.our_value, &gap.unit),
                value_with_unit(gap.best_value, &gap.unit),
                gap.best_provider
            )
        })
        .unwrap_or_default();

    format!(
        "On {} we rank {} of {} across {} tracked dimensions, {:.1} points behind {}, and {}.{}",
        report.matrix.product_line.name.to_lowercase(),
        position.rank,
        position.provider_count,
        report.matrix.features.len(),
        position.score_behind_leader,
        position.leader_name,
        direction,
        reason
    )
}

/// Keeps units glued to their value the way a reader expects: `1.62%`, `47 days`.
fn value_with_unit(value: f64, unit: &str) -> String {
    let label = unit_label(unit);
    if label == "%" {
        format!("{}%", format_number(value))
    } else {
        format!("{} {}", format_number(value), label)
    }
}

fn format_number(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 {
        format!("{}", value.round() as i64)
    } else {
        let text = format!("{value:.2}");
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn short_date(timestamp: &str) -> &str {
    timestamp.get(..10).unwrap_or(timestamp)
}

// ---------------------------------------------------------------------------
// Markdown
// ---------------------------------------------------------------------------

pub fn markdown(data: &ReportData) -> String {
    let run = &data.run;
    let mut report = String::new();

    writeln!(report, "# MarketShield competitive position report").ok();
    writeln!(report).ok();
    writeln!(
        report,
        "Run `{}` · trigger `{}` · {}",
        run.run_id, run.trigger, run.started_at
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
        if run.breached {
            "position moved against us beyond threshold"
        } else {
            "no threshold breached"
        }
    )
    .ok();
    writeln!(report).ok();

    writeln!(report, "## What this says").ok();
    writeln!(report).ok();
    for line in &data.lines {
        writeln!(report, "- {}", plain_summary(line)).ok();
    }
    writeln!(report).ok();

    writeln!(report, "## Position by product line").ok();
    writeln!(report).ok();
    writeln!(
        report,
        "| Product line | Position | Change | Rank | Leader |"
    )
    .ok();
    writeln!(report, "| --- | ---: | ---: | :---: | --- |").ok();
    for line in &run.product_lines {
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

    writeln!(report, "## Findings").ok();
    writeln!(report).ok();
    if run.findings.is_empty() {
        writeln!(
            report,
            "No competitive drift detected since the previous run."
        )
        .ok();
    } else {
        for finding in &run.findings {
            writeln!(
                report,
                "- **[{}] {}** — {}",
                finding.severity, finding.headline, finding.detail
            )
            .ok();
        }
    }
    writeln!(report).ok();

    if !run.cases_opened.is_empty() {
        writeln!(report, "## Cases opened by the monitor").ok();
        writeln!(report).ok();
        for case_id in &run.cases_opened {
            writeln!(report, "- `{case_id}`").ok();
        }
        writeln!(report).ok();
    }

    for line in &data.lines {
        write_markdown_matrix(&mut report, &line.matrix);
    }

    report
}

fn write_markdown_matrix(report: &mut String, matrix: &ComparisonMatrix) {
    writeln!(
        report,
        "## {} — feature comparison",
        matrix.product_line.name
    )
    .ok();
    writeln!(report).ok();

    // Column order follows the standings so the leader reads first.
    let columns = &matrix.standings;

    write!(report, "| Dimension | Unit | Better |").ok();
    for column in columns {
        write!(
            report,
            " {}{} |",
            column.short_name,
            if column.is_us { " (us)" } else { "" }
        )
        .ok();
    }
    writeln!(report).ok();
    write!(report, "| --- | --- | --- |").ok();
    for _ in columns {
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
            feature.name,
            unit_label(&feature.unit),
            better
        )
        .ok();
        for column in columns {
            let cell = feature
                .providers
                .iter()
                .find(|provider| provider.competitor_id == column.competitor_id)
                .map(|provider| {
                    if provider.is_best {
                        format!("**{}**", format_number(provider.value))
                    } else {
                        format_number(provider.value)
                    }
                })
                .unwrap_or_else(|| "—".to_string());
            write!(report, " {cell} |").ok();
        }
        writeln!(report).ok();
    }

    write!(report, "| **Weighted position** | | |").ok();
    for column in columns {
        write!(report, " **{:.1}** |", column.position_score).ok();
    }
    writeln!(report).ok();
    writeln!(report).ok();

    if !matrix.our_position.largest_gaps.is_empty() {
        writeln!(report, "Largest weighted deficits:").ok();
        writeln!(report).ok();
        for gap in &matrix.our_position.largest_gaps {
            writeln!(
                report,
                "- **{}** — we are at {} against {} at {}.",
                gap.feature_name,
                value_with_unit(gap.our_value, &gap.unit),
                value_with_unit(gap.best_value, &gap.unit),
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
                short_date(&competitor_move.changed_at),
                competitor_move.competitor_name,
                competitor_move.feature_name,
                format_number(competitor_move.previous_value),
                format_number(competitor_move.current_value),
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

// ---------------------------------------------------------------------------
// HTML
// ---------------------------------------------------------------------------

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn html(data: &ReportData) -> String {
    let run = &data.run;
    let mut page = String::new();

    page.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    page.push_str("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n");
    page.push_str("<title>Competitive position report — MarketShield</title>\n");
    page.push_str(STYLE);
    page.push_str("</head>\n<body>\n<main class=\"page viz-root\">\n");

    // The banner is first and unmissable: this report is synthetic, and it must
    // not be mistaken for a real competitor assessment if it is forwarded on.
    page.push_str(
        "<div class=\"banner\"><strong>Fictional demonstration.</strong> Every provider, value and \
         financial figure in this report is synthetic. It shows what the monitoring produces, not \
         a real competitive assessment.</div>\n",
    );

    writeln!(
        page,
        "<header class=\"head\">\n<p class=\"eyebrow\">Cape Meridian Life &amp; Investments</p>\n\
         <h1>Competitive position report</h1>\n\
         <p class=\"sub\">{} · trigger <code>{}</code> · run <code>{}</code></p>\n</header>",
        escape(short_date(&run.started_at)),
        escape(&run.trigger),
        escape(&run.run_id)
    )
    .ok();

    let (status_class, status_icon, status_label) = if run.breached {
        (
            "critical",
            "▲",
            "Our position moved against us by more than the agreed threshold",
        )
    } else {
        (
            "good",
            "●",
            "No threshold breached since the previous check",
        )
    };
    writeln!(
        page,
        "<p class=\"status {status_class}\"><span class=\"icon\" aria-hidden=\"true\">{status_icon}</span> <strong>{}</strong></p>",
        escape(status_label)
    )
    .ok();

    page.push_str("<section class=\"card\">\n<h2>What this says</h2>\n<ul class=\"plain\">\n");
    for line in &data.lines {
        writeln!(page, "<li>{}</li>", escape(&plain_summary(line))).ok();
    }
    page.push_str("</ul>\n</section>\n");

    if !run.findings.is_empty() {
        page.push_str("<section class=\"card\">\n<h2>What changed</h2>\n");
        for finding in &run.findings {
            let severity_class = match finding.severity.as_str() {
                "high" => "critical",
                "medium" => "warning",
                _ => "muted",
            };
            writeln!(
                page,
                "<div class=\"finding {severity_class}\"><p class=\"finding-head\"><span class=\"pill {severity_class}\">{}</span> {}</p><p class=\"finding-detail\">{}</p></div>",
                escape(&finding.severity),
                escape(&finding.headline),
                escape(&finding.detail)
            )
            .ok();
        }
        page.push_str("</section>\n");
    }

    if !run.cases_opened.is_empty() {
        page.push_str("<section class=\"card\">\n<h2>Opened for review</h2>\n");
        writeln!(
            page,
            "<p class=\"plain\">The monitor opened {} for human diagnosis. {} carries the \
             comparison that triggered it, and every approval gate is unmet — nothing has been \
             decided, priced or communicated.</p>",
            run.cases_opened
                .iter()
                .map(|case_id| format!("<strong>{}</strong>", escape(case_id)))
                .collect::<Vec<_>>()
                .join(", "),
            if run.cases_opened.len() == 1 {
                "It"
            } else {
                "Each"
            }
        )
        .ok();
        page.push_str("</section>\n");
    }

    for line in &data.lines {
        write_html_line(&mut page, line);
    }

    page.push_str(LIMITATIONS);
    page.push_str("</main>\n</body>\n</html>\n");

    page
}

fn write_html_line(page: &mut String, report: &LineReport) {
    let matrix = &report.matrix;
    let position = &matrix.our_position;

    writeln!(
        page,
        "<section class=\"card line\">\n<h2>{}</h2>",
        escape(&matrix.product_line.name)
    )
    .ok();

    // KPI row: the handful of headline numbers, as stat tiles rather than a chart.
    writeln!(
        page,
        "<div class=\"kpis\">\n\
         <div class=\"kpi\"><span class=\"kpi-label\">Weighted position</span><span class=\"kpi-value\">{:.1}</span><span class=\"kpi-note\">out of 100</span></div>\n\
         <div class=\"kpi\"><span class=\"kpi-label\">Rank</span><span class=\"kpi-value\">{} of {}</span><span class=\"kpi-note\">tracked providers</span></div>\n\
         <div class=\"kpi\"><span class=\"kpi-label\">Behind {}</span><span class=\"kpi-value\">{:.1}</span><span class=\"kpi-note\">points</span></div>\n\
         <div class=\"kpi\"><span class=\"kpi-label\">Dimensions</span><span class=\"kpi-value\">{}</span><span class=\"kpi-note\">compared</span></div>\n\
         </div>",
        position.position_score,
        position.rank,
        position.provider_count,
        escape(&position.leader_name),
        position.score_behind_leader,
        matrix.features.len()
    )
    .ok();

    write_provider_chart(page, report);
    write_trend_chart(page, report);
    write_gap_chart(page, report);
    write_html_matrix(page, matrix);
    write_html_moves(page, matrix);

    page.push_str("</section>\n");
}

/// Emphasis form: our bar carries the accent, every other provider is
/// de-emphasised. The reader's job is "where do we sit", not "tell four
/// series apart".
fn write_provider_chart(page: &mut String, report: &LineReport) {
    let standings = &report.matrix.standings;
    if standings.is_empty() {
        return;
    }

    let row_height = 34.0;
    let label_width = 210.0;
    let value_width = 52.0;
    let chart_width = 720.0;
    let track = chart_width - label_width - value_width;
    let height = row_height * standings.len() as f64 + 8.0;

    writeln!(
        page,
        "<figure class=\"figure\">\n<figcaption>Weighted position by provider — higher is better</figcaption>\n\
         <svg viewBox=\"0 0 {chart_width} {height}\" role=\"img\" aria-label=\"Weighted position by provider\" class=\"chart\">"
    )
    .ok();

    for (index, standing) in standings.iter().enumerate() {
        let y = index as f64 * row_height + 4.0;
        // Scale is the full 0-100 the score is defined on, never trimmed to the
        // data, so the gap is not visually exaggerated.
        let width = (standing.position_score / 100.0) * track;
        let bar_class = if standing.is_us {
            "bar-us"
        } else {
            "bar-other"
        };
        let label_class = if standing.is_us { "lab-us" } else { "lab" };
        let suffix = if standing.is_us { " (us)" } else { "" };

        writeln!(
            page,
            "<text x=\"0\" y=\"{:.1}\" class=\"{label_class}\">{}{}</text>",
            y + 15.0,
            escape(&standing.short_name),
            suffix
        )
        .ok();
        writeln!(
            page,
            "<rect x=\"{label_width}\" y=\"{:.1}\" width=\"{track}\" height=\"14\" rx=\"4\" class=\"bar-track\"/>",
            y + 5.0
        )
        .ok();
        writeln!(
            page,
            "<rect x=\"{label_width}\" y=\"{:.1}\" width=\"{:.1}\" height=\"14\" rx=\"4\" class=\"{bar_class}\"><title>{}: {:.1} of 100</title></rect>",
            y + 5.0,
            width.max(2.0),
            escape(&standing.competitor_name),
            standing.position_score
        )
        .ok();
        writeln!(
            page,
            "<text x=\"{:.1}\" y=\"{:.1}\" class=\"val\">{:.1}</text>",
            chart_width - value_width + 8.0,
            y + 16.0,
            standing.position_score
        )
        .ok();
    }

    page.push_str("</svg>\n</figure>\n");
}

/// Trend over time, on a fixed 0-100 scale for the same reason.
fn write_trend_chart(page: &mut String, report: &LineReport) {
    let history = &report.history;
    if history.len() < 2 {
        return;
    }

    let width = 720.0;
    let height = 200.0;
    let left = 44.0;
    let right = 16.0;
    let top = 16.0;
    let bottom = 34.0;
    let plot_width = width - left - right;
    let plot_height = height - top - bottom;

    let x_at = |index: usize| -> f64 {
        if history.len() == 1 {
            left + plot_width / 2.0
        } else {
            left + (index as f64 / (history.len() - 1) as f64) * plot_width
        }
    };
    let y_at = |score: f64| -> f64 { top + (1.0 - score / 100.0) * plot_height };

    writeln!(
        page,
        "<figure class=\"figure\">\n<figcaption>Recorded position over time — 0 to 100 scale</figcaption>\n\
         <svg viewBox=\"0 0 {width} {height}\" role=\"img\" aria-label=\"Recorded competitive position over time\" class=\"chart\">"
    )
    .ok();

    for gridline in [0.0_f64, 25.0, 50.0, 75.0, 100.0] {
        let y = y_at(gridline);
        writeln!(
            page,
            "<line x1=\"{left}\" y1=\"{y:.1}\" x2=\"{:.1}\" y2=\"{y:.1}\" class=\"grid\"/>\n\
             <text x=\"{:.1}\" y=\"{:.1}\" class=\"tick\">{gridline:.0}</text>",
            width - right,
            left - 8.0,
            y + 4.0
        )
        .ok();
    }

    let points: Vec<String> = history
        .iter()
        .enumerate()
        .map(|(index, snapshot)| format!("{:.1},{:.1}", x_at(index), y_at(snapshot.position_score)))
        .collect();
    writeln!(
        page,
        "<polyline points=\"{}\" class=\"line\"/>",
        points.join(" ")
    )
    .ok();

    for (index, snapshot) in history.iter().enumerate() {
        let x = x_at(index);
        let y = y_at(snapshot.position_score);
        writeln!(
            page,
            "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"5\" class=\"dot\"><title>{}: {:.1}</title></circle>",
            escape(short_date(&snapshot.captured_at)),
            snapshot.position_score
        )
        .ok();
        // Label only the endpoints, so the line stays readable. They are anchored
        // inward so neither label runs off the edge of the plot.
        let is_first = index == 0;
        let is_last = index == history.len() - 1;
        if is_first || is_last {
            // Anchoring is a class, not a presentation attribute: in SVG a CSS
            // rule beats an inline attribute, so `.tick`'s anchor would win.
            let anchor = if is_first {
                "anchor-start"
            } else {
                "anchor-end"
            };
            // Nudge clear of the y-axis ticks on the left edge.
            let label_x = if is_first { x + 8.0 } else { x };
            writeln!(
                page,
                "<text x=\"{label_x:.1}\" y=\"{:.1}\" class=\"pointlab {anchor}\">{:.1}</text>",
                y - 12.0,
                snapshot.position_score
            )
            .ok();
            writeln!(
                page,
                "<text x=\"{x:.1}\" y=\"{:.1}\" class=\"datelab {anchor}\">{}</text>",
                height - 12.0,
                escape(short_date(&snapshot.captured_at))
            )
            .ok();
        }
    }

    page.push_str("</svg>\n</figure>\n");
}

fn write_gap_chart(page: &mut String, report: &LineReport) {
    let gaps = &report.matrix.our_position.largest_gaps;
    if gaps.is_empty() {
        return;
    }

    let row_height = 40.0;
    let label_width = 260.0;
    let chart_width = 720.0;
    let track = chart_width - label_width - 60.0;
    let height = row_height * gaps.len() as f64 + 8.0;
    let max_deficit = gaps
        .iter()
        .map(|gap| gap.weighted_deficit)
        .fold(0.0_f64, f64::max)
        .max(1.0);

    writeln!(
        page,
        "<figure class=\"figure\">\n<figcaption>Largest weighted deficits — what is costing us most</figcaption>\n\
         <svg viewBox=\"0 0 {chart_width} {height}\" role=\"img\" aria-label=\"Largest weighted deficits\" class=\"chart\">"
    )
    .ok();

    for (index, gap) in gaps.iter().enumerate() {
        let y = index as f64 * row_height + 4.0;
        let width = (gap.weighted_deficit / max_deficit) * track;

        writeln!(
            page,
            "<text x=\"0\" y=\"{:.1}\" class=\"lab\">{}</text>",
            y + 14.0,
            escape(&gap.feature_name)
        )
        .ok();
        writeln!(
            page,
            "<text x=\"0\" y=\"{:.1}\" class=\"sublab\">{} vs {} at {}</text>",
            y + 30.0,
            escape(&value_with_unit(gap.our_value, &gap.unit)),
            escape(&value_with_unit(gap.best_value, &gap.unit)),
            escape(&gap.best_provider)
        )
        .ok();
        writeln!(
            page,
            "<rect x=\"{label_width}\" y=\"{:.1}\" width=\"{:.1}\" height=\"14\" rx=\"4\" class=\"bar-gap\"><title>{}: weighted deficit {:.0}</title></rect>",
            y + 8.0,
            width.max(2.0),
            escape(&gap.feature_name),
            gap.weighted_deficit
        )
        .ok();
    }

    page.push_str("</svg>\n</figure>\n");
}

fn write_html_matrix(page: &mut String, matrix: &ComparisonMatrix) {
    page.push_str("<h3>What we offer against each provider</h3>\n");
    page.push_str(
        "<p class=\"note\">The best value on each row is marked. Hover a value to see its source.</p>\n",
    );
    page.push_str("<div class=\"scroll\">\n<table>\n<thead>\n<tr><th scope=\"col\">Dimension</th><th scope=\"col\">Better</th>");

    for standing in &matrix.standings {
        write!(
            page,
            "<th scope=\"col\" class=\"num{}\" title=\"{}\">{}{}</th>",
            if standing.is_us { " col-us" } else { "" },
            escape(&standing.competitor_name),
            escape(&standing.short_name),
            if standing.is_us {
                "<br><span class=\"tiny\">(us)</span>"
            } else {
                ""
            }
        )
        .ok();
    }
    page.push_str("</tr>\n</thead>\n<tbody>\n");

    for feature in &matrix.features {
        write!(
            page,
            "<tr><th scope=\"row\">{}<span class=\"unit\">{}</span></th><td class=\"dir\">{}</td>",
            escape(&feature.name),
            escape(unit_label(&feature.unit)),
            if feature.direction == "lower_is_better" {
                "lower"
            } else {
                "higher"
            }
        )
        .ok();

        for standing in &matrix.standings {
            let cell = feature
                .providers
                .iter()
                .find(|provider| provider.competitor_id == standing.competitor_id);
            match cell {
                Some(provider) => {
                    write!(
                        page,
                        "<td class=\"num{}{}\" title=\"{}\">{}{}</td>",
                        if standing.is_us { " col-us" } else { "" },
                        if provider.is_best { " best" } else { "" },
                        escape(&provider.source_reference),
                        format_number(provider.value),
                        if provider.is_best {
                            "<span class=\"tick-mark\" aria-label=\"best\">✓</span>"
                        } else {
                            ""
                        }
                    )
                    .ok();
                }
                None => {
                    write!(
                        page,
                        "<td class=\"num{}\">—</td>",
                        if standing.is_us { " col-us" } else { "" }
                    )
                    .ok();
                }
            }
        }
        page.push_str("</tr>\n");
    }

    page.push_str("<tr class=\"total\"><th scope=\"row\">Weighted position</th><td></td>");
    for standing in &matrix.standings {
        write!(
            page,
            "<td class=\"num{}\">{:.1}</td>",
            if standing.is_us { " col-us" } else { "" },
            standing.position_score
        )
        .ok();
    }
    page.push_str("</tr>\n</tbody>\n</table>\n</div>\n");
}

fn write_html_moves(page: &mut String, matrix: &ComparisonMatrix) {
    if matrix.recent_moves.is_empty() {
        return;
    }

    page.push_str("<h3>Recorded competitor moves</h3>\n");
    page.push_str(
        "<p class=\"note\">Every value carries the published source it came from.</p>\n<ul class=\"moves\">\n",
    );

    for competitor_move in matrix.recent_moves.iter().take(10) {
        writeln!(
            page,
            "<li><span class=\"move-date\">{}</span> <strong>{}</strong> moved {} from {} to {} — <span class=\"src\">{}</span></li>",
            escape(short_date(&competitor_move.changed_at)),
            escape(&competitor_move.competitor_name),
            escape(&competitor_move.feature_name),
            escape(&value_with_unit(competitor_move.previous_value, &competitor_move.unit)),
            escape(&value_with_unit(competitor_move.current_value, &competitor_move.unit)),
            escape(&competitor_move.source_reference)
        )
        .ok();
    }

    page.push_str("</ul>\n");
}

/// Stated plainly, because the first question a product actuary will ask is what
/// this number is not.
const LIMITATIONS: &str = r#"<section class="card limits">
<h2>What this report does not tell you</h2>
<ul class="plain">
<li><strong>The position score is relative, not absolute.</strong> Each dimension is scored 100 at the best value observed and 0 at the worst, then weighted. A score says where we sit among these providers on these dimensions. It is not a rating of how competitive the product is in the market.</li>
<li><strong>A leader extending its lead shows up as our score falling</strong>, not theirs rising, because the scale is anchored to the observed range.</li>
<li><strong>The weights are a judgement, not a measurement.</strong> They decide which gaps look largest and have not yet been agreed with product owners.</li>
<li><strong>Nothing here is a validated actuarial model.</strong> The scoring is a deterministic demonstration model and must not, on its own, justify a pricing or product decision.</li>
<li><strong>Commercial sizing is synthetic.</strong> Premium at risk and conversion figures stand in for a quote-to-issue data feed that is not yet connected.</li>
</ul>
</section>
"#;

const STYLE: &str = r#"<style>
:root { color-scheme: light dark; }
.viz-root {
  --surface-1: #fcfcfb; --plane: #f9f9f7;
  --text-primary: #0b0b0b; --text-secondary: #52514e; --muted: #898781;
  --grid: #e1e0d9; --axis: #c3c2b7; --border: rgba(11,11,11,0.10);
  --series-1: #2a78d6; --deemph: #898781; --track: #eceae4;
  --good: #0ca30c; --warning: #fab219; --critical: #d03b3b;
}
@media (prefers-color-scheme: dark) {
  :root:where(:not([data-theme="light"])) .viz-root {
    --surface-1: #1a1a19; --plane: #0d0d0d;
    --text-primary: #ffffff; --text-secondary: #c3c2b7; --muted: #898781;
    --grid: #2c2c2a; --axis: #383835; --border: rgba(255,255,255,0.10);
    --series-1: #3987e5; --deemph: #898781; --track: #262624;
    --good: #0ca30c; --warning: #fab219; --critical: #d03b3b;
  }
}
:root[data-theme="dark"] .viz-root {
  --surface-1: #1a1a19; --plane: #0d0d0d;
  --text-primary: #ffffff; --text-secondary: #c3c2b7; --muted: #898781;
  --grid: #2c2c2a; --axis: #383835; --border: rgba(255,255,255,0.10);
  --series-1: #3987e5; --deemph: #898781; --track: #262624;
}
* { box-sizing: border-box; }
body {
  margin: 0; background: var(--plane); color: var(--text-primary);
  font: 15px/1.6 system-ui, -apple-system, "Segoe UI", sans-serif;
}
.page { max-width: 900px; margin: 0 auto; padding: 28px 20px 64px; }
.banner {
  border: 1px solid var(--border); border-left: 4px solid var(--warning);
  background: var(--surface-1); border-radius: 10px; padding: 12px 14px;
  color: var(--text-secondary); font-size: 14px; margin-bottom: 24px;
}
.eyebrow { margin: 0; color: var(--text-secondary); font-size: 12px; letter-spacing: .09em; text-transform: uppercase; }
h1 { margin: 6px 0 4px; font-size: 30px; line-height: 1.2; }
h2 { margin: 0 0 12px; font-size: 20px; }
h3 { margin: 26px 0 6px; font-size: 16px; }
.sub { margin: 0; color: var(--text-secondary); font-size: 13px; }
code { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: .9em; }
.status {
  display: flex; gap: 9px; align-items: baseline; margin: 20px 0 24px;
  padding: 12px 14px; border-radius: 10px; border: 1px solid var(--border);
  background: var(--surface-1); font-size: 15px;
}
.status .icon { font-size: 13px; }
.status.critical { border-left: 4px solid var(--critical); }
.status.critical .icon { color: var(--critical); }
.status.good { border-left: 4px solid var(--good); }
.status.good .icon { color: var(--good); }
.card {
  background: var(--surface-1); border: 1px solid var(--border);
  border-radius: 12px; padding: 20px; margin-bottom: 20px;
}
.plain { margin: 0; padding-left: 20px; }
.plain li { margin-bottom: 10px; }
.plain li:last-child { margin-bottom: 0; }
.note { margin: 0 0 10px; color: var(--text-secondary); font-size: 13px; }
.finding { border-top: 1px solid var(--border); padding: 12px 0; }
.finding:first-of-type { border-top: 0; padding-top: 0; }
.finding-head { margin: 0 0 4px; font-weight: 600; }
.finding-detail { margin: 0; color: var(--text-secondary); font-size: 14px; }
.pill {
  display: inline-block; border-radius: 99px; padding: 2px 9px; font-size: 11px;
  text-transform: uppercase; letter-spacing: .05em; border: 1px solid var(--border);
  color: var(--text-secondary); margin-right: 6px;
}
.pill.critical { border-color: var(--critical); color: var(--critical); }
.pill.warning { border-color: var(--warning); color: var(--text-secondary); }
.kpis { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin: 4px 0 22px; }
.kpi { border: 1px solid var(--border); border-radius: 10px; padding: 12px; }
.kpi-label { display: block; font-size: 12px; color: var(--text-secondary); }
.kpi-value { display: block; font-size: 26px; font-weight: 650; line-height: 1.25; }
.kpi-note { display: block; font-size: 12px; color: var(--muted); }
.figure { margin: 22px 0; }
.figure figcaption { font-size: 13px; color: var(--text-secondary); margin-bottom: 8px; }
.chart { width: 100%; height: auto; overflow: visible; }
.chart .lab { fill: var(--text-primary); font-size: 13px; }
.chart .lab-us { fill: var(--text-primary); font-size: 13px; font-weight: 650; }
.chart .sublab { fill: var(--muted); font-size: 11px; }
.chart .val { fill: var(--text-primary); font-size: 13px; font-variant-numeric: tabular-nums; }
.chart .tick { fill: var(--muted); font-size: 11px; text-anchor: end; }
.chart .datelab { fill: var(--muted); font-size: 11px; }
.chart .anchor-start { text-anchor: start; }
.chart .anchor-end { text-anchor: end; }

.chart .pointlab { fill: var(--text-primary); font-size: 12px; font-variant-numeric: tabular-nums; }
.chart .grid { stroke: var(--grid); stroke-width: 1; }
.chart .line { fill: none; stroke: var(--series-1); stroke-width: 2; stroke-linejoin: round; }
.chart .dot { fill: var(--series-1); stroke: var(--surface-1); stroke-width: 2; }
.chart .bar-track { fill: var(--track); }
.chart .bar-us { fill: var(--series-1); }
.chart .bar-other { fill: var(--deemph); }
.chart .bar-gap { fill: var(--series-1); }
.scroll { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: 14px; }
th, td { padding: 9px 10px; border-bottom: 1px solid var(--border); text-align: left; vertical-align: top; }
thead th { font-size: 12px; color: var(--text-secondary); font-weight: 600; }
td.num, th.num { text-align: right; font-variant-numeric: tabular-nums; }
td.dir { color: var(--muted); font-size: 12px; }
.unit { display: block; font-size: 11px; color: var(--muted); font-weight: 400; }
.tiny { font-size: 10px; color: var(--muted); font-weight: 400; }
.col-us { background: color-mix(in srgb, var(--series-1) 8%, transparent); }
td.best { font-weight: 700; }
.tick-mark { color: var(--good); margin-left: 4px; font-size: 11px; }
tr.total td, tr.total th { border-top: 2px solid var(--axis); font-weight: 700; }
.moves { margin: 0; padding-left: 20px; font-size: 14px; }
.moves li { margin-bottom: 8px; }
.move-date { color: var(--muted); font-variant-numeric: tabular-nums; }
.src { color: var(--text-secondary); font-size: 13px; }
.limits { border-left: 4px solid var(--axis); }
.limits li { font-size: 14px; color: var(--text-secondary); }
@media (max-width: 720px) { .kpis { grid-template-columns: repeat(2, 1fr); } }
@media print {
  body { background: #fff; }
  .card, .status, .banner { break-inside: avoid; }
  .page { max-width: none; padding: 0; }
}
</style>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_drop_trailing_zeros() {
        assert_eq!(format_number(47.0), "47");
        assert_eq!(format_number(1.62), "1.62");
        assert_eq!(format_number(0.30), "0.3");
    }

    #[test]
    fn markup_is_escaped() {
        assert_eq!(
            escape("Ridgeline & <script>"),
            "Ridgeline &amp; &lt;script&gt;"
        );
    }
}
