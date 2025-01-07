// use crate::io::{AssetReader, AssetReaderError, Reader};
// use crate::io::{AssetSource, PathStream};
use crate::AssetApp;
use bevy::{
    asset::io::{AssetReader, AssetReaderError, AssetSource, PathStream, Reader, VecReader},
    prelude::*,
    utils::ConditionalSendFuture,
};
// use bevy_app::App;
// use bevy_utils::ConditionalSendFuture;
use std::path::{Path, PathBuf};

// use super::wasm::HttpWasmAssetReader;
/// Adds the `http` and `https` asset sources to the app.
/// Any asset path that begins with `http` or `https` will be loaded from the web
/// via `fetch`(wasm) or `ureq`(native).
///
/// Due to [licensing complexities](https://github.com/briansmith/ring/issues/1827)
/// secure `https` requests are disabled by default in non-wasm builds.
/// To enable add this to your dependencies in Cargo.toml:
/// ```toml
/// ureq = { version = "*", features = ["tls"] }
/// ```
pub fn http_source_plugin(app: &mut App) {
    app.register_asset_source(
        "http",
        AssetSource::build().with_reader(|| Box::new(HttpSourceAssetReader::Http)),
    );
    app.register_asset_source(
        "https",
        AssetSource::build().with_reader(|| Box::new(HttpSourceAssetReader::Https)),
    );
}

/// Asset reader that treats paths as urls to load assets from.
pub enum HttpSourceAssetReader {
    /// Unencrypted connections.
    Http,
    /// Use TLS for setting up connections.
    Https,
}

impl HttpSourceAssetReader {
    fn make_uri(&self, path: &Path) -> PathBuf {
        PathBuf::from(match self {
            Self::Http => "http://",
            Self::Https => "https://",
        })
        .join(path)
    }

    /// See [`crate::io::get_meta_path`]
    fn make_meta_uri(&self, path: &Path) -> Option<PathBuf> {
        let mut uri = self.make_uri(path);
        let mut extension = path.extension()?.to_os_string();
        extension.push(".meta");
        uri.set_extension(extension);
        Some(uri)
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
fn js_value_to_err(context: &str) -> impl FnOnce(JsValue) -> std::io::Error + '_ {
    use js_sys::JSON;
    move |value| {
        let message = match JSON::stringify(&value) {
            Ok(js_str) => format!("Failed to {context}: {js_str}"),
            Err(_) => {
                format!("Failed to {context} and also failed to stringify the JSValue of the error")
            }
        };

        std::io::Error::new(std::io::ErrorKind::Other, message)
    }
}

#[cfg(target_arch = "wasm32")]
async fn get<'a>(path: PathBuf) -> Result<Box<dyn Reader>, AssetReaderError> {
    // use super::wasm::HttpWasmAssetReader;
    use js_sys::{Uint8Array, JSON};
    use std::sync::Arc;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::Response;
    let window = web_sys::window().unwrap();

    let resp_value = JsFuture::from(window.fetch_with_str(path.to_str().unwrap()))
        .await
        .map_err(js_value_to_err("fetch path"))?;

    let resp = resp_value
        .dyn_into::<Response>()
        .map_err(js_value_to_err("convert fetch to Response"))?;

    match resp.status() {
        200 => {
            let data = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
            let bytes = Uint8Array::new(&data).to_vec();
            let reader: Box<dyn Reader> = Box::new(VecReader::new(bytes));
            Ok(reader)
        }
        404 => Err(AssetReaderError::NotFound(path)),
        status => Err(AssetReaderError::Io(Arc::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Encountered unexpected HTTP status {status}"),
        )))),
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn get<'a>(path: PathBuf) -> Result<Box<dyn Reader>, AssetReaderError> {
    // use crate::io::VecReader;
    use bevy::asset::io::VecReader;
    use std::io;
    // use ureq::Agent;
    // use bevy::asset::io::{AssetReaderError, Reader, VecReader};

    let str_path = path.to_str().ok_or_else(|| {
        AssetReaderError::Io(
            io::Error::new(
                io::ErrorKind::Other,
                format!("non-utf8 path: {}", path.display()),
            )
            .into(),
        )
    })?;

    // #[cfg(feature = "http_source_cache")]
    if let Some(data) = http_asset_cache::try_load_from_cache(str_path)? {
        return Ok(Box::new(VecReader::new(data)));
    }

    match ureq::get(str_path).call() {
        Ok(response) => {
            let mut reader = response.into_reader();
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer)?;

            // #[cfg(feature = "http_source_cache")]
            http_asset_cache::save_to_cache(str_path, &buffer)?;

            Ok(Box::new(VecReader::new(buffer)))
        }
        // ureq considers all >=400 status codes as errors
        Err(ureq::Error::Status(code, _response)) => {
            if code == 404 {
                Err(AssetReaderError::NotFound(path))
            } else {
                Err(AssetReaderError::HttpError(code))
            }
        }
        Err(ureq::Error::Transport(err)) => Err(AssetReaderError::Io(
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "unexpected error while loading asset {}: {}",
                    path.display(),
                    err
                ),
            )
            .into(),
        )),
    }
}

