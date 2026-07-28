use std::fmt;

pub use crate::ast::{StyleDecl, StyleProp, MediaQuery, AnimDecl};

/// Platform target enum
#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Web,
    MacOS,
    Windows,
    Linux,
    Ios,
    Android,
}

impl Target {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "web" => Self::Web,
            "macos" => Self::MacOS,
            "windows" => Self::Windows,
            "linux" => Self::Linux,
            "ios" => Self::Ios,
            "android" => Self::Android,
            _ => Self::Web,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Web => "web",
            Self::MacOS => "macos",
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Ios => "ios",
            Self::Android => "android",
        }
    }
}

/// A route definition parsed from `<route path="..." component=@comp ...>`
#[derive(Clone, Debug)]
pub struct RouteConfig {
    pub path: String,
    pub component: String,
    pub layout: Option<String>,
    pub title: Option<String>,
    pub guard: Option<String>,
    pub lazy: bool,
    pub target_config: Vec<TargetConfig>,
}

/// Platform-specific config block inside a route or view
#[derive(Clone, Debug)]
pub struct TargetConfig {
    pub target: Target,
    pub props: Vec<StyleProp>,
}

/// Built-in type: file_data — represents a file selected via file picker
/// Mapped in JS to: { name: str, size: i64, mime: str, content: bytes, last_modified: i64 }
/// Mapped in Swift to: a struct with same fields
pub const FILE_DATA_TYPE_FIELDS: &[(&str, &str)] = &[
    ("name", "str"),
    ("size", "i64"),
    ("mime", "str"),
    ("content", "bytes"),
    ("last_modified", "i64"),
];

/// A complete .kyx program (UI-IR)
#[derive(Clone, Debug)]
pub struct UiProgram {
    pub routes: Vec<RouteConfig>,
    pub code_blocks: Vec<String>,
    pub styles: Vec<StyleDecl>,
    pub animations: Vec<AnimDecl>,
    pub body: Vec<UiNode>,
    /// Named component renderers from resolved dependencies
    pub component_renderers: Vec<ComponentRenderer>,
}

/// A named component that can be rendered separately (from a resolved .kyx file)
#[derive(Clone, Debug)]
pub struct ComponentRenderer {
    pub name: String,
    pub code_blocks: Vec<String>,
    pub styles: Vec<StyleDecl>,
    pub animations: Vec<AnimDecl>,
    pub body: Vec<UiNode>,
}

/// Platform-agnostic UI node
#[derive(Clone, Debug)]
pub enum UiNode {
    Element {
        tag: ComponentTag,
        attrs: Vec<UiAttr>,
        children: Vec<UiNode>,
    },
    SelfClosing {
        tag: ComponentTag,
        attrs: Vec<UiAttr>,
    },
    Slot {
        name: String,
        fallback: Vec<UiNode>,
    },
    If {
        condition: String,
        then_branch: Vec<UiNode>,
        else_branch: Vec<UiNode>,
    },
    For {
        item: String,
        list: String,
        body: Vec<UiNode>,
    },
    Match {
        expr: String,
        cases: Vec<MatchCase>,
    },
    Expr(String),
    CodeBlock(String),
    Text(String),
}

#[derive(Clone, Debug)]
pub struct MatchCase {
    pub pattern: String,
    pub body: Vec<UiNode>,
}

#[derive(Clone, Debug)]
pub struct UiAttr {
    pub name: String,
    pub value: AttrValue,
}

#[derive(Clone, Debug)]
pub enum AttrValue {
    String(String),
    Expr(String),
    Flag,
}

/// Platform-agnostic component tag — each is a type (final class)
#[derive(Clone, Debug, PartialEq)]
pub enum ComponentTag {
    // Containers
    App, Router, Route, Layout, View, Group, Scroll,

    // Layout
    VStack, HStack, ZStack, Spacer, Divider,

    // Elements
    Text, Button, Image, Link, Slot,
    Input, TextField, PasswordField, TextArea,
    Checkbox, Radio, Switch, Slider,
    Progress, Spinner,

    // Overlays
    Modal, Sheet, Alert, Tooltip,

    // Navigation
    Navbar, Sidebar, TabBar, Footer,

    // Form
    Form, Select, List, Card, Surface,

    // File
    FilePicker,

    // Other
    Portal, ErrorBoundary, Canvas, Video, Audio, Icon,

    // Custom (user-defined component)
    Custom(String),
}

impl ComponentTag {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "app" => Self::App,
            "router" => Self::Router,
            "route" => Self::Route,
            "layout" => Self::Layout,
            "view" => Self::View,
            "group" => Self::Group,
            "scroll" => Self::Scroll,

            "vstack" | "v_stack" => Self::VStack,
            "hstack" | "h_stack" => Self::HStack,
            "zstack" | "z_stack" => Self::ZStack,
            "spacer" => Self::Spacer,
            "divider" => Self::Divider,

