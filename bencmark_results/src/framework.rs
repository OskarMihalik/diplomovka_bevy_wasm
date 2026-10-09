//! Experiment A from framework_bench/PLAN.md: Axum vs Express without a database.
//! Reads framework_load.csv (written by framework_bench's runner) and writes the `fw_*` tables.

use crate::stats::{Summary, holm, welch};
use serde::Deserialize;
use std::fmt::Write as _;
use std::path::Path;

/// In the order they appear in the tables
const ENDPOINTS: &[&str] = &["plaintext", "json", "echo-json", "json-large", "cpu", "delay", "file", "upload"];
const PROFILES: &[&str] = &["P0", "P3", "P5"];

/// A difference counts only if it is significant after Holm and at least this large (PLAN.md §10)
const MIN_DIFF_PERCENT: f64 = 5.;
const ALPHA: f64 = 0.05;

#[derive(Clone, Copy, PartialEq)]
pub enum Framework {
    Axum,
    Express,
}

impl Framework {
    const ALL: [Framework; 2] = [Framework::Axum, Framework::Express];

    fn label(self) -> &'static str {
        match self {
            Framework::Axum => "Axum",
            Framework::Express => "ExpressJS",
        }
    }
}

/// The four configurations, in table order
const CONFIGS: [(Framework, u32); 4] = [
    (Framework::Axum, 1),
    (Framework::Express, 1),
    (Framework::Axum, 4),
    (Framework::Express, 4),
];

#[derive(Deserialize)]
pub struct Row {
    framework: String,
    cores: u32,
    profile: String,
    endpoint: String,
    server_state: String,
    oom_kills: u64,
    client_saturated: bool,
    err_pct: f64,
    measured_rtt_ms: f64,
    pub rps: f64,
    pub lat_p50: f64,
    pub lat_p99: f64,
    pub cpu_s_per_1k_req: f64,
    pub mem_idle_mib: f64,
    pub mem_max_mib: f64,
}

impl Row {
    fn framework(&self) -> Option<Framework> {
        match self.framework.as_str() {
            "axum" => Some(Framework::Axum),
            "express" => Some(Framework::Express),
            _ => None,
        }
    }
}

pub fn read(path: &Path) -> Result<Vec<Row>, String> {
    let mut reader = csv::Reader::from_path(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut rows = Vec::new();
    for (line, row) in reader.deserialize::<Row>().enumerate() {
        let row: Row = row.map_err(|e| format!("{} line {}: {e}", path.display(), line + 2))?;
        // A dead server or a saturated load generator would make the row meaningless; the runner
        // records both, and the final run had neither, so stop rather than silently skip.
        if row.server_state != "running" || row.oom_kills > 0 || row.client_saturated || row.err_pct > 0. {
            return Err(format!(
                "{} line {}: {} {}c {}/{} is not a clean run (state {}, {} OOM kills, saturated {}, {}% errors)",
                path.display(), line + 2, row.framework, row.cores, row.endpoint, row.profile,
                row.server_state, row.oom_kills, row.client_saturated, row.err_pct
            ));
        }
        rows.push(row);
    }
    eprintln!("{}: {} rows used", path.display(), rows.len());
    Ok(rows)
}

fn summary(rows: &[Row], framework: Framework, cores: u32, endpoint: &str, profile: &str, f: fn(&Row) -> f64) -> Option<Summary> {
    let values: Vec<f64> = rows
        .iter()
        .filter(|r| r.framework() == Some(framework) && r.cores == cores && r.endpoint == endpoint && r.profile == profile)
        .map(f)
        .collect();
    Summary::of(&values)
}

fn endpoint_label(endpoint: &str) -> String {
    format!("\\texttt{{/{endpoint}}}")
}

/// 86738.4 -> "86{,}738" (thousands separators inside math mode)
fn grouped(value: f64, precision: usize) -> String {
    let text = format!("{value:.precision$}");
    let (int, frac) = text.split_once('.').map_or((text.as_str(), None), |(i, f)| (i, Some(f)));
    let (sign, digits) = int.strip_prefix('-').map_or(("", int), |d| ("-", d));
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push_str("{,}");
        }
        out.push(c);
    }
    match frac {
        Some(f) => format!("{sign}{out}.{f}"),
        None => format!("{sign}{out}"),
    }
}

