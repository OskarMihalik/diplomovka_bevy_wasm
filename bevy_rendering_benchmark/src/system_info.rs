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

/// The local time zone, read once by `init_local_offset`
#[cfg(not(target_arch = "wasm32"))]
static LOCAL_OFFSET: std::sync::OnceLock<time::UtcOffset> = std::sync::OnceLock::new();

/// Reads the local time zone. Has to run first in `main`: `time` can only read it while the
/// process has a single thread.
#[cfg(not(target_arch = "wasm32"))]
pub fn init_local_offset() {
    let offset = time::UtcOffset::current_local_offset().unwrap_or_else(|error| {
        eprintln!("can't read the local time zone, dates are in UTC: {error}");
        time::UtcOffset::UTC
    });
    let _ = LOCAL_OFFSET.set(offset);
}

/// Local date and time like on the web, `YYYY-MM-DD HH:MM:SS`
#[cfg(not(target_arch = "wasm32"))]
pub fn date() -> String {
    let offset = LOCAL_OFFSET.get().copied().unwrap_or(time::UtcOffset::UTC);
    let date = time::OffsetDateTime::now_utc().to_offset(offset);
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

#[cfg(all(target_arch = "wasm32", feature = "webgpu"))]
thread_local! {
    static WEBGPU_ADAPTER: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// Asks the browser for a WebGPU adapter with the power preference wgpu uses, for `gpu_name`:
/// wgpu only takes the adapter's `description`, which both Chrome and Firefox leave empty
#[cfg(all(target_arch = "wasm32", feature = "webgpu"))]
pub fn request_webgpu_adapter(power_preference: bevy::render::settings::PowerPreference) {
    use bevy::render::settings::PowerPreference;
    let preference = match power_preference {
        PowerPreference::LowPower => "low-power",
        PowerPreference::HighPerformance => "high-performance",
        PowerPreference::None => "",
    };
    let request = js_sys::Function::new_with_args(
        "preference",
        "return navigator.gpu ? navigator.gpu.requestAdapter(preference ? { powerPreference: preference } : {})\
         .then((a) => (a ? [a.info.vendor, a.info.architecture].filter(Boolean).join(' ') : '')) : Promise.resolve('')",
    );
    let Ok(promise) = request.call1(&wasm_bindgen::JsValue::NULL, &preference.into()) else {
        return;
    };
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(name) = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await {
            WEBGPU_ADAPTER.with(|adapter| *adapter.borrow_mut() = name.as_string().unwrap_or_default());
        }
    });
}

/// The adapter name, on WebGPU `vendor architecture` (`intel gen-12lp` in Chrome; Firefox hides
/// the adapter info, so empty)
pub fn gpu_name(adapter_name: &str) -> String {
    #[cfg(all(target_arch = "wasm32", feature = "webgpu"))]
    if adapter_name.is_empty() {
        return WEBGPU_ADAPTER.with(|adapter| adapter.borrow().clone());
    }
    adapter_name.to_string()
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