            "text" => Self::Text,
            "button" => Self::Button,
            "image" | "img" => Self::Image,
            "link" => Self::Link,
            "slot" => Self::Slot,

            "input" => Self::Input,
            "text_field" | "textfield" => Self::TextField,
            "password_field" | "passwordfield" => Self::PasswordField,
            "text_area" | "textarea" => Self::TextArea,

            "checkbox" => Self::Checkbox,
            "radio" => Self::Radio,
            "switch" => Self::Switch,
            "slider" => Self::Slider,

            "progress" => Self::Progress,
            "spinner" => Self::Spinner,

            "modal" => Self::Modal,
            "sheet" => Self::Sheet,
            "alert" => Self::Alert,
            "tooltip" => Self::Tooltip,

            "navbar" | "nav_bar" => Self::Navbar,
            "sidebar" | "side_bar" => Self::Sidebar,
            "tab_bar" | "tabbar" => Self::TabBar,
            "footer" => Self::Footer,

            "form" => Self::Form,
            "select" => Self::Select,
            "list" => Self::List,
            "card" => Self::Card,
            "surface" => Self::Surface,

            "file_picker" | "filepicker" => Self::FilePicker,
            "portal" => Self::Portal,
            "error_boundary" | "errorboundary" => Self::ErrorBoundary,
            "canvas" => Self::Canvas,
            "video" => Self::Video,
            "audio" => Self::Audio,
            "icon" => Self::Icon,

            other => Self::Custom(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::App => "app",
            Self::Router => "router",
            Self::Route => "route",
            Self::Layout => "layout",
            Self::View => "view",
            Self::Group => "group",
            Self::Scroll => "scroll",

            Self::VStack => "vstack",
            Self::HStack => "hstack",
            Self::ZStack => "zstack",
            Self::Spacer => "spacer",
            Self::Divider => "divider",

            Self::Text => "text",
            Self::Button => "button",
            Self::Image => "image",
            Self::Link => "link",
            Self::Slot => "slot",

            Self::Input => "input",
            Self::TextField => "text_field",
            Self::PasswordField => "password_field",
            Self::TextArea => "text_area",

            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Switch => "switch",
            Self::Slider => "slider",

            Self::Progress => "progress",
            Self::Spinner => "spinner",

            Self::Modal => "modal",
            Self::Sheet => "sheet",
            Self::Alert => "alert",
            Self::Tooltip => "tooltip",

            Self::Navbar => "navbar",
            Self::Sidebar => "sidebar",
            Self::TabBar => "tab_bar",
            Self::Footer => "footer",

            Self::Form => "form",
            Self::Select => "select",
            Self::List => "list",
            Self::Card => "card",
            Self::Surface => "surface",

            Self::FilePicker => "file_picker",
            Self::Portal => "portal",
            Self::ErrorBoundary => "error_boundary",
            Self::Canvas => "canvas",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Icon => "icon",

            Self::Custom(s) => s,
        }
    }
}

// ── Display impls ──

impl fmt::Display for UiProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for route in &self.routes {
            writeln!(f, "route {} -> {} (layout: {:?})", route.path, route.component, route.layout)?;
        }
        for block in &self.code_blocks {
            writeln!(f, "@({})", block)?;
        }
        for node in &self.body {
            write!(f, "{}", node)?;
        }
        Ok(())
    }
}

impl fmt::Display for UiNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiNode::Element { tag, attrs, children } => {
                write!(f, "<{}", tag.as_str())?;
                for a in attrs { write!(f, " {}", a)?; }
                if children.is_empty() {
                    writeln!(f, " />")?;
                } else {
                    writeln!(f, ">")?;
                    for c in children { write!(f, "  {}", c)?; }
                    writeln!(f, "</{}>", tag.as_str())?;
                }
                Ok(())
            }
            UiNode::SelfClosing { tag, attrs } => {
                write!(f, "<{}", tag.as_str())?;
                for a in attrs { write!(f, " {}", a)?; }
                writeln!(f, " />")
            }
            UiNode::Slot { name, .. } => writeln!(f, "<slot name=\"{}\" />", name),
            UiNode::If { condition, .. } => writeln!(f, "@if({}): ...", condition),
            UiNode::For { item, list, .. } => writeln!(f, "@for({} in {}): ...", item, list),
            UiNode::Match { expr, .. } => writeln!(f, "@match({}): ...", expr),
            UiNode::Expr(e) => writeln!(f, "@{}", e),
            UiNode::CodeBlock(b) => writeln!(f, "@({})", b),
            UiNode::Text(t) => write!(f, "{}", t),
        }
    }
}

impl fmt::Display for UiAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            AttrValue::String(s) => write!(f, "{}=\"{}\"", self.name, s),
            AttrValue::Expr(e) => write!(f, "{}={}", self.name, e),
            AttrValue::Flag => write!(f, "{}", self.name),
        }
    }
}
