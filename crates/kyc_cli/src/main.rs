use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use kyc_core::resolver::RegistryBackend;
use kyc_tools::package::{
    Manifest, LockFile, find_project_root, main_source_path,
    cache,
    registry::{FileRegistry, RegistryClient},
};

fn bin_name() -> String {
    "ky".to_string()
}

fn exe_path(path: &Path) -> PathBuf {
    let ext = std::env::consts::EXE_EXTENSION;
    if ext.is_empty() { path.to_path_buf() } else { path.with_extension(ext) }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "build" => cmd_build(&args),
        "run" => cmd_run(&args),
        "check" => cmd_check(&args),
        "serve" => cmd_serve(&args),
        "parse" => cmd_parse(&args),
        "mir" => cmd_mir(&args),
        "test" => cmd_test(&args),
        "fmt" => cmd_fmt(&args),
        "new" | "init" => cmd_new(&args),
        "add" => cmd_add(&args),
        "remove" => cmd_remove(&args),
        "install" => cmd_install(&args),
        "info" => cmd_info(&args),
        "publish" => cmd_publish(&args),
        "login" => cmd_login(&args),
        "update" => cmd_update(&args),
        "outdated" => cmd_outdated(&args),
        "uninstall" => cmd_uninstall(),
        "lsp" => cmd_lsp(&args),
        "completions" => cmd_completions(&args),
        "_complete" => cmd_complete(&args),
        "help" | "-h" => print_usage(),
        "--version" | "-v" => {
            println!("{} v{}", bin_name(), env!("CARGO_PKG_VERSION"));
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}

// ── Dependency Resolution ──

fn resolve_project_dependencies(project_root: &Path, manifest: &Manifest) -> Result<(), String> {
    if manifest.dependencies.is_empty() {
        return Ok(());
    }

    println!("Resolving dependencies...");

    let registry_url = std::env::var("KL_REGISTRY").unwrap_or_default();
    let graph = if registry_url.starts_with("file://") {
        let path = std::path::PathBuf::from(registry_url.strip_prefix("file://").unwrap_or(""));
        let registry = FileRegistry::new(&path);
        manifest.resolve_dependencies(&registry)?
    } else {
        let registry = RegistryClient::new();
        manifest.resolve_dependencies(&registry)?
    };

    if !graph.conflicts.is_empty() {
        for conflict in &graph.conflicts {
            eprintln!("Conflict: {}", conflict);
        }
        return Err("Dependency resolution failed due to conflicts".to_string());
    }

    cache::ensure_cache_dir()?;
    let source = "registry";

    for name in &graph.order {
        if let Some(version) = graph.version_str(name) {
            if !cache::is_cached(name, &version) {
                print!("  Downloading {} v{} ... ", name, version);
                let data = kyc_tools::package::registry::download_package(name, &version)?;
                let dest = cache::package_cache_dir(name, &version);
                cache::extract_tarball(&data, &dest)?;
                println!("done");
            }
            install_package_to_std(project_root, name, &version)?;
        }
    }

    let lock_path = project_root.join("ky.lock");
    let mut lock = LockFile::read(&lock_path).unwrap_or_default();
    lock.update_from_graph(&graph, source);
    lock.write(&lock_path)?;

    println!("Dependencies resolved ({} packages)", graph.packages.len());
    Ok(())
}

fn resolve_and_check(project_root: &Path) {
    if let Ok(manifest) = Manifest::find_in_directory(project_root) {
        if let Err(errors) = manifest.validate() {
            for err in &errors {
                eprintln!("Warning: ky.toml: {}", err);
            }
        }
        if !manifest.dependencies.is_empty() {
            if let Err(e) = resolve_project_dependencies(project_root, &manifest) {
                eprintln!("Dependency resolution error: {}", e);
                process::exit(1);
            }
        }
    }
}

/// Copy a package's main source file to the project's std/<name>.ky.
fn install_package_to_std(project_root: &Path, name: &str, version: &str) -> Result<(), String> {
    let src_dir = cache::package_src_dir(name, version);
    if !src_dir.exists() {
        return Err(format!("Package '{}' not found in cache at {}", name, src_dir.display()));
    }
    let pkg_dir = project_root.join("std").join(name);
    fs::create_dir_all(&pkg_dir)
        .map_err(|e| format!("Failed to create std/{name} dir: {}", e))?;
    copy_dir(&src_dir, &pkg_dir)?;
    println!("  Installed to std/{name}/");
    Ok(())
}

// ── Command Implementations ──

fn is_kyx_file(file: &str) -> bool {
    Path::new(file).extension().map_or(false, |ext| ext == "kyx")
}

/// Extract the first non-flag argument from args, skipping values of known flags.
fn first_file_arg(args: &[String]) -> Option<String> {
    let mut skip_next = false;
    for a in args.iter().skip(2) {
        if skip_next { skip_next = false; continue; }
        if a.starts_with("--") {
            skip_next = matches!(a.as_str(), "--target" | "--port" | "--host");
            continue;
        }
        if *a == "--release" { continue; }
        return Some(a.clone());
    }
    None
}

/// Extract the build target from args. Checks both positional arg (web/desktop/ios) and --target flag.
fn build_target(args: &[String]) -> Option<String> {
    // Check ALL positional args for target names (any position)
    for a in args.iter().skip(2) {
        match a.as_str() {
            "web" | "desktop" | "ios" | "native" | "freestanding" => return Some(a.clone()),
            _ => {}
        }
    }
    // Fallback to --target flag
    args.iter().position(|a| a == "--target")
        .and_then(|i| args.get(i + 1)).cloned()
}

fn cmd_build(args: &[String]) {
    let release = args.iter().any(|a| a == "--release");
    let target = build_target(args);
    let is_desktop = target.as_deref() == Some("desktop") || target.as_deref() == Some("native");
    let is_ios = target.as_deref() == Some("ios") || target.as_deref() == Some("iphone");
    let is_web = target.as_deref() == Some("web") || target.as_deref() == Some("wasm32");
    let is_freestanding = target.as_deref() == Some("freestanding");
    let ui_backend_name = if is_ios { Some("ios") } else if is_desktop { Some("desktop") } else if is_web { Some("web") } else { None };
    let file_arg = first_file_arg(args);

    // In project mode, filter out the target keyword from file_arg
    let file_arg = if let Some(ref f) = file_arg {
        match f.as_str() {
            "web" | "desktop" | "ios" | "native" | "freestanding" => {
                // Find next non-flag, non-target arg
                args.iter().skip(2).filter(|a| {
                    !a.starts_with("--") && *a != "--release"
                        && *a != "web" && *a != "desktop" && *a != "ios" && *a != "native" && *a != "freestanding"
                }).next().cloned()
            }
            _ => file_arg
        }
    } else { None };

    if file_arg.is_none() {
        // Project mode
        let project_root = match find_project_root(&std::env::current_dir().unwrap()) {
            Some(r) => r,
            None => { eprintln!("No ky.toml found"); process::exit(1); }
        };
        resolve_and_check(&project_root);

        if let Some(source_path) = main_source_path(&project_root) {
            let source = fs::read_to_string(&source_path).unwrap_or_else(|e| {
                eprintln!("Error reading {}: {}", source_path.display(), e);
                process::exit(1);
            });
            let file = source_path.to_string_lossy().to_string();
            let build_dir = project_root.join("target").join(if release { "release" } else { "debug" });
            let _ = fs::create_dir_all(&build_dir);

            if is_freestanding {
                // Freestanding: compile raw .ky to kernel binary (no UI, no runtime)
                let output = exe_path(&build_dir.join("kernel"));
                let build_result = if release {
                    kyc_driver::pipeline::Pipeline::build_source_with_artifacts_release_target(&source, &file, &output, &build_dir, Some("freestanding"))
                } else {
                    kyc_driver::pipeline::Pipeline::build_source_with_artifacts_target(&source, &file, &output, &build_dir, Some("freestanding"))
                };
                match build_result {
                    Ok(()) => println!("Build complete (freestanding): {}", output.display()),
                    Err(e) => { eprintln!("Build error: {}", e); process::exit(1); }
                }
            } else if let Some(backend_name) = ui_backend_name {
                if is_kyx_file(&file) {
                    build_ui_backend_kyx(&source, &file, &build_dir, backend_name, release);
                }
            } else if is_kyx_file(&file) {
                let js_output = build_dir.join("main.js");
                let src_path = std::path::Path::new(&file);
                match kyc_driver::pipeline::Pipeline::build_kyx_source(&source, Some(src_path), &js_output) {
                    Ok(()) => {}
                    Err(e) => { eprintln!("Build error: {}", e); process::exit(1); }
                }
            } else {
                let output = exe_path(&build_dir.join("main"));
                let build_result = if release {
                    kyc_driver::pipeline::Pipeline::build_source_with_artifacts_release_target(&source, &file, &output, &build_dir, target.as_deref())
                } else {
                    kyc_driver::pipeline::Pipeline::build_source_with_artifacts_target(&source, &file, &output, &build_dir, target.as_deref())
                };
                match build_result {
                    Ok(()) => println!("Build complete: {}", output.display()),
                    Err(e) => { eprintln!("Build error: {}", e); process::exit(1); }
                }
            }
        } else {
            eprintln!("No src/main.ky or src/main.kyx found in project");
            process::exit(1);
        }
        return;
    }
    // Single-file mode: also resolve project dependencies if in a project
    if let Some(project_root) = find_project_root(&std::env::current_dir().unwrap()) {
        resolve_and_check(&project_root);
    }

    let file = file_arg.unwrap();
    let file_idx = args.iter().position(|a| *a == file).unwrap();
    let source = load_source(args, file_idx);
    let file_stem = Path::new(&file).file_stem().unwrap_or_default().to_string_lossy().to_string();
    let build_dir = Path::new("target").join(if release { "release" } else { "debug" });
    let _ = fs::create_dir_all(&build_dir);

    if let Some(backend_name) = ui_backend_name {
        if is_kyx_file(&file) {
            build_ui_backend_kyx(&source, &file, &build_dir, backend_name, release);
        }
    } else if is_kyx_file(&file) {
        let js_output = build_dir.join(&file_stem).with_extension("js");
        let src_path = std::path::Path::new(&file);
        match kyc_driver::pipeline::Pipeline::build_kyx_source(&source, Some(src_path), &js_output) {
            Ok(()) => {}
            Err(e) => { eprintln!("Build error: {}", e); process::exit(1); }
        }
    } else {
        let output = exe_path(&build_dir.join(&file_stem));
        let build_result = if release {
            kyc_driver::pipeline::Pipeline::build_source_with_artifacts_release_target(&source, &file, &output, &build_dir, target.as_deref())
        } else {
            kyc_driver::pipeline::Pipeline::build_source_with_artifacts_target(&source, &file, &output, &build_dir, target.as_deref())
        };
        match build_result {
            Ok(()) => println!("Build complete: {}", output.display()),
            Err(e) => { eprintln!("Build error: {}", e); process::exit(1); }
        }
    }
}

/// Build a .kyx file using a named UI backend (desktop, ios, etc.)
fn build_ui_backend_kyx(source: &str, file: &str, build_dir: &Path, backend_name: &str, release: bool) {
    // Use multi-file resolution
    let source_path = std::path::Path::new(file);
    let program = match kyc_ui::resolver::build_multifile_program(source, source_path) {
        Ok(p) => p,
        Err(e) => { eprintln!("kyx build error: {}", e); process::exit(1); }
    };
    let backend = match kyc_ui::backend::get_backend(backend_name) {
        Some(b) => b,
        None => { eprintln!("Backend '{}' not available", backend_name); process::exit(1); }
    };
    let output_files = backend.generate(&program);

    // Write generated files
    for gen_file in &output_files.files {
        let dst = if gen_file.path.starts_with("ios-app/") {
            build_dir.join(&gen_file.path)
        } else {
            build_dir.join(&gen_file.path)
        };
        if let Some(parent) = dst.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::write(&dst, &gen_file.content) {
            eprintln!("Error writing {}: {}", gen_file.path, e);
            process::exit(1);
        }
    }

    // For web, write runtime JS files from embedded content
    if backend_name == "web" || backend_name == "wasm32" {
        if let Err(e) = kyc_ui::embedded_runtime::write_runtime_files(build_dir) {
            eprintln!("Error writing runtime files: {}", e);
        }
        // Write HTML shell
        if let Some(html) = &output_files.html_shell {
            if let Err(e) = fs::write(&build_dir.join("index.html"), html) {
                eprintln!("Error writing index.html: {}", e);
            }
        }
        println!("Build complete (web target)");
    } else if backend_name == "desktop" || backend_name == "native" {
        let main_ky_path = build_dir.join("main.ky");
        let main_ky_source = match fs::read_to_string(&main_ky_path) {
            Ok(s) => s,
            Err(e) => { eprintln!("Error reading generated main.ky: {}", e); process::exit(1); }
        };
        let output = exe_path(&build_dir.join("main"));
        let build_result = if release {
            kyc_driver::pipeline::Pipeline::build_source_with_artifacts_release_target(
                &main_ky_source, &main_ky_path.to_string_lossy(), &output, build_dir, None)
        } else {
            kyc_driver::pipeline::Pipeline::build_source_with_artifacts_target(
                &main_ky_source, &main_ky_path.to_string_lossy(), &output, build_dir, None)
        };
        match build_result {
            Ok(()) => println!("Build complete: {} (desktop target)", output.display()),
            Err(e) => { eprintln!("Desktop build error: {}", e); process::exit(1); }
        }
    } else if backend_name == "ios" {
        let ios_dir = build_dir.join("ios-app");
        println!("Generated iOS project at: {}", ios_dir.display());
        println!("To build:");
        println!("  cd {} && swift build", ios_dir.display());
        println!("  cd {} && xcodebuild -scheme KyleApp -destination 'platform=iOS Simulator,name=iPhone 15' build", ios_dir.display());
    } else {
        println!("Build complete ({} target)", backend_name);
    }
}

fn cmd_run(args: &[String]) {
    let release = args.iter().any(|a| a == "--release");
    let target = build_target(args);
    let is_desktop = target.as_deref() == Some("desktop") || target.as_deref() == Some("native");
    let is_ios = target.as_deref() == Some("ios") || target.as_deref() == Some("iphone");
    let is_web = target.as_deref() == Some("web") || target.as_deref() == Some("wasm32");
    let ui_backend_name = if is_ios { Some("ios") } else if is_desktop { Some("desktop") } else { None };
    let port = args.iter().position(|a| a == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);

    // Filter out target name from file arguments
    let is_target_word = |a: &str| matches!(a, "web" | "desktop" | "ios" | "native" | "wasm32");
    let file_arg = args.iter().skip(2).find(|a| {
        !a.starts_with("--") && *a != "--release" && !is_target_word(a)
    }).cloned();

    if let Some(file) = &file_arg {
        // Single-file mode
        if let Some(project_root) = find_project_root(&std::env::current_dir().unwrap()) {
            resolve_and_check(&project_root);
        }
        let file_idx = args.iter().position(|a| a == file).unwrap();
        let source = load_source(args, file_idx);

        if is_web {
            run_dev_server(&source, file, port, true);
        } else if let Some(backend_name) = ui_backend_name {
            if is_kyx_file(file) {
                run_ui_app(&source, file, backend_name, release);
            }
        } else if is_kyx_file(file) || kyc_ui::app_config::has_ui_content(&source) {
            run_dev_server(&source, file, port, true);
        } else {
            compile_and_run_native(&source, file, &args, release, target.as_deref());
        }
    } else {
        // Project mode
        let project_root = match find_project_root(&std::env::current_dir().unwrap()) {
            Some(r) => r,
            None => { eprintln!("No ky.toml found"); process::exit(1); }
        };
        resolve_and_check(&project_root);

        if let Some(source_path) = main_source_path(&project_root) {
            let source = fs::read_to_string(&source_path).unwrap_or_else(|e| {
                eprintln!("Error reading {}: {}", source_path.display(), e);
                process::exit(1);
            });
            let file = source_path.to_string_lossy().to_string();

            if is_web {
                run_dev_server_for_project(&project_root, &source, &file, port);
            } else if let Some(backend_name) = ui_backend_name {
                if is_kyx_file(&file) {
                    run_ui_app(&source, &file, backend_name, release);
                }
            } else if is_kyx_file(&file) || kyc_ui::app_config::has_ui_content(&source) {
                run_dev_server_for_project(&project_root, &source, &file, port);
            } else {
                let build_dir = project_root.join("target").join(if release { "release" } else { "debug" });
                let _ = fs::create_dir_all(&build_dir);
                let output = exe_path(&build_dir.join("main"));
                let run_result = if release {
                    kyc_driver::pipeline::Pipeline::build_source_with_artifacts_release_target(&source, &file, &output, &build_dir, None)
                } else {
                    kyc_driver::pipeline::Pipeline::build_source_with_artifacts_target(&source, &file, &output, &build_dir, None)
                };
                match run_result {
                    Ok(()) => {
                        let status = process::Command::new(&output)
                            .args(args.iter().skip(2))
                            .status()
                            .expect("Failed to execute binary");
                        if !status.success() {
                            process::exit(status.code().unwrap_or(1));
                        }
                    }
                    Err(e) => { eprintln!("Run error: {}", e); process::exit(1); }
                }
            }
        } else {
            eprintln!("No src/main.ky or src/main.kyx found in project");
            process::exit(1);
        }
    }
}

/// Build and run a .kyx file as a desktop/ios application
fn run_ui_app(source: &str, file: &str, backend_name: &str, release: bool) {
    let build_dir = Path::new("target").join(if release { "release" } else { "debug" });
    let _ = fs::create_dir_all(&build_dir);

    build_ui_backend_kyx(source, file, &build_dir, backend_name, release);

    if backend_name == "desktop" || backend_name == "native" {
        let output = exe_path(&build_dir.join("main"));
        let status = process::Command::new(&output)
            .status()
            .expect("Failed to execute desktop app");
        if !status.success() {
            process::exit(status.code().unwrap_or(1));
        }
    }
}

fn compile_and_run_native(source: &str, file: &str, args: &[String], release: bool, target: Option<&str>) {
    let file_stem = Path::new(file).file_stem().unwrap_or_default().to_string_lossy().to_string();
    let build_dir = Path::new("target").join(if release { "release" } else { "debug" });
    let _ = fs::create_dir_all(&build_dir);
    let output = exe_path(&build_dir.join(&file_stem));
    let run_result = if release {
        kyc_driver::pipeline::Pipeline::build_source_with_artifacts_release_target(source, file, &output, &build_dir, target)
    } else {
        kyc_driver::pipeline::Pipeline::build_source_with_artifacts_target(source, file, &output, &build_dir, target)
    };
    match run_result {
        Ok(()) => {
            let status = process::Command::new(&output)
                .args(args.iter().skip(3))
                .status()
                .expect("Failed to execute binary");
            if !status.success() {
                process::exit(status.code().unwrap_or(1));
            }
        }
        Err(e) => { eprintln!("Run error: {}", e); process::exit(1); }
    }
}

fn run_dev_server(source: &str, file: &str, port: u16, _single_file: bool) {
    let project_root = match find_project_root(&std::env::current_dir().unwrap()) {
        Some(r) => r,
        None => {
            eprintln!("No project context found. Run from a project directory.");
            process::exit(1);
        }
    };
    let build_dir = project_root.join("target").join("debug");
    let _ = fs::create_dir_all(&build_dir);
    let js_output = build_dir.join("ui-runtime.js");

    // Build kyx source to JS with multi-file resolution
    println!("Building project...");
    let source_path = Path::new(file);
    match kyc_driver::pipeline::Pipeline::build_kyx_source(source, Some(source_path), &js_output) {
        Ok(()) => {}
        Err(e) => { eprintln!("Build error: {}", e); process::exit(1); }
    }

    serve_static(port, &project_root);
}

fn run_dev_server_for_project(project_root: &Path, source: &str, _file: &str, port: u16) {
    let build_dir = project_root.join("target").join("debug");
    let _ = fs::create_dir_all(&build_dir);
    let js_output = build_dir.join("ui-runtime.js");

    println!("Building project...");
    let source_path = project_root.join("app.kyx");
    match kyc_driver::pipeline::Pipeline::build_kyx_source(source, Some(&source_path), &js_output) {
        Ok(()) => {}
        Err(e) => { eprintln!("Build error: {}", e); process::exit(1); }
    }

    serve_static(port, project_root);
}

fn serve_static(port: u16, project_root: &Path) {
    println!("Serving on http://localhost:{}", port);
    println!("Watch mode: edit src/main.kyx then refresh browser");

    let addr = format!("0.0.0.0:{}", port);
    let listener = std::net::TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("Failed to bind to port {}: {}", port, e);
        process::exit(1);
    });

    // Try to open browser
    let url = format!("http://localhost:{}", port);
    let _ = std::process::Command::new("open").arg(&url).spawn();

    println!("Press Ctrl+C to stop");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream, project_root);
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }
}

