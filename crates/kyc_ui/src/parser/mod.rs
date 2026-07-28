use crate::ast::*;

/// Parse a .kyx file content into a KyxFile AST.
pub fn parse(source: &str) -> Result<KyxFile, String> {
    let mut parser = KyxParser::new(source);
    parser.parse_file()
}

struct KyxParser {
    chars: Vec<char>,
    pos: usize,
}

impl KyxParser {
    fn new(source: &str) -> Self {
        Self { chars: source.chars().collect(), pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' { self.advance(); }
            else { break; }
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        match self.advance() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(format!("Expected '{}', found '{}'", expected, c)),
            None => Err(format!("Expected '{}', found EOF", expected)),
        }
    }

    fn parse_file(&mut self) -> Result<KyxFile, String> {
        let mut imports = Vec::new();
        let mut routes = Vec::new();
        let mut code_blocks = Vec::new();
        let mut styles = Vec::new();
        let mut animations = Vec::new();
        let mut body = Vec::new();

        loop {
            self.skip_whitespace();
            match self.peek() {
                None => break,
                Some('#') => {
                    while self.peek() != Some('\n') && self.peek().is_some() { self.advance(); }
                }
                Some('f') if self.starts_with("from ") => {
                    // Parse: from views.home import home
                    self.pos += 5; // "from "
                    let module = self.read_while(|c| c != ' ' && c != '\t' && c != '\n')?;
                    self.skip_whitespace();
                    self.expect_string("import")?;
                    self.skip_whitespace();
                    let name = self.read_while(|c| c != '\n' && c != ' ' && c != '\t')?;
                    imports.push(crate::ast::ImportDecl { module, name });
                }
                Some('s') if self.starts_with("style<") => {
                    styles.push(self.parse_style_decl("style", false)?);
                }
                Some('l') if self.starts_with("layout<") => {
                    styles.push(self.parse_style_decl("layout", false)?);
                }
                Some('t') if self.starts_with("tpl<") => {
                    styles.push(self.parse_style_decl("tpl", false)?);
                }
                Some('t') if self.starts_with("theme ") => {
                    self.pos += 5;
                    let name = self.read_while(|c| c != ':' && c != '\n')?.trim().to_string();
                    self.expect_char(':')?;
                    let props = self.parse_style_props()?;
                    styles.push(StyleDecl::Theme { name, props });
                }
                Some('a') if self.starts_with("animation<") => {
                    self.pos += 9;
                    self.expect_char('<')?;
                    let component = self.read_while(|c| c != '>')?;
                    self.expect_char('>')?;
                    self.skip_whitespace();
                    let name = self.read_while(|c| c != ':' && c != ' ' && c != '\t')?.trim().to_string();
                    self.expect_char(':')?;
                    let props = self.parse_style_props()?;
                    animations.push(AnimDecl { component, name, props });
                }
                Some('@') => {
                    self.advance();
                    if self.peek() == Some('(') {
                        let content_start = self.pos + 1; // after (
                        self.advance();
                        let after_paren = self.find_matching_paren()?;
                        let block = self.extract_and_advance(after_paren, content_start);
                        code_blocks.push(block);
                    } else if self.starts_with("if(") {
                        self.pos += 2;
                        self.advance();
                        let condition = self.read_while(|c| c != ')')?;
                        self.expect_char(')')?;
                        self.expect_char(':')?;
                        let then_branch = self.parse_body_until(&["@else", "@elif"])?;
                        let mut else_branch = Vec::new();
                        if self.starts_with("@else") {
                            self.pos += 5;
                            while self.peek() != Some(':') && self.peek() != Some('\n') && self.peek().is_some() { self.advance(); }
                            if self.peek() == Some(':') { self.advance(); }
                            else_branch = self.parse_body_until(&[])?;
                        }
                        body.push(KyxNode::If { condition, then_branch, else_branch });
                    } else if self.starts_with("for(") {
                        self.pos += 3;
                        let item = self.read_while(|c| c != ' ' && c != '\t')?.trim().to_string();
                        self.skip_whitespace();
                        self.expect_string("in")?;
                        self.skip_whitespace();
                        let list = self.read_while(|c| c != ')')?.trim().to_string();
                        self.expect_char(')')?;
                        self.expect_char(':')?;
                        let body_content = self.parse_body_until(&[])?;
                        body.push(KyxNode::For { item, list, body: body_content });
                    } else if self.starts_with("match(") {
                        self.pos += 5;
                        let expr = self.read_while(|c| c != ')')?.trim().to_string();
                        self.expect_char(')')?;
                        self.expect_char(':')?;
                        let mut cases = Vec::new();
                        loop {
                            self.skip_whitespace();
                            if self.peek().is_none() || self.peek() == Some('<') { break; }
                            let pattern = self.read_while(|c| c != ':')?.trim().to_string();
                            if pattern.is_empty() { break; }
                            self.expect_char(':')?;
                            let case_body = self.parse_body_until(&["@case"])?;
                            cases.push(MatchCase { pattern, body: case_body });
                        }
                        body.push(KyxNode::Match { expr, cases });
                    } else {
                        let expr = self.read_while(|c| c != '\n')?.trim().to_string();
                        body.push(KyxNode::Expr(expr));
                    }
                }
                Some('<') => {
                    let node = self.parse_element()?;
                    body.push(node);
                }
                Some('f') if self.starts_with("fn ") => {
                    // fn declarations at top level are Kyle code blocks.
                    // Consume the fn line + all indented lines that follow.
                    let mut block = String::new();
                    loop {
                        match self.peek() {
                            None => break,
                            Some('\n') => {
                                block.push('\n');
                                self.advance();
                                // Check if next line is indented (part of fn body)
                                if self.peek().map_or(true, |ch| ch != ' ' && ch != '\t') {
                                    break;
                                }
                                // Read the indented line
                                while let Some(c) = self.peek() {
                                    if c == '\n' { break; }
                                    block.push(c);
                                    self.advance();
                                }
                            }
                            Some(c) => {
                                block.push(c);
                                self.advance();
                            }
                        }
                    }
                    if !block.trim().is_empty() {
                        code_blocks.push(block.trim().to_string());
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Ok(KyxFile { imports, routes, code_blocks, styles, animations, body })
    }

    fn parse_style_decl(&mut self, kind: &str, _is_theme: bool) -> Result<StyleDecl, String> {
        // style<comp> Name: or layout<comp> Name: or tpl<comp> Name:
        for c in kind.chars() { self.advance(); }
        self.expect_char('<')?;
        let component = self.read_while(|c| c != '>')?;
        self.expect_char('>')?;
        self.skip_whitespace();
        let name = self.read_while(|c| c != ':' && c != ' ' && c != '\t')?.trim().to_string();
        self.expect_char(':')?;
        let (props, media) = self.parse_style_block()?;
        match kind {
            "layout" => Ok(StyleDecl::Layout { component, name, props, media }),
            "tpl" => Ok(StyleDecl::Template { component, name, props, media }),
            _ => Ok(StyleDecl::Style { component, name, props, media }),
        }
    }

    fn parse_style_props(&mut self) -> Result<Vec<StyleProp>, String> {
        let mut props = Vec::new();
        // Expect newline + indent
        if !self.at_newline() { return Ok(props); }
        self.advance();
        if !self.at_indent() { return Ok(props); }
        self.advance();
        loop {
            self.skip_whitespace_inline();
            match self.peek() {
                None | Some('\n') | Some('\r') => break,
                _ => {}
            }
            let name = self.read_while(|c| c != ' ' && c != '\t' && c != '=')?.trim().to_string();
            if name.is_empty() { break; }
            self.skip_whitespace();
            if self.peek() == Some('=') {
                self.advance();
                self.skip_whitespace();
                let value = self.read_while(|c| c != '\n' && c != '\r')?.trim().to_string();
                props.push(StyleProp { name, value });
            }
            // Skip to next line
            while self.peek() == Some(' ') || self.peek() == Some('\t') { self.advance(); }
            if self.peek() == Some('\n') || self.peek() == Some('\r') {
                self.advance();
                while self.peek() == Some('\n') || self.peek() == Some('\r') { self.advance(); }
                if !self.at_indent() && !self.at_space_or_tab() { break; }
            }
        }
        Ok(props)
    }

    fn parse_style_block(&mut self) -> Result<(Vec<StyleProp>, Vec<MediaQuery>), String> {
        let mut props = Vec::new();
        let mut media = Vec::new();
        if !self.at_newline() { return Ok((props, media)); }
        self.advance();
        if !self.at_indent() { return Ok((props, media)); }
        self.advance();
        loop {
            self.skip_whitespace_inline();
            match self.peek() {
                None | Some('\n') | Some('\r') => break,
                Some('@') if self.starts_with("@media(") => {
                    // Parse @media query
                    self.pos += 6; // skip "@media"
                    let condition = self.read_while(|c| c != ')')?.trim().to_string();
                    self.expect_char(')')?;
                    self.expect_char(':')?;
                    let media_props = self.parse_style_props()?;
                    media.push(MediaQuery { condition, props: media_props });
                }
                _ => {
                    let name = self.read_while(|c| c != ' ' && c != '\t' && c != '=')?.trim().to_string();
                    if name.is_empty() { break; }
                    self.skip_whitespace();
                    if self.peek() == Some('=') {
                        self.advance();
                        self.skip_whitespace();
                        let value = self.read_while(|c| c != '\n' && c != '\r')?.trim().to_string();
                        props.push(StyleProp { name, value });
                    }
                }
            }
            while self.peek() == Some(' ') || self.peek() == Some('\t') { self.advance(); }
            if self.peek() == Some('\n') || self.peek() == Some('\r') {
                self.advance();
                while self.peek() == Some('\n') || self.peek() == Some('\r') { self.advance(); }
                if !self.at_indent() && !self.at_space_or_tab() { break; }
            }
        }
        Ok((props, media))
    }

    fn at_newline(&self) -> bool {
        self.peek() == Some('\n') || self.peek() == Some('\r')
    }

    fn at_indent(&self) -> bool {
        self.peek() == Some('\t') || self.peek() == Some(' ')
    }

    fn at_space_or_tab(&self) -> bool {
        self.peek() == Some(' ') || self.peek() == Some('\t')
    }

    fn skip_whitespace_inline(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' { self.advance(); }
            else { break; }
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        let cs: Vec<char> = s.chars().collect();
        self.chars[self.pos..].starts_with(&cs)
    }

    fn parse_at_directive(&mut self) -> Result<KyxNode, String> {
        // @(...) code block
        if self.peek() == Some('(') {
            let content_start = self.pos + 1;
            self.advance();
            let after_paren = self.find_matching_paren()?;
            let block = self.extract_and_advance(after_paren, content_start);
            return Ok(KyxNode::CodeBlock(block));
        }
        // @if cond: ... @else: ... or @if(cond): ... @else: ... or @for(item in list): or @match(expr): or @expr
        if self.starts_with("if ") || self.starts_with("if(") {
            self.pos += 2; // skip 'if'
            self.skip_whitespace();
            let condition = if self.peek() == Some('(') {
                self.advance(); // skip '('
                let cond = self.read_while(|c| c != ')')?;
                self.expect_char(')')?;
                cond
            } else {
                self.read_while(|c| c != ':' && c != '\n')?.trim().to_string()
            };
            self.expect_char(':')?;
            let then_branch = self.parse_body_until(&["@else", "@elif"])?;
            let mut else_branch = Vec::new();
            if self.starts_with("@else") {
                self.pos += 5;
                if self.peek() == Some(' ') || self.peek() == Some(':') {
                    while self.peek() != Some(':') && self.peek() != Some('\n') { self.advance(); }
                }
                if self.peek() == Some(':') { self.advance(); }
                else_branch = self.parse_body_until(&[])?;
            }
            return Ok(KyxNode::If { condition, then_branch, else_branch });
        }
        if self.starts_with("for(") {
            self.pos += 3;
            self.advance(); // skip '('
            let item = self.read_while(|c| c != ' ' && c != '\t' && c != ')')?.trim().to_string();
            self.skip_whitespace();
            self.expect_string("in")?;
            self.skip_whitespace();
            let list = self.read_while(|c| c != ')')?.trim().to_string();
            self.expect_char(')')?;
            self.skip_whitespace();
            if self.peek() == Some(':') { self.advance(); }
            let body = self.parse_body_until(&[])?;
            return Ok(KyxNode::For { item, list, body });
        }
        if self.starts_with("match(") {
            self.pos += 5;
            let expr = self.read_while(|c| c != ')')?.trim().to_string();
            self.expect_char(')')?;
            self.expect_char(':')?;
            let mut cases = Vec::new();
            loop {
                self.skip_whitespace();
                if self.peek().is_none() || self.peek() == Some('<') { break; }
                let pattern = self.read_while(|c| c != ':')?.trim().to_string();
                if pattern.is_empty() { break; }
                self.expect_char(':')?;
                let case_body = self.parse_body_until(&["@case"])?;
                cases.push(MatchCase { pattern, body: case_body });
            }
            return Ok(KyxNode::Match { expr, cases });
        }
        // Simple @expr (inline expression)
        let expr = self.read_while(|c| c != '\n')?.trim().to_string();
        Ok(KyxNode::Expr(expr))
    }

    fn parse_element(&mut self) -> Result<KyxNode, String> {
        self.expect_char('<')?;
        let tag = self.read_while(|c| c != ' ' && c != '\t' && c != '>' && c != '/' && c != '\n')?;
        let mut attrs = Vec::new();

        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('>') => {
                    self.advance();
                    let children = self.parse_children(&tag)?;
                    return Ok(KyxNode::Element { tag, attrs, children });
                }
                Some('/') => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        return Ok(KyxNode::SelfClosing { tag, attrs });
                    }
                    return Err(format!("Expected '>' after '/' in tag '{}'", tag));
                }
                None => return Err(format!("Unexpected EOF in tag '{}'", tag)),
                _ => {
                    let attr = self.parse_attr()?;
                    attrs.push(attr);
                }
            }
        }
    }

