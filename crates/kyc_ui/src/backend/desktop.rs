use crate::ir::*;
use super::{GeneratedFile, BackendOutput, UiBackend};

pub struct DesktopBackend;

impl DesktopBackend {
    pub fn new() -> Self { Self }
}

impl UiBackend for DesktopBackend {
    fn name(&self) -> &str { "desktop" }
    fn target_triple(&self) -> &str { "native" }

    fn generate(&self, program: &UiProgram) -> BackendOutput {
        let mut k = String::new();

        // ── Headers ──
        k.push_str("# Kyle UI — Desktop (Skia + GLFW)\n\n");
        k.push_str("@link \"-L/opt/homebrew/lib\"\n@link \"-L/usr/local/lib\"\n");
        k.push_str("@link \"glfw\"\n@link \"-framework Cocoa\"\n@link \"-framework OpenGL\"\n@link \"-framework IOKit\"\n");
        k.push_str("@link \"skia\"\n@link \"-framework Skia\"\n\n");

        // ── GLFW externs ──
        k.push_str("extern fn glfw_init() i32\nextern fn glfw_terminate()\n");
        k.push_str("extern fn glfw_create_window(w: i32, h: i32, title: ptr, mon: ptr, share: ptr) ptr\n");
        k.push_str("extern fn glfw_destroy_window(w: ptr)\n");
        k.push_str("extern fn glfw_make_context_current(w: ptr)\n");
        k.push_str("extern fn glfw_swap_buffers(w: ptr)\n");
        k.push_str("extern fn glfw_poll_events()\n");
        k.push_str("extern fn glfw_window_should_close(w: ptr) i32\n");
        k.push_str("extern fn glfw_set_window_title(w: ptr, title: ptr)\n");
        k.push_str("extern fn glfw_get_framebuffer_size(w: ptr, w2: ptr, h: ptr)\n");
        k.push_str("extern fn glfw_get_cursor_pos(w: ptr, x: ptr, y: ptr)\n");
        k.push_str("extern fn glfw_get_mouse_button(w: ptr, btn: i32) i32\n\n");

        // ── OpenGL externs ──
        k.push_str("extern fn gl_viewport(x: i32, y: i32, w: i32, h: i32)\n");
        k.push_str("extern fn gl_clear_color(r: f32, g: f32, b: f32, a: f32)\n");
        k.push_str("extern fn gl_clear(mask: i32)\n");
        k.push_str("extern fn gl_gen_texture(n: i32, tex: ptr)\n");
        k.push_str("extern fn gl_bind_texture(target: i32, tex: i32)\n");
        k.push_str("extern fn gl_tex_image_2d(target: i32, lvl: i32, ifmt: i32, w: i32, h: i32, border: i32, fmt: i32, typ: i32, pixels: ptr)\n");
        k.push_str("extern fn gl_tex_parameter_i(target: i32, pname: i32, param: i32)\n");
        k.push_str("extern fn gl_enable(cap: i32)\n");
        k.push_str("extern fn gl_begin(mode: i32)\n");
        k.push_str("extern fn gl_end()\n");
        k.push_str("extern fn gl_tex_coord_2f(s: f32, t: f32)\n");
        k.push_str("extern fn gl_vertex_2f(x: f32, y: f32)\n");
        k.push_str("extern fn gl_color_4f(r: f32, g: f32, b: f32, a: f32)\n\n");

        // ── Skia externs ──
        k.push_str("extern fn sk_surface_new_raster(info: ptr, row_bytes: i64, pixels: ptr) ptr\n");
        k.push_str("extern fn sk_surface_get_canvas(s: ptr) ptr\n");
        k.push_str("extern fn sk_surface_make_image_snapshot(s: ptr) ptr\n");
        k.push_str("extern fn sk_surface_unref(s: ptr)\n");
        k.push_str("extern fn sk_canvas_clear(c: ptr, color: i32)\n");
        k.push_str("extern fn sk_canvas_save(c: ptr)\n");
        k.push_str("extern fn sk_canvas_restore(c: ptr)\n");
        k.push_str("extern fn sk_canvas_translate(c: ptr, dx: f32, dy: f32)\n");
        k.push_str("extern fn sk_canvas_draw_rect(c: ptr, rect: ptr, paint: ptr)\n");
        k.push_str("extern fn sk_canvas_draw_round_rect(c: ptr, rect: ptr, rx: f32, ry: f32, paint: ptr)\n");
        k.push_str("extern fn sk_canvas_draw_circle(c: ptr, cx: f32, cy: f32, r: f32, paint: ptr)\n");
        k.push_str("extern fn sk_canvas_draw_simple_text(c: ptr, text: ptr, len: i64, x: f32, y: f32, font: ptr, paint: ptr)\n");
        k.push_str("extern fn sk_paint_new() ptr\n");
        k.push_str("extern fn sk_paint_unref(p: ptr)\n");
        k.push_str("extern fn sk_paint_set_color(p: ptr, color: i32)\n");
        k.push_str("extern fn sk_paint_set_alpha(p: ptr, alpha: f32)\n");
        k.push_str("extern fn sk_paint_set_anti_alias(p: ptr, aa: i32)\n");
        k.push_str("extern fn sk_paint_set_stroke(p: ptr, stroke: i32)\n");
        k.push_str("extern fn sk_paint_set_stroke_width(p: ptr, w: f32)\n");
        k.push_str("extern fn sk_font_new(typeface: ptr, size: f32) ptr\n");
        k.push_str("extern fn sk_font_unref(f: ptr)\n");
        k.push_str("extern fn sk_font_set_size(f: ptr, size: f32)\n");
        k.push_str("extern fn sk_typeface_create_from_name(name: ptr, style: i32) ptr\n");
        k.push_str("extern fn sk_typeface_unref(t: ptr)\n");
        k.push_str("extern fn sk_image_info_new(w: i32, h: i32, ct: i32, at: i32) ptr\n\n");

        // ── Kyle runtime externs ──
        k.push_str("extern fn ky_alloc(size: i64) ptr\n");
        k.push_str("extern fn ky_free(ptr)\n");
        k.push_str("extern fn ky_memset(ptr, val: i32, size: i64)\n");
        k.push_str("extern fn ky_now_ms() i64\n");
        k.push_str("extern fn ky_sleep(ms: i64)\n\n");

        // ── Constants ──
        k.push_str("final class Gl:\n");
        k.push_str("    ColorBufferBit: i32 = 0x00004000\n");
        k.push_str("    TriangleFan: i32 = 0x0006\n");
        k.push_str("    Texture2d: i32 = 0x0DE1\n");
        k.push_str("    Rgba: i32 = 0x1908\n");
        k.push_str("    UnsignedByte: i32 = 0x1401\n");
        k.push_str("    TextureMinFilter: i32 = 0x2800\n");
        k.push_str("    TextureMagFilter: i32 = 0x2801\n");
        k.push_str("    Nearest: i32 = 0x2600\n");
        k.push_str("    Linear: i32 = 0x2601\n");
        k.push_str("    Blend: i32 = 0x0BE2\n\n");
        
        k.push_str("final class SkiaCT:\n");
        k.push_str("    Rgba8888: i32 = 0\n\n");
        k.push_str("final class SkiaAT:\n");
        k.push_str("    Premul: i32 = 1\n\n");

        k.push_str("struct SkRect:\n    x: f32\n    y: f32\n    w: f32\n    h: f32\n\n");

        // ── Inline declarations (must precede code blocks for variable scoping) ──
        k.push_str("# ── Declarations ──\n");
        for node in &program.body {
            match node {
                UiNode::Expr(e) => {
                    let cleaned = e.replace("@", "");
                    let line = fix_declaration(&cleaned);
                    if !line.trim().is_empty() {
                        k.push_str(&line);
                        k.push('\n');
                    }
                }
                UiNode::CodeBlock(b) => {
                    let cleaned = b.replace("@", "").replace(":=", "=");
                    for line in cleaned.lines() {
                        if !line.trim().is_empty() {
                            k.push_str(line);
                            k.push('\n');
                        }
                    }
                }
                _ => {}
            }
        }
        // Emit code blocks (functions) after declarations
        k.push_str("# ── Functions ──\n");
        for block in &program.code_blocks {
            let cleaned = block.replace("@", "");
            k.push_str(&cleaned);
            if !cleaned.ends_with('\n') { k.push('\n'); }
        }
        k.push_str("# ── UI State ──\n");

        k.push_str("_win_w: i32 = 800\n_win_h: i32 = 600\n\n");

        // ── Hit targets (buttons, inputs) ──
        k.push_str("final class HitTarget:\n    x: i32\n    y: i32\n    w: i32\n    h: i32\n    id: str\n    handler: fn()\n\n");

        // Emit hit targets array
        k.push_str("fn _ky_noop(): 0\n\n");
        k.push_str("_hit_targets: ^[HitTarget] = [HitTarget{x: 0, y: 0, w: 0, h: 0, id: \"\", handler: _ky_noop}]\n\n");

        // ── Register hit targets ──
        k.push_str("fn ky_register_hit(x: i32, y: i32, w: i32, h: i32, id: &str, handler: fn()):\n");
        k.push_str("    _hit_targets.push(HitTarget{x: x, y: y, w: w, h: h, id: id, handler: handler})\n\n");

        // ── Hit test ──
        k.push_str("fn ky_hit_test(mx: i32, my: i32):\n");
        k.push_str("    i: ^i32 = 0\n");
        k.push_str("    while i < len(_hit_targets):\n");
        k.push_str("        t = _hit_targets[i]\n");
        k.push_str("        if mx >= t.x and mx <= t.x + t.w and my >= t.y and my <= t.y + t.h:\n");
        k.push_str("            t.handler()\n");
        k.push_str("            return\n");
        k.push_str("        i = i + 1\n\n");
        
        // ── Drawing helpers ──
        k.push_str("fn ky_draw_text(c: ptr, x: i32, y: i32, fs: i32, text: str, color: i32):\n");
        k.push_str("    paint = sk_paint_new()\n");
        k.push_str("    sk_paint_set_color(paint, color)\n");
        k.push_str("    sk_paint_set_anti_alias(paint, 1)\n");
        k.push_str("    font = sk_font_new(0 as ptr, fs as f32)\n");
        k.push_str("    sk_canvas_draw_simple_text(c, text as ptr, len(text) as i64, x as f32, y as f32, font, paint)\n");
        k.push_str("    sk_font_unref(font)\n");
        k.push_str("    sk_paint_unref(paint)\n\n");
        
        k.push_str("fn ky_draw_rect(c: ptr, x: i32, y: i32, w: i32, h: i32, color: i32):\n");
        k.push_str("    paint = sk_paint_new()\n");
        k.push_str("    sk_paint_set_color(paint, color)\n");
        k.push_str("    rect = SkRect{x: x as f32, y: y as f32, w: w as f32, h: h as f32}\n");
        k.push_str("    sk_canvas_draw_rect(c, &rect as ptr, paint)\n");
        k.push_str("    sk_paint_unref(paint)\n\n");
        
        k.push_str("fn ky_draw_button(c: ptr, x: i32, y: i32, w: i32, h: i32, text: str, color: i32):\n");
        k.push_str("    ky_draw_rect(c, x, y, w, h, color)\n");
        k.push_str("    ky_draw_text(c, x + 4, y + h / 2 + 6, 14, text, 0xFFFFFFFF)\n\n");
        
        k.push_str("fn ky_draw_checkbox(c: ptr, x: i32, y: i32, checked: i32, label: str):\n");
        k.push_str("    ky_draw_rect(c, x, y, 16, 16, 0xFFCCCCCC)\n");
        k.push_str("    if checked != 0:\n");
        k.push_str("        ky_draw_text(c, x + 2, y + 12, 12, \"\\u2713\", 0xFF333333)\n");
        k.push_str("    ky_draw_text(c, x + 22, y + 12, 14, label, 0xFF333333)\n\n");
        
        k.push_str("fn ky_draw_switch(c: ptr, x: i32, y: i32, on: i32):\n");
        k.push_str("    if on != 0:\n");
        k.push_str("        ky_draw_rect(c, x, y, 36, 20, 0xFF4CAF50)\n");
        k.push_str("    else:\n");
        k.push_str("        ky_draw_rect(c, x, y, 36, 20, 0xFFCCCCCC)\n\n");
        
        k.push_str("fn ky_draw_slider(c: ptr, x: i32, y: i32, width: i32, val: i32, min: i32):\n");
        k.push_str("    ky_draw_rect(c, x, y + 8, width, 4, 0xFFCCCCCC)\n");
        k.push_str("    fill_w = ((val - min) * width) / 100\n");
        k.push_str("    ky_draw_rect(c, x, y + 8, fill_w, 4, 0xFF4CAF50)\n");
        k.push_str("    ky_draw_text(c, x + width + 8, y + 12, 12, val.to_str(), 0xFF333333)\n\n");
        
        k.push_str("fn ky_draw_text_field(c: ptr, x: i32, y: i32, w: i32, text: str):\n");
        k.push_str("    ky_draw_rect(c, x, y, w, 24, 0xFFFFFFFF)\n");
        k.push_str("    ky_draw_rect(c, x, y, w, 24, 0xFFCCCCCC)\n");
        k.push_str("    ky_draw_text(c, x + 4, y + 16, 12, text, 0xFF333333)\n\n");
        
        k.push_str("fn ky_draw_image_placeholder(c: ptr, x: i32, y: i32, w: i32, h: i32, src: str):\n");
        k.push_str("    ky_draw_rect(c, x, y, w, h, 0xFFEEEEEE)\n");
        k.push_str("    ky_draw_text(c, x + 4, y + h / 2 + 6, 14, \"[img]\", 0xFF999999)\n\n");
        
        k.push_str("fn ky_draw_divider(c: ptr, x: i32, y: i32, w: i32):\n");
        k.push_str("    ky_draw_rect(c, x, y, w, 1, 0xFFCCCCCC)\n\n");
        
        k.push_str("fn ky_draw_progress(c: ptr, x: i32, y: i32, w: i32, val: i32, max: i32):\n");
        k.push_str("    ky_draw_rect(c, x, y, w, 8, 0xFFEEEEEE)\n");
        k.push_str("    fill_w = (val * w) / max\n");
        k.push_str("    ky_draw_rect(c, x, y, fill_w, 8, 0xFF4CAF50)\n\n");
        
        k.push_str("fn ky_draw_spinner(c: ptr, x: i32, y: i32):\n");
        k.push_str("    ky_draw_text(c, x, y + 12, 16, \"...\", 0xFF666666)\n\n");
        
        // ── Render function ──
        k.push_str("fn ky_render(c: ptr, fb_w: i32, fb_h: i32):\n");
        k.push_str("    _hit_targets = [HitTarget{x: 0, y: 0, w: 0, h: 0, id: \"\", handler: _ky_noop}]\n");
        k.push_str("    _hit_targets.clear()\n");
        k.push_str("    sk_canvas_clear(c, 0xFFF0F0F0)\n");
        let ui_nodes: Vec<&UiNode> = program.body.iter().filter(|n| {
            !matches!(n, UiNode::Expr(_) | UiNode::CodeBlock(_))
        }).collect();
        gen_nodes_ref(&ui_nodes, &mut k, 1, "c", "", 0, 0, true);
        k.push_str("\n");

        // ── Main ──
        k.push_str("fn main(args: {str}):\n");
        k.push_str("    result = glfw_init()\n");
        k.push_str("    if result == 0: return\n");
        k.push_str("    win = glfw_create_window(_win_w, _win_h, &(\"Kyle\" as str) as ptr, 0 as ptr, 0 as ptr)\n");
        k.push_str("    if win == 0 as ptr: glfw_terminate() return\n");
        k.push_str("    glfw_make_context_current(win)\n");
        k.push_str("    gl_enable(Gl.Blend)\n\n");

        k.push_str("    # Create Skia surface\n");
        k.push_str("    pixel_size = _win_w * _win_h * 4\n");
        k.push_str("    pixels = ky_alloc(pixel_size as i64)\n");
        k.push_str("    ky_memset(pixels, 0, pixel_size as i64)\n");
        k.push_str("    info = sk_image_info_new(_win_w, _win_h, SkiaCT.Rgba8888, SkiaAT.Premul)\n");
        k.push_str("    sk_surface = sk_surface_new_raster(info, (_win_w * 4) as i64, pixels)\n");
        k.push_str("    canvas = sk_surface_get_canvas(sk_surface)\n\n");

        k.push_str("    # Create OpenGL texture\n");
        k.push_str("    tex: i32 = 0\n    gl_gen_texture(1, ^&tex)\n\n");

        k.push_str("    # Mouse state\n");
        k.push_str("    mouse_x: ^f64 = 0.0\n    mouse_y: ^f64 = 0.0\n");
        k.push_str("    mouse_down: ^i32 = 0\n\n");
        
        k.push_str("    # Main loop\n");
        k.push_str("    running: ^i32 = 1\n");
        k.push_str("    while running != 0:\n");
        k.push_str("        glfw_poll_events()\n");
        k.push_str("        if glfw_window_should_close(win) != 0: running = 0\n\n");
        
        k.push_str("        # Poll mouse\n");
        k.push_str("        glfw_get_cursor_pos(win, ^&mouse_x, ^&mouse_y)\n");
        k.push_str("        mb = glfw_get_mouse_button(win, 0)\n");
        k.push_str("        if mb == 1 and mouse_down == 0:\n");
        k.push_str("            ky_hit_test(mouse_x as i32, mouse_y as i32)\n");
        k.push_str("        mouse_down = mb\n\n");
        
        k.push_str("        # Get framebuffer size\n");
        k.push_str("        fb_w: ^i32 = 0\n        fb_h: ^i32 = 0\n");
        k.push_str("        glfw_get_framebuffer_size(win, ^&fb_w, ^&fb_h)\n\n");
        
        k.push_str("        # Render\n");
        k.push_str("        ky_render(canvas, fb_w, fb_h)\n\n");

        k.push_str("        # Blit to screen\n");
        k.push_str("        gl_viewport(0, 0, fb_w, fb_h)\n");
        k.push_str("        gl_clear_color(0.0, 0.0, 0.0, 1.0)\n");
        k.push_str("        gl_clear(Gl.ColorBufferBit)\n");

        // Blit texture quad
        k.push_str("        gl_bind_texture(Gl.Texture2d, tex)\n");
        k.push_str("        gl_tex_image_2d(Gl.Texture2d, 0, Gl.Rgba, _win_w, _win_h, 0, Gl.Rgba, Gl.UnsignedByte, pixels)\n");
        k.push_str("        gl_tex_parameter_i(Gl.Texture2d, Gl.TextureMinFilter, Gl.Nearest)\n");
        k.push_str("        gl_tex_parameter_i(Gl.Texture2d, Gl.TextureMagFilter, Gl.Nearest)\n");

        // Full-screen quad
        k.push_str("        gl_begin(Gl.TriangleFan)\n");
        k.push_str("        gl_tex_coord_2f(0.0, 0.0) gl_vertex_2f(-1.0, -1.0)\n");
        k.push_str("        gl_tex_coord_2f(1.0, 0.0) gl_vertex_2f(1.0, -1.0)\n");
        k.push_str("        gl_tex_coord_2f(1.0, 1.0) gl_vertex_2f(1.0, 1.0)\n");
        k.push_str("        gl_tex_coord_2f(0.0, 1.0) gl_vertex_2f(-1.0, 1.0)\n");
        k.push_str("        gl_end()\n");
        k.push_str("        glfw_swap_buffers(win)\n");
        k.push_str("        ky_sleep(16)\n");

        // Cleanup
        k.push_str("    glfw_destroy_window(win)\n    glfw_terminate()\n");

        BackendOutput {
            files: vec![GeneratedFile { path: "main.ky".to_string(), content: k }],
            html_shell: None,
        }
    }
}

