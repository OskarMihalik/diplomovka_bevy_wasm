//! Experiment B from framework_bench/PLAN.md: Cornucopia vs Kysely, one database scenario with no
//! HTTP. Reads query_layer.csv (written by framework_bench/query_bench/run.sh, one row per run) and
//! writes the `qb_*` tables.

use crate::stats::{Summary, holm, welch};
use serde::Deserialize;
use std::fmt::Write as _;
use std::path::Path;

/// The Postgres cpuset of the runs in the tables (PLAN.md §6.8); rows with another one are only
/// in the overview
const PG_CPUS: &str = "4 6 8 10";
/// Rows read or written by one scenario (1 user + 100 tags + 1 tag + 2 in the transaction + 1 delete)
const ROWS_PER_SCENARIO: u64 = 105;

/// Same rule as experiment A (PLAN.md §10)
const MIN_DIFF_PERCENT: f64 = 5.;
const ALPHA: f64 = 0.05;

#[derive(Clone, Copy, PartialEq)]
enum Variant {
    Cornucopia,
    Kysely,
}

#[derive(Deserialize)]
pub struct Row {
    turbo: String,
    pg_cpus: String,
    variant: String,
    scenarios: u64,
    errors: u64,
    rows: u64,
    scenarios_per_s: f64,
    scen_mean: f64,
    scen_p50: f64,
    scen_p95: f64,
    scen_p99: f64,
    q1_p50: f64,
    q1_p99: f64,
    q2_p50: f64,
    q2_p99: f64,
    q3_p50: f64,
    q3_p99: f64,
    q4_p50: f64,
    q4_p99: f64,
    cpu_ms_per_scenario: f64,
    rss_peak_mib: f64,
}

impl Row {
    fn variant(&self) -> Option<Variant> {
        match self.variant.as_str() {
            "cornucopia" => Some(Variant::Cornucopia),
            "kysely" => Some(Variant::Kysely),
            _ => None,
        }
    }
}