    fn parse_attr(&mut self) -> Result<KyxAttr, String> {
        let name = self.read_while(|c| c != '=' && c != ' ' && c != '\t' && c != '>' && c != '/' && c != '\n')?;
        self.skip_whitespace();
        match self.peek() {
            Some('=') => {
                self.advance();
                self.skip_whitespace();
                match self.peek() {
                    Some('"') => {
                        self.advance();
                        let val = self.read_until('"')?;
                        Ok(KyxAttr { name, value: AttrValue::String(val) })
                    }
                    Some('@') => {
                        self.advance();
                        // Read expression, handling string literals with spaces
                        let expr = self.read_expression()?;
                        Ok(KyxAttr { name, value: AttrValue::Expr(expr) })
                    }
                    _ => {
                        let val = self.read_expression()?;
                        Ok(KyxAttr { name, value: AttrValue::Expr(val) })
                    }
                }
            }
            _ => Ok(KyxAttr { name, value: AttrValue::Flag }),
        }
    }

    fn parse_children(&mut self, closing_tag: &str) -> Result<Vec<KyxNode>, String> {
        let mut children = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => return Err(format!("Unexpected EOF, expected </{}>", closing_tag)),
                Some('<') => {
                    if self.starts_with("</") {
                        // Closing tag
                        let start = self.pos;
                        self.pos += 2;
                        let tag = self.read_while(|c| c != '>')?;
                        self.expect_char('>')?;
                        if tag == closing_tag {
                            return Ok(children);
                        }
                        // Not the right closing tag — treat as child element
                        self.pos = start;
                        let child = self.parse_element()?;
                        children.push(child);
                    } else if self.starts_with("<!--") {
                        // HTML comment — skip
                        self.pos += 4;
                        while !self.starts_with("-->") && self.peek().is_some() { self.advance(); }
                        if self.starts_with("-->") { self.pos += 3; }
                    } else {
                        // Child element
                        let child = self.parse_element()?;
                        children.push(child);
                    }
                }
                Some('@') => {
                    // @directive or @expr
                    self.advance();
                    let node = self.parse_at_directive()?;
                    children.push(node);
                }
                _ => {
                    // Text content
                    let text = self.read_while(|c| c != '<' && c != '@')?;
                    if !text.trim().is_empty() {
                        children.push(KyxNode::Text(text));
                    } else { self.advance(); }
                }
            }
        }
    }

    fn parse_body_until(&mut self, delimiters: &[&str]) -> Result<Vec<KyxNode>, String> {
        let mut nodes = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => break,
                Some('<') => {
                    // Check if this is a closing tag (</) — always break for parent to handle
                    if self.starts_with("</") {
                        break;
                    }
                    // Check if this is a closing delimiter tag
                    let mut is_delim = false;
                    for d in delimiters {
                        if self.starts_with(d) { is_delim = true; break; }
                    }
                    if is_delim { break; }
                    let child = self.parse_element()?;
                    nodes.push(child);
                }
                Some('@') => {
                    let mut is_delim = false;
                    for d in delimiters {
                        if self.starts_with(d) { is_delim = true; break; }
                    }
                    if is_delim { break; }
                    self.advance();
                    let node = self.parse_at_directive()?;
                    nodes.push(node);
                }
                _ => break,
            }
        }
        Ok(nodes)
    }

    fn expect_string(&mut self, s: &str) -> Result<(), String> {
        let cs: Vec<char> = s.chars().collect();
        for &c in &cs {
            match self.advance() {
                Some(ch) if ch == c => {}
                Some(ch) => return Err(format!("Expected '{}', found '{}' in string '{}'", c, ch, s)),
                None => return Err(format!("Unexpected EOF, expected '{}'", s)),
            }
        }
        Ok(())
    }

    fn read_while<F>(&mut self, f: F) -> Result<String, String>
    where F: Fn(char) -> bool {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if f(c) { s.push(c); self.advance(); }
            else { break; }
        }
        Ok(s)
    }

    fn read_until(&mut self, delimiter: char) -> Result<String, String> {
        let mut s = String::new();
        while let Some(c) = self.advance() {
            if c == delimiter { break; }
            s.push(c);
        }
        Ok(s)
    }

    /// Read a Kyle expression with string literal and paren support.
    fn read_expression(&mut self) -> Result<String, String> {
        let mut s = String::new();
        let mut depth: i32 = 0;
        let mut in_string = false;
        let mut after_operator = false;
        loop {
            match self.peek() {
                None => break,
                Some(c) => {
                    if c == '"' {
                        in_string = !in_string;
                        s.push(c);
                        self.advance();
                        after_operator = false;
                    } else if c == '(' && !in_string {
                        depth += 1;
                        s.push(c);
                        self.advance();
                        after_operator = false;
                    } else if c == ')' && !in_string {
                        depth -= 1;
                        s.push(c);
                        self.advance();
                        after_operator = false;
                    } else if (c == ' ' || c == '\t') && !in_string && depth == 0 {
                        if after_operator {
                            // After an operator, spaces are part of the expression
                            s.push(c);
                            self.advance();
                            after_operator = false;
                        } else {
                            // Peek ahead: if followed by an operator, continue reading
                            let mut skip_pos = self.pos + 1;
                            while let Some(&nc) = self.chars.get(skip_pos) {
                                if nc == ' ' || nc == '\t' { skip_pos += 1; }
                                else { break; }
                            }
                            let next = self.chars.get(skip_pos).copied().unwrap_or(' ');
                            let is_operator = matches!(next, '+' | '-' | '*' | '/' | '!' | '<' | '>' | '&' | '|' | '?' | ':');
                            if is_operator {
                                s.push(c);
                                self.advance();
                            } else {
                                break; // end of expression
                            }
                        }
                    } else if c == '\n' && !in_string && depth == 0 {
                        break;
                    } else if c == '>' && !in_string {
                        break;
                    } else if c == '/' && !in_string
                        && self.chars.get(self.pos + 1) == Some(&'>') {
                        break;
                    } else {
                        // Check if this char is an operator (for tracking)
                        let is_op = matches!(c, '+' | '-' | '*' | '/' | '=' | '!' | '<' | '>' | '&' | '|' | '?' | ':');
                        after_operator = is_op;
                        s.push(c);
                        self.advance();
                    }
                }
            }
        }
        Ok(s.trim().to_string())
    }

    fn find_matching_paren(&mut self) -> Result<usize, String> {
        let mut depth = 1;
        loop {
            match self.advance() {
                Some('(') => depth += 1,
                Some(')') => {
                    depth -= 1;
                    if depth == 0 { return Ok(self.pos); } // pos is AFTER ) now
                }
                Some(_) => {}
                None => return Err("Unexpected EOF in @() block".to_string()),
            }
        }
    }

    fn extract_and_advance(&mut self, after_paren: usize, content_start: usize) -> String {
        self.pos = after_paren;
        self.chars[content_start..after_paren - 1].iter().collect::<String>().trim().to_string()
    }
}