fn gen_nodes(nodes: &[UiNode], k: &mut String, indent: usize, canvas: &str,
             parent_var: &str, bx: i32, by: i32, top_level: bool) -> i32 {
    let mut y = by;
    for n in nodes {
        y = gen_node(n, k, indent, canvas, parent_var, bx, y, top_level);
    }
    y
}

fn gen_nodes_ref(nodes: &[&UiNode], k: &mut String, indent: usize, canvas: &str,
                 parent_var: &str, bx: i32, by: i32, top_level: bool) -> i32 {
    let mut y = by;
    for n in nodes {
        y = gen_node(n, k, indent, canvas, parent_var, bx, y, top_level);
    }
    y
}

fn gen_node(node: &UiNode, k: &mut String, indent: usize, canvas: &str,
            parent_var: &str, bx: i32, by: i32, top_level: bool) -> i32 {
    match node {
        UiNode::Element { tag, attrs, children } => {
            let children = children.as_slice();
            gen_element(tag, attrs, children, k, indent, canvas, parent_var, bx, by, top_level)
        }
        UiNode::SelfClosing { tag, attrs } => {
            gen_element(tag, attrs, &[], k, indent, canvas, parent_var, bx, by, top_level)
        }
        UiNode::Slot { .. } => by,
        UiNode::If { condition, then_branch, else_branch } => {
            let cond = condition.trim_start_matches('@');
            emit(k, indent, &format!("if {}:", cond));
            gen_nodes(then_branch, k, indent + 1, canvas, parent_var, bx, by + 10, false);
            if !else_branch.is_empty() {
                emit(k, indent, "else:");
                gen_nodes(else_branch, k, indent + 1, canvas, parent_var, bx, by + 10, false);
            }
            by
        }
        UiNode::For { item, list, body } => {
            let list_var = list.trim_start_matches('@');
            emit(k, indent, &format!("for {} in {}:", item, list_var));
            gen_nodes(body, k, indent + 1, canvas, parent_var, bx, by + 10, false);
            by
        }
        UiNode::Match { expr, cases } => {
            let e = expr.trim_start_matches('@');
            emit(k, indent, &format!("match {}:", e));
            for case in cases {
                emit(k, indent + 1, &format!("{}:", case.pattern));
                gen_nodes(&case.body, k, indent + 2, canvas, parent_var, bx, by + 10, false);
            }
            by
        }
        UiNode::Expr(e) => {
            let line = e.replace("@", "").replace(":=", "=");
            if !line.is_empty() {
                emit(k, indent, &line);
            }
            by
        }
        UiNode::CodeBlock(b) => {
            let cleaned = b.replace("@", "").replace(":=", "=");
            for line in cleaned.lines() {
                if !line.trim().is_empty() {
                    emit(k, indent, line.trim());
                }
            }
            by
        }
        _ => by,
    }
}