pub fn read(path: &Path) -> Result<Vec<Row>, String> {
    let mut reader = csv::Reader::from_path(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut rows = Vec::new();
    for (line, row) in reader.deserialize::<Row>().enumerate() {
        let row: Row = row.map_err(|e| format!("{} line {}: {e}", path.display(), line + 2))?;
        // A failed query or a scenario that read or wrote other rows makes the run meaningless
        if row.variant().is_none()
            || row.errors > 0
            || row.rows != row.scenarios * ROWS_PER_SCENARIO
            || row.turbo != "off"
        {
            return Err(format!(
                "{} line {}: {} is not a clean run ({} errors, {} rows for {} scenarios, turbo {})",
                path.display(), line + 2, row.variant, row.errors, row.rows, row.scenarios, row.turbo
            ));
        }
        rows.push(row);
    }
    eprintln!("{}: {} rows used", path.display(), rows.len());
    Ok(rows)
}

fn summary(rows: &[Row], variant: Variant, f: fn(&Row) -> f64) -> Option<Summary> {
    let values: Vec<f64> = rows
        .iter()
        .filter(|r| r.variant() == Some(variant) && r.pg_cpus == PG_CPUS)
        .map(f)
        .collect();
    Summary::of(&values)
}

/// A metric of the tables, compared between the two variants
struct Metric {
    label: &'static str,
    f: fn(&Row) -> f64,
    higher_is_better: bool,
    precision: usize,
}

const SUMMARY_METRICS: [Metric; 7] = [
    Metric { label: "Throughput (scenarios/s)", f: |r| r.scenarios_per_s, higher_is_better: true, precision: 1 },
    Metric { label: "Scenario latency, mean (ms)", f: |r| r.scen_mean, higher_is_better: false, precision: 2 },
    Metric { label: "Scenario latency, p50 (ms)", f: |r| r.scen_p50, higher_is_better: false, precision: 2 },
    Metric { label: "Scenario latency, p95 (ms)", f: |r| r.scen_p95, higher_is_better: false, precision: 2 },
    Metric { label: "Scenario latency, p99 (ms)", f: |r| r.scen_p99, higher_is_better: false, precision: 2 },
    Metric { label: "Client CPU time per scenario (ms)", f: |r| r.cpu_ms_per_scenario, higher_is_better: false, precision: 3 },
    Metric { label: "Client peak memory (MiB)", f: |r| r.rss_peak_mib, higher_is_better: false, precision: 1 },
];

/// The steps of the scenario: label, p50, p99
const STEPS: [(&str, fn(&Row) -> f64, fn(&Row) -> f64); 4] = [
    ("User by email", |r| r.q1_p50, |r| r.q1_p99),
    ("Tags by project (100 rows)", |r| r.q2_p50, |r| r.q2_p99),
    ("Insert tag", |r| r.q3_p50, |r| r.q3_p99),
    ("Create project (transaction) + delete", |r| r.q4_p50, |r| r.q4_p99),
];

struct Comparison {
    cornucopia: Summary,
    kysely: Summary,
    /// (Cornucopia - Kysely) / Kysely in %
    diff_percent: f64,
    /// Welch's p, Holm-adjusted over all comparisons of experiment B
    p_holm: f64,
}

impl Comparison {
    fn winner(&self, higher_is_better: bool) -> Option<Variant> {
        if self.p_holm >= ALPHA || self.diff_percent.abs() < MIN_DIFF_PERCENT {
            return None;
        }
        let cornucopia_higher = self.cornucopia.mean > self.kysely.mean;
        Some(if cornucopia_higher == higher_is_better { Variant::Cornucopia } else { Variant::Kysely })
    }
}

/// Every comparison of experiment B (the summary metrics, then the step p50s), Holm-adjusted
/// together, as PLAN.md §10 adjusts within each experiment
fn comparisons(rows: &[Row]) -> Vec<Option<Comparison>> {
    let metrics = SUMMARY_METRICS.iter().map(|m| m.f).chain(STEPS.iter().map(|s| s.1));
    let raw: Vec<Option<(Summary, Summary, f64, f64)>> = metrics
        .map(|f| {
            let cornucopia = summary(rows, Variant::Cornucopia, f)?;
            let kysely = summary(rows, Variant::Kysely, f)?;
            let w = welch(cornucopia, kysely)?;
            Some((cornucopia, kysely, w.diff_percent, w.p))
        })
        .collect();
    let ps: Vec<f64> = raw.iter().flatten().map(|r| r.3).collect();
    let mut adjusted = holm(&ps).into_iter();
    raw.into_iter()
        .map(|r| {
            r.map(|(cornucopia, kysely, diff_percent, _)| Comparison {
                cornucopia,
                kysely,
                diff_percent,
                p_holm: adjusted.next().expect("one adjusted p per test"),
            })
        })
        .collect()
}

fn format_p(p: f64) -> String {
    if p < 0.001 { "$<0.001$".into() } else { format!("{p:.3}") }
}

fn cell(s: Summary, precision: usize, bold: bool) -> String {
    let body = format!("{:.precision$} \\pm {:.precision$}", s.mean, s.sd);
    if bold { format!("$\\mathbf{{{body}}}$") } else { format!("${body}$") }
}

/// Throughput, scenario latency, client CPU and memory: mean ± sd over the runs, Δ and p
pub fn summary_table(rows: &[Row]) -> String {
    let all = comparisons(rows);
    let tested = all.iter().flatten().count();
    let mut tex = format!(
        "% generated by bencmark_results, don't edit\n\
         % mean $\\pm$ sd over runs (Postgres on CPUs {PG_CPUS}); \\Delta = (Cornucopia - Kysely) / Kysely;\n\
         % p: Welch's t-test, Holm-adjusted over the {tested} comparisons of qb_summary and qb_steps;\n\
         % bold: p < {ALPHA} and |\\Delta| >= {MIN_DIFF_PERCENT}%, on the better side\n\
         \\begin{{tabular}}{{lcccc}}\n\\toprule\n\
         Metric & Cornucopia & Kysely & $\\Delta$ (\\%) & $p$ \\\\\n\\midrule\n"
    );
    for (metric, comparison) in SUMMARY_METRICS.iter().zip(&all) {
        let _ = write!(tex, "{}", metric.label);
        match comparison {
            Some(c) => {
                let winner = c.winner(metric.higher_is_better);
                let _ = write!(
                    tex,
                    " & {} & {} & {:+.1} & {}",
                    cell(c.cornucopia, metric.precision, winner == Some(Variant::Cornucopia)),
                    cell(c.kysely, metric.precision, winner == Some(Variant::Kysely)),
                    c.diff_percent,
                    format_p(c.p_holm),
                );
            }
            None => tex.push_str(" & -- & -- & -- & --"),
        }
        tex.push_str(" \\\\\n");
    }
    tex.push_str("\\bottomrule\n\\end{tabular}\n");
    tex
}

/// Latency of each step: p50 and p99 in ms (mean over runs), Δ and p on the p50
pub fn steps_table(rows: &[Row]) -> String {
    let all = comparisons(rows);
    let mut tex = String::from(
        "% generated by bencmark_results, don't edit\n\
         % latency in ms per step, including taking a pool connection; p50 and p99 of each run, mean over runs;\n\
         % \\Delta and p (Holm, see qb_summary) on the p50; bold as in qb_summary\n\
         \\begin{tabular}{lcccccc}\n\\toprule\n\
         & \\multicolumn{2}{c}{Cornucopia} & \\multicolumn{2}{c}{Kysely} & & \\\\\n\
         \\cmidrule(lr){2-3} \\cmidrule(lr){4-5}\n\
         Step & p50 & p99 & p50 & p99 & $\\Delta_{p50}$ (\\%) & $p$ \\\\\n\\midrule\n",
    );
    for (i, (label, _, p99)) in STEPS.iter().enumerate() {
        let _ = write!(tex, "{label}");
        let p99_cornucopia = summary(rows, Variant::Cornucopia, *p99);
        let p99_kysely = summary(rows, Variant::Kysely, *p99);
        match (&all[SUMMARY_METRICS.len() + i], p99_cornucopia, p99_kysely) {
            (Some(c), Some(p99_cornucopia), Some(p99_kysely)) => {
                let winner = c.winner(false);
                let p50 = |s: Summary, bold: bool| {
                    if bold { format!("$\\mathbf{{{:.2}}}$", s.mean) } else { format!("${:.2}$", s.mean) }
                };
                let _ = write!(
                    tex,
                    " & {} & ${:.2}$ & {} & ${:.2}$ & {:+.1} & {}",
                    p50(c.cornucopia, winner == Some(Variant::Cornucopia)),
                    p99_cornucopia.mean,
                    p50(c.kysely, winner == Some(Variant::Kysely)),
                    p99_kysely.mean,
                    c.diff_percent,
                    format_p(c.p_holm),
                );
            }
            _ => tex.push_str(" & -- & -- & -- & -- & -- & --"),
        }
        tex.push_str(" \\\\\n");
    }
    tex.push_str("\\bottomrule\n\\end{tabular}\n");
    tex
}

/// Throughput of every Postgres cpuset in the CSV, with run counts
pub fn print_overview(rows: &[Row]) {
    eprintln!("\nquery layer, scenarios/s mean ± sd (runs):");
    let mut cpusets: Vec<&str> = rows.iter().map(|r| r.pg_cpus.as_str()).collect();
    cpusets.sort_unstable();
    cpusets.dedup();
    for cpus in cpusets {
        let line: Vec<String> = [(Variant::Cornucopia, "Cornucopia"), (Variant::Kysely, "Kysely")]
            .iter()
            .filter_map(|&(variant, label)| {
                let values: Vec<f64> = rows
                    .iter()
                    .filter(|r| r.variant() == Some(variant) && r.pg_cpus == cpus)
                    .map(|r| r.scenarios_per_s)
                    .collect();
                Summary::of(&values).map(|s| format!("{label} {:.1} ± {:.1} ({})", s.mean, s.sd, s.n))
            })
            .collect();
        let used = if cpus == PG_CPUS { "" } else { " (not in the tables)" };
        eprintln!("Postgres CPUs {cpus}{used}: {}", line.join(" | "));
    }
}
