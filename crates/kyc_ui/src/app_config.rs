/// App configuration extracted from the entry point (.ky / .kyx) file.
/// This provides platform-agnostic config with target-specific overrides.

#[derive(Clone, Debug, Default)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub web: WebConfig,
    pub desktop: DesktopConfig,
    pub android: AndroidConfig,
    pub ios: IosConfig,
}

#[derive(Clone, Debug)]
pub struct WebConfig {
    pub port: u16,
    pub title: String,
    pub index_template: Option<String>,
    pub host: String,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            title: "Kyle App".to_string(),
            index_template: None,
            host: "127.0.0.1".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DesktopConfig {
    pub window_title: String,
    pub window_width: u32,
    pub window_height: u32,
    pub resizable: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AndroidConfig {
    pub package: String,
    pub min_sdk: u32,
    pub target_sdk: u32,
}

#[derive(Clone, Debug, Default)]
pub struct IosConfig {
    pub deployment_target: String,
    pub bundle_id: String,
}

/// Parse a config block from source (simple convention-based parsing).
/// Looks for lines like:
///   port = 8080
///   title = "My App"
///   name = "MyApp"
pub fn parse_config(source: &str) -> AppConfig {
    let mut config = AppConfig::default();

    for line in source.lines() {
        let line = line.trim();
        // Skip comments and non-assignment lines
        if line.is_empty() || line.starts_with('#') || !line.contains('=') {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            continue;
        }
        let key = parts[0].trim();
        let value = parts[1].trim().trim_matches('"');

        match key {
            "port" => {
                if let Ok(p) = value.parse::<u16>() {
                    config.web.port = p;
                }
            }
            "title" => {
                config.web.title = value.to_string();
                if config.desktop.window_title.is_empty() {
                    config.desktop.window_title = value.to_string();
                }
            }
            "name" => config.name = value.to_string(),
            "version" => config.version = value.to_string(),
            "host" => config.web.host = value.to_string(),
            "window_title" => config.desktop.window_title = value.to_string(),
            "window_width" | "width" => {
                if let Ok(w) = value.parse::<u32>() {
                    config.desktop.window_width = w;
                }
            }
            "window_height" | "height" => {
                if let Ok(h) = value.parse::<u32>() {
                    config.desktop.window_height = h;
                }
            }
            "package" => config.android.package = value.to_string(),
            "min_sdk" => {
                if let Ok(v) = value.parse::<u32>() {
                    config.android.min_sdk = v;
                }
            }
            "target_sdk" => {
                if let Ok(v) = value.parse::<u32>() {
                    config.android.target_sdk = v;
                }
            }
            "bundle_id" => config.ios.bundle_id = value.to_string(),
            "deployment_target" => config.ios.deployment_target = value.to_string(),
            _ => {}
        }
    }

    config
}

/// Detect if a source file has UI-related content (kyx or UI config).
pub fn has_ui_content(source: &str) -> bool {
    source.contains("view(") || source.contains("<view") || source.contains("<app")
        || source.contains("style<") || source.contains("animation<")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_port() {
        let source = "port = 9090";
        let config = parse_config(source);
        assert_eq!(config.web.port, 9090);
    }

    #[test]
    fn test_parse_title() {
        let source = r#"title = "Mi App""#;
        let config = parse_config(source);
        assert_eq!(config.web.title, "Mi App");
    }

    #[test]
    fn test_parse_multiple() {
        let source = r#"
port = 3000
title = "Test"
name = "MyProject"
version = "1.0.0"
"#;
        let config = parse_config(source);
        assert_eq!(config.web.port, 3000);
        assert_eq!(config.web.title, "Test");
        assert_eq!(config.name, "MyProject");
        assert_eq!(config.version, "1.0.0");
    }

    #[test]
    fn test_has_ui_content() {
        assert!(has_ui_content("<view>"));
        assert!(has_ui_content("view(\"/\")"));
        assert!(has_ui_content("style<button>"));
        assert!(!has_ui_content("fn main():\n    println(\"hello\")"));
    }
}
