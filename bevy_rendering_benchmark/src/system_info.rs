//! Information about the machine and build, for rows of measurement results pasted into CSV.

/// Name of the computer, set when building: `PC_NAME="desktop" trunk serve`
pub const PC_NAME: &str = match option_env!("PC_NAME") {
    Some(name) => name,
    None => "unknown",
};

pub const BUILD: &str = if cfg!(debug_assertions) {
    "debug"
} else {
    "release"
};

/// Local date and time, `YYYY-MM-DD HH:MM:SS`
#[cfg(target_arch = "wasm32")]
pub fn date() -> String {
    let date = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    )
}

/// UTC date and time, `YYYY-MM-DD HH:MM:SS`
#[cfg(not(target_arch = "wasm32"))]
pub fn date() -> String {
    let date = time::OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.year(),
        date.month() as u8,
        date.day(),
        date.hour(),
        date.minute(),
        date.second()
    )
}

/// `(name, version)` of the browser, parsed from the user agent
#[cfg(target_arch = "wasm32")]
pub fn browser() -> (String, String) {
    let user_agent = web_sys::window()
        .and_then(|window| window.navigator().user_agent().ok())
        .unwrap_or_default();
    // order matters, Edge and Opera also contain "Chrome/", Chrome also contains "Safari/"
    for (token, name) in [
        ("Firefox/", "Firefox"),
        ("Edg/", "Edge"),
        ("OPR/", "Opera"),
        ("Chrome/", "Chrome"),
        ("Version/", "Safari"),
    ] {
        if let Some((_, rest)) = user_agent.split_once(token) {
            let version = rest.split([' ', ';', ')']).next().unwrap_or_default();
            return (name.to_string(), version.to_string());
        }
    }
    (user_agent, String::new())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn browser() -> (String, String) {
    ("native".to_string(), String::new())
}

/// Operating system as reported by the browser (or the target OS natively)
#[cfg(target_arch = "wasm32")]
pub fn os() -> String {
    web_sys::window()
        .and_then(|window| window.navigator().platform().ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn os() -> String {
    std::env::consts::OS.to_string()
}

/// Logical CPU cores
#[cfg(target_arch = "wasm32")]
pub fn cpu_cores() -> usize {
    web_sys::window()
        .map(|window| window.navigator().hardware_concurrency() as usize)
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn cpu_cores() -> usize {
    std::thread::available_parallelism()
        .map(|cores| cores.get())
        .unwrap_or_default()
}

/// Joins the fields into one CSV row, quoting the ones that need it
pub fn csv_row(fields: &[String]) -> String {
    fields
        .iter()
        .map(|field| {
            if field.contains([',', '"', '\n']) {
                format!("\"{}\"", field.replace('"', "\"\""))
            } else {
                field.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}
