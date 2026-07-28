use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Linker {
    target_triple: Option<String>,
}

impl Linker {
    pub fn new() -> Self {
        Self { target_triple: None }
    }

    pub fn new_with_target(target: &str) -> Self {
        Self { target_triple: Some(target.to_string()) }
    }

    fn is_target(&self, suffix: &str) -> bool {
        self.target_triple.as_ref().map_or(false, |t| t.contains(suffix))
    }

    fn is_freestanding(&self) -> bool {
        self.target_triple.as_deref() == Some("freestanding")
    }

    fn linker_cmd(&self) -> Command {
        let host_os = std::env::consts::OS;

        // Freestanding: use native host linker (clang or cc)
        if self.is_freestanding() {
            if Command::new("clang").arg("--version").output().is_ok() {
                return Command::new("clang");
            }
            if Command::new("cc").arg("--version").output().is_ok() {
                return Command::new("cc");
            }
            // Fallback to ld (may need manual linking)
            return Command::new("ld");
        }

        // Cross-compilation: use appropriate linker based on target
        if self.is_target("windows") || self.is_target("win32") {
            if Command::new("clang").arg("--version").output().is_ok() {
                return Command::new("clang");
            }
            if Command::new("lld-link.exe").arg("--version").output().is_ok() {
                return Command::new("lld-link.exe");
            }
            return Command::new("link.exe");
        }
        if self.is_target("linux") || self.is_target("unknown-linux") {
            if Command::new("clang").arg("--version").output().is_ok() {
                return Command::new("clang");
            }
            // For cross-compilation: try <target>-gcc (e.g. aarch64-linux-gnu-gcc)
            if let Some(triple) = &self.target_triple {
                let cross_gcc = format!("{}-gcc", triple);
                if Command::new(&cross_gcc).arg("--version").output().is_ok() {
                    return Command::new(cross_gcc);
                }
            }
            return Command::new("cc");
        }
        if self.is_target("wasm32") || self.is_target("wasm64") {
            // WASM target: use wasm-ld or clang with wasm triple
            if Command::new("wasm-ld").arg("--version").output().is_ok() {
                let mut c = Command::new("wasm-ld");
                return c;
            }
            // clang can also link wasm with appropriate flags
            if Command::new("clang").arg("--version").output().is_ok() {
                return Command::new("clang");
            }
        }
        // Default: native host linker
        if host_os == "windows" {
            if Command::new("clang").arg("--version").output().is_ok() {
                Command::new("clang")
            } else if Command::new("lld-link.exe").arg("--version").output().is_ok() {
                Command::new("lld-link.exe")
            } else {
                Command::new("link.exe")
            }
        } else if host_os == "macos" {
            Command::new("clang")
        } else {
            if Command::new("clang").arg("--version").output().is_ok() {
                Command::new("clang")
            } else {
                Command::new("cc")
            }
        }
    }

    pub fn link(&self, object_files: &[&Path], output: &Path, runtime_lib: Option<&Path>, release: bool, links: &[String]) -> Result<(), String> {
        if object_files.is_empty() {
            return Err("No object files to link".to_string());
        }

        let mut cmd = self.linker_cmd();

        // Freestanding: use _start as entry point instead of main
        // We still link the runtime library (for ky_alloc, etc.)
        // The user's _start function replaces C's main as entry point.
        // NOTE: For a true bare-metal kernel, you'd also need -nostdlib -nostartfiles
        // and a bare-metal version of the runtime. That comes in Phase 3.
        if self.is_freestanding() {
            cmd.arg("-e").arg("_start");
        }

        cmd.arg("-o").arg(output);

        if release {
            cmd.arg("-O3");
            if !self.is_target("wasm32") {
                cmd.arg("-flto");
            }
        }

        for obj in object_files {
            cmd.arg(obj);
        }

        if let Some(runtime) = runtime_lib {
            cmd.arg(runtime);
        }

        for link in links {
            if link.starts_with("-framework ") {
                let framework_name = link.trim_start_matches("-framework ");
                cmd.arg("-framework");
                cmd.arg(framework_name);
            } else if link.starts_with("-L") {
                cmd.arg(link);
            } else {
                cmd.arg(format!("-l{}", link));
            }
        }

        if self.is_target("wasm32") || self.is_target("wasm64") {
            cmd.arg("--no-entry");
            cmd.arg("-lc");
        } else if self.is_target("windows") || self.is_target("win32") {
            cmd.arg("-Wl,/NODEFAULTLIB:msvcrt");
            cmd.arg("-lkernel32");
            cmd.arg("-lws2_32");
            cmd.arg("-lbcrypt");
            cmd.arg("-luserenv");
            cmd.arg("-lntdll");
            cmd.arg("-ladvapi32");
            cmd.arg("-lcfgmgr32");
            cmd.arg("-lshlwapi");
            cmd.arg("-liphlpapi");
        }

        if release && !self.is_target("wasm32") {
            cmd.arg("-lm");
        }

        // macOS-specific: frameworks and Homebrew paths
        // (also applies to freestanding since we still link the runtime)
        if (self.target_triple.is_none() || self.is_freestanding()) && std::env::consts::OS == "macos" {
            let ver = std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| {
                std::process::Command::new("sw_vers")
                    .arg("-productVersion")
                    .output()
                    .ok()
                    .and_then(|o| if o.status.success() {
                        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        let parts: Vec<&str> = s.split('.').collect();
                        if parts.len() >= 2 {
                            Some(format!("{}.{}", parts[0], parts[1]))
                        } else { None }
                    } else { None })
                    .unwrap_or_else(|| "26.0".to_string())
            });
            if !ver.is_empty() {
                cmd.arg(format!("-mmacosx-version-min={}", ver));
            }
            cmd.arg("-framework").arg("CoreFoundation");
            let homebrew_paths = [
                "/opt/homebrew/opt/libpq/lib",
                "/opt/homebrew/lib",
                "/usr/local/lib",
            ];
            for p in &homebrew_paths {
                if std::path::Path::new(p).exists() {
                    cmd.arg(format!("-L{}", p));
                    cmd.arg(format!("-Wl,-rpath,{}", p));
                }
            }
        }

