use crate::ir::*;
use super::{GeneratedFile, BackendOutput, UiBackend};

pub struct IosBackend;

impl IosBackend {
    pub fn new() -> Self { Self }
}

impl UiBackend for IosBackend {
    fn name(&self) -> &str { "ios" }
    fn target_triple(&self) -> &str { "aarch64-apple-ios" }

    fn generate(&self, program: &UiProgram) -> BackendOutput {
        let mut files = Vec::new();

        // ── App.swift ──
        let mut app = String::new();
        app.push_str("import SwiftUI\n\n");
        app.push_str("@main\n");
        app.push_str("struct KyleApp: App {\n");
        app.push_str("    var body: some Scene {\n");
        app.push_str("        WindowGroup {\n");
        app.push_str("            ContentView()\n");
        app.push_str("        }\n");
        app.push_str("    }\n");
        app.push_str("}\n");
        // ── ColorExtension.swift ──
        let color_ext = r#"import SwiftUI

extension Color {
    init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let a, r, g, b: UInt64
        switch hex.count {
        case 6:
            (a, r, g, b) = (255, (int >> 16) & 0xFF, (int >> 8) & 0xFF, int & 0xFF)
        case 8:
            (a, r, g, b) = ((int >> 24) & 0xFF, (int >> 16) & 0xFF, (int >> 8) & 0xFF, int & 0xFF)
        default:
            (a, r, g, b) = (255, 0, 0, 0)
        }
        self.init(.sRGB, red: Double(r) / 255, green: Double(g) / 255, blue: Double(b) / 255, opacity: Double(a) / 255)
    }
}
"#.to_string();
        files.push(GeneratedFile { path: "ios-app/ColorExtension.swift".to_string(), content: color_ext });

        // ── ContentView.swift ──
        let mut cv = String::new();
        cv.push_str("import SwiftUI\n\n");
        cv.push_str("struct ContentView: View {\n");

        // Generate @State properties from code blocks
        let states = extract_states(&program.code_blocks);
        for (name, default_val) in &states {
            cv.push_str(&format!("    @State private var {}: {} = {}\n", name, ios_type(default_val), default_val));
        }
        if !states.is_empty() {
            cv.push_str("\n");
        }

        cv.push_str("    var body: some View {\n");

        // Generate the UI tree
        gen_ios_nodes(&program.body, &mut cv, 3, "");

        cv.push_str("\n    }\n");

        // Generate functions from code blocks
        let funcs = extract_functions(&program.code_blocks);
        for (sig, body) in &funcs {
            cv.push_str(&format!("\n    {}\n", sig));
            for line in body {
                cv.push_str(&format!("        {}\n", ios_translate_expr(line)));
            }
            cv.push_str("    }\n");
        }

        cv.push_str("}\n");
        files.push(GeneratedFile { path: "ios-app/ContentView.swift".to_string(), content: cv });

        // ── Info.plist ──
        let plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>KyleApp</string>
    <key>CFBundleIdentifier</key>
    <string>com.kyle.app</string>
    <key>CFBundleName</key>
    <string>KyleApp</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>UILaunchStoryboardName</key>
    <string>LaunchScreen</string>
</dict>
</plist>
"#.to_string();
        files.push(GeneratedFile { path: "ios-app/Info.plist".to_string(), content: plist });

        // ── Package.swift (for building with swift) ──
        let pkg = format!(r#"// swift-tools-version:5.9
import PackageDescription
let package = Package(
    name: "KyleApp",
    platforms: [.iOS(.v16)],
    products: [
        .library(name: "KyleApp", targets: ["KyleApp"])
    ],
    dependencies: [],
    targets: [
        .target(
            name: "KyleApp",
            path: ".",
            exclude: ["Info.plist"]
        )
    ]
)
"#);
        files.push(GeneratedFile { path: "ios-app/Package.swift".to_string(), content: pkg });

        BackendOutput { files, html_shell: None }
    }
}

fn gen_ios_nodes(nodes: &[UiNode], swift: &mut String, indent: usize, _parent: &str) {
    for node in nodes {
        gen_ios_node(node, swift, indent);
    }
}

