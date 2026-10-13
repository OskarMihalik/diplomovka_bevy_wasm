//! Full-stack load test (`loadtest/`): Axum vs Express with the database.
//! Reads the Goose HTML reports and writes, per report, the request metrics table
//! (`tables/lt_*.tex`) and the data of the two charts on top of the report (`data/lt_*.csv`).

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// One Goose report and what its table is called in the paper
pub struct Run {
    /// File in `loadtest/`
    pub file: &'static str,
    /// Used in the names of the written files
    pub key: &'static str,
    caption: &'static str,
    label: &'static str,
}

pub const RUNS: [Run; 4] = [
    Run {
        file: "axumReport.html",
        key: "axum",
        caption: "Axum request metrics tag creation",
        label: "tab:axum-request-metrics",
    },
    Run {
        file: "expressReport.html",
        key: "express",
        caption: "Express request metrics tag creation",
        label: "tab:express-request-metrics",
    },
    Run {
        file: "axumReport-login.html",
        key: "axum_login",
        caption: "Axum request metrics login",
        label: "tab:axum_request_metrics_login",
    },
    Run {
        file: "expressReport-login.html",
        key: "express_login",
        caption: "Express request metrics login",
        label: "tab:express_request_metrics_login",
    },
];

/// The resource graphs drawn by the load test itself, copied next to the tables
pub const GRAPHS: [&str; 4] = [
    "cpu_usage.png",
    "mem_usage.png",
    "cpu_usage-login.png",
    "mem_usage-login.png",
];

/// A row of the "Request Metrics" table, the cells as Goose printed them
struct Request {
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
    requests: Vec<Request>,
    /// Requests per second over time
    pub rps: Vec<Series>,
    /// Average response time in ms over time
    pub response: Vec<Series>,
}

pub fn read(path: &Path) -> Result<Report, String> {
    let html = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let error = |what: &str| format!("{}: {what}", path.display());
    Ok(Report {
        requests: requests(&html).map_err(|e| error(&e))?,
        rps: chart(&html, "graph-rps").map_err(|e| error(&e))?,
        response: chart(&html, "graph-avg-response-time").map_err(|e| error(&e))?,
    })
}

/// Columns of the request metrics table, the first two are the method and the name
const COLUMNS: usize = 9;

fn requests(html: &str) -> Result<Vec<Request>, String> {
    let section = between(html, "<h2>Request Metrics</h2>", "</table>")
        .and_then(|section| section.split_once("<tbody>"))
        .ok_or("no request metrics table")?
        .1;

    let mut requests = Vec::new();
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
        if cells.len() != COLUMNS {
            return Err(format!("request row with {} cells: {cells:?}", cells.len()));
        }
        requests.push(Request { breakdown, cells });
    }
    if requests.is_empty() {
        return Err("empty request metrics table".into());
    }
    Ok(requests)
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

fn tex_escape(text: &str) -> String {
    let mut tex = String::new();
    for c in text.chars() {
        if "&%_#{}".contains(c) {
            tex.push('\\');
        }
        tex.push(c);
    }
    tex
}

/// The request metrics table of the report as a complete `table*`
pub fn request_table(run: &Run, report: &Report) -> String {
    let mut tex = format!(
        "% generated by bencmark_results from loadtest/{}, don't edit\n\
         \\begin{{table*}}[!ht]\n    \\centering\n    \\caption{{{}}}\n    \\label{{{}}}\n    \
         \\begin{{tabular}}{{l l r r r r r r r}}\n        \\toprule\n        \
         \\textbf{{Method}} & \\textbf{{Name}} & \\textbf{{\\# Requests}} & \\textbf{{\\# Fails}} & \
         \\textbf{{Average (ms)}} & \\textbf{{Min (ms)}} & \\textbf{{Max (ms)}} & \\textbf{{RPS}} & \
         \\textbf{{Failures/s}} \\\\\n        \\midrule\n",
        run.file, run.caption, run.label,
    );
    for request in &report.requests {
        let cell = |i: usize| match request.cells[i].as_str() {
            "" => "~".to_string(),
            text => tex_escape(text),
        };
        let head = if request.breakdown {
            format!("\\multicolumn{{2}}{{l}}{{--- {}}}", cell(0))
        } else {
            format!("{} & {}", cell(0), cell(1))
        };
        // the last row sums up the others
        if request.cells[1] == "Aggregated" {
            tex.push_str("        \\midrule\n");
        }
        let rest: Vec<_> = (2..COLUMNS).map(cell).collect();
        let _ = writeln!(tex, "        {head} & {} \\\\", rest.join(" & "));
    }
    tex.push_str("        \\bottomrule\n    \\end{tabular}\n\\end{table*}\n");
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

pub fn print_overview(run: &Run, report: &Report) {
    let Some(total) = report.requests.last() else {
        return;
    };
    eprintln!(
        "{:<26}{:>10} requests{:>8} fails{:>10} ms avg{:>10} rps",
        run.file, total.cells[2], total.cells[3], total.cells[4], total.cells[7],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_rows_and_timestamps() {
        let html = r#"<h2>Request Metrics</h2><table><thead></thead><tbody>
            <tr><td>POST</td><td></td><td>47,485</td><td>882</td><td>1367.01</td><td>6</td>
                <td>60,002</td><td>719.47</td><td>13.36</td></tr>
            <tr class="status-breakdown"><td colspan="2">&nbsp;&nbsp;└─ 500 (1.5%)</td><td>730</td>
                <td></td><td>10369.42</td><td>818</td><td>58,136</td><td></td><td></td></tr>
            </tbody></table>"#;
        let report = Report {
            requests: requests(html).unwrap(),
            rps: Vec::new(),
            response: Vec::new(),
        };
        let tex = request_table(&RUNS[0], &report);
        assert!(tex.contains("POST & ~ & 47,485 & 882 & 1367.01 & 6 & 60,002 & 719.47 & 13.36 \\\\"));
        assert!(tex.contains(
            "\\multicolumn{2}{l}{--- 500 (1.5\\%)} & 730 & ~ & 10369.42 & 818 & 58,136 & ~ & ~ \\\\"
        ));

        assert_eq!(timestamp("1970-01-01 00:00:00"), Ok(0));
        assert_eq!(timestamp("2026-10-13 16:21:37"), Ok(1_791_908_497));
        // a run over midnight at the end of February of a leap year
        let before = timestamp("2028-02-29 23:59:59").unwrap();
        assert_eq!(timestamp("2028-03-01 00:00:01").unwrap() - before, 2);
    }
}
