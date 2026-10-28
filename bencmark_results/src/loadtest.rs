//! Full-stack load test (`loadtest/`): Axum vs Express with the database, repeated runs.
//! Reads the Goose HTML reports `{backend}Report{scenario}-r{n}.html` and the container stats
//! `container_stats{scenario}-r{n}.csv` of every run and writes the summary table
//! (`tables/lt_summary.tex`) and the data of the two charts of run 1 (`data/lt_*.csv`).

use crate::stats::{Summary, holm, welch};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// Runs per scenario and backend, `LOADTEST_RUN=1..=RUNS`
pub const RUNS: u32 = 5;
/// The run whose charts and resource graphs are shown in the paper
pub const CHART_RUN: u32 = 1;

const MIN_DIFF_PERCENT: f64 = 5.;
const ALPHA: f64 = 0.05;

pub struct Scenario {
    /// Suffix of the file names, before `-r<n>`
    pub suffix: &'static str,
    label: &'static str,
    /// The request the scenario repeats
    request: &'static str,
    request_label: &'static str,
    /// The scenario sends no other requests, so the row of the request repeats the total
    only_request: bool,
}

pub const SCENARIOS: [Scenario; 2] = [
    Scenario {
        suffix: "",
        label: "Tag creation (1{,}000 users)",
        request: "/tags",
        request_label: "Tag submission",
        only_request: false,
    },
    Scenario {
        suffix: "-login",
        label: "Login (100 users)",
        request: "/login",
        request_label: "Login",
        only_request: true,
    },
];

/// File prefix of the reports and container name in the stats, in table order
pub const BACKENDS: [&str; 2] = ["axum", "express"];

/// The resource graphs drawn by the load test itself, without the scenario suffix
pub const GRAPHS: [&str; 2] = ["cpu_usage", "mem_usage"];

pub fn report_path(dir: &Path, backend: &str, scenario: &Scenario, run: u32) -> PathBuf {
    dir.join(format!("{backend}Report{}-r{run}.html", scenario.suffix))
}

pub fn stats_path(dir: &Path, scenario: &Scenario, run: u32) -> PathBuf {
    dir.join(format!("container_stats{}-r{run}.csv", scenario.suffix))
}

pub fn graph_path(dir: &Path, graph: &str, scenario: &Scenario, run: u32) -> PathBuf {
    dir.join(format!("{graph}{}-r{run}.png", scenario.suffix))
}

/// A row of a Goose table, the cells as Goose printed them
struct TableRow {
    /// A status code row (`200 (98.1%)`) under the request it belongs to
    breakdown: bool,
    cells: Vec<String>,
}

/// A line of a chart: seconds since the Unix epoch and the value
pub struct Series {
    name: String,
    points: Vec<(i64, f64)>,
}

pub struct Report {
    /// Method, name, # requests, # fails, average, min, max (ms), RPS, failures/s
    requests: Vec<TableRow>,
    /// Method, name, 50, 60, 70, 80, 90, 95, 99, 100 %ile (ms)
    percentiles: Vec<TableRow>,
    /// Requests per second over time
    pub rps: Vec<Series>,
    /// Average response time in ms over time
    pub response: Vec<Series>,
}

pub fn read(path: &Path) -> Result<Report, String> {
    let html = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let error = |what: &str| format!("{}: {what}", path.display());
    Ok(Report {
        requests: table(&html, "Request Metrics", 9).map_err(|e| error(&e))?,
        percentiles: table(&html, "Response Time Metrics", 10).map_err(|e| error(&e))?,
        rps: chart(&html, "graph-rps").map_err(|e| error(&e))?,
        response: chart(&html, "graph-avg-response-time").map_err(|e| error(&e))?,
    })
}