fn cmd_check(args: &[String]) {
    if args.len() < 3 || args[2].starts_with("--") {
        // Project mode
        if let Some(project_root) = find_project_root(&std::env::current_dir().unwrap()) {
            resolve_and_check(&project_root);

            if let Some(source_path) = main_source_path(&project_root) {
                let source = fs::read_to_string(&source_path).unwrap_or_else(|e| {
                    eprintln!("Error reading {}: {}", source_path.display(), e);
                    process::exit(1);
                });
                let file = source_path.to_string_lossy().to_string();
                if is_kyx_file(&file) {
                    match kyc_driver::pipeline::Pipeline::check_kyx_source(&source) {
                        Ok(()) => println!("No errors found."),
                        Err(e) => { eprintln!("Check error: {}", e); process::exit(1); }
                    }
                } else {
                    match kyc_driver::pipeline::Pipeline::check_source(&source, &file) {
                        Ok(output) => {
                            if output.analyzer.has_errors() {
                                output.analyzer.emit_diagnostics();
                                process::exit(1);
                            } else {
                                println!("No errors found.");
                            }
                        }
                        Err(e) => { eprintln!("Check error: {}", e); process::exit(1); }
                    }
                }
            } else {
                eprintln!("No src/main.ky or src/main.kyx found in project");
                process::exit(1);
            }
        } else {
            eprintln!("Usage: {} check <file.ky>", bin_name());
            process::exit(1);
        }
        return;
    }
    // Single-file mode: also resolve project dependencies if in a project
    if let Some(project_root) = find_project_root(&std::env::current_dir().unwrap()) {
        resolve_and_check(&project_root);
    }

    let source = load_source(args, 2);
    let file = &args[2];
    if is_kyx_file(file) {
        match kyc_driver::pipeline::Pipeline::check_kyx_source(&source) {
            Ok(()) => println!("No errors found."),
            Err(e) => { eprintln!("Check error: {}", e); process::exit(1); }
        }
    } else {
        match kyc_driver::pipeline::Pipeline::check_source(&source, file) {
            Ok(output) => {
                if output.analyzer.has_errors() {
                    output.analyzer.emit_diagnostics();
                    process::exit(1);
                } else {
                    println!("No errors found.");
                }
            }
            Err(e) => { eprintln!("Check error: {}", e); process::exit(1); }
        }
    }
}