/// Requests per second: whole numbers from 1,000 up, one decimal below
fn rps_precision(value: f64) -> usize {
    if value >= 1000. { 0 } else { 1 }
}

/// Milliseconds with three significant-ish digits
fn ms(value: f64) -> String {
    let precision = if value < 10. {
        2
    } else if value < 1000. {
        1
    } else {
        0
    };
    grouped(value, precision)
}

fn rps_cell(s: Summary, bold: bool) -> String {
    let p = rps_precision(s.mean);
    let body = format!("{} \\pm {}", grouped(s.mean, p), grouped(s.sd, p));
    if bold { format!("$\\mathbf{{{body}}}$") } else { format!("${body}$") }
}

fn format_p(p: f64) -> String {
    if p < 0.001 { "$<0.001$".into() } else { format!("{p:.3}") }
}

/// One Axum vs Express comparison of a metric
struct Comparison {
    axum: Summary,
    express: Summary,
    diff_percent: f64,
    /// Welch's p, Holm-adjusted over the table's comparisons
    p_holm: f64,
}

impl Comparison {
    /// The significantly and meaningfully better side, if any
    fn winner(&self, higher_is_better: bool) -> Option<Framework> {
        if self.p_holm >= ALPHA || self.diff_percent.abs() < MIN_DIFF_PERCENT {
            return None;
        }
        let axum_higher = self.axum.mean > self.express.mean;
        Some(if axum_higher == higher_is_better { Framework::Axum } else { Framework::Express })
    }
}

/// Welch's t-test for every (endpoint, profile, cores) that has both frameworks, Holm-adjusted
/// over all of them.
fn compare(rows: &[Row], cells: &[(&'static str, &'static str, u32)], f: fn(&Row) -> f64) -> Vec<Option<Comparison>> {
    let raw: Vec<Option<(Summary, Summary, f64, f64)>> = cells
        .iter()
        .map(|&(endpoint, profile, cores)| {
            let axum = summary(rows, Framework::Axum, cores, endpoint, profile, f)?;
            let express = summary(rows, Framework::Express, cores, endpoint, profile, f)?;
            let w = welch(axum, express)?;
            Some((axum, express, w.diff_percent, w.p))
        })
        .collect();
    let ps: Vec<f64> = raw.iter().flatten().map(|r| r.3).collect();
    let mut adjusted = holm(&ps).into_iter();
    raw.into_iter()
        .map(|r| {
            r.map(|(axum, express, diff_percent, _)| Comparison {
                axum,
                express,
                diff_percent,
                p_holm: adjusted.next().expect("one adjusted p per test"),
            })
        })
        .collect()
}

