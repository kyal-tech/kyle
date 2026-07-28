use std::fmt;

/// An import declaration: `from views.home import home`
#[derive(Clone, Debug)]
pub struct ImportDecl {
    /// Module path: "views.home" → resolves to src/views/home.kyx
    pub module: String,
    /// Import name: "home" → available as `home` in scope
    pub name: String,
}

/// A route extracted from `<route path="..." component=@comp layout=@layout>`
#[derive(Clone, Debug)]
pub struct RouteDecl {
    pub path: String,
    pub component: String,
    pub layout: Option<String>,
    pub title: Option<String>,
    pub guard: Option<String>,
    pub lazy: bool,
}

/// A parsed .kyx file
#[derive(Clone, Debug)]
pub struct KyxFile {
    /// Import declarations: from X import Y
    pub imports: Vec<ImportDecl>,
    /// Route declarations extracted from <route> elements
    pub routes: Vec<RouteDecl>,
    /// Kyle code blocks: @(...)
    pub code_blocks: Vec<String>,
    /// Style/template/theme declarations
    pub styles: Vec<StyleDecl>,
    /// Animation declarations
    pub animations: Vec<AnimDecl>,
    /// Root XML elements
    pub body: Vec<KyxNode>,
}

/// A style/template/theme declaration
#[derive(Clone, Debug)]
pub enum StyleDecl {
    /// style<component> Name: prop = value, ...
    Style {
        component: String,
        name: String,
        props: Vec<StyleProp>,
        media: Vec<MediaQuery>,
    },
    /// layout<component> Name: prop = value, ...
    Layout {
        component: String,
        name: String,
        props: Vec<StyleProp>,
        media: Vec<MediaQuery>,
    },
    /// tpl<component> Name: style = X, animation = Y, ...
    Template {
        component: String,
        name: String,
        props: Vec<StyleProp>,
        media: Vec<MediaQuery>,
    },
    /// theme Name: prop = value, ...
    Theme {
        name: String,
        props: Vec<StyleProp>,
    },
}

/// A single style property: color = Color("#0066FF")
#[derive(Clone, Debug)]
pub struct StyleProp {
    pub name: String,
    pub value: String,
}

/// A @media query within a style declaration
#[derive(Clone, Debug)]
pub struct MediaQuery {
    pub condition: String,  // e.g. "min_width: 640"
    pub props: Vec<StyleProp>,
}

/// An XML element or control-flow directive
#[derive(Clone, Debug)]
pub enum KyxNode {
    /// XML element: <tag attrs>children</tag>
    Element {
        tag: String,
        attrs: Vec<KyxAttr>,
        children: Vec<KyxNode>,
    },
    /// Self-closing element: <tag attrs />
    SelfClosing {
        tag: String,
        attrs: Vec<KyxAttr>,
    },
    /// Slot declaration: <slot name="header" />
    Slot {
        name: String,
        fallback: Vec<KyxNode>,
    },
    /// @if(condition): ...
    If {
        condition: String,
        then_branch: Vec<KyxNode>,
        else_branch: Vec<KyxNode>,
    },
    /// @for(item in list): ...
    For {
        item: String,
        list: String,
        body: Vec<KyxNode>,
    },
    /// @match(expr): cases
    Match {
        expr: String,
        cases: Vec<MatchCase>,
    },
    /// @expr — inline Kyle expression
    Expr(String),
    /// @(code) — Kyle code block
    CodeBlock(String),
    /// Raw text content
    Text(String),
}

/// A match case in @match
#[derive(Clone, Debug)]
pub struct MatchCase {
    pub pattern: String,
    pub body: Vec<KyxNode>,
}

/// An XML attribute
#[derive(Clone, Debug)]
pub struct KyxAttr {
    pub name: String,
    pub value: AttrValue,
}

/// Attribute value: literal string or Kyle expression
#[derive(Clone, Debug)]
pub enum AttrValue {
    /// Plain string: attr="value"
    String(String),
    /// Kyle expression: attr=@expr
    Expr(String),
    /// Boolean flag: attr (no value)
    Flag,
}

/// Animation declaration: animation<comp> Name: ...
#[derive(Clone, Debug)]
pub struct AnimDecl {
    pub component: String,
    pub name: String,
    pub props: Vec<StyleProp>,
}

impl fmt::Display for KyxFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for imp in &self.imports {
            writeln!(f, "from {} import {}", imp.module, imp.name)?;
        }
        for route in &self.routes {
            writeln!(f, "route {} -> {}", route.path, route.component)?;
        }
        for block in &self.code_blocks {
            writeln!(f, "@({})", block)?;
        }
        for anim in &self.animations {
            writeln!(f, "animation<{}> {}:", anim.component, anim.name)?;
            for p in &anim.props { writeln!(f, "    {} = {}", p.name, p.value)?; }
        }
        for style in &self.styles {
            write!(f, "{}", style)?;
        }
        for node in &self.body {
            write!(f, "{}", node)?;
        }
        Ok(())
    }
}

impl fmt::Display for KyxNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KyxNode::Element { tag, attrs, children } => {
                write!(f, "<{}", tag)?;
                for a in attrs { write!(f, " {}", a)?; }
                if children.is_empty() {
                    writeln!(f, " />")?;
                } else {
                    writeln!(f, ">")?;
                    for c in children { write!(f, "  {}", c)?; }
                    writeln!(f, "</{}>", tag)?;
                }
                Ok(())
            }
            KyxNode::SelfClosing { tag, attrs } => {
                write!(f, "<{}", tag)?;
                for a in attrs { write!(f, " {}", a)?; }
                writeln!(f, " />")
            }
            KyxNode::Slot { name, .. } => writeln!(f, "<slot name=\"{}\" />", name),
            KyxNode::If { condition, .. } => writeln!(f, "@if({}): ...", condition),
            KyxNode::For { item, list, .. } => writeln!(f, "@for({} in {}): ...", item, list),
            KyxNode::Match { expr, .. } => writeln!(f, "@match({}): ...", expr),
            KyxNode::Expr(e) => writeln!(f, "@{}", e),
            KyxNode::CodeBlock(b) => writeln!(f, "@({})", b),
            KyxNode::Text(t) => write!(f, "{}", t),
        }
    }
}

impl fmt::Display for KyxAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            AttrValue::String(s) => write!(f, "{}=\"{}\"", self.name, s),
            AttrValue::Expr(e) => write!(f, "{}={}", self.name, e),
            AttrValue::Flag => write!(f, "{}", self.name),
        }
    }
}

impl fmt::Display for StyleDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (keyword, component, name, props, media) = match self {
            StyleDecl::Style { component, name, props, media } => ("style", component, name, props, media),
            StyleDecl::Layout { component, name, props, media } => ("layout", component, name, props, media),
            StyleDecl::Template { component, name, props, media } => ("tpl", component, name, props, media),
            StyleDecl::Theme { name, props } => {
                writeln!(f, "theme {}:", name)?;
                for p in props { writeln!(f, "    {} = {}", p.name, p.value)?; }
                return Ok(());
            }
        };
        writeln!(f, "{}<{}> {}:", keyword, component, name)?;
        for p in props { writeln!(f, "    {} = {}", p.name, p.value)?; }
        for m in media {
            writeln!(f, "    @media({}):", m.condition)?;
            for p in &m.props { writeln!(f, "        {} = {}", p.name, p.value)?; }
        }
        Ok(())
    }
}

impl fmt::Display for StyleProp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.name, self.value)
    }
}
