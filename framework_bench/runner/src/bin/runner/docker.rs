use std::process::Command;

/// Runs `docker <args>` and returns stdout, or stderr as the error.
pub fn docker(args: &[&str]) -> Result<String, String> {
    let output = Command::new("docker")
        .args(args)
        .output()
        .map_err(|e| format!("docker {}: {e}", args.join(" ")))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!(
            "docker {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

pub fn remove(name: &str) {
    let _ = docker(&["rm", "-f", name]);
}

pub fn container_id(name: &str) -> Result<String, String> {
    Ok(docker(&["inspect", "-f", "{{.Id}}", name])?.trim().to_string())
}

pub fn ensure_network(name: &str) -> Result<(), String> {
    if docker(&["network", "inspect", name]).is_err() {
        docker(&["network", "create", name])?;
    }
    Ok(())
}

/// Builds the three images from the repository root, each with its own .dockerignore.
pub fn build_images() -> Result<(), String> {
    for (tag, dockerfile) in [
        ("fwbench-axum", "framework_bench/docker/axum.Dockerfile"),
        ("fwbench-express", "framework_bench/docker/express.Dockerfile"),
        ("fwbench-attack", "framework_bench/docker/attack.Dockerfile"),
    ] {
        println!("building {tag}");
        let status = Command::new("docker")
            .args(["build", "-f", dockerfile, "-t", tag, "."])
            .status()
            .map_err(|e| format!("docker build {tag}: {e}"))?;
        if !status.success() {
            return Err(format!("docker build {tag} failed"));
        }
    }
    Ok(())
}