fn cmd_parse(args: &[String]) {
    let source = load_source(args, 2);
    let file = &args[2];
    match kyc_driver::pipeline::Pipeline::parse_source(&source) {
        Ok(output) => {
            println!("--- AST for {} ---", file);
            println!("{}", output.program);
        }
        Err(e) => { eprintln!("Parse error: {}", e); process::exit(1); }
    }
}

fn cmd_mir(args: &[String]) {
    let source = load_source(args, 2);
    let file = &args[2];
    match kyc_driver::pipeline::Pipeline::mir_source(&source, file) {
        Ok(output) => {
            if output.analyzer.has_errors() {
                output.analyzer.emit_diagnostics();
                process::exit(1);
            }
            println!("{}", output.module);
        }
        Err(e) => { eprintln!("MIR error: {}", e); process::exit(1); }
    }
}

fn cmd_fmt(args: &[String]) {
    let check = args.iter().any(|a| a == "--check");

    // Determine project root from cwd or explicit path
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Try loading format config from ky.toml (from either cwd or the first explicit path)
    let config = {
        let root = if args.len() > 2 && !args[2].starts_with("--") {
            let first = Path::new(&args[2]);
            if first.is_file() {
                first.parent().and_then(|p| kyc_tools::package::find_project_root(p))
            } else if first.is_dir() {
                kyc_tools::package::find_project_root(first)
            } else {
                kyc_tools::package::find_project_root(&cwd)
            }
        } else {
            kyc_tools::package::find_project_root(&cwd)
        };
        if let Some(root) = root {
            let manifest_path = root.join("ky.toml");
            if manifest_path.exists() {
                if let Ok(toml_str) = fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = kyc_tools::package::Manifest::from_str(&toml_str) {
                        manifest.format
                    } else {
                        kyc_tools::package::FormatConfig::default()
                    }
                } else {
                    kyc_tools::package::FormatConfig::default()
                }
            } else {
                kyc_tools::package::FormatConfig::default()
            }
        } else {
            kyc_tools::package::FormatConfig::default()
        }
    };
    let formatter = kyc_tools::formatter::Formatter::with_config(config);
    let mut any_fail = false;

    // Collect files to format
    let files: Vec<PathBuf> = if args.len() > 2 && !args[2].starts_with("--") {
        // Explicit path(s) given
        args[2..].iter()
            .take_while(|a| !a.starts_with("--"))
            .flat_map(|p| {
                let path = Path::new(p);
                if path.is_dir() {
                    // Format all .ky files in directory
                    fs::read_dir(path).ok().into_iter().flat_map(|rd| {
                        rd.flatten().filter_map(|e| {
                            let p = e.path();
                            if p.extension().map_or(false, |ext| ext == "ky") {
                                Some(p)
                            } else {
                                None
                            }
                        }).collect::<Vec<_>>()
                    }).collect::<Vec<_>>()
                } else {
                    vec![path.to_path_buf()]
                }
            }).collect()
    } else {
        // No path — try project mode (format all in src/ and tests/)
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if let Some(root) = kyc_tools::package::find_project_root(&cwd) {
            let src_dir = root.join("src");
            let tests_dir = root.join("tests");
            let mut files = Vec::new();
            if src_dir.is_dir() {
                if let Ok(rd) = fs::read_dir(&src_dir) {
                    for entry in rd.flatten() {
                        let p = entry.path();
                        if p.extension().map_or(false, |ext| ext == "ky") {
                            files.push(p);
                        }
                    }
                }
            }
            if tests_dir.is_dir() {
                if let Ok(rd) = fs::read_dir(&tests_dir) {
                    for entry in rd.flatten() {
                        let p = entry.path();
                        if p.extension().map_or(false, |ext| ext == "ky") {
                            files.push(p);
                        }
                    }
                }
            }
            files
        } else {
            eprintln!("Error: no project found and no file specified");
            eprintln!("Usage: {} fmt [<file.ky> | <dir/>]", bin_name());
            process::exit(1);
        }
    };

    if files.is_empty() {
        eprintln!("No .ky files found to format");
        process::exit(1);
    }

    for path in &files {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading '{}': {}", path.display(), e);
                any_fail = true;
                continue;
            }
        };
        match formatter.format(&source) {
            Ok(formatted) => {
                if check {
                    if source != formatted {
                        eprintln!("{}: would reformat", path.display());
                        any_fail = true;
                    }
                } else {
                    if source != formatted {
                        fs::write(path, &formatted).unwrap_or_else(|e| {
                            eprintln!("Error writing '{}': {}", path.display(), e);
                            any_fail = true;
                        });
                        println!("Formatted: {}", path.display());
                    }
                }
            }
            Err(e) => {
                eprintln!("{}: format error: {}", path.display(), e);
                any_fail = true;
            }
        }
    }

    if any_fail {
        process::exit(1);
    }
}

