#[derive(Debug)]
pub struct CliExit {
    pub code: i32,
    pub message: String,
}

impl CliExit {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn fail(code: i32, err: impl std::fmt::Display) -> CliExit {
    CliExit::new(code, err.to_string())
}
