pub mod web;
pub mod desktop;
pub mod ios;

use crate::ir::*;

/// A generated file as output of a UI backend
#[derive(Clone, Debug)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
}

/// Output of a UI backend translation
#[derive(Clone, Debug)]
pub struct BackendOutput {
    /// Generated source files (JS, HTML, XML, etc.)
    pub files: Vec<GeneratedFile>,
    /// Optional HTML shell for web target
    pub html_shell: Option<String>,
}

/// Platform backend: translates UI-IR to platform-specific code.
pub trait UiBackend {
    /// Backend name (e.g. "web", "desktop", "android")
    fn name(&self) -> &str;

    /// LLVM target triple (e.g. "wasm32-unknown-unknown", "x86_64-unknown-linux-gnu")
    fn target_triple(&self) -> &str;

    /// Translate UI-IR to platform code
    fn generate(&self, program: &UiProgram) -> BackendOutput;
}

/// Get a backend by name. Returns None if unknown.
pub fn get_backend(name: &str) -> Option<Box<dyn UiBackend>> {
    match name {
        "web" | "wasm32" => Some(Box::new(web::WebBackend::new())),
        "desktop" | "native" => Some(Box::new(desktop::DesktopBackend::new())),
        "ios" | "iphone" | "ios-app" => Some(Box::new(ios::IosBackend::new())),
        _ => None,
    }
}

/// All registered backend names
pub fn available_backends() -> Vec<&'static str> {
    vec!["web (wasm32-unknown-unknown)", "desktop (native)", "ios (aarch64-apple-ios)"]
}
