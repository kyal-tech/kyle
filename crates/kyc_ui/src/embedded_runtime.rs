/// Embedded JS runtime files — compiled into the binary at build time.
/// This avoids needing to find runtime files at runtime (e.g. in installed binaries).

pub const REACTIVITY_JS: &str = include_str!("runtime/reactivity.js");
pub const ROUTER_JS: &str = include_str!("runtime/router.js");
pub const A11Y_JS: &str = include_str!("runtime/a11y.js");
pub const PORTAL_JS: &str = include_str!("runtime/portal.js");
pub const ERROR_BOUNDARY_JS: &str = include_str!("runtime/error_boundary.js");
pub const I18N_JS: &str = include_str!("runtime/i18n.js");
pub const SSR_JS: &str = include_str!("runtime/ssr.js");
pub const TESTING_JS: &str = include_str!("runtime/testing.js");
pub const GLUE_JS: &str = include_str!("runtime/glue.js");

pub const RUNTIME_FILES: &[(&str, &str)] = &[
    ("reactivity.js", REACTIVITY_JS),
    ("router.js", ROUTER_JS),
    ("a11y.js", A11Y_JS),
    ("portal.js", PORTAL_JS),
    ("error_boundary.js", ERROR_BOUNDARY_JS),
    ("i18n.js", I18N_JS),
    ("ssr.js", SSR_JS),
    ("testing.js", TESTING_JS),
    ("glue.js", GLUE_JS),
];

/// Write all runtime JS files to a directory.
pub fn write_runtime_files(dir: &std::path::Path) -> std::io::Result<()> {
    for (name, content) in RUNTIME_FILES {
        let dst = dir.join(name);
        std::fs::write(&dst, content)?;
    }
    Ok(())
}
