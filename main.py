#!/usr/bin/env python3
"""
PR Title Generator CLI
A command-line tool to generate meaningful PR titles using ML models.
"""

import os
import sys
import subprocess
import argparse
from pathlib import Path


def is_git_repository(directory: str) -> bool:
    """Check if the given directory is a git repository."""
    git_dir = os.path.join(directory, '.git')
    return os.path.exists(git_dir) and os.path.isdir(git_dir)


def get_git_root(directory: str) -> str:
    """Get the root directory of the git repository."""
    try:
        result = subprocess.run(
            ['git', 'rev-parse', '--show-toplevel'],
            cwd=directory,
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        return None


def validate_git_repository(directory: str) -> tuple[bool, str]:
    """Validate that the directory is a git repository and return status and root."""
    if not is_git_repository(directory):
        return False, None
    
    git_root = get_git_root(directory)
    if not git_root:
        return False, None
    
    return True, git_root


def run_pr_title_generator(args):
    """Run the PR title generator with the given arguments."""
    # Get the directory where this script is located
    script_dir = Path(__file__).parent.absolute()
    generator_script = script_dir / 'generate-pr-title-ml.py'
    
    if not generator_script.exists():
        print(f"Error: Could not find generate-pr-title-ml.py at {generator_script}", file=sys.stderr)
        sys.exit(1)
    
    # Build the command to run the generator
    cmd = [sys.executable, str(generator_script)]
    
    # Add arguments
    if args.branch:
        cmd.extend(['--branch', args.branch])
    if args.base:
        cmd.extend(['--base', args.base])
    if args.max_commits:
        cmd.extend(['--max-commits', str(args.max_commits)])
    if args.model:
        cmd.extend(['--model', args.model])
    if args.temperature:
        cmd.extend(['--temperature', str(args.temperature)])
    if args.max_length:
        cmd.extend(['--max-length', str(args.max_length)])
    if args.verbose:
        cmd.append('--verbose')
    
    # Run the generator
    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print(result.stdout.strip())
    except subprocess.CalledProcessError as e:
        print(f"Error running PR title generator: {e}", file=sys.stderr)
        if e.stderr:
            print(e.stderr, file=sys.stderr)
        sys.exit(1)


def main():
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        description='Generate meaningful PR titles using ML models',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  generate-pr-title                    # Generate title for current branch
  generate-pr-title --verbose          # Enable verbose output
  generate-pr-title --branch feature/auth  # Generate for specific branch
  generate-pr-title --model phi-2      # Use different ML model
        """
    )
    
    parser.add_argument(
        '--branch',
        help='Branch to analyze (defaults to current branch)'
    )
    
    parser.add_argument(
        '--base',
        default='main',
        help='Base branch to compare against (default: main)'
    )
    
    parser.add_argument(
        '--max-commits',
        type=int,
        default=20,
        help='Maximum number of commits to analyze (default: 20)'
    )
    
    parser.add_argument(
        '--model',
        choices=['tiny-llama', 'phi-2', 'gemma-2b', 'llama-2-7b'],
        default='tiny-llama',
        help='LLM model to use (default: tiny-llama)'
    )
    
    parser.add_argument(
        '--temperature',
        type=float,
        default=0.7,
        help='Generation temperature (0.1-1.0, default: 0.7)'
    )
    
    parser.add_argument(
        '--max-length',
        type=int,
        default=50,
        help='Maximum title length (default: 50)'
    )
    
    parser.add_argument(
        '--verbose',
        action='store_true',
        help='Enable verbose output'
    )
    
    args = parser.parse_args()
    
    # Get current working directory
    current_dir = os.getcwd()
    
    # Validate git repository
    is_git, git_root = validate_git_repository(current_dir)
    
    if not is_git:
        print(f"Error: '{current_dir}' is not a git repository.", file=sys.stderr)
        print("Please run this command from within a git repository.", file=sys.stderr)
        sys.exit(1)
    
    if args.verbose:
        print(f"Git repository found at: {git_root}")
        print(f"Current directory: {current_dir}")
    
    # Run the PR title generator
    run_pr_title_generator(args)


if __name__ == '__main__':
    main()
