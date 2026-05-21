use dto::auth::LoginDto;
use goose::prelude::*;
use plotters::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct StatPoint {
    time_sec: f32,
    cpu: f32,
    mem: f32,
}

#[derive(Default, Clone)]
struct ContainerStats {
    backend: Vec<StatPoint>,
    backend_express: Vec<StatPoint>,
}

async fn loadtest_index(user: &mut GooseUser) -> TransactionResult {
    let body = LoginDto {
        email: "user@mail.com".to_string(),
        password: "user@mail.com".to_string(),
    };
    let _goose_metrics = user.post_json("/login", &body).await?;

    Ok(())
}

fn generate_graph(stats: &ContainerStats) {
    // CPU graph
    {
        let root = BitMapBackend::new("cpu_usage.png", (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let max_time = stats
            .backend
            .last()
            .map(|p| p.time_sec)
            .unwrap_or(10.0)
            .max(
                stats
                    .backend_express
                    .last()
                    .map(|p| p.time_sec)
                    .unwrap_or(10.0),
            );
        let max_cpu = stats
            .backend
            .iter()
            .map(|p| p.cpu)
            .fold(0.0, f32::max)
            .max(
                stats
                    .backend_express
                    .iter()
                    .map(|p| p.cpu)
                    .fold(0.0, f32::max),
            )
            .max(1.0)
            * 1.2;

        let mut chart = ChartBuilder::on(&root)
            .caption("CPU Usage over Time (%)", ("sans-serif", 30).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0f32..max_time, 0f32..max_cpu)
            .unwrap();

        chart
            .configure_mesh()
            .x_desc("Time (s)")
            .y_desc("CPU (%)")
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                stats.backend.iter().map(|p| (p.time_sec, p.cpu)),
                &RED,
            ))
            .unwrap()
            .label("Axum (Rust)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .draw_series(LineSeries::new(
                stats.backend_express.iter().map(|p| (p.time_sec, p.cpu)),
                &BLUE,
            ))
            .unwrap()
            .label("Express (Node)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart
            .configure_series_labels()
            .background_style(&WHITE)
            .border_style(&BLACK)
            .draw()
            .unwrap();
        root.present().unwrap();
    }

    // Mem graph
    {
        let root = BitMapBackend::new("mem_usage.png", (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let max_time = stats
            .backend
            .last()
            .map(|p| p.time_sec)
            .unwrap_or(10.0)
            .max(
                stats
                    .backend_express
                    .last()
                    .map(|p| p.time_sec)
                    .unwrap_or(10.0),
            );
        let max_mem = stats
            .backend
            .iter()
            .map(|p| p.mem)
            .fold(0.0, f32::max)
            .max(
                stats
                    .backend_express
                    .iter()
                    .map(|p| p.mem)
                    .fold(0.0, f32::max),
            )
            .max(1.0)
            * 1.2;

        let mut chart = ChartBuilder::on(&root)
            .caption("Memory Usage over Time (%)", ("sans-serif", 30).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0f32..max_time, 0f32..max_mem)
            .unwrap();

        chart
            .configure_mesh()
            .x_desc("Time (s)")
            .y_desc("Memory (%)")
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                stats.backend.iter().map(|p| (p.time_sec, p.mem)),
                &RED,
            ))
            .unwrap()
            .label("Axum (Rust)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .draw_series(LineSeries::new(
                stats.backend_express.iter().map(|p| (p.time_sec, p.mem)),
                &BLUE,
            ))
            .unwrap()
            .label("Express (Node)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart
            .configure_series_labels()
            .background_style(&WHITE)
            .border_style(&BLACK)
            .draw()
            .unwrap();
        root.present().unwrap();
    }
    println!("Graphs saved to cpu_usage.png and mem_usage.png");
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    println!("Starting docker-compose...");
    let _ = tokio::process::Command::new("docker")
        .args([
            "compose",
            "up",
            "-d",
            "--build",
            "postgres",
            "backend",
            "backend-express",
        ])
        .current_dir("..")
        .status()
        .await
        .expect("Failed to run docker compose");

    println!("Waiting for containers to be ready (15s)...");
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    let stats = Arc::new(Mutex::new(ContainerStats::default()));
    let stats_clone = stats.clone();
    let start_time = std::time::Instant::now();
    let cancel_token = Arc::new(tokio::sync::Notify::new());
    let cancel = cancel_token.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                    let output = tokio::process::Command::new("docker")
                        .args(["stats", "--no-stream", "--format", "{{.Name}},{{.CPUPerc}},{{.MemPerc}}"])
                        .output()
                        .await;

                    if let Ok(out) = output {
                        if let Ok(text) = String::from_utf8(out.stdout) {
                            let elapsed = start_time.elapsed().as_secs_f32();
                            let mut locked = stats_clone.lock().unwrap();
                            for line in text.lines() {
                                let parts: Vec<&str> = line.split(',').collect();
                                if parts.len() == 3 {
                                    let name = parts[0].trim();
                                    let cpu = parts[1].replace("%", "").parse::<f32>().unwrap_or(0.0);
                                    let mem = parts[2].replace("%", "").parse::<f32>().unwrap_or(0.0);

                                    let point = StatPoint { time_sec: elapsed, cpu, mem };
                                    if name.contains("backend-express") {
                                        locked.backend_express.push(point);
                                    } else if name.starts_with("diplomovka_bevy_wasm-backend-1") || (name.contains("backend") && !name.contains("express")) {
                                        locked.backend.push(point);
                                    }
                                }
                            }
                        }
                    }
                }
                _ = cancel.notified() => {
                    break;
                }
            }
        }
    });

    println!("Starting GooseAttack for axum...");
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("LoadtestTransactions").register_transaction(transaction!(loadtest_index)),
        )
        .set_default(GooseDefault::Host, "http://localhost:4000/")?
        .set_default(GooseDefault::ReportFile, "axumReport.html")?
        .set_default(GooseDefault::NoResetMetrics, true)?
        .execute()
        .await?;

    println!("Starting GooseAttack for express...");
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("LoadtestTransactions").register_transaction(transaction!(loadtest_index)),
        )
        .set_default(GooseDefault::Host, "http://localhost:4001/")?
        .set_default(GooseDefault::ReportFile, "expressReport.html")?
        .set_default(GooseDefault::NoResetMetrics, true)?
        .execute()
        .await?;

    cancel_token.notify_one();

    generate_graph(&stats.lock().unwrap());

    Ok(())
}
