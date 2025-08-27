# PR Title Generator

A high-performance Rust-based PR title generator that creates meaningful and specific pull request titles based on commit messages and branch context. Available as a global CLI tool that can be called from any git repository.

## Features

- **Fast Rust implementation**: Built with Rust for speed and reliability
- **Global CLI tool**: Install once, use from any git repository  
- **Intelligent pattern matching**: Uses advanced text processing for smart title generation
- **Context filtering**: Automatically removes noise and redundant information
- **Smart analysis**: Analyzes both branch context and commit messages
- **Customizable**: Configurable generation parameters and models
- **Git validation**: Automatically validates that you're in a git repository
- **Clean code**: Follows Rust best practices with comprehensive .cursorrules

## Installation

### Prerequisites
- Rust and Cargo (install from [rustup.rs](https://rustup.rs/))
- Git

### Option 1: Build from Source (Recommended)
```bash
# Clone the repository
git clone https://github.com/alessandropac96/pr-title-generator.git
cd pr-title-generator

# Build and install
cargo build --release
cargo install --path .
```

### Option 2: Direct Cargo Install (when published)
```bash
# Install directly from crates.io
cargo install pr-title-generator
```

## Usage

### Basic Usage
```bash
# From any git repository
generate-pr-title
```

### Advanced Usage
```bash
generate-pr-title --branch feature/my-feature --verbose
generate-pr-title --model phi-2 --temperature 0.6
generate-pr-title --max-commits 30 --base develop
```

### Options
- `--branch`: Branch to analyze (defaults to current branch)
- `--base`: Base branch to compare against (default: main)
- `--max-commits`: Maximum number of commits to analyze (default: 20)
- `--model`: Pattern model to use (default: tiny-llama)
- `--temperature`: Generation creativity (0.1-1.0, default: 0.7)
- `--max-length`: Maximum title length (default: 50)
- `--verbose`: Enable verbose output

## Supported Models

Currently uses intelligent pattern-based generation:
- `tiny-llama`: Fast pattern matching (default)
- `phi-2`: Enhanced context analysis
- `gemma-2b`: Advanced pattern recognition
- `llama-2-7b`: Maximum context understanding

*Note: Full ML model integration coming in future releases*

## How It Works

1. **Context Extraction**: Analyzes branch name and commit messages using Rust's powerful text processing
2. **Noise Filtering**: Removes commit hashes, branch prefixes, and redundant information with regex patterns
3. **Intelligent Pattern Matching**: Uses contextual patterns to generate meaningful titles
4. **Post-processing**: Ensures the title is concise and properly formatted

## Example Output

**Input:**
- Branch: `cursor/CRU-310-fix-bottle-stuck-issue-with-remediation-f8b5`
- Commits: Block remediation system implementation, test improvements

**Output:**
- `CRU-310: Fix bottle stuck with remediation and improve test coverage`

## Requirements

- Rust 1.70+ (for compilation)
- Git repository
- Minimal system resources (fast startup, low memory usage)

## Development

This project follows clean code principles with comprehensive Rust best practices:

### Code Quality Features
- **Comprehensive .cursorrules**: Enforces clean, idiomatic Rust code
- **Modular architecture**: Separate modules for git, context processing, ML, and CLI
- **Error handling**: Proper Result types with meaningful error messages
- **Type safety**: Leverages Rust's type system for bug prevention
- **Memory safety**: Zero-cost abstractions with ownership model
- **Testing**: Unit and integration tests for reliability

### Architecture
```
src/
├── lib.rs          # Library exports and configuration
├── main.rs         # CLI entry point
├── cli.rs          # Command line argument parsing
├── git.rs          # Git repository operations
├── context.rs      # Text processing and context extraction
├── ml.rs           # Pattern-based title generation
└── error.rs        # Error types and handling
```

### Building from Source
```bash
# Clone and build
git clone <repository-url>
cd pr-title-generator
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt --check
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Follow the .cursorrules for code quality
4. Add tests for new functionality  
5. Ensure `cargo test` and `cargo clippy` pass
6. Submit a pull request

## Future Roadmap

- [ ] Full ML model integration using candle-rs
- [ ] Web interface for PR title generation
- [ ] GitHub/GitLab integration
- [ ] Custom pattern configuration
- [ ] Performance optimizations