/// The table under the heading `<h2>{heading}</h2>`, whose first two columns are the method and
/// the name
fn table(html: &str, heading: &str, columns: usize) -> Result<Vec<TableRow>, String> {
    let section = between(html, &format!("<h2>{heading}</h2>"), "</table>")
        .and_then(|section| section.split_once("<tbody>"))
        .ok_or_else(|| format!("no {heading} table"))?
        .1;

    let mut rows = Vec::new();
    for row in section.split("<tr").skip(1) {
        let (tag, body) = row.split_once('>').ok_or("unclosed <tr")?;
        let mut cells = Vec::new();
        for cell in body.split("<td").skip(1) {
            let text = between(cell, ">", "</td>").ok_or("unclosed <td")?;
            cells.push(text.replace("&nbsp;", "").replace("&amp;", "&"));
        }
        let breakdown = tag.contains("status-breakdown");
        if breakdown {
            // the first cell spans the method and the name: `└─ 200 (98.1%)`
            let status = cells.first().ok_or("empty status row")?;
            cells[0] = status.trim_start_matches(['└', '─', ' ']).to_string();
            cells.insert(1, String::new());
        }
        if cells.len() != columns {
            return Err(format!("{heading} row with {} cells: {cells:?}", cells.len()));
        }
        rows.push(TableRow { breakdown, cells });
    }
    if rows.is_empty() {
        return Err(format!("empty {heading} table"));
    }
    Ok(rows)
}

/// The cells of the row of the request `name` (`/tags`, or `Aggregated` for the total)
fn row<'a>(rows: &'a [TableRow], name: &str) -> Result<&'a [String], String> {
    rows.iter()
        .find(|r| !r.breakdown && r.cells[1] == name)
        .map(|r| r.cells.as_slice())
        .ok_or_else(|| format!("no row {name}"))
}

/// `1,000` -> 1000
fn number(cell: &str) -> Result<f64, String> {
    cell.replace(',', "")
        .parse()
        .map_err(|e| format!("number {cell:?}: {e}"))
}

/// The series of the ECharts graph drawn into the element `id`
fn chart(html: &str, id: &str) -> Result<Vec<Series>, String> {
    let script = between(html, &format!("getElementById('{id}')"), "</script>")
        .ok_or_else(|| format!("no chart {id}"))?;

    let mut series = Vec::new();
    // every series starts with its name, the axis title before them has no data
    for part in script.split("name: '").skip(1) {
        let (name, rest) = part.split_once('\'').ok_or("unclosed series name")?;
        let Some(data) = between(rest, "data: [[", "]]") else {
            continue;
        };
        let mut points = Vec::new();
        for point in data.split("],[") {
            let (time, value) = point
                .split_once(',')
                .ok_or_else(|| format!("chart point {point:?}"))?;
            let value = value
                .parse()
                .map_err(|e| format!("chart value {value:?}: {e}"))?;
            points.push((timestamp(time.trim_matches('"'))?, value));
        }
        series.push(Series {
            name: name.to_string(),
            points,
        });
    }
    if series.is_empty() {
        return Err(format!("chart {id} has no series"));
    }
    Ok(series)
}

fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let text = text.split_once(start)?.1;
    Some(text.split_once(end)?.0)
}

/// `2026-10-13 16:21:37` in seconds since the Unix epoch, as if it was UTC: only differences
/// are used
fn timestamp(text: &str) -> Result<i64, String> {
    let numbers: Vec<i64> = text
        .split(['-', ' ', ':'])
        .map(|part| part.parse().map_err(|e| format!("timestamp {text:?}: {e}")))
        .collect::<Result<_, _>>()?;
    let [year, month, day, hour, minute, second] = numbers[..] else {
        return Err(format!("timestamp {text:?}"));
    };
    // days since 1970-01-01 of a Gregorian date, the year starts in March to end on the leap day
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    Ok(((days * 24 + hour) * 60 + minute) * 60 + second)
}

/// What the summary table uses of one run of one backend
#[derive(Clone, Copy)]
pub struct RunResult {
    total_rps: f64,
    fails: f64,
    request_rps: f64,
    request_mean_ms: f64,
    request_p50_ms: f64,
    request_p99_ms: f64,
    request_max_ms: f64,
    /// Median CPU of the backend container while it was attacked, 100 = one logical CPU
    backend_cpu: f64,
    /// Median CPU of the PostgreSQL container while the backend was attacked
    postgres_cpu: f64,
}

/// The run of `backend` from its report and the container stats of the same run
pub fn run_result(
    report: &Report,
    stats: &str,
    scenario: &Scenario,
    backend: &str,
) -> Result<RunResult, String> {
    let total = row(&report.requests, "Aggregated")?;
    let request = row(&report.requests, scenario.request)?;
    let percentiles = row(&report.percentiles, scenario.request)?;
    Ok(RunResult {
        total_rps: number(&total[7])?,
        fails: number(&total[3])?,
        request_rps: number(&request[7])?,
        request_mean_ms: number(&request[4])?,
        request_p50_ms: number(&percentiles[2])?,
        request_p99_ms: number(&percentiles[8])?,
        request_max_ms: number(&request[6])?,
        backend_cpu: median_cpu(stats, backend, backend)?,
        postgres_cpu: median_cpu(stats, "postgres", backend)?,
    })
}

