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
        k.push_str("# Kyle UI — Desktop (SDL2)\n\n");
        k.push_str("@link \"-L/opt/homebrew/lib\"\n@link \"-L/usr/local/lib\"\n@link \"SDL2\"\n\n");

        k.push_str("extern fn SDL_Init(flags: i32) i32\nextern fn SDL_Quit()\n");
        k.push_str("extern fn SDL_CreateWindow(title: ptr, x: i32, y: i32, w: i32, h: i32, f: i32) ptr\n");
        k.push_str("extern fn SDL_DestroyWindow(w: ptr)\n");
        k.push_str("extern fn SDL_CreateRenderer(w: ptr, idx: i32, f: i32) ptr\n");
        k.push_str("extern fn SDL_DestroyRenderer(r: ptr)\n");
        k.push_str("extern fn SDL_SetRenderDrawColor(r: ptr, r2: i32, g2: i32, b2: i32, a2: i32) i32\n");
        k.push_str("extern fn SDL_RenderClear(r: ptr) i32\n");
        k.push_str("extern fn SDL_RenderFillRect(r: ptr, rect: ptr) i32\n");
        k.push_str("extern fn SDL_RenderPresent(r: ptr)\n");
        k.push_str("extern fn SDL_PollEvent(event: ptr) i32\n");
        k.push_str("extern fn SDL_Delay(ms: i32)\n\n");

        k.push_str("struct SDL_Rect:\n");
        k.push_str("    x: i32\n    y: i32\n    w: i32\n    h: i32\n\n");

        k.push_str("fn fill_rect(ren: ptr, x: i32, y: i32, rw: i32, rh: i32):\n");
        k.push_str("    rect = SDL_Rect{x: x, y: y, w: rw, h: rh}\n");
        k.push_str("    SDL_RenderFillRect(ren, &rect as ptr)\n\n");

        k.push_str("fn render_app(ren: ptr, w: i32, h: i32):\n");
        gen_nodes(&program.body, &mut k, 1, "ren", 0, 0);
        k.push_str("\n");

        k.push_str("fn main(args: {str}):\n");
        k.push_str("    SDL_Init(32)\n");
        k.push_str("    win = SDL_CreateWindow(\"Kyle\" as ptr, 0x2FFF0000, 0x2FFF0000, 800, 600, 4)\n");
        k.push_str("    if win == 0 as ptr: SDL_Quit() return\n");
        k.push_str("    ren = SDL_CreateRenderer(win, -1, 1)\n");
        k.push_str("    if ren == 0 as ptr: SDL_DestroyWindow(win) SDL_Quit() return\n");
        k.push_str("    # Allocate event buffer (128 bytes for SDL_Event)\n");
        k.push_str("    run: ^i32 = 1\n    ev: [i32, 8] = [0,0,0,0,0,0,0,0]\n    while run != 0:\n");
        k.push_str("        while SDL_PollEvent(&ev as ptr) != 0:\n");
        k.push_str("            if ev[0] == 256: run = 0  # SDL_QUIT\n");
        k.push_str("        SDL_SetRenderDrawColor(ren, 230, 230, 230, 255)\n");
        k.push_str("        SDL_RenderClear(ren)\n");
        k.push_str("        render_app(ren, 800, 600)\n");
        k.push_str("        SDL_RenderPresent(ren)\n");
        k.push_str("        SDL_Delay(16)\n");
        k.push_str("    SDL_DestroyRenderer(ren)\n    SDL_DestroyWindow(win)\n    SDL_Quit()\n");

        BackendOutput {
            files: vec![GeneratedFile { path: "main.ky".to_string(), content: k }],
            html_shell: None,
        }
    }
}

fn gen_nodes(nodes: &[UiNode], k: &mut String, indent: usize, ren: &str, bx: i32, by: i32) -> i32 {
    let mut y = by;
    for n in nodes { y = gen_node(n, k, indent, ren, bx, y); }
    y
}

fn gen_node(node: &UiNode, k: &mut String, indent: usize, ren: &str, bx: i32, by: i32) -> i32 {
    match node {
        UiNode::Element { tag, attrs, children } => {
            match tag {
                ComponentTag::Text => {
                    let x = get_int(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int(attrs, "y").unwrap_or(by + 30);
                    emit(k, indent, "SDL_SetRenderDrawColor(ren, 240, 240, 240, 255)");
                    emit(k, indent, &format!("fill_rect(ren, {}, {}, 200, 24)", x, y));
                    emit(k, indent, "SDL_SetRenderDrawColor(ren, 0, 102, 255, 255)");
                    emit(k, indent, &format!("fill_rect(ren, {}, {}, 4, 24)", x, y));
                    by + 34
                }
                ComponentTag::Button => {
                    let x = get_int(attrs, "x").unwrap_or(bx + 20);
                    let y = get_int(attrs, "y").unwrap_or(by + 30);
                    emit(k, indent, "SDL_SetRenderDrawColor(ren, 0, 102, 255, 255)");
                    emit(k, indent, &format!("fill_rect(ren, {}, {}, 100, 36)", x, y));
                    by + 46
                }
                ComponentTag::View | ComponentTag::VStack | ComponentTag::HStack |
                ComponentTag::ZStack | ComponentTag::Card | ComponentTag::Spacer | ComponentTag::Surface => {
                    let pad = get_int(attrs, "padding").unwrap_or(10);
                    gen_nodes(children, k, indent, ren, bx + pad, by + pad)
                }
                ComponentTag::FilePicker => by,
                _ => gen_nodes(children, k, indent, ren, bx, by),
            }
        }
        UiNode::SelfClosing { tag, attrs } => gen_node(
            &UiNode::Element { tag: tag.clone(), attrs: attrs.clone(), children: vec![] },
            k, indent, ren, bx, by),
        _ => by,
    }
}

fn emit(k: &mut String, indent: usize, line: &str) {
    k.push_str(&"    ".repeat(indent));
    k.push_str(line);
    k.push('\n');
}

fn get_int(attrs: &[UiAttr], key: &str) -> Option<i32> {
    for a in attrs {
        if a.name == key {
            if let AttrValue::String(ref s) = a.value { return s.parse().ok(); }
        }
    }
    None
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
        assert!(o.files[0].content.contains("fill_rect"));
    }
}
