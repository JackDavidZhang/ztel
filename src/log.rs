use tokio::fs::File;

struct Log {
}

pub static LOG: Log = Log{
};
impl Log {
    pub fn debug(&self, message: &str) {
        
    }
    pub fn info(&self, message: &str) {
    }
    pub fn warn(&self, message: &str) {}
    pub fn error(&self, message: &str) {}
}