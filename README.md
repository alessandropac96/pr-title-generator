# PR Title Generator

A machine learning-based PR title generator that creates meaningful and specific pull request titles based on commit messages and branch context. Available as a global CLI tool that can be called from any git repository.

## Features

- **Global CLI tool**: Install once, use from any git repository
- **LLM-based generation**: Uses TinyLlama model for intelligent title generation
- **Context filtering**: Automatically removes noise and redundant information
- **Smart analysis**: Analyzes both branch context and commit messages
- **Customizable**: Configurable temperature and generation parameters
- **Git validation**: Automatically validates that you're in a git repository

## Installation

### Option 1: Quick Install (Recommended)
```bash
# Clone the repository
git clone https://github.com/alessandropac96/pr-title-generator.git
cd pr-title-generator

# Run the installation script
./install.sh
```

### Option 2: Manual Installation
```bash
# Clone the repository
git clone https://github.com/alessandropac96/pr-title-generator.git
cd pr-title-generator

# Install dependencies
pip3 install -r requirements.txt

# Install as a Python package
pip3 install -e .
```

### Option 3: Virtual Environment
```bash
# Create a virtual environment
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Use directly
python3 main.py
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
- `--model`: LLM model to use (default: tiny-llama)
- `--temperature`: Generation temperature (0.1-1.0, default: 0.7)
- `--max-length`: Maximum title length (default: 50)
- `--verbose`: Enable verbose output

## Supported Models

- `tiny-llama`: Fast and efficient (default)
- `phi-2`: Microsoft's Phi-2 model
- `gemma-2b`: Google's Gemma 2B model
- `llama-2-7b`: Meta's Llama 2 7B model

## How It Works

1. **Context Extraction**: Analyzes branch name and commit messages
2. **Noise Filtering**: Removes commit hashes, branch prefixes, and redundant information
3. **LLM Generation**: Uses the cleaned context to generate a meaningful title
4. **Post-processing**: Ensures the title is concise and properly formatted

## Example Output

**Input:**
- Branch: `cursor/CRU-310-fix-bottle-stuck-issue-with-remediation-f8b5`
- Commits: Block remediation system implementation, test improvements

**Output:**
- `Fix bottle stuck with remediation and improve test coverage`

## Requirements

- Python 3.7+
- Git repository
- ~2GB disk space for model download (first run)
- GPU recommended for faster generation



