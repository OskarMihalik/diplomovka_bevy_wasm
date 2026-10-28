//! Turns the benchmark CSVs into what the paper uses:
//! - `out/tables/*.tex`  LaTeX tables (booktabs), `\input{}` them
//! - `out/data/*.csv`    data for the pgfplots figures in `paper/figures`
//!
//! `cargo run -p bencmark_results`, rerun after adding benchmark rows.
//! Reads rendering_fps.csv, rendering_load.csv, the app's fps.csv and load.csv (see app.rs),
//! framework_load.csv (see framework.rs),
//! query_layer.csv (see query.rs) and the
//! Goose reports in `../loadtest` (see loadtest.rs), whose resource graphs go to `out/figures`.

mod app;
mod data;
mod framework;
mod loadtest;
mod query;
mod stats;

use data::{MODELS, Measurement, Platform, Row};
use stats::{Summary, welch};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fps = data::read_fps(&root.join("rendering_fps.csv"))?;
    let load = data::read_load(&root.join("rendering_load.csv"))?;

    let out = Output::new(root.join("out"))?;
    out.write("tables/models.tex", &models_table(&fps))?;
    out.write("tables/fps.tex", &fps_table(&fps, Platform::COMPARED))?;
    out.write(
        "tables/ttest.tex",
        &ttest_table(
            &fps,
            [
                (Platform::BevyWasm, Platform::BabylonWasm),
                (Platform::BevyNative, Platform::BevyWasm),
            ],
        ),
    )?;
    // the same on the dedicated GPU (rendering_runner/matrices/nvidia.csv), once it ran
    if fps.iter().any(|m| Platform::RTX.contains(&m.platform)) {
        out.write("tables/fps_rtx.tex", &fps_table(&fps, Platform::RTX))?;
        out.write(
            "tables/ttest_rtx.tex",
            &ttest_table(
                &fps,
                [
                    (Platform::BevyWasmRtx, Platform::BabylonWasmRtx),
                    (Platform::BevyNativeRtx, Platform::BevyWasmRtx),
                ],
            ),
        )?;
    }
    // the older notebook (rendering_runner/matrices/main.csv with --pc asus), once it ran
    if fps.iter().any(|m| Platform::ASUS.contains(&m.platform)) {
        out.write("tables/fps_asus.tex", &fps_table(&fps, Platform::ASUS))?;
        out.write(
            "tables/ttest_asus.tex",
            &ttest_table(
                &fps,
                [
                    (Platform::BevyWasmAsus, Platform::BabylonWasmAsus),
                    (Platform::BevyNativeAsus, Platform::BevyWasmAsus),
                ],
            ),
        )?;
        out.write("tables/load_asus.tex", &load_table(&load, Platform::ASUS))?;
    }
    out.write("tables/load.tex", &load_table(&load, Platform::COMPARED))?;
    out.write("data/fps.csv", &fps_data(&fps))?;
    out.write("data/frametimes.csv", &frametime_data(&fps))?;
    for platform in Platform::ALL {
        if let Some(csv) = scalability_data(&fps, platform) {
            out.write(&format!("data/scalability_{}.csv", platform.key()), &csv)?;
        }
        if let Some(csv) = load_data(&load, platform) {
            out.write(&format!("data/load_{}.csv", platform.key()), &csv)?;
        }
    }
    print_overview(&fps);

    // the whole app in bench mode (rendering_runner, matrices/app.csv), skipped until it ran
    let app_fps_csv = root.join("fps.csv");
    let app_load_csv = root.join("load.csv");
    if app_fps_csv.exists() && app_load_csv.exists() {
        let app_fps = app::read(&app_fps_csv, true)?;
        let app_load = app::read(&app_load_csv, false)?;
        out.write("tables/app.tex", &app::table(&app_fps, &app_load, &fps))?;
    }

    // Experiment A of framework_bench (Axum vs Express), skipped until the CSV exists
    let framework_csv = root.join("framework_load.csv");
    if framework_csv.exists() {
        let rows = framework::read(&framework_csv)?;
        out.write(
            "tables/fw_throughput.tex",
            &framework::throughput_table(&rows),
        )?;
        out.write("tables/fw_latency.tex", &framework::latency_table(&rows))?;
        out.write(
            "tables/fw_resources.tex",
            &framework::resources_table(&rows),
        )?;
        out.write("tables/fw_scaling.tex", &framework::scaling_table(&rows))?;
        out.write("tables/fw_network.tex", &framework::network_table(&rows))?;
        framework::print_overview(&rows);
    }

    // Experiment B of framework_bench (Cornucopia vs Kysely), skipped until the CSV exists
    let query_csv = root.join("query_layer.csv");
    if query_csv.exists() {
        let rows = query::read(&query_csv)?;
        out.write("tables/qb_summary.tex", &query::summary_table(&rows))?;
        out.write("tables/qb_steps.tex", &query::steps_table(&rows))?;
        query::print_overview(&rows);
    }

    // Full-stack load test (Axum vs Express with the database), from the Goose reports of the
    // repeated runs
    let loadtest_dir = root.join("../loadtest");
    let first = loadtest::report_path(&loadtest_dir, "axum", &loadtest::SCENARIOS[0], 1);
    if first.exists() {
        eprintln!("\nload test, aggregated:");
        let mut all_runs = Vec::new();
        for scenario in &loadtest::SCENARIOS {
            let mut runs: loadtest::ScenarioRuns = Default::default();
            for run in 1..=loadtest::RUNS {
                let stats_path = loadtest::stats_path(&loadtest_dir, scenario, run);
                let stats = fs::read_to_string(&stats_path)
                    .map_err(|e| format!("{}: {e}", stats_path.display()))?;
                for (backend, results) in loadtest::BACKENDS.iter().zip(&mut runs) {
                    let path = loadtest::report_path(&loadtest_dir, backend, scenario, run);
                    let report = loadtest::read(&path)?;
                    let result = loadtest::run_result(&report, &stats, scenario, backend)
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    loadtest::print_overview(scenario, backend, run, &result);
                    results.push(result);
                    // the charts of the paper show the tag creation of one run
                    if scenario.suffix.is_empty() && run == loadtest::CHART_RUN {
                        out.write(
                            &format!("data/lt_{backend}_rps.csv"),
                            &loadtest::chart_data(&report.rps),
                        )?;
                        out.write(
                            &format!("data/lt_{backend}_response.csv"),
                            &loadtest::chart_data(&report.response),
                        )?;
                    }
                }
            }
            for graph in loadtest::GRAPHS {
                let path = loadtest::graph_path(&loadtest_dir, graph, scenario, loadtest::CHART_RUN);
                out.copy(&path, &format!("figures/{graph}{}.png", scenario.suffix))?;
            }
            all_runs.push(runs);
        }
        out.write("tables/lt_summary.tex", &loadtest::summary_table(&all_runs))?;
    }
    Ok(())
}