        let status = cmd.status().map_err(|e| format!("Linker failed: {}", e))?;

        if !status.success() {
            return Err("Linking failed".to_string());
        }

        Ok(())
    }

    pub fn link_shared(&self, object_files: &[&Path], output: &Path) -> Result<(), String> {
        if object_files.is_empty() {
            return Err("No object files to link".to_string());
        }

        let mut cmd = self.linker_cmd();
        cmd.arg("-shared").arg("-o").arg(output);

        for obj in object_files {
            cmd.arg(obj);
        }

        let status = cmd.status().map_err(|e| format!("Linker failed: {}", e))?;

        if !status.success() {
            return Err("Linking failed".to_string());
        }

        Ok(())
    }

    pub fn verify_output(output: &Path) -> bool {
        output.exists()
    }

    /// Locate the kyc_runtime static library.
    ///
    /// Search order:
    /// 1. Relative to the kl binary (installed: /usr/local/lib/kl/libkyc_runtime.a)
    /// 2. Cargo workspace (debug/release)
    /// 3. Current working directory
    pub fn find_runtime_lib() -> Option<PathBuf> {
        let runtime_lib = if cfg!(target_os = "windows") {
            "kyc_runtime.lib"
        } else {
            "libkyc_runtime.a"
        };
        let wasm_lib = "libkyc_runtime_wasm.a";

        let mut paths = Vec::new();

        // 1. Relative to the running binary
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // /usr/local/bin/ky → /usr/local/lib/libkyc_runtime.a
                paths.push(exe_dir.join("../lib").join(runtime_lib));
                // /usr/local/bin/ky → /usr/local/lib/ky/libkyc_runtime.a (legacy)
                paths.push(exe_dir.join("../lib/ky").join(runtime_lib));
                // Alongside binary
                paths.push(exe_dir.join(runtime_lib));
            }
        }

        // 2. Cargo workspace
        if let Some(root) = workspace_root() {
            paths.push(root.join("target").join("debug").join(runtime_lib));
            paths.push(root.join("target").join("release").join(runtime_lib));
            // WASM runtime
            paths.push(root.join("target/wasm32-unknown-unknown/release").join(wasm_lib));
            paths.push(root.join("target/wasm32-unknown-unknown/debug").join(wasm_lib));
        }

        // 3. Current working directory
        paths.push(PathBuf::from("./target/debug").join(runtime_lib));
        paths.push(PathBuf::from("./target/release").join(runtime_lib));
        paths.push(PathBuf::from(runtime_lib));

        for p in &paths {
            if p.exists() {
                return Some(p.clone());
            }
        }
        None
    }
}

fn workspace_root() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for ancestor in manifest.ancestors() {
        if ancestor.join("Cargo.toml").exists() {
            // Check it's the workspace root (has [workspace])
            if let Ok(content) = std::fs::read_to_string(ancestor.join("Cargo.toml")) {
                if content.contains("[workspace]") {
                    return Some(ancestor.to_path_buf());
                }
            }
        }
    }
    None
}
