#[derive(Debug)]
pub enum AccessError {
    NotFound,
    AlreadyExists,
    DatabaseError(String),
}