fn gen_ios_node(node: &UiNode, swift: &mut String, indent: usize) {
    let ind = "    ".repeat(indent);
    match node {
        UiNode::Element { tag, attrs, children } => {
            match tag {
                ComponentTag::View | ComponentTag::Card | ComponentTag::Surface => {
                    swift.push_str(&format!("{}ZStack(alignment: .topLeading) {{\n", ind));
                    gen_ios_nodes(children, swift, indent + 1, "");
                    swift.push_str(&format!("{}}}\n", ind));
                }
                ComponentTag::VStack => {
                    let spacing = get_style_attr(attrs, "spacing", "16");
                    swift.push_str(&format!("{}VStack(spacing: {}) {{\n", ind, spacing));
                    gen_ios_nodes(children, swift, indent + 1, "");
                    swift.push_str(&format!("{}}}\n", ind));
                }
                ComponentTag::HStack => {
                    let spacing = get_style_attr(attrs, "spacing", "16");
                    swift.push_str(&format!("{}HStack(spacing: {}) {{\n", ind, spacing));
                    gen_ios_nodes(children, swift, indent + 1, "");
                    swift.push_str(&format!("{}}}\n", ind));
                }
                ComponentTag::Text => {
                    let text = get_text_attr(attrs);
                    // If text is already a Text() expression, use it directly
                    if text.starts_with("Text(") {
                        swift.push_str(&format!("{}{}\n", ind, text));
                    } else {
                        swift.push_str(&format!("{}Text({})\n", ind, text));
                    }
                    let mods = ios_text_modifiers(attrs);
                    for m in &mods {
                        swift.push_str(&format!("{}{}\n", ind, m));
                    }
                }
                ComponentTag::Button => {
                    let label = get_str_attr(attrs, "text", "value").unwrap_or("\"Button\"");
                    let label_swift = if label.starts_with('"') {
                        format!("Text({})", label)
                    } else {
                        format!("Text(\"{}\")", label.trim_matches('"'))
                    };
                    let action = get_click_attr(attrs);
                    swift.push_str(&format!("{}Button(action: {{ {} }}) {{\n", ind, action));
                    swift.push_str(&format!("{}    {}\n", ind, label_swift));
                    let mods = ios_button_modifiers(attrs);
                    for m in &mods {
                        swift.push_str(&format!("{}{}\n", ind, m));
                    }
                    swift.push_str(&format!("{}}}\n", ind));
                }
                ComponentTag::TextField | ComponentTag::PasswordField => {
                    let placeholder = get_str_attr(attrs, "placeholder", "label").unwrap_or("\"\"");
                    let binding = get_bind_attr(attrs);
                    swift.push_str(&format!("{}TextField({}, text: ${})\n", ind, placeholder, binding));
                }
                ComponentTag::Image => {
                    let src = get_str_attr(attrs, "src", "source").unwrap_or("\"\"");
                    swift.push_str(&format!("{}Image(systemName: {})\n", ind, src));
                }
                ComponentTag::Spacer => {
                    swift.push_str(&format!("{}Spacer()\n", ind));
                }
                ComponentTag::Scroll => {
                    swift.push_str(&format!("{}ScrollView {{\n", ind));
                    gen_ios_nodes(children, swift, indent + 1, "");
                    swift.push_str(&format!("{}}}\n", ind));
                }
                ComponentTag::FilePicker => {
                    swift.push_str(&format!("{}// FilePicker: not implemented for iOS yet\n", ind));
                }
                _ => {
                    // Custom/unknown: render children
                    gen_ios_nodes(children, swift, indent, "");
                }
            }
        }
        UiNode::SelfClosing { tag, attrs } => {
            gen_ios_node(&UiNode::Element { tag: tag.clone(), attrs: attrs.clone(), children: vec![] }, swift, indent);
        }
        UiNode::Text(text) => {
            swift.push_str(&format!("{}Text(\"{}\")\n", ind, text));
        }
        UiNode::If { condition, then_branch, else_branch } => {
            let cond = ios_translate_expr(condition);
            swift.push_str(&format!("{}if {} {{\n", ind, cond));
            gen_ios_nodes(then_branch, swift, indent + 1, "");
            if !else_branch.is_empty() {
                swift.push_str(&format!("{}}} else {{\n", ind));
                gen_ios_nodes(else_branch, swift, indent + 1, "");
            }
            swift.push_str(&format!("{}}}\n", ind));
        }
        UiNode::For { item, list, body } => {
            swift.push_str(&format!("{}ForEach({}, id: \\.self) {{ {} in\n", ind, list, item));
            gen_ios_nodes(body, swift, indent + 1, "");
            swift.push_str(&format!("{}}}\n", ind));
        }
        _ => {}
    }
}

// ── Helpers ──

fn get_str_attr<'a>(attrs: &'a [UiAttr], k1: &str, k2: &str) -> Option<&'a str> {
    for a in attrs {
        if a.name == k1 || a.name == k2 {
            if let AttrValue::String(ref s) = a.value { return Some(s); }
            if let AttrValue::Expr(ref e) = a.value { return Some(e); }
        }
    }
    None
}