/// Median CPU of `container` over the samples taken while `target` was attacked, from the
/// `time_s,container,target,cpu_percent,mem_percent` CSV of the load test
fn median_cpu(stats: &str, container: &str, target: &str) -> Result<f64, String> {
    let mut cpu = Vec::new();
    for line in stats.lines().skip(1) {
        let cells: Vec<&str> = line.split(',').collect();
        let [_, c, t, value, _] = cells[..] else {
            return Err(format!("container stats line {line:?}"));
        };
        if c == container && t == target {
            cpu.push(number(value)?);
        }
    }
    if cpu.is_empty() {
        return Err(format!("no CPU samples of {container} during the {target} attack"));
    }
    cpu.sort_by(f64::total_cmp);
    let mid = cpu.len() / 2;
    Ok(if cpu.len() % 2 == 1 { cpu[mid] } else { (cpu[mid - 1] + cpu[mid]) / 2. })
}

struct Metric {
    label: fn(&Scenario) -> String,
    /// Throughput of the repeated request, left out where it is the total
    per_request: bool,
    f: fn(&RunResult) -> f64,
    higher_is_better: bool,
    precision: usize,
    /// Welch's t-test in the table; the percentiles are bucketed by Goose, the CPU is sampled
    tested: bool,
}

const METRICS: [Metric; 8] = [
    Metric { label: |_| "Throughput, all requests (req/s)".into(), per_request: false, f: |r| r.total_rps, higher_is_better: true, precision: 1, tested: true },
    Metric { label: |s| format!("{}s (req/s)", s.request_label), per_request: true, f: |r| r.request_rps, higher_is_better: true, precision: 1, tested: true },
    Metric { label: |s| format!("{} latency, mean (ms)", s.request_label), per_request: false, f: |r| r.request_mean_ms, higher_is_better: false, precision: 1, tested: true },
    Metric { label: |s| format!("{} latency, p50 (ms)", s.request_label), per_request: false, f: |r| r.request_p50_ms, higher_is_better: false, precision: 0, tested: false },
    Metric { label: |s| format!("{} latency, p99 (ms)", s.request_label), per_request: false, f: |r| r.request_p99_ms, higher_is_better: false, precision: 0, tested: false },
    Metric { label: |s| format!("{} latency, max (ms)", s.request_label), per_request: false, f: |r| r.request_max_ms, higher_is_better: false, precision: 0, tested: false },
    Metric { label: |_| "Backend CPU, median (\\%)".into(), per_request: false, f: |r| r.backend_cpu, higher_is_better: false, precision: 0, tested: false },
    Metric { label: |_| "PostgreSQL CPU, median (\\%)".into(), per_request: false, f: |r| r.postgres_cpu, higher_is_better: false, precision: 0, tested: false },
];

/// The rows of the scenario
fn metrics(scenario: &Scenario) -> impl Iterator<Item = &'static Metric> {
    METRICS.iter().filter(|m| !(m.per_request && scenario.only_request))
}

/// The runs of both backends of one scenario, in the order of `BACKENDS`
pub type ScenarioRuns = [Vec<RunResult>; 2];

struct Comparison {
    axum: Summary,
    express: Summary,
    /// (Axum - Express) / Express in %
    diff_percent: f64,
    /// Welch's p, Holm-adjusted over all tested comparisons of the table
    p_holm: Option<f64>,
}

