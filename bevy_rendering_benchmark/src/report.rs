//! Sends results to rendering_runner when the page was opened with `?report=<url>`. index.html
//! defines `window.benchReport`, which posts the messages to that url one after another. Natively
//! the rows go to `--out` instead and these do nothing.

/// One CSV row, `header` includes the common columns
pub fn row(file: &str, header: &str, row: &str) {
    #[cfg(target_arch = "wasm32")]
    send(&[
        ("file", file.into()),
        ("header", header.into()),
        ("row", row.into()),
    ]);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (file, header, row);
}

/// The benchmark finished, every row was sent before
pub fn done() {
    #[cfg(target_arch = "wasm32")]
    send(&[("done", wasm_bindgen::JsValue::TRUE)]);
}

/// The benchmark can't go on: the model didn't load, a panic, bad settings
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