/// Convert KyxFile to platform-agnostic UI-IR UiProgram
/// Extracts <route> elements from the body into UiProgram.routes.
pub fn to_ui_program(file: KyxFile) -> crate::ir::UiProgram {
    use crate::ir::RouteConfig;

    // Convert RouteDecl from KyxFile
    let mut routes: Vec<RouteConfig> = file.routes.into_iter().map(|r| {
        RouteConfig {
            path: r.path,
            component: r.component,
            layout: r.layout,
            title: r.title,
            guard: r.guard,
            lazy: r.lazy,
            target_config: Vec::new(),
        }
    }).collect();

    // Convert body, extracting <route> elements
    let mut body = Vec::new();
    for node in file.body {
        let ir_node = convert_node(node, &mut routes);
        if let Some(n) = ir_node {
            body.push(n);
        }
    }

    crate::ir::UiProgram {
        routes,
        code_blocks: file.code_blocks,
        styles: file.styles,
        animations: file.animations,
        body,
        component_renderers: Vec::new(),
    }
}

/// Convert a KyxNode to UiNode, optionally extracting RouteConfig from <route> elements.
/// Returns None if the node is a <route> that was extracted into routes.
fn convert_node(node: KyxNode, routes: &mut Vec<crate::ir::RouteConfig>) -> Option<crate::ir::UiNode> {
    use crate::ir::{UiNode, ComponentTag};

    match node {
        KyxNode::Element { tag, attrs, children } => {
            // Check if this is a <route> element — extract as RouteConfig
            if tag.to_lowercase() == "route" {
                let mut path = String::new();
                let mut component = String::new();
                let mut layout = None;
                let mut title = None;
                let mut guard = None;
                let mut lazy = false;
                for a in &attrs {
                    match a.name.as_str() {
                        "path" => path = attr_value_str(&a.value),
                        "component" => component = strip_at(attr_value_str(&a.value)),
                        "layout" => layout = Some(strip_at(attr_value_str(&a.value))),
                        "title" => title = Some(attr_value_str(&a.value)),
                        "guard" => guard = Some(strip_at(attr_value_str(&a.value))),
                        "lazy" => lazy = true,
                        _ => {}
                    }
                }
                routes.push(crate::ir::RouteConfig {
                    path, component, layout, title, guard, lazy,
                    target_config: Vec::new(),
                });
                return None; // Don't add to body
            }

            let converted_children: Vec<crate::ir::UiNode> = children.into_iter()
                .filter_map(|c| convert_node(c, routes))
                .collect();

            Some(UiNode::Element {
                tag: ComponentTag::from_str(&tag),
                attrs: attrs.into_iter().map(convert_attr).collect(),
                children: converted_children,
            })
        }
        KyxNode::SelfClosing { tag, attrs } => {
            if tag.to_lowercase() == "route" {
                let mut path = String::new();
                let mut component = String::new();
                let mut layout = None;
                let mut title = None;
                let mut guard = None;
                let mut lazy = false;
                for a in &attrs {
                    match a.name.as_str() {
                        "path" => path = attr_value_str(&a.value),
                        "component" => component = strip_at(attr_value_str(&a.value)),
                        "layout" => layout = Some(strip_at(attr_value_str(&a.value))),
                        "title" => title = Some(attr_value_str(&a.value)),
                        "guard" => guard = Some(strip_at(attr_value_str(&a.value))),
                        "lazy" => lazy = true,
                        _ => {}
                    }
                }
                routes.push(crate::ir::RouteConfig {
                    path, component, layout, title, guard, lazy,
                    target_config: Vec::new(),
                });
                return None;
            }
            Some(crate::ir::UiNode::SelfClosing {
                tag: ComponentTag::from_str(&tag),
                attrs: attrs.into_iter().map(convert_attr).collect(),
            })
        }
        KyxNode::Slot { name, fallback } => Some(UiNode::Slot {
            name,
            fallback: fallback.into_iter().filter_map(|c| convert_node(c, routes)).collect(),
        }),
        KyxNode::If { condition, then_branch, else_branch } => Some(UiNode::If {
            condition,
            then_branch: then_branch.into_iter().filter_map(|c| convert_node(c, routes)).collect(),
            else_branch: else_branch.into_iter().filter_map(|c| convert_node(c, routes)).collect(),
        }),
        KyxNode::For { item, list, body } => Some(UiNode::For {
            item, list,
            body: body.into_iter().filter_map(|c| convert_node(c, routes)).collect(),
        }),
        KyxNode::Match { expr, cases } => Some(UiNode::Match {
            expr,
            cases: cases.into_iter().map(|c| crate::ir::MatchCase {
                pattern: c.pattern,
                body: c.body.into_iter().filter_map(|n| convert_node(n, routes)).collect(),
            }).collect(),
        }),
        KyxNode::Expr(e) => Some(UiNode::Expr(e)),
        KyxNode::CodeBlock(b) => Some(UiNode::CodeBlock(b)),
        KyxNode::Text(t) => Some(UiNode::Text(t)),
    }
}