/// Every metric of every scenario, the tested ones Holm-adjusted together
fn comparisons(runs: &[ScenarioRuns]) -> Vec<Vec<Option<Comparison>>> {
    let mut ps = Vec::new();
    let mut raw = Vec::new();
    for (scenario, [axum, express]) in SCENARIOS.iter().zip(runs) {
        let mut comparisons = Vec::new();
        for metric in metrics(scenario) {
            let values = |runs: &[RunResult]| runs.iter().map(metric.f).collect::<Vec<_>>();
            let (Some(a), Some(e)) = (Summary::of(&values(axum)), Summary::of(&values(express)))
            else {
                comparisons.push(None);
                continue;
            };
            let w = if metric.tested { welch(a, e) } else { None };
            if let Some(w) = &w {
                ps.push(w.p);
            }
            comparisons.push(Some((a, e, (a.mean - e.mean) / e.mean * 100., w.is_some())));
        }
        raw.push(comparisons);
    }
    let mut adjusted = holm(&ps).into_iter();
    raw.into_iter()
        .map(|scenario| {
            scenario
                .into_iter()
                .map(|c| {
                    c.map(|(axum, express, diff_percent, tested)| Comparison {
                        axum,
                        express,
                        diff_percent,
                        p_holm: tested.then(|| adjusted.next().expect("one adjusted p per test")),
                    })
                })
                .collect()
        })
        .collect()
}

/// 2808.6 -> "2{,}808.6" (thousands separators inside math mode)
fn grouped(value: f64, precision: usize) -> String {
    let text = format!("{value:.precision$}");
    let (int, frac) = text.split_once('.').map_or((text.as_str(), None), |(i, f)| (i, Some(f)));
    let mut out = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push_str("{,}");
        }
        out.push(c);
    }
    match frac {
        Some(f) => format!("{out}.{f}"),
        None => out,
    }
}

fn cell(s: Summary, precision: usize, bold: bool) -> String {
    let body = format!("{} \\pm {}", grouped(s.mean, precision), grouped(s.sd, precision));
    if bold { format!("$\\mathbf{{{body}}}$") } else { format!("${body}$") }
}

fn format_p(p: f64) -> String {
    if p < 0.001 { "$<0.001$".into() } else { format!("{p:.3}") }
}

/// Throughput, latency and CPU of both scenarios: mean ± sd over the runs, Δ and p
pub fn summary_table(runs: &[ScenarioRuns]) -> String {
    let all = comparisons(runs);
    let tested = all.iter().flatten().flatten().filter(|c| c.p_holm.is_some()).count();
    let mut tex = format!(
        "% generated by bencmark_results from the reports in loadtest/, don't edit\n\
         % mean $\\pm$ sd over runs; \\Delta = (Axum - ExpressJS) / ExpressJS;\n\
         % p: Welch's t-test, Holm-adjusted over the {tested} tested comparisons;\n\
         % bold: p < {ALPHA} and |\\Delta| >= {MIN_DIFF_PERCENT}%, on the better side\n\
         \\begin{{tabular}}{{lcccc}}\n\\toprule\n\
         Metric & Axum & ExpressJS & $\\Delta$ (\\%) & $p$ \\\\\n"
    );
    for ((scenario, comparisons), [axum, express]) in SCENARIOS.iter().zip(&all).zip(runs) {
        let _ = write!(
            tex,
            "\\midrule\n\\multicolumn{{5}}{{l}}{{\\textit{{{}}}}} \\\\\n",
            scenario.label
        );
        for (metric, comparison) in metrics(scenario).zip(comparisons) {
            let _ = write!(tex, "{}", (metric.label)(scenario));
            match comparison {
                Some(c) => {
                    let significant = c
                        .p_holm
                        .is_some_and(|p| p < ALPHA && c.diff_percent.abs() >= MIN_DIFF_PERCENT);
                    let axum_better = (c.axum.mean > c.express.mean) == metric.higher_is_better;
                    let _ = write!(
                        tex,
                        " & {} & {} & {:+.1} & {}",
                        cell(c.axum, metric.precision, significant && axum_better),
                        cell(c.express, metric.precision, significant && !axum_better),
                        c.diff_percent,
                        c.p_holm.map_or("--".into(), format_p),
                    );
                }
                None => tex.push_str(" & -- & -- & -- & --"),
            }
            tex.push_str(" \\\\\n");
        }
        let fails = |runs: &[RunResult]| runs.iter().map(|r| r.fails).sum::<f64>();
        let _ = writeln!(
            tex,
            "Failed requests, all runs & {} & {} & -- & -- \\\\",
            grouped(fails(axum), 0),
            grouped(fails(express), 0),
        );
    }
    tex.push_str("\\bottomrule\n\\end{tabular}\n");
    tex
}

/// `POST /model/{project_id}/{model_name}` -> `post_model_project_id_model_name`
fn column(name: &str) -> String {
    let words: Vec<_> = name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    words.join("_").to_lowercase()
}

