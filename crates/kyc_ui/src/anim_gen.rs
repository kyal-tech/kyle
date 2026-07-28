use crate::ast::*;

/// Generate CSS keyframes and animation objects from animation declarations.
pub fn generate_animations(animations: &[AnimDecl]) -> String {
    let mut js = String::new();

    if animations.is_empty() {
        js.push_str("const animations = {};\n\n");
        js.push_str("function applyAnimation(el, animName) {\n");
        js.push_str("  const a = animations[animName];\n");
        js.push_str("  if (!a) return;\n");
        js.push_str("  el.animate(a.keyframes, a.options);\n");
        js.push_str("}\n\n");
        return js;
    }

    js.push_str("const animations = {\n");
    for anim in animations {
        js.push_str(&format!("  '{}': {{\n", anim.name));
        js.push_str("    keyframes: [\n");

        let mut from = String::new();
        let mut to = String::new();
        let mut duration = 300i64;
        let mut easing = "ease".to_string();
        let mut iterations = 1i64;
        let mut delay = 0i64;
        let mut direction = "normal".to_string();
        let mut fill = "none".to_string();

        for prop in &anim.props {
            let v = prop.value.trim();
            match prop.name.as_str() {
                "duration" => { if let Ok(n) = v.parse::<i64>() { duration = n; } }
                "easing" => easing = to_js_easing(v),
                "delay" => { if let Ok(n) = v.parse::<i64>() { delay = n; } }
                "iterations" => { if let Ok(n) = v.parse::<i64>() { iterations = n; } }
                "direction" => direction = v.trim_matches('"').to_string(),
                "fill_mode" => fill = v.trim_matches('"').to_string(),
                "from" => from = parse_anim_state(v),
                "to" => to = parse_anim_state(v),
                _ => {}
            }
        }

        if !from.is_empty() {
            js.push_str(&format!("      {},\n", from));
        }
        if !to.is_empty() {
            js.push_str(&format!("      {},\n", to));
        }

        js.push_str("    ],\n");
        js.push_str(&format!("    options: {{\n"));
        js.push_str(&format!("      duration: {},\n", duration));
        js.push_str(&format!("      easing: '{}',\n", easing));
        js.push_str(&format!("      iterations: {},\n", iterations));
        if delay > 0 { js.push_str(&format!("      delay: {},\n", delay)); }
        if direction != "normal" { js.push_str(&format!("      direction: '{}',\n", direction)); }
        if fill != "none" { js.push_str(&format!("      fill: '{}',\n", fill)); }
        js.push_str("    },\n");
        js.push_str("  },\n");
    }
    js.push_str("};\n\n");

    js.push_str("function applyAnimation(el, animName) {\n");
    js.push_str("  const a = animations[animName];\n");
    js.push_str("  if (!a) return;\n");
    js.push_str("  _a11y.animate(el, a.keyframes, a.options);\n");
    js.push_str("}\n\n");

    js
}

fn parse_anim_state(s: &str) -> String {
    // AnimationState(prop1: val1, prop2: val2)
    let s = s.trim();
    if !s.starts_with("AnimationState(") {
        return format!("{{}}");
    }
    let inner = s.trim_start_matches("AnimationState(").trim_end_matches(')');
    let mut props = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();
        if let Some(eq) = part.find(':') {
            let key = &part[..eq].trim();
            let val = &part[eq+1..].trim();
            let css_key = anim_prop_to_css(key);
            props.push(format!("'{}': {}", css_key, val));
        }
    }
    format!("{{{}}}", props.join(", "))
}

fn anim_prop_to_css(prop: &str) -> &str {
    match prop {
        "opacity" => "opacity",
        "translate_x" => "transform",
        "translate_y" => "transform",
        "scale_x" => "transform",
        "scale_y" => "transform",
        "rotate" => "transform",
        "background" => "backgroundColor",
        "color" => "color",
        "border_radius" => "borderRadius",
        "shadow" => "boxShadow",
        "width" => "width",
        "height" => "height",
        _ => prop,
    }
}

fn to_js_easing(easing: &str) -> String {
    match easing.trim() {
        "Easing.Linear" => "linear",
        "Easing.Ease" => "ease",
        "Easing.EaseIn" => "ease-in",
        "Easing.EaseOut" => "ease-out",
        "Easing.EaseInOut" => "ease-in-out",
        "Easing.EaseInSine" => "cubic-bezier(0.47, 0, 0.745, 0.715)",
        "Easing.EaseOutSine" => "cubic-bezier(0.39, 0.575, 0.565, 1)",
        "Easing.EaseInCubic" => "cubic-bezier(0.55, 0.055, 0.675, 0.19)",
        "Easing.EaseOutCubic" => "cubic-bezier(0.215, 0.61, 0.355, 1)",
        "Easing.Bounce" => "cubic-bezier(0.68, -0.55, 0.265, 1.55)",
        _ => "ease",
    }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anim_gen_fadein() {
        let anims = vec![AnimDecl {
            component: "view".to_string(),
            name: "FadeIn".to_string(),
            props: vec![
                StyleProp { name: "duration".to_string(), value: "300".to_string() },
                StyleProp { name: "easing".to_string(), value: "Easing.EaseOut".to_string() },
                StyleProp { name: "from".to_string(), value: "AnimationState(opacity: 0.0)".to_string() },
                StyleProp { name: "to".to_string(), value: "AnimationState(opacity: 1.0)".to_string() },
            ],
        }];
        let js = generate_animations(&anims);
        assert!(js.contains("FadeIn"));
        assert!(js.contains("opacity"));
        assert!(js.contains("300"));
        assert!(js.contains("ease-out"));
    }
}
