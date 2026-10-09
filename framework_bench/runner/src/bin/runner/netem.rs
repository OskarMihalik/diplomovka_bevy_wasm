//! Network emulation with `tc netem`, applied from a sidecar that joins the container's network
//! namespace (the attack image ships iproute2), so the server images need no extra capabilities.

use crate::config::Profile;
use crate::docker::docker;

const IMAGE: &str = "fwbench-attack";
const DEV: &str = "eth0";

fn tc(container: &str, args: &[&str]) -> Result<String, String> {
    let net = format!("container:{container}");
    let mut full = vec!["run", "--rm", "--net", &net, "--cap-add", "NET_ADMIN", IMAGE, "tc"];
    full.extend_from_slice(args);
    docker(&full)
}

/// Applies half of the profile's RTT and jitter/sqrt(2) to each side, so the round trip has the
/// profile's mean and standard deviation; loss applies to each direction.
/// The queue limit is raised from netem's default of 1000 packets, which would otherwise drop
/// packets on large responses (3.7 MiB at 60 ms RTT is far more than 1000 packets in flight).
pub fn apply(container: &str, profile: &Profile) -> Result<(), String> {
    clear(container);
    if profile.rtt_ms == 0.0 && profile.loss_pct == 0.0 {
        return Ok(());
    }
    let delay = format!("{}ms", profile.rtt_ms / 2.0);
    let jitter = format!("{:.3}ms", profile.jitter_ms / std::f64::consts::SQRT_2);
    let loss = format!("{}%", profile.loss_pct);
    let mut args = vec!["qdisc", "add", "dev", DEV, "root", "netem", "limit", "100000", "delay", &delay];
    if profile.jitter_ms > 0.0 {
        args.extend_from_slice(&[&jitter, "distribution", "normal"]);
    }
    if profile.loss_pct > 0.0 {
        args.extend_from_slice(&["loss", &loss]);
    }
    tc(container, &args).map(|_| ())
}

pub fn clear(container: &str) {
    // Fails when no root qdisc is set, which is fine.
    let _ = tc(container, &["qdisc", "del", "dev", DEV, "root"]);
}

/// The qdisc statistics and the number of packets netem dropped (0 without netem).
pub fn stats(container: &str) -> (String, u64) {
    let text = tc(container, &["-s", "qdisc", "show", "dev", DEV]).unwrap_or_default();
    // "qdisc netem 8001: root ... \n Sent 123 bytes 4 pkt (dropped 5, overlimits 0 requeues 0)"
    let dropped = if text.contains("netem") {
        text.split("dropped ")
            .nth(1)
            .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|n| n.parse().ok())
            .unwrap_or(0)
    } else {
        0
    };
    (text, dropped)
}