impl AssetReader for HttpSourceAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl ConditionalSendFuture<Output = Result<Box<dyn Reader>, AssetReaderError>> {
        get(self.make_uri(path))
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<Box<dyn Reader>, AssetReaderError> {
        match self.make_meta_uri(path) {
            Some(uri) => get(uri).await,
            None => Err(AssetReaderError::NotFound(
                "source path has no extension".into(),
            )),
        }
    }

    async fn is_directory<'a>(&'a self, _path: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(false)
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Err(AssetReaderError::NotFound(self.make_uri(path)))
    }
}

/// A naive implementation of an HTTP asset cache that never invalidates.
/// `ureq` currently does not support caching, so this is a simple workaround.
/// It should eventually be replaced by `http-cache` or similar, see [tracking issue](https://github.com/06chaynes/http-cache/issues/91)
// #[cfg(feature = "http_source_cache")]
mod http_asset_cache {
    use core::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    use std::fs::{self, File};
    use std::io::{self, Read, Write};
    use std::path::PathBuf;

    const CACHE_DIR: &str = ".http-asset-cache";

    fn url_to_hash(url: &str) -> String {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    pub fn try_load_from_cache(url: &str) -> Result<Option<Vec<u8>>, io::Error> {
        let filename = url_to_hash(url);
        let cache_path = PathBuf::from(CACHE_DIR).join(&filename);

        if cache_path.exists() {
            let mut file = File::open(&cache_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            Ok(Some(buffer))
        } else {
            Ok(None)
        }
    }

    pub fn save_to_cache(url: &str, data: &[u8]) -> Result<(), io::Error> {
        let filename = url_to_hash(url);
        let cache_path = PathBuf::from(CACHE_DIR).join(&filename);

        fs::create_dir_all(CACHE_DIR).ok();

        let mut cache_file = File::create(&cache_path)?;
        cache_file.write_all(data)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_http_uri() {
        assert_eq!(
            HttpSourceAssetReader::Http
                .make_uri(Path::new("example.com/favicon.png"))
                .to_str()
                .unwrap(),
            "http://example.com/favicon.png"
        );
    }

    #[test]
    fn make_https_uri() {
        assert_eq!(
            HttpSourceAssetReader::Https
                .make_uri(Path::new("example.com/favicon.png"))
                .to_str()
                .unwrap(),
            "https://example.com/favicon.png"
        );
    }

    #[test]
    fn make_http_meta_uri() {
        assert_eq!(
            HttpSourceAssetReader::Http
                .make_meta_uri(Path::new("example.com/favicon.png"))
                .expect("cannot create meta uri")
                .to_str()
                .unwrap(),
            "http://example.com/favicon.png.meta"
        );
    }

    #[test]
    fn make_https_meta_uri() {
        assert_eq!(
            HttpSourceAssetReader::Https
                .make_meta_uri(Path::new("example.com/favicon.png"))
                .expect("cannot create meta uri")
                .to_str()
                .unwrap(),
            "https://example.com/favicon.png.meta"
        );
    }

    #[test]
    fn make_https_without_extension_meta_uri() {
        assert_eq!(
            HttpSourceAssetReader::Https.make_meta_uri(Path::new("example.com/favicon")),
            None
        );
    }
}
