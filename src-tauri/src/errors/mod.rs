#[derive(Debug)]
pub enum AppError {
    Runtime(String),
}

impl AppError {
    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime(message.into())
    }

    pub fn into_string(self) -> String {
        match self {
            Self::Runtime(message) => message,
        }
    }
}