fn cmd_new(args: &[String]) {
    let mut project_type = "bare";
    let mut project_name_arg: &str;

    if args.len() == 4 {
        // ky new <type> <name>
        project_type = &args[2];
        project_name_arg = &args[3];
    } else if args.len() >= 3 {
        // ky new <name>
        project_name_arg = &args[2];
        // Check if the first arg is actually a type keyword
        let first = &args[2];
        if first == "api" || first == "bare" || first == "webapp" || first == "kyui" {
            eprintln!("Error: missing project name for type '{}'", first);
            eprintln!("Usage: {} new {} <project>", bin_name(), first);
            process::exit(1);
        }
    } else {
        eprintln!("Error: missing project name");
        eprintln!("Usage: {} new [type] <project>", bin_name());
        eprintln!("Types: bare    — simple script (default)");
        eprintln!("       api     — HTTP API server project");
        eprintln!("       kyui    — UI project (web + desktop + iOS)");
        process::exit(1);
    }

    let project_dir = Path::new(project_name_arg);
    let project_name = project_dir.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if project_dir.exists() {
        eprintln!("Error: directory '{}' already exists", project_name);
        process::exit(1);
    }

    let exe_path = std::env::current_exe().ok()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "ky".to_string());

    match project_type {
        "bare" => cmd_new_bare(&project_dir, &project_name, &exe_path),
        "api" => cmd_new_api(&project_dir, &project_name, &exe_path),
        "kyui" | "webapp" => cmd_new_app(&project_dir, &project_name, &exe_path),
        _ => cmd_new_app(&project_dir, &project_name, &exe_path),
    }
}

fn cmd_new_app(project_dir: &Path, project_name: &str, exe_path: &str) {
    // Create directories: src/views, src/layouts, src/components, src
    fs::create_dir_all(project_dir.join("src").join("views")).unwrap_or_else(|e| {
        eprintln!("Error creating project: {}", e);
        process::exit(1);
    });
    fs::create_dir_all(project_dir.join("src").join("layouts")).unwrap_or_else(|e| {
        eprintln!("Error creating project: {}", e);
        process::exit(1);
    });
    fs::create_dir_all(project_dir.join("src").join("components")).unwrap_or_else(|e| {
        eprintln!("Error creating project: {}", e);
        process::exit(1);
    });
    fs::create_dir_all(project_dir.join("src")).unwrap_or_else(|e| {
        eprintln!("Error creating project: {}", e);
        process::exit(1);
    });

    // Entry point: app.kyx (root level)
    let app_kyx = format!(
        r##"# {name} — Kyle UI App
from views.home import home
from views.not_found import not_found
from layouts.main import main

<app title="{name}">
    <router>
        <route path="/" component=home layout=main title="Home" />
        <route path="*" component=not_found layout=main title="404" />
    </router>
</app>
"##,
        name = project_name
    );
    fs::write(project_dir.join("app.kyx"), &app_kyx).unwrap_or_else(|e| {
        eprintln!("Error writing app.kyx: {}", e);
        process::exit(1);
    });

    // Home view: src/views/home.kyx
    let home_kyx = format!(
        r##"# Home view
style<text> Title:
    font_size = 24
    font_weight = font_weight.bold
    color = color("#1A1A1A")

style<button> Primary:
    background = color("#0066FF")
    color = color("#FFFFFF")
    font_size = 16
    font_weight = font_weight.medium
    border_radius = 8
    padding = spacing.all(12)
    cursor = cursor.pointer

<view>
    @(
        count: ^i32 = 0
        fn increment():
            count += 1
    )

    <vstack alignment=alignment.center spacing=16>
        <text style=Title value="Bienvenido a Kyle UI!" />
        <text value=@"Contador: " + count.to_str() />
        <button style=Primary text="+" click=@increment />
    </vstack>
</view>
"##
    );
    fs::write(project_dir.join("src").join("views").join("home.kyx"), &home_kyx).unwrap_or_else(|e| {
        eprintln!("Error writing src/views/home.kyx: {}", e);
        process::exit(1);
    });

    // Not found view: src/views/not_found.kyx
    let not_found_kyx = r##"# 404 view
style<text> Display:
    font_size = 48
    font_weight = font_weight.bold
    color = color("#0066FF")

<view>
    @(set_title("404 — No Encontrado"))

    <vstack alignment=alignment.center spacing=12>
        <text value="404" style=Display />
        <text value="Página no encontrada" />
        <link to="/">Volver al inicio</link>
    </vstack>
</view>
"##;
    fs::write(project_dir.join("src").join("views").join("not_found.kyx"), &not_found_kyx).unwrap_or_else(|e| {
        eprintln!("Error writing src/views/not_found.kyx: {}", e);
        process::exit(1);
    });

    // Main layout: src/layouts/main.kyx
    let main_layout_kyx = r##"# Persistent layout with navbar
<layout>
    <navbar title="Mi App" />
    <main>
        <slot />
    </main>
    <footer>
        <text value="Kyle UI" />
    </footer>
</layout>
"##;
    fs::write(project_dir.join("src").join("layouts").join("main.kyx"), &main_layout_kyx).unwrap_or_else(|e| {
        eprintln!("Error writing src/layouts/main.kyx: {}", e);
        process::exit(1);
    });

    // Library module
    let lib_kl = format!(
        "# {} — library module\nfn greet(greeting: str, name: str) str:\n    greeting + \", \" + name + \"!\"\n",
        project_name
    );
    fs::write(project_dir.join("src").join("lib.ky"), &lib_kl).unwrap_or_else(|e| {
        eprintln!("Error writing src/lib.ky: {}", e);
        process::exit(1);
    });

    write_manifest(project_dir, project_name, "app.kyx");
    write_gitignore(project_dir);
    write_vscode_settings(project_dir, exe_path);

    println!("✅ Created kyui project '{}'", project_name);
    println!("   ├── app.kyx               — entry point");
    println!("   ├── src/views/home.kyx    — home page view");
    println!("   ├── src/views/not_found.kyx — 404 page");
    println!("   ├── src/layouts/main.kyx  — persistent layout");
    println!("   ├── src/components/        — shared components");
    println!("   ├── src/lib.ky             — library module");
    println!("   ├── ky.toml               — manifest");
    println!("   ├── .gitignore");
    println!("   └── .vscode/              — VS Code settings");
    println!();
    println!("   cd {} && ky run web      # Web (browser)", project_name);
    println!("   cd {} && ky run desktop  # Desktop (SDL2)", project_name);
    println!("   cd {} && ky run ios      # iOS (Xcode)", project_name);
}

fn cmd_new_api(project_dir: &Path, project_name: &str, exe_path: &str) {
    fs::create_dir_all(project_dir.join("src")).unwrap_or_else(|e| {
        eprintln!("Error creating project: {}", e);
        process::exit(1);
    });

    let main_kl = format!(
        "# {} — API entry point\nfrom http.server import Router\n\napp = Router()\n\napp.get(\"/ping\", (req, res):\n    res.json({{\"status\": \"ok\"}}, 200)\n)\n\napp.listen(8080)\n",
        project_name
    );
    fs::write(project_dir.join("src").join("main.ky"), &main_kl).unwrap_or_else(|e| {
        eprintln!("Error writing src/main.ky: {}", e);
        process::exit(1);
    });

    write_manifest_with_deps(project_dir, project_name, "src/main.ky", &["http", "json"]);
    write_gitignore(project_dir);
    write_vscode_settings(project_dir, exe_path);

    println!("✅ Created API project '{}'", project_name);
    println!("   ├── src/main.ky     — HTTP server");
    println!("   ├── ky.toml         — manifest");
    println!("   ├── .gitignore");
    println!("   └── .vscode/        — VS Code settings");
    println!();
    println!("   cd {} && {} run", project_name, bin_name());
    println!("   curl http://localhost:8080/ping");
}

fn cmd_new_bare(project_dir: &Path, project_name: &str, exe_path: &str) {
    fs::create_dir_all(project_dir).unwrap_or_else(|e| {
        eprintln!("Error creating project: {}", e);
        process::exit(1);
    });
    // ky.toml manifest
    let toml = format!(
        "[project]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        project_name
    );
    fs::write(project_dir.join("ky.toml"), &toml).unwrap_or_else(|e| {
        eprintln!("Error writing ky.toml: {}", e);
        process::exit(1);
    });
    let bare_kl = format!(
        "# {} — bare script\nfn main():\n    name := \"Kyle\"  # compile-time constant\n    println(\"Hello from \" + name)\n",
        project_name
    );
    let file = format!("{}.ky", project_name);
    fs::write(project_dir.join(&file), &bare_kl).unwrap_or_else(|e| {
        eprintln!("Error writing {}: {}", file, e);
        process::exit(1);
    });

    write_gitignore(project_dir);

    println!("✅ Created bare script '{}'", project_name);
    println!("   ├── ky.toml         — manifest");
    println!("   ├── {}.ky        — script", project_name);
    println!("   └── .gitignore");
    println!();
    println!("   cd {} && {} run {}.ky", project_name, bin_name(), project_name);
}