/// Throughput at P0: Axum vs Express at 1 and 4 cores, Δ = (Axum − Express) / Express,
/// Welch's t-test, Holm over the 13 comparisons. Light endpoints have no 4c cells.
pub fn throughput_table(rows: &[Row]) -> String {
    let mut cells = Vec::new();
    for cores in [1, 4] {
        for &endpoint in ENDPOINTS {
            cells.push((endpoint, "P0", cores));
        }
    }
    let comparisons = compare(rows, &cells, |r| r.rps);
    let tested = comparisons.iter().flatten().count();

    let mut tex = format!(
        "% generated by bencmark_results, don't edit\n\
         % requests/s at P0, mean $\\pm$ sd over runs; \\Delta = (Axum - ExpressJS) / ExpressJS;\n\
         % p: Welch's t-test, Holm-adjusted over the {tested} comparisons; bold: p < {ALPHA} and |\\Delta| >= {MIN_DIFF_PERCENT}%\n\
         \\begin{{tabular}}{{lcccccccc}}\n\\toprule\n\
         & \\multicolumn{{4}}{{c}}{{1 core}} & \\multicolumn{{4}}{{c}}{{4 cores}} \\\\\n\
         \\cmidrule(lr){{2-5}} \\cmidrule(lr){{6-9}}\n\
         Endpoint & Axum & ExpressJS & $\\Delta$ (\\%) & $p$ & Axum & ExpressJS & $\\Delta$ (\\%) & $p$ \\\\\n\\midrule\n"
    );
    for (i, &endpoint) in ENDPOINTS.iter().enumerate() {
        let _ = write!(tex, "{}", endpoint_label(endpoint));
        for comparison in [&comparisons[i], &comparisons[ENDPOINTS.len() + i]] {
            match comparison {
                Some(c) => {
                    let winner = c.winner(true);
                    let _ = write!(
                        tex,
                        " & {} & {} & {:+.1} & {}",
                        rps_cell(c.axum, winner == Some(Framework::Axum)),
                        rps_cell(c.express, winner == Some(Framework::Express)),
                        c.diff_percent,
                        format_p(c.p_holm),
                    );
                }
                None => tex.push_str(" & -- & -- & -- & --"),
            }
        }
        tex.push_str(" \\\\\n");
    }
    tex.push_str("\\bottomrule\n\\end{tabular}\n");
    tex
}

/// Latency at P0: "p50 / p99" in ms, mean over runs, for the four configurations
pub fn latency_table(rows: &[Row]) -> String {
    let mut tex = String::from(
        "% generated by bencmark_results, don't edit\n\
         % latency in ms at P0, p50 / p99 of each run, mean over runs\n\
         \\begin{tabular}{lcccc}\n\\toprule\n\
         & \\multicolumn{2}{c}{1 core} & \\multicolumn{2}{c}{4 cores} \\\\\n\
         \\cmidrule(lr){2-3} \\cmidrule(lr){4-5}\n\
         Endpoint & Axum & ExpressJS & Axum & ExpressJS \\\\\n\\midrule\n",
    );
    for &endpoint in ENDPOINTS {
        let _ = write!(tex, "{}", endpoint_label(endpoint));
        for (framework, cores) in CONFIGS {
            let p50 = summary(rows, framework, cores, endpoint, "P0", |r| r.lat_p50);
            let p99 = summary(rows, framework, cores, endpoint, "P0", |r| r.lat_p99);
            match p50.zip(p99) {
                Some((p50, p99)) => {
                    let _ = write!(tex, " & ${} \\,/\\, {}$", ms(p50.mean), ms(p99.mean));
                }
                None => tex.push_str(" & --"),
            }
        }
        tex.push_str(" \\\\\n");
    }
    tex.push_str("\\bottomrule\n\\end{tabular}\n");
    tex
}

