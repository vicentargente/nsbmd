#[derive(Debug)]
pub struct AppError {
    message: String
}

impl AppError {
    pub fn new(message: &str) -> AppError {
        AppError {
            message: message.to_string()
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