fn cmd_serve(args: &[String]) {
    eprintln!("Note: 'ky serve' is deprecated. Use 'ky run web' instead.");
    let mut run_args = vec![args[0].clone(), "run".to_string(), "web".to_string()];
    run_args.extend(args.iter().skip(2).cloned());
    cmd_run(&run_args);
}

fn handle_connection(mut stream: std::net::TcpStream, project_root: &Path) {
    use std::io::{Read, Write};
    let mut buffer = [0; 4096];
    if stream.read(&mut buffer).is_err() { return; }
    let request = String::from_utf8_lossy(&buffer[..]);
    let path = request.lines().next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/");

    // Auto-rebuild if source changed (check app.kyx or src/main.kyx)
    let source_path = if project_root.join("app.kyx").exists() {
        project_root.join("app.kyx")
    } else {
        project_root.join("src/main.kyx")
    };
    if source_path.exists() {
        if let Ok(meta) = std::fs::metadata(&source_path) {
            if let Ok(mtime) = meta.modified() {
                if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
                    use std::sync::atomic::{AtomicU64, Ordering};
                    static LAST_BUILD: AtomicU64 = AtomicU64::new(0);
                    let ms = dur.as_millis() as u64;
                    if ms > LAST_BUILD.load(Ordering::Relaxed) {
                        if let Ok(src) = std::fs::read_to_string(&source_path) {
                            let bd = project_root.join("target/debug");
                            let _ = std::fs::create_dir_all(&bd);
                            let _ = kyc_driver::pipeline::Pipeline::build_kyx_source(&src, Some(&source_path), &bd.join("ui-runtime.js"));
                            println!("  [auto-rebuild] {} changed", source_path.file_name().unwrap_or_default().to_string_lossy());
                        }
                        LAST_BUILD.store(ms, Ordering::Relaxed);
                    }
                }
            }
        }
    }

    let (status, content, mime) = if path == "/" || path == "/index.html" {
        // Try custom index.html first, then generated one, then auto-generate
        let index_path = project_root.join("index.html");
        let gen_index = project_root.join("target/debug/index.html");
        if index_path.exists() {
            let content = fs::read_to_string(&index_path).unwrap_or_default();
            (200, content, "text/html")
        } else if gen_index.exists() {
            let content = fs::read_to_string(&gen_index).unwrap_or_default();
            (200, content, "text/html")
        } else {
            let html = generate_html_shell();
            (200, html, "text/html")
        }
    } else if path == "/favicon.ico" {
        (204, String::new(), "image/x-icon")
    } else if path.starts_with("/target/debug/") {
        let file_path = project_root.join(&path[1..]);
        match fs::read(&file_path) {
            Ok(data) => {
                let mime = if path.ends_with(".js") { "application/javascript" }
                    else if path.ends_with(".css") { "text/css" }
                    else if path.ends_with(".wasm") { "application/wasm" }
                    else if path.ends_with(".html") { "text/html" }
                    else { "application/octet-stream" };
                (200, String::from_utf8_lossy(&data).to_string(), mime)
            }
            Err(_) => (404, "Not Found".to_string(), "text/plain"),
        }
    } else if path == "/ui-runtime.js" || path.ends_with(".js") {
        // Direct file serving for runtime JS files
        let file_path = project_root.join(&path[1..]);
        match fs::read_to_string(&file_path) {
            Ok(content) => (200, content, "application/javascript"),
            Err(_) => (404, "Not Found".to_string(), "text/plain"),
        }
    } else {
        (404, "Not Found".to_string(), "text/plain")
    };

    let response = format!(
        "HTTP/1.1 {status} OK\r\nContent-Type: {mime}\r\nContent-Length: {len}\r\nCache-Control: no-cache\r\nAccess-Control-Allow-Origin: *\r\n\r\n{content}",
        status = status,
        mime = mime,
        len = content.len(),
        content = content
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

/// Generate a minimal HTML shell for development (fallback).
fn generate_html_shell() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Kyle App</title>
</head>
<body>
    <div id="app"></div>
    <script type="module">
        import { render } from './target/debug/ui-runtime.js';
        const app = document.getElementById('app');
        const result = render();
        app.appendChild(result.element);
    </script>
</body>
</html>"#.to_string()
}

/// Write VS Code settings with correct ky compiler path.
fn write_vscode_settings(project_dir: &Path, exe_path: &str) {
    let vscode_dir = project_dir.join(".vscode");
    fs::create_dir_all(&vscode_dir).unwrap_or_else(|e| {
        eprintln!("Error creating .vscode: {}", e);
        process::exit(1);
    });
    // ky.kycPath must point to the compiler binary
    let settings = format!(
        r#"{{
  "ky.kycPath": "{}",
  "files.associations": {{
    "*.ky": "ky",
    "ky.toml": "json"
  }},
  "editor.semanticHighlighting.enabled": true,
  "[ky]": {{
    "editor.tabSize": 4,
    "editor.insertSpaces": true,
    "editor.formatOnSave": true
  }}
}}
"#,
        exe_path
    );
    fs::write(vscode_dir.join("settings.json"), settings).unwrap_or_else(|e| {
        eprintln!("Error writing .vscode/settings.json: {}", e);
        process::exit(1);
    });
}

fn write_manifest(project_dir: &Path, project_name: &str, main_path: &str) {
    write_manifest_with_deps(project_dir, project_name, main_path, &[])
}

fn write_manifest_with_deps(project_dir: &Path, project_name: &str, main_path: &str, deps: &[&str]) {
    let deps_str = if deps.is_empty() {
        String::new()
    } else {
        deps.iter().map(|d| format!("{} = \"*\"", d)).collect::<Vec<_>>().join("\n") + "\n"
    };
    let manifest = format!(
        "[project]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nauthors = [\"You <you@example.com>\"]\nlicense = \"MIT\"\ndescription = \"A Kyle language project\"\nmain = \"{}\"\n\n[compiler]\noptimization = \"O2\"\ntarget = \"native\"\n\n[dependencies]\n{}",
        project_name, main_path, deps_str
    );
    fs::write(project_dir.join("ky.toml"), &manifest).unwrap_or_else(|e| {
        eprintln!("Error writing ky.toml: {}", e);
        process::exit(1);
    });
}

fn write_gitignore(project_dir: &Path) {
    let gitignore = "target/\n*.kyc-build/\nky.lock\nstd/\n.vscode/\n";
    fs::write(project_dir.join(".gitignore"), gitignore).unwrap_or_else(|e| {
        eprintln!("Error writing .gitignore: {}", e);
        process::exit(1);
    });
}