struct Output {
    dir: PathBuf,
}

impl Output {
    fn new(dir: PathBuf) -> Result<Self, String> {
        for sub in ["tables", "data", "figures"] {
            fs::create_dir_all(dir.join(sub)).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        Ok(Self { dir })
    }

    fn write(&self, path: &str, content: &str) -> Result<(), String> {
        let path = self.dir.join(path);
        fs::write(&path, content).map_err(|e| format!("{}: {e}", path.display()))?;
        eprintln!("wrote {}", path.display());
        Ok(())
    }

    fn copy(&self, from: &Path, path: &str) -> Result<(), String> {
        let path = self.dir.join(path);
        fs::copy(from, &path).map_err(|e| format!("{}: {e}", path.display()))?;
        eprintln!("wrote {}", path.display());
        Ok(())
    }
}

/// Values of one metric over the runs of a model on a platform
fn values(rows: &[Measurement], model: usize, platform: Platform, f: fn(&Row) -> f64) -> Vec<f64> {
    rows.iter()
        .filter(|m| m.model == model && m.platform == platform)
        .map(|m| f(&m.row))
        .collect()
}

fn summary(
    rows: &[Measurement],
    model: usize,
    platform: Platform,
    f: fn(&Row) -> f64,
) -> Option<Summary> {
    Summary::of(&values(rows, model, platform, f))
}

/// What the scene contains, from Bevy's measurement: native first, it is the most complete
fn model_metric(rows: &[Measurement], model: usize, f: fn(&Row) -> f64) -> Option<f64> {
    [
        Platform::BevyNative,
        // Platform::BevyNativeRtx,
        Platform::BevyWasm,
        Platform::BabylonWasm,
    ]
    .into_iter()
    .find_map(|platform| summary(rows, model, platform, f))
    .map(|s| s.mean)
}

/// Cell for a missing measurement: OOM where Bevy can't load the model in the browser, or natively
/// on the asus, whose Vulkan driver runs out of GPU memory
fn missing(model: usize, platform: Platform) -> &'static str {
    let model = &MODELS[model];
    if (model.wasm_oom && platform.on_integrated() == Platform::BevyWasm)
        || (model.asus_native_oom && platform == Platform::BevyNativeAsus)
    {
        "OOM"
    } else {
        "--"
    }
}

