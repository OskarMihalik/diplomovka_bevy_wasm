use dto::{
    auth::{AuthDtoResponse, LoginDto, RegisterDto, UserDto},
    default::{ErrorDto, NewTagDto},
    model::ModelDto,
    project::{NewProjectDto, ProjectDto},
};
use goose::{goose::GooseResponse, prelude::*};
use plotters::prelude::*;
use serde::de::{DeserializeOwned, IgnoredAny};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const AXUM_HOST: &str = "http://localhost:4000/";
const EXPRESS_HOST: &str = "http://localhost:4001/";
const PROJECT_NAME_PREFIX: &str = "goose-project-";
const PROJECT_DESCRIPTION: &str = "goose loadtest session project";
const LOADTEST_PASSWORD: &str = "goose-loadtest-password";

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

/// Returns the `Ok` payload of a backend response.
///
/// Both backends answer most errors with HTTP 200 and an `Err` body, so the status code alone
/// does not tell whether a request succeeded. Everything else is recorded as a failed request.
async fn ok_body<T: DeserializeOwned>(
    user: &mut GooseUser,
    goose_response: GooseResponse,
) -> Result<T, Box<TransactionError>> {
    let GooseResponse {
        mut request,
        response,
    } = goose_response;
    // a request without response (timeout) was already recorded as failed by goose
    let response = response?;

    let status = response.status();
    let tag = if !status.is_success() {
        // already recorded as failed by goose, `set_failure` only returns the error
        format!("error: status {status}")
    } else {
        match response.json::<Result<T, ErrorDto>>().await {
            Ok(Ok(body)) => return Ok(body),
            Ok(Err(e)) => format!("error: {}", e.message),
            Err(e) => format!("error: invalid response body: {e}"),
        }
    };

    user.set_failure(&tag, &mut request, None, None)?;
    Err(tag.into())
}

/// Stops a user without a usable session from looping without ever yielding.
async fn missing_session(what: &str) -> TransactionResult {
    tokio::time::sleep(Duration::from_secs(1)).await;
    Err(format!("error: missing {what} in session").into())
}

