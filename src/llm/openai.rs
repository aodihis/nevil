

pub enum Model {
    Gpt4Turbo,
    Gpt4,
    Gpt35Turbo,
    Gpt35Turbo16k,
    TextDavinci003,
    TextDavinci002,
    CodeDavinci002,
}

impl Model {
    fn name(&self) -> &'static str {
        match self {
            Model::Gpt4Turbo => "gpt-4-turbo",
            Model::Gpt4 => "gpt-4",
            Model::Gpt35Turbo => "gpt-3.5-turbo",
            Model::Gpt35Turbo16k => "gpt-3.5-turbo-16k",
            Model::TextDavinci003 => "text-davinci-003",
            Model::TextDavinci002 => "text-davinci-002",
            Model::CodeDavinci002 => "code-davinci-002",
        }
    }
}