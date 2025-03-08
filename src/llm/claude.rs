

pub enum Model {
    Claude37,
}

impl Model {
    pub fn name(&self) -> &str {
        match self {
            Model::Claude37 => "claude-3-7-sonnet-20250219",
        }
    }
}