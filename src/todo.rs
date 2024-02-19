#[derive(Debug, Clone)]
pub struct Todo {
    pub id: usize,
    pub title: String,
    pub done: bool,
}

impl Todo {
    pub fn new(title: String) -> Self {
        Self {
            title,
            done: false,
            id: 0,
        }
    }
}