/// One line per second of the attack: `t` in s since the first sample, then a column per series,
/// `nan` where the series has no sample (requests that are sent only at the start or the end)
pub fn chart_data(series: &[Series]) -> String {
    let mut times: Vec<_> = series
        .iter()
        .flat_map(|s| s.points.iter().map(|p| p.0))
        .collect();
    times.sort_unstable();
    times.dedup();

    let mut csv = String::from("t");
    for s in series {
        let _ = write!(csv, ",{}", column(&s.name));
    }
    csv.push('\n');
    for &time in &times {
        let _ = write!(csv, "{}", time - times[0]);
        for s in series {
            match s.points.iter().find(|p| p.0 == time) {
                Some((_, value)) => {
                    let _ = write!(csv, ",{value:.1}");
                }
                None => csv.push_str(",nan"),
            }
        }
        csv.push('\n');
    }
    csv
}

pub fn print_overview(scenario: &Scenario, backend: &str, run: u32, r: &RunResult) {
    eprintln!(
        "{backend:<8}{:<7} r{run}{:>10.1} rps{:>6} fails  {}{:>9.1} rps{:>8.1} ms avg  cpu{:>5.0}%  postgres{:>5.0}%",
        scenario.suffix,
        r.total_rps,
        r.fails,
        scenario.request,
        r.request_rps,
        r.request_mean_ms,
        r.backend_cpu,
        r.postgres_cpu,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_and_timestamps() {
        let html = r#"<h2>Request Metrics</h2><table><thead></thead><tbody>
            <tr><td>POST</td><td>/tags</td><td>47,485</td><td>882</td><td>1367.01</td><td>6</td>
                <td>60,002</td><td>719.47</td><td>13.36</td></tr>
            <tr class="status-breakdown"><td colspan="2">&nbsp;&nbsp;└─ 500 (1.5%)</td><td>730</td>
                <td></td><td>10369.42</td><td>818</td><td>58,136</td><td></td><td></td></tr>
            </tbody></table>
            <h2>Response Time Metrics</h2><table><thead></thead><tbody>
            <tr><td>POST</td><td>/tags</td><td>330</td><td>370</td><td>390</td><td>600</td>
                <td>800</td><td>1,000</td><td>2,000</td><td>4,000</td></tr>
            </tbody></table>"#;
        let requests = table(html, "Request Metrics", 9).unwrap();
        assert_eq!(requests.len(), 2);
        assert!(requests[1].breakdown);
        assert_eq!(requests[1].cells[0], "500 (1.5%)");
        let tags = row(&requests, "/tags").unwrap();
        assert_eq!(number(&tags[2]), Ok(47_485.));
        assert_eq!(number(&tags[7]), Ok(719.47));
        let percentiles = table(html, "Response Time Metrics", 10).unwrap();
        assert_eq!(number(&row(&percentiles, "/tags").unwrap()[8]), Ok(2_000.));

        assert_eq!(timestamp("1970-01-01 00:00:00"), Ok(0));
        assert_eq!(timestamp("2026-10-13 16:21:37"), Ok(1_791_908_497));
        // a run over midnight at the end of February of a leap year
        let before = timestamp("2028-02-29 23:59:59").unwrap();
        assert_eq!(timestamp("2028-03-01 00:00:01").unwrap() - before, 2);
    }

    #[test]
    fn median_cpu_of_the_attacked_backend() {
        let stats = "time_s,container,target,cpu_percent,mem_percent\n\
                     1.0,axum,axum,100.0,0.5\n2.0,axum,axum,300.0,0.5\n3.0,axum,axum,200.0,0.5\n\
                     4.0,axum,express,5.0,0.5\n1.0,postgres,axum,50.0,0.3\n2.0,postgres,axum,70.0,0.3\n";
        assert_eq!(median_cpu(stats, "axum", "axum"), Ok(200.));
        assert_eq!(median_cpu(stats, "postgres", "axum"), Ok(60.));
        assert!(median_cpu(stats, "express", "express").is_err());
    }

    #[test]
    fn thousands_separators() {
        assert_eq!(grouped(2808.64, 1), "2{,}808.6");
        assert_eq!(grouped(999.0, 0), "999");
        assert_eq!(grouped(181128.0, 0), "181{,}128");
    }
}
