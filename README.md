# neVil 🖥️📊

## Overview
neVil is an experimental desktop application built with Rust, egui, and eframe that simplifies database querying through an intuitive interface. Designed to make data exploration easier, neVil allows users to connect to databases and retrieve information using natural language.

## Features
- 🤖 LLM-Powered Querying
- 🔒 Support for Claude and OpenAI API
- 📊 Database Data Retrieval

## Prerequisites
- Rust (latest stable version)
- API Key from Claude or OpenAI

## How to Get API Keys
- **Claude API Key**:
    - Visit [Anthropic's Platform](https://www.anthropic.com/product)
    - Sign up for an account
    - Generate an API key in your dashboard

- **OpenAI API Key**:
    - Visit [OpenAI Platform](https://platform.openai.com/)
    - Create an account
    - Navigate to API keys section
    - Generate a new API key

## How to Use the Application

### Initial Setup
1. Launch the application
2. Add your LLM API key (Claude or OpenAI)
3. Create a new database connection
4. Save the connection details

### Querying Data
1. Open your saved database connection
2. Use natural language
3. Execute and view results

## Local Development

### Prerequisites
- Rust programming language
- Cargo package manager

### Setup Steps
```bash
# Clone the repository
git clone https://github.com/your-username/neVil.git

# Navigate to project directory
cd neVil

# Build the project
cargo build

# Run the application
cargo run
```

## Limitations
- Currently supports only SELECT queries
- Does not support UPDATE, INSERT, or DELETE operations
- Executable generation coming in future releases

## Future Roadmap
- Support for more query types
- Executable file generation
- Enhanced database compatibility

## Contributing
Contributions are welcome! Please open an issue or submit a pull request.

## License
MIT