/// Server CPU time per 1,000 requests and peak working set at P0, plus the idle working set
pub fn resources_table(rows: &[Row]) -> String {
    let mut tex = String::from(
        "% generated by bencmark_results, don't edit\n\
         % CPU-s per 1,000 requests and peak working set (MiB) in the measurement window at P0, mean over runs;\n\
         % idle: working set 5 s after start, before any load\n\
         \\begin{tabular}{lcccccccc}\n\\toprule\n\
         & \\multicolumn{4}{c}{CPU time per 1{,}000 requests (s)} & \\multicolumn{4}{c}{Peak memory (MiB)} \\\\\n\
         \\cmidrule(lr){2-5} \\cmidrule(lr){6-9}\n\
         & \\multicolumn{2}{c}{1 core} & \\multicolumn{2}{c}{4 cores} & \\multicolumn{2}{c}{1 core} & \\multicolumn{2}{c}{4 cores} \\\\\n\
         \\cmidrule(lr){2-3} \\cmidrule(lr){4-5} \\cmidrule(lr){6-7} \\cmidrule(lr){8-9}\n\
         Endpoint & Axum & ExpressJS & Axum & ExpressJS & Axum & ExpressJS & Axum & ExpressJS \\\\\n\\midrule\n",
    );
    for &endpoint in ENDPOINTS {
        let _ = write!(tex, "{}", endpoint_label(endpoint));
        for (framework, cores) in CONFIGS {
            let cell = summary(rows, framework, cores, endpoint, "P0", |r| r.cpu_s_per_1k_req)
                .map_or_else(|| "--".into(), |s| cpu_seconds(s.mean));
            let _ = write!(tex, " & {cell}");
        }
        for (framework, cores) in CONFIGS {
            let cell = summary(rows, framework, cores, endpoint, "P0", |r| r.mem_max_mib)
                .map_or_else(|| "--".into(), |s| format!("${}$", grouped(s.mean, if s.mean < 100. { 1 } else { 0 })));
            let _ = write!(tex, " & {cell}");
        }
        tex.push_str(" \\\\\n");
    }
    // The idle working set doesn't depend on the endpoint, so it is averaged over all P0 cells
    tex.push_str("\\midrule\nIdle & & & &");
    for (framework, cores) in CONFIGS {
        let values: Vec<f64> = rows
            .iter()
            .filter(|r| r.framework() == Some(framework) && r.cores == cores)
            .map(|r| r.mem_idle_mib)
            .collect();
        let cell = Summary::of(&values).map_or_else(|| "--".into(), |s| format!("${:.1}$", s.mean));
        let _ = write!(tex, " & {cell}");
    }
    tex.push_str(" \\\\\n\\bottomrule\n\\end{tabular}\n");
    tex
}

fn cpu_seconds(value: f64) -> String {
    let precision = if value < 0.1 {
        4
    } else if value < 10. {
        3
    } else {
        1
    };
    format!("${value:.precision$}$")
}

/// Speedup of throughput from 1 to 4 cores, on the endpoints where the server is the bottleneck
/// at P0 (at /delay the 256 users waiting 20 ms cap the rate, so its speedup says nothing).
/// The requests/s behind it are in the throughput table.
pub fn scaling_table(rows: &[Row]) -> String {
    let endpoints = ["json-large", "cpu", "file", "upload"];
    let mut tex = String::from(
        "% generated by bencmark_results, don't edit\n\
         % requests/s at P0, mean over runs; speedup = 4 cores / 1 core, efficiency (speedup / 4) in parentheses\n\
         \\begin{tabular}{lcc}\n\\toprule\n\
         Endpoint & Axum & ExpressJS \\\\\n\\midrule\n",
    );
    for endpoint in endpoints {
        let _ = write!(tex, "{}", endpoint_label(endpoint));
        for framework in Framework::ALL {
            let one = summary(rows, framework, 1, endpoint, "P0", |r| r.rps);
            let four = summary(rows, framework, 4, endpoint, "P0", |r| r.rps);
            match one.zip(four) {
                Some((one, four)) => {
                    let speedup = four.mean / one.mean;
                    let _ = write!(tex, " & ${speedup:.2}\\times$ ({:.0}\\%)", speedup / 4. * 100.);
                }
                None => tex.push_str(" & --"),
            }
        }
        tex.push_str(" \\\\\n");
    }
    tex.push_str("\\bottomrule\n\\end{tabular}\n");
    tex
}