fn cmd_lsp(_args: &[String]) {
    match kyc_tools::lsp::LanguageServer::new() {
        Ok(mut server) => {
            if let Err(e) = server.run() {
                eprintln!("LSP error: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to start LSP server: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_completions(args: &[String]) {
    let name = bin_name();
    let shell = if args.len() > 2 { &args[2] } else { "bash" };
    match shell {
        "bash" => print!("{}", bash_completions(&name)),
        "zsh" => print!("{}", zsh_completions(&name)),
        "fish" => print!("{}", fish_completions(&name)),
        "powershell" | "pwsh" => print!("{}", powershell_completions(&name)),
        _ => {
            eprintln!("Unknown shell '{}'. Supported: bash, zsh, fish, powershell", shell);
            process::exit(1);
        }
    }
}

/// Hidden command `kl _complete <cmd> <prefix>` — outputs completion candidates
/// for dynamic completion (e.g., package names for `ky add`).
fn cmd_complete(args: &[String]) {
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let prefix = args.get(3).map(|s| s.as_str()).unwrap_or("");
    match sub {
        "add" => {
            // Suggest cached packages matching prefix
            if let Ok(packages) = kyc_tools::package::cache::list_cached_packages() {
                for (name, _version) in &packages {
                    if name.starts_with(prefix) {
                        println!("{}", name);
                    }
                }
            }
        }
        _ => {}
    }
}

fn bash_completions(name: &str) -> String {
    format!(
        "_{name}() {{
    local cur prev words cword
    _init_completion || return
    local cmds=\"build run check test parse mir fmt new add remove info publish login update outdated lsp completions help\"
    if [[ $cword -eq 1 ]]; then
        COMPREPLY=($(compgen -W \"$cmds\" -- \"$cur\"))
    elif [[ $cword -ge 2 ]]; then
        case \"${{prev}}\" in
            build|run|check|test|parse|mir|fmt)
                COMPREPLY=($(compgen -f -X '!*.ky' -- \"$cur\"))
                ;;
            add)
                local packages=$({name} _complete add \"$cur\" 2>/dev/null)
                COMPREPLY=($(compgen -W \"$packages\" -- \"$cur\"))
                ;;
            *) ;;
        esac
    fi
}} &&
complete -F _{name} {name}
"
    )
}

fn zsh_completions(name: &str) -> String {
    format!(
        "#compdef {name}
local -a cmds
cmds=(
    'build:Compile project or file'
    'run:Compile and execute'
    'check:Type-check without codegen'
    'parse:Parse and dump AST'
    'mir:Parse and dump MIR'
    'fmt:Format source code'
    'new:Create new project'
    'add:Add dependency'
    'remove:Remove dependency'
    'info:Show project info'
    'publish:Publish package to registry'
    'login:Login to package registry'
    'update:Update lock file'
    'outdated:List outdated dependencies'
    'lsp:Start LSP server'
    'completions:Generate shell completions'
    'help:Show help'
)
_describe -t commands '{name}' cmds && ret=0

# Dynamic completion for `ky add` — suggest cached packages
_{{name}}_add() {{
  local -a packages
  packages=(${{(f)\"$({name} _complete add \\\"$words[CURRENT]\\\" 2>/dev/null)\"}})
  _describe -t packages 'package' packages
}}
compdef _{{name}}_add {name}
"
    )
}

fn fish_completions(name: &str) -> String {
    format!(
        "complete -c {name} -f -a build -d \"Compile project or file\"
complete -c {name} -f -a run -d \"Compile and execute\"
complete -c {name} -f -a check -d \"Type-check without codegen\"
complete -c {name} -f -a parse -d \"Parse and dump AST\"
complete -c {name} -f -a mir -d \"Parse and dump MIR\"
complete -c {name} -f -a fmt -d \"Format source code\"
complete -c {name} -f -a new -d \"Create new project\"
complete -c {name} -f -a add -d \"Add dependency\"
complete -c {name} -f -a remove -d \"Remove dependency\"
complete -c {name} -f -a info -d \"Show project info\"
complete -c {name} -f -a publish -d \"Publish package to registry\"
complete -c {name} -f -a login -d \"Login to package registry\"
complete -c {name} -f -a update -d \"Update lock file\"
complete -c {name} -f -a outdated -d \"List outdated dependencies\"
complete -c {name} -f -a lsp -d \"Start LSP server\"
complete -c {name} -f -a completions -d \"Generate shell completions\"
complete -c {name} -f -a help -d \"Show help\"
# Dynamic completion for `ky add` — suggest cached packages
complete -c {name} -f -n \"__fish_seen_subcommand_from add\" -a \"({name} _complete add (commandline -ct) 2>/dev/null)\" -d \"Cached package\"
"
    )
}

fn powershell_completions(name: &str) -> String {
    format!(
        "Register-ArgumentCompleter -Native -CommandName '{name}' -ScriptBlock {{
    param($wordToComplete, $commandAst, $cursorPosition)
    $commands = @(
        'build', 'run', 'check', 'test', 'parse', 'mir', 'fmt',
        'new', 'add', 'remove', 'info', 'publish', 'login',
        'update', 'outdated', 'lsp', 'completions', 'help'
    )
    $cmd = $commandAst.CommandElements[1].Value
    if ($commandAst.CommandElements.Count -eq 2) {{
        $commands | Where-Object {{ $_ -like \"$wordToComplete*\" }}
    }} elseif ($commandAst.CommandElements.Count -eq 3 -and @('build','run','check','test','parse','mir','fmt') -contains $cmd) {{
        Get-ChildItem -Filter *.ky | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{ $_.Name }}
    }} elseif ($commandAst.CommandElements.Count -eq 3 -and $cmd -eq 'add') {{
        # Suggest cached packages
        $home = if ($env:KY_HOME) {{ \"$env:KY_HOME/cache\" }} else {{ \"$env:USERPROFILE\\.kl\\cache\" }}
        if (Test-Path $home) {{
            Get-ChildItem $home -Directory | ForEach-Object {{ $_.Name -replace '-[0-9]+\\\\.[0-9]+\\\\.[0-9]+$', '' }} | Sort-Object -Unique | Where-Object {{ $_ -like \"$wordToComplete*\" }}
        }}
    }}
}}
"
    )
}

fn print_usage() {
    let name = bin_name();
    eprintln!("{} v{} — Kyle Programming Language Compiler", name, env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("Project commands (run from a project directory with ky.toml):");
    eprintln!("  {name} build [web|desktop|ios] [--release]   Build for platform");
    eprintln!("  {name} run   [web|desktop|ios] [--port N]    Build and run for platform");
    eprintln!("  {name} test                Run project tests");
    eprintln!("  {name} info                Show project info");
    eprintln!("  {name} add   <dep>[@<ver>] Add dependency");
    eprintln!("  {name} remove <dep>        Remove dependency");
    eprintln!("  {name} install             Install all dependencies from ky.lock");
    eprintln!("  {name} update              Update lock file to latest compatible versions");
    eprintln!("  {name} outdated            List outdated dependencies");
    eprintln!("  {name} publish             Publish project to registry");
    eprintln!("  {name} login               Login to package registry");
    eprintln!();
    eprintln!("File commands:");
    eprintln!("  {name} build <file.ky> [--target <triple>]     Compile single file");
    eprintln!("  {name} run   <file.kyx>                        Compile and serve UI project");
    eprintln!("  {name} run   <file.ky>  [--target <triple>]    Compile and run single file");
    eprintln!("  {name} check <file.ky>     Type-check without codegen");
    eprintln!("  {name} parse <file.ky>     Parse and dump AST");
    eprintln!("  {name} mir   <file.ky>     Parse and dump MIR");
    eprintln!("  {name} fmt   [file/dir]    Format source files");
    eprintln!();
    eprintln!("Project creation:");
    eprintln!("  {name} new   <project>     Create new project (default: bare template)");
    eprintln!("  {name} new   kyui <name>    Create UI project (web + desktop + iOS)");
    eprintln!("  {name} new   bare <name>    Create minimal script project");
    eprintln!("  {name} new   api <name>     Create HTTP API server project");
    eprintln!();
    eprintln!("System:");
    eprintln!("  {name} uninstall           Remove ky and LLVM files");
    eprintln!();
    eprintln!("Tools:");
    eprintln!("  {name} lsp                 Start LSP server (stdio)");
    eprintln!("  {name} completions <shell>  Generate shell completions (bash, zsh, fish, powershell)");
    eprintln!("  {name} help                Show this help");
}

fn cmd_test(args: &[String]) {
    let release = args.iter().any(|a| a == "--release");
    let project_root = match find_project_root(&std::env::current_dir().unwrap()) {
        Some(r) => r,
        None => { eprintln!("No ky.toml found"); process::exit(1); }
    };
    resolve_and_check(&project_root);

    let test_files = kyc_tools::package::test_source_paths(&project_root);
    if test_files.is_empty() {
        println!("No test files found in tests/");
        println!("All checks passed.");
        return;
    }

    let mut total = 0u32;
    let mut passed = 0u32;
    let mut failed: Vec<String> = Vec::new();

    let build_dir = project_root.join("target").join(if release { "release" } else { "debug" });
    let _ = fs::create_dir_all(&build_dir);

    for test_file_path in &test_files {
        let source = fs::read_to_string(test_file_path).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {}", test_file_path.display(), e);
            process::exit(1);
        });
        let file_name = test_file_path.file_stem().unwrap_or_default().to_string_lossy().to_string();

        // Parse to find #[test] functions
        let parsed = match kyc_driver::pipeline::Pipeline::parse_source(&source) {
            Ok(p) => p,
            Err(e) => { eprintln!("Parse error in {}: {}", test_file_path.display(), e); process::exit(1); }
        };

        // Collect #[test] functions
        let test_fns: Vec<&kyc_core::ast::FunctionDecl> = parsed.program.declarations.iter()
            .filter_map(|d| {
                if let kyc_core::ast::Decl::Function(f) = d {
                    if f.is_test || f.is_bench { Some(f) } else { None }
                } else {
                    None
                }
            })
            .collect();

        if test_fns.is_empty() {
            println!("  {}: no #[test] functions", test_file_path.file_name().unwrap_or_default().to_string_lossy());
            continue;
        }

        for test_fn in &test_fns {
            total += 1;
            let test_name = format!("{}::{}", file_name, test_fn.name);

            // Validate test function signature
            if !test_fn.params.is_empty() {
                eprintln!("  FAIL {}: test function must have no parameters", test_name);
                failed.push(test_name);
                continue;
            }

            // Generate wrapper: original source + test main
            let wrapper = format!(
                "{}\nfn main() i32:\n    {}()\n    println(\"  PASS {}\")\n    0\n",
                source, test_fn.name, test_name
            );

            print!("  test {} ... ", test_name);
            let output = exe_path(&build_dir.join(format!("test_{}", test_fn.name)));
            let build_result = if release {
                kyc_driver::pipeline::Pipeline::build_source_with_artifacts_release(&wrapper, test_file_path.to_str().unwrap(), &output, &build_dir)
            } else {
                kyc_driver::pipeline::Pipeline::build_source_with_artifacts(&wrapper, test_file_path.to_str().unwrap(), &output, &build_dir)
            };

            match build_result {
                Ok(()) => {
                    let run_result = std::process::Command::new(&output)
                        .output()
                        .map_err(|e| format!("failed to execute test binary: {}", e));

                    match run_result {
                        Ok(run_output) => {
                            if run_output.status.success() {
                                passed += 1;
                                let stdout = String::from_utf8_lossy(&run_output.stdout);
                                println!("OK");
                                print!("{}", stdout);
                            } else {
                                let stdout = String::from_utf8_lossy(&run_output.stdout);
                                let stderr = String::from_utf8_lossy(&run_output.stderr);
                                println!("FAILED");
                                if !stderr.is_empty() {
                                    eprintln!("{}", stderr);
                                }
                                if !stdout.is_empty() {
                                    println!("{}", stdout);
                                }
                                failed.push(test_name);
                                // Clean up binary
                                let _ = fs::remove_file(&output);
                            }
                        }
                        Err(e) => {
                            println!("ERROR");
                            eprintln!("  {}", e);
                            failed.push(test_name);
                        }
                    }
                }
                Err(e) => {
                    println!("BUILD FAILED");
                    eprintln!("  {}", e);
                    failed.push(test_name);
                }
            }
            // Clean up binary
            let _ = fs::remove_file(&output);
        }
    }

    println!("\nTest results: {}/{} passed", passed, total);
    if !failed.is_empty() {
        eprintln!("Failed tests:");
        for f in &failed {
            eprintln!("  - {}", f);
        }
        process::exit(1);
    }
}

fn cmd_add(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: {} add <dependency>[@<version>]", bin_name());
        process::exit(1);
    }
    let dep_str = &args[2];
    let (name, version) = if let Some(at_pos) = dep_str.find('@') {
        (&dep_str[..at_pos], &dep_str[at_pos + 1..])
    } else {
        (dep_str.as_str(), "*")
    };

    let cwd = std::env::current_dir().unwrap();
    match Manifest::find_in_cwd() {
        Ok(mut manifest) => {
            manifest.add_dependency(name, version);
            if let Err(e) = manifest.save_to_dir(&cwd) {
                eprintln!("Error saving manifest: {}", e);
                process::exit(1);
            }
            println!("Added dependency '{}' version '{}'", name, version);

            if let Some(project_root) = find_project_root(&cwd) {
                if let Err(e) = resolve_project_dependencies(&project_root, &manifest) {
                    eprintln!("Warning: could not resolve dependencies: {}", e);
                    eprintln!("Run 'ky update' to resolve later");
                }
            }
        }
        Err(e) => { eprintln!("Error: {}", e); process::exit(1); }
    }
}

/// Recursively copy a directory.
fn copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create dir '{}': {}", dst.display(), e))?;
    for entry in fs::read_dir(src)
        .map_err(|e| format!("Failed to read dir '{}': {}", src.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let file_type = entry.file_type()
            .map_err(|e| format!("Failed to get file type: {}", e))?;
        let name = entry.file_name();
        let src_path = entry.path();
        let dst_path = dst.join(&name);
        if file_type.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy '{}': {}", src_path.display(), e))?;
        }
    }
    Ok(())
}

fn cmd_remove(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: {} remove <dependency>", bin_name());
        process::exit(1);
    }
    let name = &args[2];
    match Manifest::find_in_cwd() {
        Ok(mut manifest) => {
            if manifest.remove_dependency(name) {
                if let Err(e) = manifest.save_to_dir(&std::env::current_dir().unwrap()) {
                    eprintln!("Error saving manifest: {}", e);
                    process::exit(1);
                }
                // Remove from std/<name>/
                let std_dir = std::env::current_dir().unwrap().join("std").join(name);
                if std_dir.exists() {
                    fs::remove_dir_all(&std_dir)
                        .map_err(|e| format!("Failed to remove std/{name}: {}", e)).ok();
                    println!("  Removed std/{name}/");
                }
                println!("Removed dependency '{}'", name);
            } else {
                eprintln!("Dependency '{}' not found", name);
                process::exit(1);
            }
        }
        Err(e) => { eprintln!("Error: {}", e); process::exit(1); }
    }
}

fn cmd_install(args: &[String]) {
    let cwd = std::env::current_dir().unwrap();
    let project_root = match find_project_root(&cwd) {
        Some(r) => r,
        None => { eprintln!("No ky.toml found in current or parent directories"); process::exit(1); }
    };
    let lock_path = project_root.join("ky.lock");
    let lock = match LockFile::read(&lock_path) {
        Ok(l) => l,
        Err(e) => { eprintln!("Failed to read ky.lock: {}", e); process::exit(1); }
    };
    if lock.packages.is_empty() {
        eprintln!("No packages in ky.lock");
        process::exit(1);
    }

    println!("Installing packages from ky.lock...");
    let mut count = 0;
    for entry in &lock.packages {
        if !cache::is_cached(&entry.name, &entry.version) {
            print!("  Downloading {} v{} ... ", entry.name, entry.version);
            match kyc_tools::package::registry::download_package(&entry.name, &entry.version) {
                Ok(data) => {
                    let dest = cache::package_cache_dir(&entry.name, &entry.version);
                    if let Err(e) = cache::extract_tarball(&data, &dest) {
                        eprintln!("Failed to extract {}: {}", entry.name, e);
                        continue;
                    }
                    println!("done");
                }
                Err(e) => {
                    eprintln!("Failed to download {}: {}", entry.name, e);
                    continue;
                }
            }
        }
        if let Err(e) = install_package_to_std(&project_root, &entry.name, &entry.version) {
            eprintln!("Warning: {}: {}", entry.name, e);
        } else {
            count += 1;
        }
    }
    println!("Installed {}/{} packages", count, lock.packages.len());
}

fn cmd_info(args: &[String]) {
    let dir = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        std::env::current_dir().unwrap()
    };
    match Manifest::find_in_directory(&dir) {
        Ok(manifest) => {
            println!("Project: {}", manifest.project_name());
            println!("Version: {}", manifest.project_version());
            println!("Edition: {}", manifest.project_edition());
            let authors = manifest.project_authors();
            if !authors.is_empty() {
                println!("Authors: {}", authors.join(", "));
            }
            println!("License: {}", manifest.project_license());
            let desc = manifest.project_description();
            if !desc.is_empty() {
                println!("Description: {}", desc);
            }
            let main = manifest.project_main();
            if !main.is_empty() {
                println!("Main: {}", main);
            }
            println!();
            println!("Compiler: {} ({})", manifest.compiler.target, manifest.compiler.optimization);
            println!();
            if manifest.dependencies.is_empty() {
                println!("Dependencies: (none)");
            } else {
                println!("Dependencies:");
                for line in manifest.dependency_summary() {
                    println!("{}", line);
                }
            }

            let lock_path = dir.join("ky.lock");
            if let Ok(lock) = LockFile::read(&lock_path) {
                if !lock.packages.is_empty() {
                    println!();
                    println!("Resolved packages (ky.lock):");
                    for pkg in &lock.packages {
                        let cached = if cache::is_cached(&pkg.name, &pkg.version) {
                            "cached"
                        } else {
                            "not cached"
                        };
                        println!("  {} v{} [{}]", pkg.name, pkg.version, cached);
                    }
                }
            }
        }
        Err(e) => { eprintln!("Error: {}", e); process::exit(1); }
    }
}

// ── New Commands ──

fn cmd_publish(_args: &[String]) {
    let cwd = std::env::current_dir().unwrap();
    let manifest = match Manifest::find_in_cwd() {
        Ok(m) => m,
        Err(e) => { eprintln!("{}", e); process::exit(1); }
    };

    if let Err(errors) = manifest.validate() {
        for err in &errors {
            eprintln!("Validation error: {}", err);
        }
        process::exit(1);
    }

    let name = manifest.project_name().to_string();
    let version = manifest.project_version().to_string();

    println!("Publishing '{}' v{} ...", name, version);

    let tarball_data = match create_package_tarball(&cwd) {
        Ok(data) => data,
        Err(e) => { eprintln!("Failed to create package: {}", e); process::exit(1); }
    };

    match upload_package(&name, &version, &tarball_data) {
        Ok(()) => println!("✅ Published '{}' v{}", name, version),
        Err(e) => {
            eprintln!("Failed to publish: {}", e);
            eprintln!();
            eprintln!("To publish, set the KL_REGISTRY environment variable or ensure");
            eprintln!("the registry server is running. For local testing:");
            eprintln!("  export KL_REGISTRY=http://localhost:8080/v1");
            process::exit(1);
        }
    }
}

fn create_package_tarball(project_dir: &Path) -> Result<Vec<u8>, String> {
    let mut buf: Vec<u8> = Vec::new();
    let encoder = flate2::write::GzEncoder::new(&mut buf, flate2::Compression::default());
    let mut tar = tar::Builder::new(encoder);

    let manifest_path = project_dir.join("ky.toml");
    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read ky.toml: {}", e))?;
    let mut header = tar::Header::new_ustar();
    header.set_path("ky.toml").map_err(|e| format!("tar header error: {}", e))?;
    header.set_size(manifest_content.len() as u64);
    header.set_mode(0o644);
    tar.append(&header, manifest_content.as_bytes())
        .map_err(|e| format!("tar append error: {}", e))?;

    let src_dir = project_dir.join("src");
    if src_dir.exists() {
        tar.append_dir_all("src", &src_dir)
            .map_err(|e| format!("Failed to add src/: {}", e))?;
    }

    let readme_path = project_dir.join("README.md");
    if readme_path.exists() {
        tar.append_path_with_name(&readme_path, "README.md")
            .map_err(|e| format!("Failed to add README.md: {}", e))?;
    }

    let encoder = tar.into_inner()
        .map_err(|e| format!("tar finalize error: {}", e))?;
    encoder.finish()
        .map_err(|e| format!("gzip finish error: {}", e))?;

    Ok(buf)
}

fn upload_package(name: &str, version: &str, data: &[u8]) -> Result<(), String> {
    let registry_url = std::env::var("KL_REGISTRY")
        .unwrap_or_else(|_| "https://registry.kyle-lang.org/v1".to_string());
    let url = format!("{}/packages/{}/{}/upload", registry_url, name, version);

    let resp = ureq::put(&url)
        .header("Content-Type", "application/gzip")
        .send(data)
        .map_err(|e| format!("Upload failed: {}", e))?;

    if resp.status() == 200 || resp.status() == 201 {
        Ok(())
    } else {
        Err(format!("Registry returned {} (expected 200/201)", resp.status()))
    }
}

fn cmd_login(_args: &[String]) {
    let registry_url = std::env::var("KL_REGISTRY")
        .unwrap_or_else(|_| "https://registry.kyle-lang.org/v1".to_string());

    println!("Login to {}", registry_url);
    println!("Enter your API key (or press Enter to skip):");

    let mut api_key = String::new();
    std::io::stdin().read_line(&mut api_key).unwrap_or_default();
    let api_key = api_key.trim();

    if api_key.is_empty() {
        println!("Skipped login.");
        return;
    }

    let verify_url = format!("{}/auth/verify", registry_url);
    match ureq::get(&verify_url)
        .header("Authorization", &format!("Bearer {}", api_key))
        .call()
    {
        Ok(resp) if resp.status() == 200 => {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let config_dir = PathBuf::from(&home).join(".ky");
            fs::create_dir_all(&config_dir).unwrap_or_else(|e| {
                eprintln!("Warning: could not create {}: {}", config_dir.display(), e);
            });

            let config_path = config_dir.join("config.toml");
            let config_content = format!("registry = \"{}\"\napi_key = \"{}\"\n", registry_url, api_key);
            fs::write(&config_path, &config_content).unwrap_or_else(|e| {
                eprintln!("Warning: could not write {}: {}", config_path.display(), e);
            });

            println!("✅ Logged in successfully.");
            println!("   API key saved to {}", config_path.display());
        }
        Ok(resp) => {
            eprintln!("Login failed: registry returned {}", resp.status());
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Could not contact registry: {}", e);
            eprintln!("API key not saved. You can set KL_API_KEY environment variable instead.");
            process::exit(1);
        }
    }
}

fn cmd_update(_args: &[String]) {
    let cwd = std::env::current_dir().unwrap();
    let project_root = match find_project_root(&cwd) {
        Some(r) => r,
        None => { eprintln!("No ky.toml found"); process::exit(1); }
    };
    let manifest = match Manifest::find_in_directory(&project_root) {
        Ok(m) => m,
        Err(e) => { eprintln!("{}", e); process::exit(1); }
    };

    println!("Updating dependencies...");
    match resolve_project_dependencies(&project_root, &manifest) {
        Ok(()) => println!("✅ Lock file updated."),
        Err(e) => { eprintln!("Update failed: {}", e); process::exit(1); }
    }
}

fn cmd_outdated(_args: &[String]) {
    let cwd = std::env::current_dir().unwrap();
    let project_root = match find_project_root(&cwd) {
        Some(r) => r,
        None => { eprintln!("No ky.toml found"); process::exit(1); }
    };
    let manifest = match Manifest::find_in_directory(&project_root) {
        Ok(m) => m,
        Err(e) => { eprintln!("{}", e); process::exit(1); }
    };
    let lock_path = project_root.join("ky.lock");
    let lock = LockFile::read(&lock_path).unwrap_or_default();

    if manifest.dependencies.is_empty() {
        println!("No dependencies.");
        return;
    }

    let registry = RegistryClient::new();
    println!("Checking for outdated dependencies...");
    println!();

    let mut any_outdated = false;
    for (dep_name, dep_req_str) in &manifest.dependencies {
        let current_version = lock.get_version(dep_name).unwrap_or("—");

        match registry.get_versions(dep_name) {
            Ok(versions) => {
                let latest = versions.last();
                let latest_str = latest.map(|v| format!("{}", v)).unwrap_or_else(|| "?".to_string());

                let is_outdated = match latest {
                    Some(latest_v) => {
                        if let Ok(current_v) = kyc_core::semver::parse_version(current_version) {
                            latest_v > &current_v
                        } else {
                            false
                        }
                    }
                    None => false,
                };

                if is_outdated {
                    println!("  ⚠ {}  {}  →  {}  ({})", dep_name, current_version, latest_str, dep_req_str);
                    any_outdated = true;
                } else {
                    println!("  ✓ {}  {}  (latest)", dep_name, current_version);
                }
            }
            Err(_) => {
                println!("  ? {}  {}  (registry unavailable)", dep_name, current_version);
            }
        }
    }

    if !any_outdated {
        println!();
        println!("All dependencies are up to date.");
    } else {
        println!();
        println!("Run 'ky update' to update the lock file.");
    }
}

fn cmd_uninstall() {
    let mut uninstalled = false;

    if cfg!(target_os = "windows") {
        let home = std::env::var("USERPROFILE").unwrap_or_default();
        let appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();

        // Delete runtime libs and llvm-18 (can be deleted while we run)
        let ky_libs = [
            format!("{}\\.ky\\lib\\libkyc_runtime.a", home),
            format!("{}\\.ky\\lib\\kyc_runtime.lib", home),
        ];
        for t in &ky_libs {
            let p = Path::new(t);
            if p.exists() {
                let _ = fs::remove_file(p);
                println!("  removed {}", p.display());
                uninstalled = true;
            }
        }
        for base in &[&home, &appdata] {
            let llvm_dir = Path::new(base).join(".ky").join("llvm-18");
            if llvm_dir.exists() {
                let _ = fs::remove_dir_all(&llvm_dir);
                println!("  removed {}", llvm_dir.display());
                uninstalled = true;
            }
        }

        // ky.exe cannot self-delete on Windows (file locked by running process).
        // Create a self-destruct batch that runs after we exit.
        let reinstall_targets = [
            format!("{}\\.ky\\bin\\ky.exe", home),
            format!("{}\\.ky\\bin\\ky.exe", appdata),
        ];
        for t in &reinstall_targets {
            let ky_exe = Path::new(t);
            if ky_exe.exists() {
                let ky_dir = ky_exe.parent().unwrap().parent().unwrap();
                // Write a .cmd that deletes ky.exe, empty dirs, and itself
                let clean_script = format!(
                    "@echo off\r\n\
                     ping -n 2 127.0.0.1 >nul\r\n\
                     del /f /q \"{}\" 2>nul\r\n\
                     rmdir /s /q \"{}\" 2>nul\r\n\
                     del /f /q \"%~f0\" 2>nul\r\n",
                    t, ky_dir.display()
                );
                let batch_path = ky_exe.parent().unwrap().join("ky_cleanup.cmd");
                let _ = fs::write(&batch_path, &clean_script);
                let _ = std::process::Command::new("cmd")
                    .args(&["/c", &batch_path.to_string_lossy().to_string()])
                    .spawn();
                println!("  queued removal of {}", t);
                uninstalled = true;
            }
        }
    } else {
        let home = std::env::var("HOME").unwrap_or_default();
        let targets = [
            "/usr/local/bin/ky",
            "/usr/local/lib/ky/libkyc_runtime.a",
            &format!("{}/.ky/bin/ky", home),
            &format!("{}/.ky/lib/ky/libkyc_runtime.a", home),
            &format!("{}/.ky/lib/libkyc_runtime.a", home),
            &format!("{}/.kl/bin/ky", home),
            &format!("{}/.kl/lib/ky/libkyc_runtime.a", home),
            &format!("{}/.kl/lib/libkyc_runtime.a", home),
        ];
        for target in &targets {
            if Path::new(target).exists() {
                let _ = fs::remove_file(target);
                println!("  removed {}", target);
                uninstalled = true;
            }
        }
    }

    if uninstalled {
        println!("ky uninstalled.");
        println!("NOTE: Remove the install directories from your PATH manually.");
    } else {
        println!("ky is not installed, or installation path is unknown.");
    }
}

fn load_source(args: &[String], index: usize) -> String {
    if args.len() <= index {
        eprintln!("Error: missing file argument");
        print_usage();
        process::exit(1);
    }
    let path = &args[index];
    match fs::read_to_string(path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            process::exit(1);
        }
    }
}

fn ensure_main(source: &str) -> String {
    if source.contains("fn main(") || source.contains("fn main ") {
        source.to_string()
    } else {
        let mut top = Vec::new();
        let mut body = Vec::new();
        let mut in_top_decl = false;
        for line in source.lines() {
            let trimmed = line.trim();
            let leading = line.len() - trimmed.len();
            if leading == 0 && !in_top_decl {
                let is_top = trimmed.is_empty()
                    || trimmed.starts_with('#')
                    || trimmed.starts_with("from ")
                    || trimmed.starts_with("import ")
                    || trimmed.starts_with("final ")
                    || trimmed.starts_with("class ")
                    || trimmed.starts_with("abstract ")
                    || trimmed.starts_with("struct ")
                    || trimmed.starts_with("enum ")
                    || trimmed.starts_with("type ")
                    || trimmed.starts_with("fn ");
                if is_top {
                    top.push(line.to_string());
                    in_top_decl = trimmed.ends_with(':');
                    continue;
                }
                body.push(line.to_string());
            } else if leading > 0 && in_top_decl {
                top.push(line.to_string());
                in_top_decl = trimmed.ends_with(':');
            } else {
                body.push(line.to_string());
                in_top_decl = false;
            }
        }
        let top_str = top.join("\n");
        let body_str: String = body.iter()
            .map(|l| format!("    {}", l))
            .collect::<Vec<_>>()
            .join("\n");
        if top_str.trim().is_empty() {
            format!("fn main() i32:\n{}\n    0", body_str)
        } else {
            format!("{}\n\nfn main() i32:\n{}\n    0", top_str, body_str)
        }
    }
}
