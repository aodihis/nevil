

pub enum Model {
    Claude37,
}

impl Model {
    pub fn name(&self) ->  &'static str {
        match self {
            Model::Claude37 => "claude-3-7-sonnet-20250219",
        }
    }
    pub fn variants() -> Vec<Model> {
        use Model::*;
        vec![
            Claude37
        ]
    }

    pub fn variants_name() -> Vec<&'static str> {
        Self::variants().iter().map(|model| model.name()).collect()
    }
}