fn interpolate_text(val: &str) -> String {
    // Replace @name with \(name) for SwiftUI string interpolation
    let mut result = String::new();
    let mut last = 0;
    for (i, _) in val.match_indices('@') {
        result.push_str(&val[last..i]);
        let rest = &val[i + 1..];
        let key_len = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').count();
        if key_len > 0 {
            result.push_str("\\(");
            result.push_str(&rest[..key_len]);
            result.push(')');
            last = i + 1 + key_len;
        } else {
            result.push('@');
            last = i + 1;
        }
    }
    if last < val.len() {
        result.push_str(&val[last..]);
    }
    result
}

fn get_text_attr(attrs: &[UiAttr]) -> String {
    for a in attrs {
        if a.name == "value" || a.name == "text" {
            match &a.value {
                AttrValue::String(s) => {
                    if s.contains('@') {
                        return format!("\"{}\"", interpolate_text(s));
                    }
                    return format!("\"{}\"", s);
                }
                AttrValue::Expr(e) => {
                    return ios_expr_to_text(e);
                }
                AttrValue::Flag => {}
            }
        }
    }
    "\"\"".to_string()
}

/// Convert a Kyle expression to a SwiftUI Text expression
fn ios_expr_to_text(expr: &str) -> String {
    let e = expr.trim();
    // Handle "string" + expr patterns
    if e.contains('+') {
        let parts: Vec<&str> = e.split('+').collect();
        let mut result = String::new();
        for part in parts {
            let p = part.trim().trim_matches('"');
            if part.trim().starts_with('"') {
                if result.is_empty() {
                    result = format!("Text(\"{}\")", p);
                } else {
                    result = format!("{} + Text(\"{}\")", result, p);
                }
            } else {
                let swift = ios_translate_expr(p);
                if result.is_empty() {
                    result = format!("Text(\"\\({})\")", swift);
                } else {
                    result = format!("{} + Text(\"\\({})\")", result, swift);
                }
            }
        }
        return result;
    }
    // Single expression (like count)
    let swift = ios_translate_expr(e);
    format!("Text(\"\\({})\")", swift)
}

fn get_click_attr(attrs: &[UiAttr]) -> String {
    for a in attrs {
        if a.name == "click" {
            if let AttrValue::Expr(e) = &a.value {
                return format!("{}()", e);
            }
        }
    }
    "".to_string()
}

fn get_bind_attr(attrs: &[UiAttr]) -> String {
    for a in attrs {
        if a.name == "bind" {
            if let AttrValue::Expr(e) = &a.value {
                return e.to_string();
            }
        }
    }
    "value".to_string()
}

fn get_style_attr(attrs: &[UiAttr], key: &str, default: &str) -> String {
    for a in attrs {
        if a.name == key {
            if let AttrValue::String(s) = &a.value {
                if s.chars().all(|c| c.is_ascii_digit()) {
                    return format!("CGFloat({})", s);
                }
                return s.clone();
            }
        }
    }
    default.to_string()
}

fn ios_text_modifiers(attrs: &[UiAttr]) -> Vec<String> {
    let mut m = Vec::new();
    for a in attrs {
        let val = match &a.value {
            AttrValue::String(s) => s.clone(),
            AttrValue::Expr(e) => ios_translate_expr(e),
            AttrValue::Flag => "true".to_string(),
        };
        match a.name.as_str() {
            "font_size" => m.push(format!("    .font(.system(size: {}))", val)),
            "font_weight" => {
                let w = match val.as_str() {
                    "Bold" | "FontWeight.Bold" => ".bold",
                    "Light" | "FontWeight.Light" => ".light()",
                    "Medium" | "FontWeight.Medium" => ".medium()",
                    "Semibold" | "FontWeight.SemiBold" => ".semibold()",
                    _ => ".regular()",
                };
                m.push(format!("    .fontWeight({})", w));
            }
            "color" => m.push(format!("    .foregroundColor(Color(hex: {}))", val)),
            "background" | "bg" => m.push(format!("    .background(Color(hex: {}))", val)),
            "padding" => m.push(format!("    .padding({})", val)),
            "opacity" => m.push(format!("    .opacity({})", val)),
            "line_limit" | "line_limit" => m.push(format!("    .lineLimit({})", val)),
            _ => {}
        }
    }
    m
}

fn ios_button_modifiers(attrs: &[UiAttr]) -> Vec<String> {
    let mut m = Vec::new();
    for a in attrs {
        let val = match &a.value {
            AttrValue::String(s) => s.clone(),
            _ => continue,
        };
        match a.name.as_str() {
            "padding" | "tpl" => {
                m.push(format!("    .padding(.horizontal, 24)"));
                m.push(format!("    .padding(.vertical, 12)"));
                m.push(format!("    .background(Color.blue)"));
                m.push(format!("    .foregroundColor(.white)"));
                m.push(format!("    .cornerRadius(8)"));
            }
            _ => {}
        }
    }
    m
}