/// The network profiles: measured RTT, p99 latency and CPU time per 1,000 requests
pub fn network_table(rows: &[Row]) -> String {
    let cells: &[(&str, &str)] = &[
        ("json", "P0"),
        ("json", "P3"),
        ("json", "P5"),
        ("delay", "P0"),
        ("delay", "P3"),
        ("delay", "P5"),
        ("file", "P0"),
        ("file", "P3"),
        ("json-large", "P0"),
        ("json-large", "P5"),
    ];
    let mut tex = String::from(
        "% generated by bencmark_results, don't edit\n\
         % RTT: measured by ping before each run; p99 latency in ms and CPU-s per 1,000 requests, mean over runs.\n\
         % /json at P0 ran at 1 core only (at 4 cores the load generator saturates).\n\
         \\begin{tabular}{llrcccccccc}\n\\toprule\n\
         & & & \\multicolumn{4}{c}{p99 latency (ms)} & \\multicolumn{4}{c}{CPU time per 1{,}000 requests (s)} \\\\\n\
         \\cmidrule(lr){4-7} \\cmidrule(lr){8-11}\n\
         & & & \\multicolumn{2}{c}{1 core} & \\multicolumn{2}{c}{4 cores} & \\multicolumn{2}{c}{1 core} & \\multicolumn{2}{c}{4 cores} \\\\\n\
         \\cmidrule(lr){4-5} \\cmidrule(lr){6-7} \\cmidrule(lr){8-9} \\cmidrule(lr){10-11}\n\
         Endpoint & Profile & RTT (ms) & Axum & ExpressJS & Axum & ExpressJS & Axum & ExpressJS & Axum & ExpressJS \\\\\n\\midrule\n",
    );
    let mut previous = "";
    for &(endpoint, profile) in cells {
        if !previous.is_empty() && previous != endpoint {
            tex.push_str("\\addlinespace\n");
        }
        let label = if previous == endpoint { String::new() } else { endpoint_label(endpoint) };
        previous = endpoint;
        let rtt: Vec<f64> = rows
            .iter()
            .filter(|r| r.endpoint == endpoint && r.profile == profile)
            .map(|r| r.measured_rtt_ms)
            .collect();
        let rtt = Summary::of(&rtt).map_or_else(|| "--".into(), |s| format!("{:.2}", s.mean));
        let _ = write!(tex, "{label} & {profile} & {rtt}");
        for (framework, cores) in CONFIGS {
            let cell = summary(rows, framework, cores, endpoint, profile, |r| r.lat_p99)
                .map_or_else(|| "--".into(), |s| format!("${}$", ms(s.mean)));
            let _ = write!(tex, " & {cell}");
        }
        for (framework, cores) in CONFIGS {
            let cell = summary(rows, framework, cores, endpoint, profile, |r| r.cpu_s_per_1k_req)
                .map_or_else(|| "--".into(), |s| cpu_seconds(s.mean));
            let _ = write!(tex, " & {cell}");
        }
        tex.push_str(" \\\\\n");
    }
    tex.push_str("\\bottomrule\n\\end{tabular}\n");
    tex
}

/// Every cell with its run count, so a missing repetition is noticed
pub fn print_overview(rows: &[Row]) {
    eprintln!("\nframework benchmark, requests/s mean ± sd (runs):");
    for &profile in PROFILES {
        for &endpoint in ENDPOINTS {
            let cells: Vec<String> = CONFIGS
                .iter()
                .filter_map(|&(framework, cores)| {
                    summary(rows, framework, cores, endpoint, profile, |r| r.rps).map(|s| {
                        format!("{} {cores}c {:.1} ± {:.1} ({})", framework.label(), s.mean, s.sd, s.n)
                    })
                })
                .collect();
            if !cells.is_empty() {
                eprintln!("{profile} {endpoint:<11} {}", cells.join(" | "));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouped_numbers() {
        assert_eq!(grouped(86738.4, 0), "86{,}738");
        assert_eq!(grouped(1190.51, 1), "1{,}190.5");
        assert_eq!(grouped(513.0, 1), "513.0");
        assert_eq!(grouped(-1234567.0, 0), "-1{,}234{,}567");
    }
}
