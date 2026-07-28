// Kyle UI Integration Tests
// Tests that .kyx files compile correctly and generate expected JS

use std::fs;
use std::path::Path;
use kyc_ui::resolver::build_multifile_program;
use kyc_ui::backend::web::WebBackend;
use kyc_ui::backend::UiBackend;

fn test_kyx_file(path: &str, expected_patterns: &[&str]) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));

    // Parse + resolve in one step
    let program = build_multifile_program(&source, Path::new(path))
        .unwrap_or_else(|e| panic!("Failed to build program for {}: {}", path, e));

    // Generate web output
    let backend = WebBackend::new();
    let output = backend.generate(&program);

    assert!(!output.files.is_empty(), "No output files for {}", path);
    let js = &output.files[0].content;

    // Check expected patterns
    for pattern in expected_patterns {
        assert!(
            js.contains(pattern),
            "Expected pattern '{}' not found in generated JS for {}\nJS snippet (first 500 chars):\n{}",
            pattern, path,
            &js[..js.len().min(500)]
        );
    }
}

#[test]
fn test_binding() {
    test_kyx_file(
        "tests/ui/binding.kyx",
        &["state.set('name'", "state.get('name'"]
    );
}

#[test]
fn test_form_model() {
    test_kyx_file(
        "tests/ui/form_model.kyx",
        &["model", "field", "submit"]
    );
}

#[test]
fn test_events() {
    test_kyx_file(
        "tests/ui/events.kyx",
        &[
            "click",
            "mouseenter",
            "mouseleave",
            "touchstart",
            "touchend",
            "keydown",
            "keyup",
            "focus",
            "blur",
        ]
    );
}

#[test]
fn test_lifecycle() {
    test_kyx_file(
        "tests/ui/lifecycle.kyx",
        &[
            "on_created",
            "on_mounted",
            "on_updated",
            "on_unmounted",
            "state.onAnyChange",
        ]
    );
}

#[test]
fn test_conditional() {
    test_kyx_file(
        "tests/ui/conditional.kyx",
        &["if (", "Visible", "Hidden"]
    );
}

#[test]
fn test_for_loop() {
    test_kyx_file(
        "tests/ui/for_loop.kyx",
        &["for (const", "items"]
    );
}

#[test]
fn test_styles() {
    test_kyx_file(
        "tests/ui/styles.kyx",
        &[
            "'Primary'",
            "'Secondary'",
            "'Title'",
            "background",
            "borderRadius",
            "fontSize",
            "applyStyle",
        ]
    );
}

#[test]
fn test_image() {
    test_kyx_file(
        "tests/ui/image.kyx",
        &[
            "lazy",
            "https://picsum.photos/200",
        ]
    );
}

#[test]
fn test_virtual_list() {
    test_kyx_file(
        "tests/ui/virtual_list.kyx",
        &[
            "createVirtualList",
            "items.length",
        ]
    );
}

#[test]
fn test_inputs() {
    test_kyx_file(
        "tests/ui/inputs.kyx",
        &[
            "Binding.twoWay",
            "twoWay",
        ]
    );
}

#[test]
fn test_transitions() {
    test_kyx_file(
        "tests/ui/transitions.kyx",
        &["transition", "hovered"]
    );
}

#[test]
fn test_file_picker() {
    test_kyx_file(
        "tests/ui/file_picker.kyx",
        &[
            "type = 'file'",
            "accept",
            "arrayBuffer",
        ]
    );
}

#[test]
fn test_routing() {
    test_kyx_file(
        "tests/ui/routing.kyx",
        &[
            "createRouter",
            "route",
            "__KYLE_ROUTES",
        ]
    );
}

#[test]
fn test_animations() {
    test_kyx_file(
        "tests/ui/animations.kyx",
        &[
            "FadeIn",
            "keyframes",
            "duration",
        ]
    );
}

/// Test that ALL .kyx files in tests/ui/ can at least compile
#[test]
fn test_all_kyx_files_compile() {
    let dir = fs::read_dir("tests/ui")
        .expect("tests/ui/ directory not found");
    
    let mut count = 0;
    for entry in dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "kyx") {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
            
            // Parse + resolve
            let program = build_multifile_program(&source, &path)
                .unwrap_or_else(|e| panic!("Failed to build program for {:?}: {}", path, e));
            
            // Generate
            let backend = WebBackend::new();
            let output = backend.generate(&program);
            
            assert!(!output.files.is_empty(), "No output for {:?}", path);
            assert!(output.html_shell.is_some(), "No HTML shell for {:?}", path);
            
            count += 1;
        }
    }
    
    assert!(count >= 10, "Expected at least 10 .kyx test files, found {}", count);
}