/// Logs in as the account of this goose user and creates the account on the first run.
///
/// Every goose user needs its own account: the backends return the projects of the logged in
/// account, so with one shared account the users would see (and delete) each other's projects.
async fn login_own_account(user: &mut GooseUser) -> TransactionResult {
    let username = format!("goose-{}", user.weighted_users_index);
    let email = format!("{username}@loadtest.local");

    let body = LoginDto {
        email: email.clone(),
        password: LOADTEST_PASSWORD.to_string(),
    };
    let goose_response = user.post_json("/login", &body).await?;
    // not `ok_body`: an `Err` here means that the account does not exist yet
    let response = goose_response.response?.json::<AuthDtoResponse>().await?;

    let user_dto = match response {
        Ok(dto) => dto,
        Err(_) => {
            let body = RegisterDto {
                email,
                username,
                password: LOADTEST_PASSWORD.to_string(),
            };
            let goose_response = user.post_json("/register", &body).await?;
            ok_body::<UserDto>(user, goose_response).await?
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
    let Some(session) = user.get_session_data::<Session>() else {
        return missing_session("login").await;
    };
    let user_dto = session.user_dto.clone();
    // the user index keeps the name unique even if two users share a timestamp
    let project_name = format!(
        "{PROJECT_NAME_PREFIX}{}-{}",
        user.weighted_users_index,
        unique_suffix()
    );
    let body = NewProjectDto {
        name: project_name.clone(),
        description: PROJECT_DESCRIPTION.to_string(),
    };

    let reqwest_request_builder = user
        .get_request_builder(&GooseMethod::Post, "/project")?
        .json(&body)
        .header("authorization", &user_dto.token);

    let goose_request = GooseRequest::builder()
        .method(GooseMethod::Post)
        .path("/project")
        .name("/project")
        .set_request_builder(reqwest_request_builder)
        .build();

    let goose_response = user.request(goose_request).await?;
    let mut request = goose_response.request.clone();
    let projects = ok_body::<Vec<ProjectDto>>(user, goose_response).await?;

    // the response lists all projects of the account, the new one is found by its unique name
    let project_id = match projects.into_iter().find(|p| p.name == project_name) {
        Some(project) => project.id,
        None => {
            return user.set_failure(
                "error: /project did not return the new project",
                &mut request,
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
    let Some(session) = user.get_session_data::<Session>() else {
        return missing_session("login").await;
    };
    let user_dto = session.user_dto.clone();
    let project_id = match session.project_id {
        Some(id) => id,
        None => return missing_session("project_id").await,
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
        .header("authorization", &user_dto.token);

    let goose_request = GooseRequest::builder()
        .method(GooseMethod::Post)
        .path(path.as_str())
        .name("/model/{project_id}/{model_name}")
        .set_request_builder(reqwest_request_builder)
        .build();

    let goose_response = user.request(goose_request).await?;
    let mut request = goose_response.request.clone();
    let models = ok_body::<Vec<ModelDto>>(user, goose_response).await?;

    let model_id = match models.into_iter().find(|m| m.name == model_name) {
        Some(model) => model.id,
        None => {
            return user.set_failure(
                "error: /model upload did not return the new model",
                &mut request,
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
    let Some(session) = user.get_session_data::<Session>() else {
        return missing_session("login").await;
    };
    let project_id = match session.project_id {
        Some(id) => id,
        None => return missing_session("project_id").await,
    };

    let body = NewTagDto {
        title: "new test tag".to_string(),
        project_id,
        position_x: 7.089879,
        position_y: 2.999879,
        position_z: -28.943491,
    };

    let reqwest_request_builder = user
        .get_request_builder(&GooseMethod::Post, "/tags")?
        .json(&body)
        .header("authorization", &session.user_dto.token);

    let goose_request = GooseRequest::builder()
        .method(GooseMethod::Post)
        .path("/tags")
        .name("/tags")
        .set_request_builder(reqwest_request_builder)
        .build();

    let goose_response = user.request(goose_request).await?;
    // the tag list itself is not needed, only whether the backend answered with `Ok`
    ok_body::<IgnoredAny>(user, goose_response).await?;

    Ok(())
}

async fn cleanup_project(user: &mut GooseUser) -> TransactionResult {
    let Some(session) = user.get_session_data::<Session>() else {
        return Ok(());
    };
    let project_id = match session.project_id {
        Some(id) => id,
        None => return Ok(()),
    };

    let path = format!("/project/{project_id}");
    let reqwest_request_builder = user
        .get_request_builder(&GooseMethod::Delete, &path)?
        .header("authorization", &session.user_dto.token);

    let goose_request = GooseRequest::builder()
        .method(GooseMethod::Delete)
        .path(path.as_str())
        .name("/project/{project_id}")
        .set_request_builder(reqwest_request_builder)
        .build();

    let goose_response = user.request(goose_request).await?;
    ok_body::<IgnoredAny>(user, goose_response).await?;

    Ok(())
}

/// Runs a command in the directory of `docker-compose.yml` and fails if it does not succeed.
async fn docker_compose(args: &[&str]) {
    let status = tokio::process::Command::new("docker")
        .arg("compose")
        .args(args)
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
        .status()
        .await
        .expect("Failed to run docker compose");
    assert!(status.success(), "docker compose {args:?} failed: {status}");
}

/// Removes what earlier load tests left behind, so every attack starts from the same tables.
///
/// The tag queries scan the whole `Tag` table, so leftover rows (and dead rows that were not
/// vacuumed yet) of the previous attack would slow down the backend that is tested second.
async fn reset_database() {
    println!("Resetting loadtest data in the database...");
    let delete = format!(
        "DELETE FROM public.\"Project\" WHERE name LIKE '{PROJECT_NAME_PREFIX}%' AND description = '{PROJECT_DESCRIPTION}'"
    );
    docker_compose(&[
        "exec",
        "-T",
        "postgres",
        "psql",
        "-v",
        "ON_ERROR_STOP=1",
        "-U",
        "postgres",
        "-d",
        "bevy",
        "-c",
        &delete,
        "-c",
        "VACUUM (ANALYZE) public.\"Tag\", public.\"Model\", public.\"ProjectUser\", public.\"Project\", public.\"User\"",
    ])
    .await;
}

/// Waits until the backend answers HTTP requests.
async fn wait_for_backend(host: &str) {
    let client = reqwest::Client::new();
    let url = format!("{host}project");
    for _ in 0..120 {
        // unauthorized, but any HTTP response means that the server is up
        if client.get(&url).send().await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    panic!("{host} did not start in 60s");
}

/// `suffix` is appended to the file names, so the scenarios do not overwrite each other.
fn generate_graph(stats: &ContainerStats, suffix: &str) {
    let cpu_file = format!("cpu_usage{suffix}.png");
    let mem_file = format!("mem_usage{suffix}.png");
    // CPU graph
    {
        let root = BitMapBackend::new(&cpu_file, (800, 600)).into_drawing_area();
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
        let root = BitMapBackend::new(&mem_file, (800, 600)).into_drawing_area();
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
    println!("Graphs saved to {cpu_file} and {mem_file}");
}


#[tokio::main]
async fn main() -> Result<(), GooseError> {
    // LOADTEST_SCENARIO=login runs the login scenario, the default is tag creation
    let login_only = match std::env::var("LOADTEST_SCENARIO").as_deref() {
        Ok("login") => true,
        Ok("tag") | Err(_) => false,
        Ok(other) => panic!("unknown LOADTEST_SCENARIO {other:?}, expected `tag` or `login`"),
    };

    println!("Starting docker-compose...");
    // the backends connect to the database on startup, so it has to accept connections first
    docker_compose(&["up", "-d", "--wait", "postgres"]).await;
    docker_compose(&["up", "-d", "--build", "backend", "backend-express"]).await;

    println!("Waiting for the backends to be ready...");
    wait_for_backend(AXUM_HOST).await;
    wait_for_backend(EXPRESS_HOST).await;

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

    let (scenario, suffix) = if login_only {
        (
            scenario!("login").register_transaction(transaction!(login_own_account)),
            "-login",
        )
    } else {
        (
            scenario!("session tag creation")
                .register_transaction(transaction!(login_own_account).set_on_start())
                .register_transaction(transaction!(create_project).set_on_start())
                .register_transaction(transaction!(create_model).set_on_start())
                .register_transaction(transaction!(create_tag))
                .register_transaction(transaction!(cleanup_project).set_on_stop()),
            "",
        )
    };

    reset_database().await;
    GooseAttack::initialize()?
        .register_scenario(scenario.clone())
        .set_default(GooseDefault::Host, AXUM_HOST)?
        .set_default(
            GooseDefault::ReportFile,
            format!("axumReport{suffix}.html").as_str(),
        )?
        .set_default(GooseDefault::NoResetMetrics, true)?
        .execute()
        .await?;

    println!("Starting GooseAttack for express...");
    reset_database().await;
    GooseAttack::initialize()?
        .register_scenario(scenario.clone())
        .set_default(GooseDefault::Host, EXPRESS_HOST)?
        .set_default(
            GooseDefault::ReportFile,
            format!("expressReport{suffix}.html").as_str(),
        )?
        .set_default(GooseDefault::NoResetMetrics, true)?
        .execute()
        .await?;

    cancel_token.notify_one();

    generate_graph(&stats.lock().unwrap(), suffix);

    Ok(())
}
