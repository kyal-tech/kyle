use std::fmt;

pub struct KlError {
    pub message: String,
    pub code: Option<i32>,
}

impl KlError {
    pub fn new(message: &str) -> Self {
        Self { message: message.to_string(), code: None }
    }

    pub fn with_code(message: &str, code: i32) -> Self {
        Self { message: message.to_string(), code: Some(code) }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn code(&self) -> Option<i32> {
        self.code
    }
}

impl fmt::Display for KlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.code {
            write!(f, "[{}] {}", code, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl fmt::Debug for KlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.code {
            write!(f, "KlError({}, code={})", self.message, code)
        } else {
            write!(f, "KlError({})", self.message)
        }
    }
}
