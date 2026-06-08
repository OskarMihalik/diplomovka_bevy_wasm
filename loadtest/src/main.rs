use dto::{
    auth::{AuthDtoResponse, LoginDto, UserDto},
    default::NewTagDto,
    model::ModelsDtoResponse,
    project::{NewProjectDto, ProjectsDtoResponse},
};
use goose::prelude::*;
use plotters::prelude::*;
use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
#[derive(Clone, Default)]
struct StatPoint {
    time_sec: f32,
    cpu: f32,
    mem: f32,
}

#[derive(Clone)]
struct Session {
    user_dto: UserDto,
    project_id: Option<i32>,
    model_id: Option<i32>,
}

#[derive(Default, Clone)]
struct ContainerStats {
    backend: Vec<StatPoint>,
    backend_express: Vec<StatPoint>,
}

async fn login_to_system(user: &mut GooseUser) -> TransactionResult {
    let body = LoginDto {
        email: "user@mail.com".to_string(),
        password: "user@mail.com".to_string(),
    };
    let mut goose_response = user.post_json("/login", &body).await?;

    let response = goose_response.response?.json::<AuthDtoResponse>().await?;

    let user_dto = match response {
        Ok(dto) => dto,
        Err(e) => {
            return user.set_failure(
                &format!("error: {}", e.message),
                &mut goose_response.request,
                None,
                None,
            );
        }
    };

    user.set_session_data(Session {
        user_dto,
        project_id: None,
        model_id: None,
    });

    Ok(())
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    nanos.to_string()
}

async fn create_project(user: &mut GooseUser) -> TransactionResult {
    let session = user.get_session_data_unchecked::<Session>();
    let user_dto = session.user_dto.clone();
    let project_name = format!("goose-project-{}", unique_suffix());
    let body = NewProjectDto {
        name: project_name,
        description: "goose loadtest session project".to_string(),
    };

    let reqwest_request_builder = user
        .get_request_builder(&GooseMethod::Post, "/project")?
        .json(&body)
        .header("authorization", &session.user_dto.token);

    let goose_request = GooseRequest::builder()
        .set_request_builder(reqwest_request_builder)
        .build();

    let mut goose_response = user.request(goose_request).await?;
    let response = goose_response
        .response?
        .json::<ProjectsDtoResponse>()
        .await?;

    let project_id = match response {
        Ok(projects) => match projects.into_iter().next() {
            Some(project) => project.id,
            None => {
                return user.set_failure(
                    "error: /project returned no projects",
                    &mut goose_response.request,
                    None,
                    None,
                );
            }
        },
        Err(e) => {
            return user.set_failure(
                &format!("error: {}", e.message),
                &mut goose_response.request,
                None,
                None,
            );
        }
    };

    user.set_session_data(Session {
        user_dto,
        project_id: Some(project_id),
        model_id: None,
    });

    Ok(())
}

async fn create_model(user: &mut GooseUser) -> TransactionResult {
    let session = user.get_session_data_unchecked::<Session>();
    let user_dto = session.user_dto.clone();
    let project_id = match session.project_id {
        Some(id) => id,
        None => return Err("error: missing project_id in session".into()),
    };

    let model_name = format!("goose-model-{}", unique_suffix());
    let path = format!("/model/{project_id}/{model_name}");
    let part = reqwest::multipart::Part::bytes(vec![0_u8; 16])
        .file_name("loadtest.glb")
        .mime_str("model/gltf-binary")
        .unwrap();
    let form = reqwest::multipart::Form::new().part("file", part);

    let reqwest_request_builder = user
        .get_request_builder(&GooseMethod::Post, &path)?
        .multipart(form)
        .header("authorization", &session.user_dto.token);

    let goose_request = GooseRequest::builder()
        .set_request_builder(reqwest_request_builder)
        .build();

    let mut goose_response = user.request(goose_request).await?;
    let response = goose_response.response?.json::<ModelsDtoResponse>().await?;

    let model_id = match response {
        Ok(models) => match models.get(0) {
            Some(model) => model.id,
            None => {
                return user.set_failure(
                    "error: /model upload returned no models",
                    &mut goose_response.request,
                    None,
                    None,
                );
            }
        },
        Err(e) => {
            return user.set_failure(
                &format!("error: {}", e.message),
                &mut goose_response.request,
                None,
                None,
            );
        }
    };

    user.set_session_data(Session {
        user_dto,
        project_id: Some(project_id),
        model_id: Some(model_id),
    });

    Ok(())
}

async fn create_tag(user: &mut GooseUser) -> TransactionResult {
    // This will panic if the session is missing or if the session is not of the right type.
    // Use `get_session_data` to handle a missing session.
    let session = user.get_session_data_unchecked::<Session>();
    let project_id = match session.project_id {
        Some(id) => id,
        None => {
            return Err("error: missing project_id in session".to_string().into());
        }
    };

    let body = NewTagDto {
        title: "new test tag".to_string(),
        project_id,
        position_x: 7.089879,
        position_y: 2.999879,
        position_z: -28.943491,
    };

    // Create a Reqwest RequestBuilder object and configure bearer authentication when making
    // a GET request for the index.
    let reqwest_request_builder = user
        .get_request_builder(&GooseMethod::Post, "/tags")?
        .json(&body)
        .header("authorization", &session.user_dto.token);

    // Add the manually created RequestBuilder and build a GooseRequest object.
    let goose_request = GooseRequest::builder()
        .set_request_builder(reqwest_request_builder)
        .build();

    // Make the actual request.
    user.request(goose_request).await?;

    Ok(())
}

async fn cleanup_project(user: &mut GooseUser) -> TransactionResult {
    let session = user.get_session_data_unchecked::<Session>();
    let project_id = match session.project_id {
        Some(id) => id,
        None => return Ok(()),
    };

    let path = format!("/project/{project_id}");
    let reqwest_request_builder = user
        .get_request_builder(&GooseMethod::Delete, &path)?
        .header("authorization", &session.user_dto.token);

    let goose_request = GooseRequest::builder()
        .set_request_builder(reqwest_request_builder)
        .build();

    user.request(goose_request).await?;

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

    let scenario1 = scenario!("session tag creation")
        .register_transaction(transaction!(login_to_system).set_on_start())
        .register_transaction(transaction!(create_project).set_on_start())
        .register_transaction(transaction!(create_model).set_on_start())
        .register_transaction(transaction!(create_tag))
        .register_transaction(transaction!(cleanup_project).set_on_stop());

    let scenario_only_login =
        scenario!("login").register_transaction(transaction!(login_to_system));

    GooseAttack::initialize()?
        .register_scenario(scenario1.clone())
        // .register_scenario(scenario_only_login.clone())
        .set_default(GooseDefault::Host, "http://localhost:4000/")?
        .set_default(GooseDefault::ReportFile, "axumReport.html")?
        .set_default(GooseDefault::NoResetMetrics, true)?
        .execute()
        .await?;

    println!("Starting GooseAttack for express...");
    GooseAttack::initialize()?
        .register_scenario(scenario1.clone())
        // .register_scenario(scenario_only_login.clone())
        .set_default(GooseDefault::Host, "http://localhost:4001/")?
        .set_default(GooseDefault::ReportFile, "expressReport.html")?
        .set_default(GooseDefault::NoResetMetrics, true)?
        .execute()
        .await?;

    cancel_token.notify_one();

    generate_graph(&stats.lock().unwrap());

    Ok(())
}