fn extract_states(blocks: &[String]) -> Vec<(String, String)> {
    let mut states = Vec::new();
    for block in blocks {
        for line in block.lines() {
            let line = line.trim();
            if line.contains('=') && line.contains(':') && !line.starts_with("fn ") {
                if let Some(eq_pos) = line.find('=') {
                    let left = line[..eq_pos].trim();
                    let right = line[eq_pos + 1..].trim();
                    if let Some(col_pos) = left.find(':') {
                        let name = left[..col_pos].trim();
                        states.push((name.to_string(), right.to_string()));
                    }
                }
            }
        }
    }
    states
}

fn extract_functions(blocks: &[String]) -> Vec<(String, Vec<String>)> {
    let mut funcs = Vec::new();
    for block in blocks {
        let mut lines: Vec<&str> = block.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("fn ") && line.ends_with(':') {
                let sig = ios_fn_sig(line);
                let mut body = Vec::new();
                i += 1;
                while i < lines.len() {
                    let bl = lines[i];
                    if bl.trim().is_empty() || (!bl.starts_with(' ') && !bl.starts_with('\t')) {
                        break;
                    }
                    body.push(bl.trim().to_string());
                    i += 1;
                }
                funcs.push((sig, body));
            } else {
                i += 1;
            }
        }
    }
    funcs
}

fn ios_fn_sig(line: &str) -> String {
    // fn increment(): → func increment() {
    let rest = line.trim_start_matches("fn ").trim().trim_end_matches(':');
    format!("func {} {{", rest)
}

fn ios_translate_expr(expr: &str) -> String {
    // Convert Kyle expressions to Swift
    let mut s = expr.to_string();
    s = s.replace(".to_str()", ".description");
    s = s.replace("==", "==");
    s = s.replace("!=", "!=");
    s = s.replace(" and ", " && ");
    s = s.replace(" or ", " || ");
    s = s.replace(" not ", " ! ");
    s = s.replace(" is ", " is ");
    // Strip @ prefix from state variables
    s = s.replace(" @", " ");
    if s.starts_with('@') {
        s = s[1..].to_string();
    }
    // Replace := with = for variable assignment
    s = s.replace(":=", "=");
    // Remove trailing colon (if condition, while, etc.)
    if s.ends_with(':') && !s.contains("\"") {
        let trimmed = s.trim_end_matches(':');
        // Check if it's a control flow statement
        if trimmed.starts_with("if ") || trimmed.starts_with("elif ") || trimmed.starts_with("else")
            || trimmed.starts_with("while ") || trimmed.starts_with("for ") {
            s = format!("{} {{", trimmed);
        }
    }
    s
}

fn ios_type(default_val: &str) -> &str {
    if default_val.contains('.') || default_val.contains("f32") || default_val.contains("f64") {
        "CGFloat"
    } else if default_val == "true" || default_val == "false" {
        "Bool"
    } else {
        "Int"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_generates_swift() {
        let b = IosBackend::new();
        let p = UiProgram {
            routes: vec![],
            code_blocks: vec!["count: ^i32 = 0\nfn increment():\n    count = count + 1".to_string()],
            styles: vec![],
            animations: vec![],
            component_renderers: vec![],
            body: vec![
                UiNode::SelfClosing {
                    tag: ComponentTag::Text,
                    attrs: vec![
                        UiAttr { name: "value".to_string(), value: AttrValue::String("Hello".to_string()) },
                        UiAttr { name: "font_size".to_string(), value: AttrValue::String("24".to_string()) },
                        UiAttr { name: "font_weight".to_string(), value: AttrValue::String("Bold".to_string()) },
                    ],
                },
                UiNode::SelfClosing {
                    tag: ComponentTag::Button,
                    attrs: vec![
                        UiAttr { name: "text".to_string(), value: AttrValue::String("+".to_string()) },
                        UiAttr { name: "click".to_string(), value: AttrValue::Expr("increment".to_string()) },
                    ],
                },
            ],
        };
        let o = b.generate(&p);
        assert!(o.files.iter().any(|f| f.path.contains("ContentView")));
        let cv = o.files.iter().find(|f| f.path.contains("ContentView")).unwrap();
        assert!(cv.content.contains("@State"));
        assert!(cv.content.contains("Text(\"Hello\")"));
        assert!(cv.content.contains(".fontWeight"));
        assert!(cv.content.contains("Button"));
        assert!(cv.content.contains("increment()"));
    }
}