pub(crate) fn tex_escape(text: &str) -> String {
    text.replace('&', "\\&")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

pub(crate) fn mean_sd(s: Summary, precision: usize) -> String {
    format!("${:.precision$} \\pm {:.precision$}$", s.mean, s.sd)
}

fn models_table(fps: &[Measurement]) -> String {
    let mut tex = String::from(
        "% generated by bencmark_results, don't edit\n\
         \\begin{tabular}{lrrrrrrl}\n\\toprule\n\
         Model & Triangles & Vertices & Meshes & Materials & Textures & Camera & Wasm \\\\\n\
         & (M) & (M) & & & (MiB) & distance (units) & \\\\\n\\midrule\n",
    );
    for (i, model) in MODELS.iter().enumerate() {
        let metric = |f: fn(&Row) -> f64, scale: f64, precision: usize| {
            model_metric(fps, i, f)
                .map(|v| format!("{:.precision$}", v / scale))
                .unwrap_or_else(|| "--".into())
        };
        let _ = writeln!(
            tex,
            "{} & {} & {} & {} & {} & {} & {} & {} \\\\",
            tex_escape(model.name),
            metric(|r| r.triangles, 1e6, 2),
            metric(|r| r.vertices, 1e6, 2),
            metric(|r| r.mesh_instances, 1., 0),
            metric(|r| r.materials, 1., 0),
            metric(|r| r.texture_mib, 1., 1),
            format_distance(model.distance),
            if model.wasm_oom { "OOM" } else { "loads" },
        );
    }
    tex.push_str("\\bottomrule\n\\end{tabular}\n");
    tex
}

fn format_distance(distance: f64) -> String {
    if distance.fract() == 0. {
        format!("{distance:.0}")
    } else {
        format!("{distance:.2}")
    }
}

/// Average fps and 1% low, mean ± sd over the runs, the best average of a model in bold. The
/// columns don't name the GPU, the caption does.
fn fps_table(fps: &[Measurement], platforms: [Platform; 3]) -> String {
    let cols = platforms.len();
    let mut tex = format!(
        "% generated by bencmark_results, don't edit\n\
         \\begin{{tabular}}{{l{}}}\n\\toprule\n\
         & \\multicolumn{{{cols}}}{{c}}{{Average fps}} & \\multicolumn{{{cols}}}{{c}}{{1\\% low fps}} \\\\\n\
         \\cmidrule(lr){{2-{}}} \\cmidrule(lr){{{}-{}}}\n\
         Model",
        "c".repeat(cols * 2),
        cols + 1,
        cols + 2,
        cols * 2 + 1,
    );
    for _ in 0..2 {
        for platform in platforms {
            let _ = write!(tex, " & {}", platform.short_label());
        }
    }
    tex.push_str(" \\\\\n\\midrule\n");

    for (i, model) in MODELS.iter().enumerate() {
        let avg: Vec<_> = platforms
            .iter()
            .map(|&p| summary(fps, i, p, |r| r.avg_fps))
            .collect();
        let low: Vec<_> = platforms
            .iter()
            .map(|&p| summary(fps, i, p, |r| r.low_1_fps))
            .collect();
        if avg.iter().all(Option::is_none) {
            continue;
        }
        let best = avg
            .iter()
            .flatten()
            .map(|s| s.mean)
            .fold(f64::MIN, f64::max);
        let _ = write!(tex, "{}", tex_escape(model.name));
        for (column, &platform) in avg.iter().zip(&platforms) {
            let cell = match column {
                Some(s) if s.mean == best && avg.iter().flatten().count() > 1 => {
                    format!("$\\mathbf{{{:.1} \\pm {:.1}}}$", s.mean, s.sd)
                }
                Some(s) => mean_sd(*s, 1),
                None => missing(i, platform).into(),
            };
            let _ = write!(tex, " & {cell}");
        }
        for (column, &platform) in low.iter().zip(&platforms) {
            let cell = column.map_or_else(|| missing(i, platform).into(), |s| mean_sd(s, 1));
            let _ = write!(tex, " & {cell}");
        }
        tex.push_str(" \\\\\n");
    }
    tex.push_str("\\bottomrule\n\\end{tabular}\n");
    tex
}

pub(crate) fn format_p(p: f64) -> String {
    if p < 0.001 {
        "$<0.001$".into()
    } else {
        format!("{p:.3}")
    }
}

/// Welch's t-test on the average fps of the runs
fn ttest_table(fps: &[Measurement], pairs: [(Platform, Platform); 2]) -> String {
    let mut tex = String::from(
        "% generated by bencmark_results, don't edit\n\
         % Welch's t-test on average fps per run, \\Delta = (A - B) / B\n\
         \\begin{tabular}{lrrrr}\n\\toprule\n",
    );
    for (a, b) in pairs {
        let _ = write!(
            tex,
            " & \\multicolumn{{2}}{{c}}{{{} vs. {}}}",
            a.short_label(),
            b.short_label()
        );
    }
    tex.push_str(" \\\\\n\\cmidrule(lr){2-3} \\cmidrule(lr){4-5}\nModel");
    for _ in pairs {
        tex.push_str(" & $\\Delta$ (\\%) & $p$");
    }
    tex.push_str(" \\\\\n\\midrule\n");

    for (i, model) in MODELS.iter().enumerate() {
        let mut cells = Vec::new();
        for (a, b) in pairs {
            let result = summary(fps, i, a, |r| r.avg_fps)
                .zip(summary(fps, i, b, |r| r.avg_fps))
                .and_then(|(a, b)| welch(a, b));
            match result {
                Some(w) => {
                    cells.push(format!("{:+.1}", w.diff_percent));
                    cells.push(format_p(w.p));
                }
                None => {
                    let reason = if missing(i, a) == "OOM" || missing(i, b) == "OOM" {
                        "OOM"
                    } else {
                        "--"
                    };
                    cells.extend([reason.to_string(), "--".into()]);
                }
            }
        }
        if cells.iter().all(|c| c == "--" || c == "OOM") {
            continue;
        }
        let _ = writeln!(
            tex,
            "{} & {} \\\\",
            tex_escape(model.short),
            cells.join(" & ")
        );
    }
    tex.push_str("\\bottomrule\n\\end{tabular}\n");
    tex
}

/// The dedicated GPU barely changes load times, its few load rows (from the fps launches) aren't
/// in the paper
fn load_table(load: &[Measurement], platforms: [Platform; 3]) -> String {
    let platforms: Vec<_> = platforms
        .into_iter()
        .filter(|&p| load.iter().any(|m| m.platform == p))
        .collect();
    let mut tex = format!(
        "% generated by bencmark_results, don't edit\n\
         % total load time in s, mean $\\pm$ sd (number of loads)\n\
         \\begin{{tabular}}{{l{}}}\n\\toprule\nModel",
        "r".repeat(platforms.len())
    );
    for platform in &platforms {
        let _ = write!(tex, " & {}", platform.short_label());
    }
    tex.push_str(" \\\\\n\\midrule\n");
    for (i, model) in MODELS.iter().enumerate() {
        let _ = write!(tex, "{}", tex_escape(model.name));
        for &platform in &platforms {
            let cell = match summary(load, i, platform, |r| r.total_ms / 1000.) {
                Some(s) => format!("{} ({})", mean_sd(s, 2), s.n),
                None => missing(i, platform).into(),
            };
            let _ = write!(tex, " & {cell}");
        }
        tex.push_str(" \\\\\n");
    }
    tex.push_str("\\bottomrule\n\\end{tabular}\n");
    tex
}

fn number(value: Option<f64>) -> String {
    value.map_or_else(|| "nan".into(), |v| format!("{v:.3}"))
}

/// One line per model: `x,label` then mean, sd and 1% low of every platform, `nan` if missing.
/// `bevy_wasm_oom` is 0 where Bevy runs out of memory in the browser, the figure labels it.
fn fps_data(fps: &[Measurement]) -> String {
    let mut csv = String::from("x,label,bevy_wasm_oom");
    for platform in Platform::ALL {
        let k = platform.key();
        let _ = write!(csv, ",{k}_mean,{k}_sd,{k}_low1");
    }
    csv.push('\n');
    for (i, model) in MODELS.iter().enumerate() {
        let oom = model.wasm_oom.then_some(0.);
        let _ = write!(csv, "{i},{},{}", model.short, number(oom));
        for platform in Platform::ALL {
            let avg = summary(fps, i, platform, |r| r.avg_fps);
            let low = summary(fps, i, platform, |r| r.low_1_fps);
            let _ = write!(
                csv,
                ",{},{},{}",
                number(avg.map(|s| s.mean)),
                number(avg.map(|s| s.sd)),
                number(low.map(|s| s.mean)),
            );
        }
        csv.push('\n');
    }
    csv
}

/// Frame time percentiles averaged over the runs, `_up` is p99 - p50 for asymmetric error bars
fn frametime_data(fps: &[Measurement]) -> String {
    let mut csv = String::from("x,label");
    for platform in Platform::ALL {
        let k = platform.key();
        let _ = write!(csv, ",{k}_min,{k}_p50,{k}_p95,{k}_p99,{k}_max,{k}_up");
    }
    csv.push('\n');
    for (i, model) in MODELS.iter().enumerate() {
        let _ = write!(csv, "{i},{}", model.short);
        for platform in Platform::ALL {
            let mean = |f: fn(&Row) -> f64| summary(fps, i, platform, f).map(|s| s.mean);
            let p50 = mean(|r| r.frame_p50_ms);
            let p99 = mean(|r| r.frame_p99_ms);
            let _ = write!(
                csv,
                ",{},{},{},{},{},{}",
                number(mean(|r| r.frame_min_ms)),
                number(p50),
                number(mean(|r| r.frame_p95_ms)),
                number(p99),
                number(mean(|r| r.frame_max_ms)),
                number(p50.zip(p99).map(|(p50, p99)| p99 - p50)),
            );
        }
        csv.push('\n');
    }
    csv
}

/// Average fps over triangle count, sorted by triangles, for a line plot
fn scalability_data(fps: &[Measurement], platform: Platform) -> Option<String> {
    let mut points: Vec<_> = (0..MODELS.len())
        .filter_map(|i| {
            let avg = summary(fps, i, platform, |r| r.avg_fps)?;
            let triangles = model_metric(fps, i, |r| r.triangles)?;
            Some((triangles, avg, MODELS[i].short))
        })
        .collect();
    if points.is_empty() {
        return None;
    }
    points.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut csv = String::from("triangles,mean,sd,label\n");
    for (triangles, avg, label) in points {
        let _ = writeln!(csv, "{triangles:.0},{:.3},{:.3},{label}", avg.mean, avg.sd);
    }
    Some(csv)
}

/// Load phases in s for a stacked bar chart, only models with a measurement
fn load_data(load: &[Measurement], platform: Platform) -> Option<String> {
    let mut csv = String::from("x,label,download_parse,mesh_creation,first_frame,total,n\n");
    let mut x = 0;
    for (i, model) in MODELS.iter().enumerate() {
        let mean = |f: fn(&Row) -> f64| summary(load, i, platform, f).map(|s| s.mean / 1000.);
        let Some(total) = summary(load, i, platform, |r| r.total_ms) else {
            continue;
        };
        let _ = writeln!(
            csv,
            "{x},{},{},{},{},{:.3},{}",
            model.short,
            number(mean(|r| r.download_parse_ms)),
            number(mean(|r| r.mesh_creation_ms)),
            number(mean(|r| r.first_frame_ms)),
            total.mean / 1000.,
            total.n,
        );
        x += 1;
    }
    (x > 0).then_some(csv)
}

/// Which comparisons are complete, so missing runs are noticed before the paper is
fn print_overview(fps: &[Measurement]) {
    eprintln!("\naverage fps, mean ± sd (runs):");
    let mut header = format!("{:<26}", "");
    for platform in Platform::ALL {
        let _ = write!(header, "{:>26}", platform.label());
    }
    eprintln!("{header}");
    for (i, model) in MODELS.iter().enumerate() {
        let mut line = format!("{:<26}", model.name);
        for platform in Platform::ALL {
            let cell = summary(fps, i, platform, |r| r.avg_fps).map_or_else(
                || missing(i, platform).to_string(),
                |s| format!("{:.1} ± {:.1} ({})", s.mean, s.sd, s.n),
            );
            let _ = write!(line, "{cell:>26}");
        }
        eprintln!("{line}");
    }
}
