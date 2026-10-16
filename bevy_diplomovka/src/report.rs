//! Hands the CSV rows to rendering_runner in bench mode (see bench_mode.rs): on the web through
//! `window.benchReport` from index.html when the url has `?report=<url>`, natively appended to the
//! `out` folder. Without bench mode these do nothing.

use std::sync::OnceLock;

/// Native only, set from the `out` option
static OUT_DIR: OnceLock<String> = OnceLock::new();

pub fn set_out_dir(dir: &str) {
    let _ = OUT_DIR.set(dir.to_string());
}

/// One CSV row, `file` is `fps.csv` or `load.csv`
pub fn row(file: &str, header: &str, row: &str) {
    #[cfg(target_arch = "wasm32")]
    send(&[
        ("file", file.into()),
        ("header", header.into()),
        ("row", row.into()),
    ]);
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(dir) = OUT_DIR.get() {
        if let Err(error) = append_csv(std::path::Path::new(dir).join(file), header, row) {
            bevy::log::error!("can't write {file} to {dir}: {error}");
        }
    }
}

/// The benchmark finished, every row was sent before
pub fn done() {
    #[cfg(target_arch = "wasm32")]
    send(&[("done", wasm_bindgen::JsValue::TRUE)]);
}

/// The benchmark can't go on: login, unknown model, failed load, panic
pub fn error(message: &str) {
    #[cfg(target_arch = "wasm32")]
    send(&[("error", message.into())]);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = message;
}

/// Calls `window.benchReport({...fields})`, if the page defined it
#[cfg(target_arch = "wasm32")]
fn send(fields: &[(&str, wasm_bindgen::JsValue)]) {
    use wasm_bindgen::{JsCast, JsValue};

    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(function) = js_sys::Reflect::get(&window, &"benchReport".into()) else {
        return;
    };
    let Some(function) = function.dyn_ref::<js_sys::Function>() else {
        return;
    };
    let message = js_sys::Object::new();
    for (key, value) in fields {
        let _ = js_sys::Reflect::set(&message, &JsValue::from_str(key), value);
    }
    let _ = function.call1(&JsValue::NULL, &message);
}

/// Appends the row, writes the header first if the file is new
#[cfg(not(target_arch = "wasm32"))]
fn append_csv(path: std::path::PathBuf, header: &str, row: &str) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    if file.metadata()?.len() == 0 {
        writeln!(file, "{header}")?;
    }
    writeln!(file, "{row}")
}
