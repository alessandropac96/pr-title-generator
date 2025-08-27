# PR Title Generator

A machine learning-based PR title generator that creates meaningful and specific pull request titles based on commit messages and branch context.

## Features

- **LLM-based generation**: Uses TinyLlama model for intelligent title generation
- **Context filtering**: Automatically removes noise and redundant information
- **Smart analysis**: Analyzes both branch context and commit messages
- **Customizable**: Configurable temperature and generation parameters

## Installation

1. Create a virtual environment:
```bash
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
```

2. Install dependencies:
```bash
pip install -r requirements.txt
```

## Usage

### Basic Usage
```bash
python3 generate-pr-title-ml.py --branch origin/feature/my-feature
```

### Advanced Usage
```bash
python3 generate-pr-title-ml.py \
  --branch origin/feature/my-feature \
  --model tiny-llama \
  --temperature 0.6 \
  --verbose
```

### Options
- `--branch`: Branch to analyze (defaults to current branch)
- `--base`: Base branch to compare against (default: main)
- `--max-commits`: Maximum number of commits to analyze (default: 20)
- `--model`: LLM model to use (default: tiny-llama)
- `--temperature`: Generation temperature (0.1-1.0, default: 0.7)
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



