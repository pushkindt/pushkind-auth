//! Helpers for serving compiled frontend assets and resolving entrypoints.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const FRONTEND_ASSET_PREFIX: &str = "/assets/dist";

#[derive(Clone, Debug, Deserialize)]
struct ManifestEntry {
    file: String,
    #[serde(default)]
    css: Vec<String>,
}

/// CSS and JS files required to render a frontend entrypoint.
#[derive(Clone, Debug, Serialize)]
pub struct FrontendAssets {
    pub scripts: Vec<String>,
    pub styles: Vec<String>,
}

/// Mount node layout used by the direct React HTML shell.
#[derive(Clone, Copy, Debug)]
pub enum FrontendMountLayout {
    Bare,
    Container,
}

/// Errors raised while reading or resolving the frontend asset manifest.
#[derive(Debug, Error)]
pub enum FrontendAssetError {
    #[error("failed to read frontend manifest: {0}")]
    Read(#[from] std::io::Error),
    #[error("failed to parse frontend manifest: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("frontend entry `{0}` was not found in manifest")]
    MissingEntry(String),
    #[error("failed to serialize frontend bootstrap payload: {0}")]
    Bootstrap(serde_json::Error),
}

/// Manifest-backed frontend asset resolver used by routes and templates.
#[derive(Clone, Debug)]
pub struct FrontendAssetManifest {
    entries: HashMap<String, ManifestEntry>,
}

impl FrontendAssetManifest {
    /// Load a frontend manifest from disk.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, FrontendAssetError> {
        let contents = fs::read_to_string(path)?;
        let entries = serde_json::from_str(&contents)?;
        Ok(Self { entries })
    }

    /// Resolve the JS and CSS files for a Vite entrypoint.
    pub fn assets_for(&self, entry: &str) -> Result<FrontendAssets, FrontendAssetError> {
        let manifest_entry = self
            .entries
            .get(entry)
            .ok_or_else(|| FrontendAssetError::MissingEntry(entry.to_string()))?;

        Ok(FrontendAssets {
            scripts: vec![format!("{FRONTEND_ASSET_PREFIX}/{}", manifest_entry.file)],
            styles: manifest_entry
                .css
                .iter()
                .map(|path| format!("{FRONTEND_ASSET_PREFIX}/{path}"))
                .collect(),
        })
    }
}

/// Render the shared HTML document used to mount a React-owned page.
pub fn render_frontend_page<T: Serialize>(
    assets: &FrontendAssets,
    bootstrap: &T,
    mount_layout: FrontendMountLayout,
) -> Result<HttpResponse, FrontendAssetError> {
    let bootstrap_json = serde_json::to_string(bootstrap)
        .map(|json| json.replace('<', "\\u003c"))
        .map_err(FrontendAssetError::Bootstrap)?;

    let mut styles = String::new();
    for style in &assets.styles {
        let escaped = escape_html(style);
        let _ = writeln!(styles, r#"    <link rel="stylesheet" href="{escaped}">"#);
    }

    let mut scripts = String::new();
    for script in &assets.scripts {
        let escaped = escape_html(script);
        let _ = writeln!(
            scripts,
            r#"    <script type="module" src="{escaped}"></script>"#
        );
    }

    let mount_node = match mount_layout {
        FrontendMountLayout::Bare => r#"    <div id="react-root"></div>"#,
        FrontendMountLayout::Container => {
            r#"    <div class="container"><div id="react-root"></div></div>"#
        }
    };

    let html = format!(
        r#"<!doctype html>
<html lang="ru">
<head>
    <link rel="icon" href="/assets/favicon.ico" type="image/x-icon" />
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Auth</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-QWTKZyjpPEjISv5WaRU9OFeRpok6YctnYmDr5pNlyT2bRjXh0JMhjY6hW+ALEwIH" crossorigin="anonymous">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css">
{styles}
</head>
<body class="bg-light">
{mount_node}
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous"></script>
    <script id="frontend-bootstrap" type="application/json">{bootstrap_json}</script>
{scripts}</body>
</html>
"#
    );

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    use tempfile::NamedTempFile;

    #[test]
    fn resolves_assets_for_known_entry() {
        let mut manifest = NamedTempFile::new().unwrap();
        write!(
            manifest,
            r#"{{
                "src/entries/auth-signup.tsx": {{
                    "file": "auth-signup-abc123.js",
                    "css": ["auth-signup-def456.css"]
                }}
            }}"#
        )
        .unwrap();

        let manifest = FrontendAssetManifest::from_path(manifest.path()).unwrap();
        let assets = manifest.assets_for("src/entries/auth-signup.tsx").unwrap();

        assert_eq!(
            assets.scripts,
            vec!["/assets/dist/auth-signup-abc123.js".to_string()]
        );
        assert_eq!(
            assets.styles,
            vec!["/assets/dist/auth-signup-def456.css".to_string()]
        );
    }

    #[test]
    fn missing_entry_returns_error() {
        let mut manifest = NamedTempFile::new().unwrap();
        write!(manifest, "{{}}").unwrap();

        let manifest = FrontendAssetManifest::from_path(manifest.path()).unwrap();
        let error = manifest.assets_for("missing-entry").unwrap_err();

        assert!(matches!(
            error,
            FrontendAssetError::MissingEntry(ref entry) if entry == "missing-entry"
        ));
    }

    #[test]
    fn renders_frontend_page_without_htmx() {
        let response = render_frontend_page(
            &FrontendAssets {
                scripts: vec!["/assets/dist/app.js".to_string()],
                styles: vec!["/assets/dist/app.css".to_string()],
            },
            &serde_json::json!({ "ok": true }),
            FrontendMountLayout::Bare,
        )
        .unwrap();

        let body =
            actix_web::rt::System::new().block_on(actix_web::body::to_bytes(response.into_body()));
        let body = body.unwrap();
        let body = std::str::from_utf8(&body).unwrap();

        assert!(body.contains(r#"<div id="react-root"></div>"#));
        assert!(body.contains("/assets/dist/app.js"));
        assert!(!body.contains("htmx"));
    }
}