fn attr_value_str(val: &crate::ast::AttrValue) -> String {
    match val {
        crate::ast::AttrValue::String(s) => s.clone(),
        crate::ast::AttrValue::Expr(e) => e.clone(),
        crate::ast::AttrValue::Flag => String::new(),
    }
}

fn strip_at(s: String) -> String {
    if s.starts_with('@') { s[1..].to_string() } else { s }
}

fn convert_attr(attr: crate::ast::KyxAttr) -> crate::ir::UiAttr {
    crate::ir::UiAttr {
        name: attr.name,
        value: match attr.value {
            crate::ast::AttrValue::String(s) => crate::ir::AttrValue::String(s),
            crate::ast::AttrValue::Expr(e) => crate::ir::AttrValue::Expr(e),
            crate::ast::AttrValue::Flag => crate::ir::AttrValue::Flag,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_file() {
        let result = parse("");
        assert!(result.is_ok());
        let file = result.unwrap();
        assert!(file.routes.is_empty());
        assert!(file.body.is_empty());
    }

    #[test]
    fn test_route_element() {
        let result = parse(r#"<route path="/" component=@home_view layout=@main_layout title="Home" />"#);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let file = result.unwrap();
        // <route> is parsed as a regular element (extracted in to_ui_program)
        assert_eq!(file.body.len(), 1);
        match &file.body[0] {
            KyxNode::SelfClosing { tag, .. } => assert_eq!(tag, "route"),
            _ => panic!("Expected self-closing route element"),
        }
    }

    #[test]
    fn test_simple_element() {
        let result = parse("<view />");
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.body.len(), 1);
    }

    #[test]
    fn test_element_with_attrs() {
        let result = parse(r#"<text value="Hello" />"#);
        assert!(result.is_ok());
        let file = result.unwrap();
        match &file.body[0] {
            KyxNode::SelfClosing { tag, attrs } => {
                assert_eq!(tag, "text");
                assert_eq!(attrs.len(), 1);
                assert_eq!(attrs[0].name, "value");
                match &attrs[0].value {
                    AttrValue::String(s) => assert_eq!(s, "Hello"),
                    _ => panic!("Expected string attr"),
                }
            }
            _ => panic!("Expected self-closing element"),
        }
    }

    #[test]
    fn test_element_with_children() {
        let result = parse("<view>\n  <text />\n</view>");
        assert!(result.is_ok());
        let file = result.unwrap();

        match &file.body[0] {
            KyxNode::Element { tag, children, .. } => {
                assert_eq!(tag, "view");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected element with children"),
        }
    }

    #[test]
    fn test_code_block() {
        let source = r#"@(count: i32
  fn inc(): count = count + 1)"#;
        let result = parse(source);
        if let Err(ref e) = result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.code_blocks.len(), 1);
    }

    #[test]
    fn test_full_login() {
        let source = r#"@(
 email: str
 password: str
 fn login():
  print("login")
)

<view>
 <vstack alignment=alignment.center>
  <text_field bind=@email />
  <button tpl=Primary text="Ingresar" click=@login />
 </vstack>
</view>"#;
        let result = parse(source);
        if let Err(ref e) = result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let file = result.unwrap();
        assert!(file.routes.is_empty());
        assert_eq!(file.code_blocks.len(), 1);
        assert_eq!(file.body.len(), 1);
    }
}