fn gen_element(tag: &ComponentTag, attrs: &[UiAttr], children: &[UiNode],
               k: &mut String, indent: usize, canvas: &str,
               parent_var: &str, bx: i32, by: i32, top_level: bool) -> i32 {
    match tag {
        ComponentTag::Text => {
                    let val = get_str_attr(attrs, "value").unwrap_or("");
                    let fs = get_int_attr(attrs, "font_size").unwrap_or(16);
                    let color_str = get_str_attr(attrs, "color").unwrap_or("#1A1A1A");
                    let (r, g, b) = parse_hex_color(color_str);
                    let color: u32 = (255u32 << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32);
                    let x = get_int_attr(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(by + 30);

                    let text_expr = interpolate_value(val);
                    emit(k, indent, &format!(
                        "ky_draw_text({}, {}, {}, {}, {}, {})",
                        canvas, x, y, fs, text_expr, color
                    ));
                    by + fs + 8
                }

                ComponentTag::Button => {
                    let text = get_str_attr(attrs, "text").unwrap_or("Button");
                    let text_expr = interpolate_value(text);
                    let x = get_int_attr(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(by + 30);
                    let w = get_int_attr(attrs, "width").unwrap_or(100);
                    let h = 36;
                    let handler = get_expr_attr(attrs, "click").unwrap_or("");

                    emit(k, indent, &format!(
                        "ky_draw_button({}, {}, {}, {}, {}, {}, 0xFFFF6600)",
                        canvas, x, y, w, h, text_expr
                    ));
                    if !handler.is_empty() {
                        let clean = handler.trim_start_matches('@');
                        emit(k, indent, &format!(
                            "ky_register_hit({}, {}, {}, {}, &\"{}\", &{})",
                            x, y, w, h, clean, clean
                        ));
                    }
                    y + h + 8
                }

                ComponentTag::View | ComponentTag::VStack => {
                    let pad = get_int_attr(attrs, "padding").unwrap_or(10);
                    let spacing = get_int_attr(attrs, "spacing").unwrap_or(8);
                    let mut cy = by + pad;
                    for child in children {
                        cy = gen_node(child, k, indent, canvas, parent_var, bx + pad, cy, false);
                        cy += spacing;
                    }
                    cy
                }

                ComponentTag::HStack => {
                    let pad = get_int_attr(attrs, "padding").unwrap_or(10);
                    let spacing = get_int_attr(attrs, "spacing").unwrap_or(8);
                    let mut cx = bx + pad;
                    for child in children {
                        cx = gen_node(child, k, indent, canvas, parent_var, cx, by + pad, false) + spacing - bx;
                        cx += bx;  // Add back base x offset
                    }
                    by + pad + 30  // approximate height
                }

                ComponentTag::Card => {
                    let pad = get_int_attr(attrs, "padding").unwrap_or(16);
                    let x = bx + 10;
                    let y = by + 10;
                    let card_w = get_int_attr(attrs, "width").unwrap_or(300);
                    let card_h = 120;  // approximate

                    // Draw card background
                    emit_hex_color(k, indent, canvas, x, y, card_w, card_h, 0xFFFFFFFF);
                    gen_nodes(children, k, indent, canvas, parent_var, x + pad, y + pad, false);
                    y + card_h + 10
                }

                ComponentTag::Checkbox => {
                    let x = get_int_attr(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(by + 30);
                    let checked = get_str_attr(attrs, "checked").unwrap_or("");
                    let label = get_str_attr(attrs, "label").unwrap_or("");
                    let bind_var = get_expr_attr(attrs, "bind").unwrap_or("");

                    let is_checked = if !checked.is_empty() {
                        format!("({}) as i32", checked.trim_start_matches('@'))
                    } else if !bind_var.is_empty() {
                        let v = bind_var.trim_start_matches('@');
                        format!("{} as i32", v)
                    } else {
                        "0".to_string()
                    };

                    let label_expr = interpolate_value(label);
                    emit(k, indent, &format!(
                        "ky_draw_checkbox({}, {}, {}, {}, {})",
                        canvas, x, y, is_checked, label_expr
                    ));
                    y + 28
                }

                ComponentTag::Switch => {
                    let x = get_int_attr(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(by + 30);
                    let checked = get_str_attr(attrs, "checked").unwrap_or("");
                    let is_on = if !checked.is_empty() {
                        format!("({}) as i32", checked.trim_start_matches('@'))
                    } else {
                        "0".to_string()
                    };
                    emit(k, indent, &format!(
                        "ky_draw_switch({}, {}, {}, {})",
                        canvas, x, y, is_on
                    ));
                    y + 28
                }

                ComponentTag::Slider => {
                    let x = get_int_attr(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(by + 30);
                    let val = get_int_attr(attrs, "value").unwrap_or(50);
                    let min = get_int_attr(attrs, "min").unwrap_or(0);
                    let max = get_int_attr(attrs, "max").unwrap_or(100);
                    let width = get_int_attr(attrs, "width").unwrap_or(200);

                    emit(k, indent, &format!(
                        "ky_draw_slider({}, {}, {}, {}, {}, {})",
                        canvas, x, y, width, val, min
                    ));
                    y + 30
                }

                ComponentTag::TextField => {
                    let x = get_int_attr(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(by + 30);
                    let placeholder = get_str_attr(attrs, "placeholder").unwrap_or("");
                    let val = get_str_attr(attrs, "value").unwrap_or("");
                    let width = get_int_attr(attrs, "width").unwrap_or(250);

                    let display = if !val.is_empty() { val } else { placeholder };
                    let display_expr = interpolate_value(display);
                    emit(k, indent, &format!(
                        "ky_draw_text_field({}, {}, {}, {}, {})",
                        canvas, x, y, width, display_expr
                    ));
                    y + 30
                }

                ComponentTag::Image => {
                    let x = get_int_attr(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(by + 30);
                    let w = get_int_attr(attrs, "width").unwrap_or(100);
                    let h = get_int_attr(attrs, "height").unwrap_or(100);
                    let src = get_str_attr(attrs, "src").unwrap_or("");

                    let src_expr = interpolate_value(src);
                    emit(k, indent, &format!(
                        "ky_draw_image_placeholder({}, {}, {}, {}, {}, {})",
                        canvas, x, y, w, h, src_expr
                    ));
                    y + h + 10
                }

                ComponentTag::Spacer => {
                    by + 20
                }

                ComponentTag::Divider => {
                    let x = bx + 10;
                    let y = by + 10;
                    let w = get_int_attr(attrs, "width").unwrap_or(300);
                    emit(k, indent, &format!(
                        "ky_draw_divider({}, {}, {}, {})",
                        canvas, x, y, w
                    ));
                    y + 12
                }

                ComponentTag::Progress => {
                    let x = get_int_attr(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(by + 30);
                    let val = get_int_attr(attrs, "value").unwrap_or(0);
                    let w = get_int_attr(attrs, "width").unwrap_or(200);
                    let max = get_int_attr(attrs, "max").unwrap_or(100);

                    emit(k, indent, &format!(
                        "ky_draw_progress({}, {}, {}, {}, {}, {})",
                        canvas, x, y, w, val, max
                    ));
                    y + 20
                }

                ComponentTag::Spinner => {
                    let x = get_int_attr(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(by + 30);
                    emit(k, indent, &format!(
                        "ky_draw_spinner({}, {}, {})",
                        canvas, x, y
                    ));
                    y + 24
                }

                ComponentTag::ZStack => {
                    // All children render at same position
                    for child in children {
                        gen_node(child, k, indent, canvas, parent_var, bx, by, false);
                    }
                    by + 100
                }

                ComponentTag::Scroll => {
                    // Pass through — desktop handles overflow naturally
                    gen_nodes(children, k, indent, canvas, parent_var, bx, by, false)
                }

                ComponentTag::Form => {
                    gen_nodes(children, k, indent, canvas, parent_var, bx, by, false)
                }

                ComponentTag::Select => {
                    let x = get_int_attr(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int_attr(attrs, "y").unwrap_or(by + 30);
                    let selected = get_str_attr(attrs, "value").unwrap_or("Select...");
                    let selected_expr = interpolate_value(selected);
                    let w = get_int_attr(attrs, "width").unwrap_or(200);

                    emit(k, indent, &format!(
                        "ky_draw_text_field({}, {}, {}, {}, {})",
                        canvas, x, y, w, selected_expr
                    ));
                    let mut cy = y + 30;
                    for child in children {
                        if let UiNode::Element { tag, attrs, .. } = child {
                            if *tag == ComponentTag::Option {
                                let opt_val = get_str_attr(attrs, "value").unwrap_or("");
                                let opt_expr = interpolate_value(opt_val);
                                emit(k, indent, &format!(
                                    "ky_draw_text({}, {}, 14, {}, {}, 0xFF666666)",
                                    canvas, x + 5, cy, opt_expr
                                ));
                                cy += 20;
                            }
                        }
                    }
                    cy
                }

                ComponentTag::App => {
                    gen_nodes(children, k, indent, canvas, parent_var, bx, by, false)
                }

                _ => {
                    gen_nodes(children, k, indent, canvas, parent_var, bx, by, false)
                }
            }
        }
// ── Helper functions ──

/// Fix a declaration from `name := val` to `name: ^type = val` for Kyle mutable syntax
fn fix_declaration(s: &str) -> String {
    let trimmed = s.trim();
    // Handle `name := value` (Walrus operator in .kyx code)
    if let Some(wp) = trimmed.find(":=") {
        let name = trimmed[..wp].trim();
        let value = trimmed[wp + 2..].trim();
        let typ = if value.starts_with('"') || value.starts_with('\'') { "str" } else { "i32" };
        return format!("{}: ^{} = {}", name, typ, value);
    }
    s.to_string()
}

/// Convert a string with @var references to a Kyle expression.
/// "Clicks: @count" -> "\"Clicks: \" + count.to_str()"
/// "plain" -> "\"plain\""
fn interpolate_value(s: &str) -> String {
    if !s.contains('@') {
        return format!("\"{}\"", s);
    }
    let mut result = String::new();
    let mut remaining = s;
    let mut first = true;

    while let Some(at_pos) = remaining.find('@') {
        let before = &remaining[..at_pos];
        let after_at = &remaining[at_pos + 1..];
        let var_end = after_at.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(after_at.len());
        let var_name = &after_at[..var_end];

        if !before.is_empty() {
            if !first { result.push_str(" + "); }
            result.push_str(&format!("\"{}\"", before));
            first = false;
        }
        if !first { result.push_str(" + "); }
        result.push_str(&format!("{}.to_str()", var_name));
        first = false;

        remaining = &after_at[var_end..];
    }

    if !remaining.is_empty() {
        if !first { result.push_str(" + "); }
        result.push_str(&format!("\"{}\"", remaining));
    }

    result
}

fn emit(k: &mut String, indent: usize, line: &str) {
    k.push_str(&"    ".repeat(indent));
    k.push_str(line);
    k.push('\n');
}

fn get_str_attr<'a>(attrs: &'a [UiAttr], key: &str) -> Option<&'a str> {
    for a in attrs {
        if a.name == key {
            if let AttrValue::String(ref s) = a.value { return Some(s.as_str()); }
        }
    }
    None
}

fn get_int_attr(attrs: &[UiAttr], key: &str) -> Option<i32> {
    for a in attrs {
        if a.name == key {
            match &a.value {
                AttrValue::String(s) => return s.parse().ok(),
                AttrValue::Expr(e) => {
                    let clean = e.trim_start_matches('@');
                    return clean.parse().ok();
                }
                _ => {}
            }
        }
    }
    None
}

fn get_expr_attr<'a>(attrs: &'a [UiAttr], key: &str) -> Option<&'a str> {
    for a in attrs {
        if a.name == key {
            if let AttrValue::Expr(ref e) = a.value { return Some(e.as_str()); }
        }
    }
    None
}

fn emit_hex_color(k: &mut String, indent: usize, canvas: &str, x: i32, y: i32, w: i32, h: i32, color: u32) {
    emit(k, indent, &format!(
        "ky_draw_rect({}, {}, {}, {}, {}, {})",
        canvas, x, y, w, h, color
    ));
}

fn parse_hex_color(s: &str) -> (u32, u32, u32) {
    let s = s.trim_start_matches('#');
    if s.len() >= 6 {
        let r = u32::from_str_radix(&s[0..2], 16).unwrap_or(0);
        let g = u32::from_str_radix(&s[2..4], 16).unwrap_or(0);
        let b = u32::from_str_radix(&s[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else {
        (0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_desktop() {
        let b = DesktopBackend::new();
        let p = UiProgram {
            routes: vec![], code_blocks: vec![], styles: vec![], animations: vec![],
            component_renderers: vec![],
            body: vec![UiNode::SelfClosing {
                tag: ComponentTag::Text, attrs: vec![]
            }],
        };
        let o = b.generate(&p);
        let c = &o.files[0].content;
        assert!(c.contains("ky_draw_text"));
        assert!(c.contains("glfw_init"));
        assert!(c.contains("sk_canvas_draw_simple_text"));
    }
}
