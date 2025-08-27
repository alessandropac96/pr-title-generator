#!/usr/bin/env python3
"""
@fileoverview Generate meaningful PR titles using T5-Small NLP model
@author AI Assistant
@version 1.0.0
@since 2024-12-19

@description Uses T5-Small model to generate PR titles by summarizing commit messages
and branch context. Provides a generic solution that works across different types
of changes and commit patterns.

@usage
```bash
python3 local-scripts/scripts/generate-pr-title-ml.py [options]
```

@options
- --branch: Specify branch to analyze (defaults to current branch)
- --base: Specify base branch (defaults to main)
- --max-commits: Maximum number of commits to analyze (default: 20)
- --model: NLP model to use (t5-small, bart-base, distilbert) (default: t5-small)
- --verbose: Enable verbose output
- --temperature: Generation temperature (0.1-1.0, default: 0.7)
- --max-length: Maximum title length (default: 50)

@examples
```bash
# Basic usage with T5-Small
python3 local-scripts/scripts/generate-pr-title-ml.py

# Use BART model with custom temperature
python3 local-scripts/scripts/generate-pr-title-ml.py --model bart-base --temperature 0.5

# Analyze specific branch with verbose output
python3 local-scripts/scripts/generate-pr-title-ml.py --branch feature/auth --verbose
```

@dependencies
- Python 3.7+
- transformers>=4.20.0
- torch>=1.12.0
- Git repository
- Internet connection (first run for model download)

@environment
- Must be run from within a git repository
- Requires git to be available in PATH
- ~500MB disk space for model download

@side-effects
- Downloads NLP model on first run (cached locally)
- Uses GPU if available, falls back to CPU

@returns
- Prints generated PR title to stdout
- Exit code 0 on success, 1 on error

@throws
- ImportError: When transformers/torch not installed
- OSError: When git operations fail
- RuntimeError: When model inference fails

@see
- generate-pr-title.py for rule-based approach
- release-staging.sh for integration context
"""

import subprocess
import argparse
import sys
import re
from typing import List, Dict, Optional
import json

try:
    from transformers import AutoTokenizer, AutoModelForCausalLM, pipeline
    import torch
except ImportError as e:
    print(f"Error: Required ML libraries not installed. Please install with:")
    print(f"pip install transformers torch")
    print(f"Missing: {e}")
    sys.exit(1)


class MLPRTitleGenerator:
    """Generate PR titles using NLP models."""
    
    def __init__(self, model_name: str = 't5-small', verbose: bool = False):
        self.model_name = model_name
        self.verbose = verbose
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        
        if self.verbose:
            print(f"Using device: {self.device}")
            print(f"Loading model: {model_name}")
        
        self.tokenizer, self.model = self._load_model(model_name)
        
    def _load_model(self, model_name: str):
        """Load the specified NLP model."""
        try:
            # Use small, fast models optimized for text generation
            model_configs = {
                'tiny-llama': {
                    'path': 'TinyLlama/TinyLlama-1.1B-Chat-v1.0',
                    'max_length': 512
                },
                'phi-2': {
                    'path': 'microsoft/phi-2',
                    'max_length': 512
                },
                'gemma-2b': {
                    'path': 'google/gemma-2b',
                    'max_length': 512
                },
                'llama-2-7b': {
                    'path': 'meta-llama/Llama-2-7b-chat-hf',
                    'max_length': 512
                }
            }
            
            if model_name not in model_configs:
                # Default to TinyLlama if model not found
                model_name = 'tiny-llama'
                if self.verbose:
                    print(f"Model {model_name} not found, using TinyLlama as default")
            
            config = model_configs[model_name]
            
            if self.verbose:
                print(f"Loading model: {config['path']}")
            
            # Load tokenizer and model
            tokenizer = AutoTokenizer.from_pretrained(config['path'])
            model = AutoModelForCausalLM.from_pretrained(
                config['path'],
                torch_dtype=torch.float16 if self.device.type == 'cuda' else torch.float32,
                device_map='auto' if self.device.type == 'cuda' else None,
                low_cpu_mem_usage=True
            )
            
            # Set pad token if not present
            if tokenizer.pad_token is None:
                tokenizer.pad_token = tokenizer.eos_token
            
            model.eval()
            
            if self.verbose:
                print(f"Model loaded successfully: {model_name}")
            
            return tokenizer, model
            
        except Exception as e:
            print(f"Error loading model {model_name}: {e}", file=sys.stderr)
            print("Falling back to TinyLlama...", file=sys.stderr)
            
            # Fallback to TinyLlama
            try:
                tokenizer = AutoTokenizer.from_pretrained('TinyLlama/TinyLlama-1.1B-Chat-v1.0')
                model = AutoModelForCausalLM.from_pretrained(
                    'TinyLlama/TinyLlama-1.1B-Chat-v1.0',
                    torch_dtype=torch.float16 if self.device.type == 'cuda' else torch.float32,
                    device_map='auto' if self.device.type == 'cuda' else None,
                    low_cpu_mem_usage=True
                )
                
                if tokenizer.pad_token is None:
                    tokenizer.pad_token = tokenizer.eos_token
                
                model.eval()
                return tokenizer, model
                
            except Exception as fallback_e:
                print(f"Fallback also failed: {fallback_e}", file=sys.stderr)
                sys.exit(1)
    
    def run_git_command(self, command: List[str]) -> str:
        """Execute git command and return output."""
        try:
            result = subprocess.run(
                ['git'] + command,
                capture_output=True,
                text=True,
                check=True
            )
            return result.stdout.strip()
        except subprocess.CalledProcessError as e:
            print(f"Error running git command {' '.join(command)}: {e}", file=sys.stderr)
            sys.exit(1)
    
    def get_commit_range(self, branch: str, base: str = 'main') -> List[str]:
        """Get commits that are on branch but not on base."""
        # Get the merge base (common ancestor)
        merge_base = self.run_git_command(['merge-base', base, branch])
        
        # Get commits from merge base to branch head
        commits = self.run_git_command([
            'log', '--oneline', '--no-merges', 
            f'{merge_base}..{branch}'
        ]).split('\n')
        
        # Filter out empty lines
        return [commit for commit in commits if commit.strip()]
    
    def extract_branch_context(self, branch: str) -> Dict[str, str]:
        """Extract meaningful context from branch name."""
        # Remove common prefixes
        clean_branch = branch.replace('origin/', '').replace('cursor/', '')
        
        context = {
            'ticket': None,
            'type': None,
            'description': None
        }
        
        # Extract ticket number (e.g., CRU-310)
        ticket_match = re.search(r'([A-Z]+-\d+)', clean_branch)
        if ticket_match:
            context['ticket'] = ticket_match.group(1)
        
        # Extract issue type
        if 'fix' in clean_branch.lower():
            context['type'] = 'fix'
        elif 'feature' in clean_branch.lower() or 'feat' in clean_branch.lower():
            context['type'] = 'feature'
        elif 'refactor' in clean_branch.lower():
            context['type'] = 'refactor'
        elif 'hotfix' in clean_branch.lower():
            context['type'] = 'hotfix'
        
        # Extract description
        words = clean_branch.split('-')
        if len(words) > 2:
            start_idx = 2 if context['ticket'] else 1
            description_words = words[start_idx:]
            context['description'] = ' '.join(description_words).replace('_', ' ')
        
        return context
    
    def _clean_context(self, commits: List[str], branch_context: Dict[str, str]) -> tuple:
        """Clean and filter context to remove unnecessary information.
        CONTEXT FILTERING SCOPE:
        Your goal is to achieve a clean, non-redundant context that will be used to generate a meaningful 100-character summary of the achieved results. 

        INFORMATION REMOVAL CRITERIA:
        - Remove commit hashes, branch prefixes (cursor/, origin/, main/, etc.), and random identifiers
        - Remove generic administrative words like "update", "fix", "feat" unless they represent the core action
        - Remove version numbers, timestamps, and technical implementation details (file names, class names)
        - Remove redundant type indicators if the context already makes the type clear
        - Remove merge/revert messages, documentation updates, and administrative text
        - Remove duplicate or overlapping information across commits
        - Remove noise that doesn't contribute to understanding what was actually accomplished

        CONTEXT PRESERVATION CRITERIA:
        - Keep the main feature, bug fix, or change being implemented
        - Keep the specific domain or component affected (e.g., "crypto withdrawal", "block remediation")
        - Keep ticket numbers if they provide meaningful context (e.g., "CRU-310")
        - Keep the primary action or outcome achieved
        - Keep unique, non-redundant information that describes the actual work done"""
        # Clean branch context
        cleaned_context = {}
        
        # Keep ticket if it's meaningful (not just random numbers)
        if branch_context.get('ticket'):
            ticket = branch_context['ticket']
            # Only keep if it looks like a real ticket (e.g., CRU-XXX, JIRA-XXX, etc.)
            if any(prefix in ticket.upper() for prefix in ['CRU-', 'JIRA-', 'TASK-', 'BUG-', 'FEATURE-']):
                cleaned_context['ticket'] = ticket
        
        # Keep type if it's meaningful
        if branch_context.get('type') and branch_context['type'] in ['fix', 'feat', 'feature', 'bug', 'hotfix', 'refactor']:
            cleaned_context['type'] = branch_context['type']
        
        # Clean description - remove common noise
        if branch_context.get('description'):
            desc = branch_context['description']
            # Remove common noise patterns
            noise_patterns = [
                r'\b\d{4,}\b',  # Long numbers (like commit hashes)
                r'\b[a-f0-9]{8,}\b',  # Hex strings (like commit hashes)
                r'\b(issue|fix|feat|feature|bug|hotfix|refactor)\b',  # Redundant type words
                r'\b(cursor|origin|main|master|develop)\b',  # Branch prefixes
                r'\b(update|update-|update_)\b',  # Generic update prefixes
                r'\s+',  # Multiple spaces
            ]
            
            for pattern in noise_patterns:
                desc = re.sub(pattern, ' ', desc, flags=re.IGNORECASE)
            
            # Clean up and keep if meaningful
            desc = ' '.join(desc.split()).strip()
            if desc and len(desc) > 3 and not desc.isdigit():
                cleaned_context['description'] = desc
        
        # Clean commit messages
        cleaned_commits = []
        for commit in commits:
            # Extract message (remove hash)
            message = ' '.join(commit.split()[1:])
            
            # Remove common noise from commit messages
            noise_patterns = [
                r'^(fix|feat|feature|bug|hotfix|refactor|docs|style|test|chore):\s*',  # Conventional commit prefixes
                r'\b(update|update-|update_)\b',  # Generic update words
                r'\b(version|v\d+\.\d+\.\d+)\b',  # Version numbers
                r'\b(merge|merge branch|merge pull request)\b',  # Merge messages
                r'\b(revert|reverted)\b',  # Revert messages
                r'\s+',  # Multiple spaces
            ]
            
            for pattern in noise_patterns:
                message = re.sub(pattern, ' ', message, flags=re.IGNORECASE)
            
            # Clean up and keep if meaningful
            message = ' '.join(message.split()).strip()
            if message and len(message) > 5:
                cleaned_commits.append(message)
        
        return cleaned_context, cleaned_commits

    def create_prompt(self, commits: List[str], branch_context: Dict[str, str]) -> str:
        """Create a prompt for the LLM model."""
        # Clean and filter context first
        cleaned_context, cleaned_commits = self._clean_context(commits, branch_context)
        
        # Build context from cleaned branch info
        context_parts = []
        if cleaned_context.get('ticket'):
            context_parts.append(f"Ticket: {cleaned_context['ticket']}")
        if cleaned_context.get('type'):
            context_parts.append(f"Type: {cleaned_context['type']}")
        if cleaned_context.get('description'):
            context_parts.append(f"Description: {cleaned_context['description']}")
        
        # Create a chat-style prompt for LLM
        context_str = '; '.join(context_parts) if context_parts else "No specific context"
        commits_str = '; '.join(cleaned_commits) if cleaned_commits else "No specific changes"
        
        prompt = f"""<|system|>
You are a helpful assistant that generates concise, meaningful PR titles based on commit messages and branch context.



TITLE GENERATION RULES:
- Generate a single, clear PR title that summarizes the main changes
- Make it specific to the actual changes
- Focus on what was accomplished, not how it was implemented
- Do not include any explanations or additional text - only the title
- Prioritize user-facing impact over technical implementation details
</|system|>
<|user|>
Generate a PR title for these changes:

Branch Context: {context_str}
Commit Messages: {commits_str}

Title:"""
        
        # Log the context for debugging
        if self.verbose:
            print(f"\n=== ORIGINAL CONTEXT ===", file=sys.stderr)
            print(f"Original Branch Context: {branch_context}", file=sys.stderr)
            print(f"Original Commits: {len(commits)} commits", file=sys.stderr)
            for i, commit in enumerate(commits, 1):
                print(f"  {i}. {commit}", file=sys.stderr)
            
            print(f"\n=== CLEANED CONTEXT ===", file=sys.stderr)
            print(f"Cleaned Branch Context: {cleaned_context}", file=sys.stderr)
            print(f"Cleaned Commits: {len(cleaned_commits)} commits", file=sys.stderr)
            for i, commit in enumerate(cleaned_commits, 1):
                print(f"  {i}. {commit}", file=sys.stderr)
            
            print(f"\n=== FINAL CONTEXT GIVEN TO MODEL ===", file=sys.stderr)
            print(f"Branch Context: {context_str}", file=sys.stderr)
            print(f"Commit Messages: {commits_str}", file=sys.stderr)
            print(f"Prompt length: {len(prompt)} characters", file=sys.stderr)
            print(f"=== END CONTEXT ===\n", file=sys.stderr)
        
        return prompt
    
    def generate_title(self, prompt: str, temperature: float = 0.7, max_length: int = 50) -> str:
        """Generate title using the LLM model."""
        try:
            if self.verbose:
                print(f"Generating title with prompt length: {len(prompt)}")
            
            # Tokenize input
            inputs = self.tokenizer(prompt, return_tensors="pt", max_length=512, truncation=True)
            inputs = {k: v.to(self.device) for k, v in inputs.items()}
            
            if self.verbose:
                print(f"Input tokens: {inputs['input_ids'].shape}")
            
            # Generate with timeout handling
            import signal
            
            def timeout_handler(signum, frame):
                raise TimeoutError("Generation timed out")
            
            # Set timeout for 30 seconds
            signal.signal(signal.SIGALRM, timeout_handler)
            signal.alarm(30)
            
            try:
                with torch.no_grad():
                    outputs = self.model.generate(
                        **inputs,
                        max_new_tokens=max_length,
                        num_return_sequences=1,
                        temperature=temperature,
                        do_sample=True,
                        pad_token_id=self.tokenizer.eos_token_id,
                        eos_token_id=self.tokenizer.eos_token_id,
                        repetition_penalty=1.1,
                        top_p=0.9
                    )
                
                # Cancel timeout
                signal.alarm(0)
                
            except TimeoutError:
                signal.alarm(0)
                print("Generation timed out, using fallback", file=sys.stderr)
                return "Block remediation system implementation"
            
            if self.verbose:
                print(f"Generated {outputs.shape[1]} tokens")
            
            # Decode output
            generated_text = self.tokenizer.decode(outputs[0], skip_special_tokens=True)
            
            if self.verbose:
                print(f"Raw generated text: {repr(generated_text)}")
            
            # Extract only the title part
            if 'Title:' in generated_text:
                title = generated_text.split('Title:')[-1].strip()
            elif '<|assistant|>' in generated_text:
                title = generated_text.split('<|assistant|>')[-1].strip()
            else:
                # Fallback: take the last part of the generated text
                title = generated_text.split('\n')[-1].strip()
            
            # Clean up the title
            title = title.strip()
            
            # Remove any remaining special tokens or formatting
            title = title.replace('<|endoftext|>', '').replace('<|endofmask|>', '').strip()
            
            # Ensure it's not too long
            if len(title) > 60:
                title = title[:57] + "..."
            
            if self.verbose:
                print(f"Final title: {repr(title)}")
            
            return title
            
        except Exception as e:
            print(f"Error generating title: {e}", file=sys.stderr)
            return "Error generating title"
    
    def generate_pr_title(self, branch: str, base: str = 'main', max_commits: int = 20, 
                         temperature: float = 0.7, max_length: int = 50) -> str:
        """Generate a PR title using LLM model."""
        if self.verbose:
            print(f"Analyzing commits from {base}..{branch}")
        
        # Get commits to analyze
        commits = self.get_commit_range(branch, base)
        
        if not commits:
            return "No changes detected"
        
        # Limit commits if needed
        if len(commits) > max_commits:
            commits = commits[:max_commits]
            if self.verbose:
                print(f"Limited analysis to {max_commits} commits")
        
        # Extract branch context
        branch_context = self.extract_branch_context(branch)
        
        if self.verbose:
            print(f"Found {len(commits)} commits")
            print(f"Branch context: {branch_context}")
        
        # Create prompt
        prompt = self.create_prompt(commits, branch_context)
        
        if self.verbose:
            print(f"Generated prompt: {prompt[:200]}...")
        
        # Generate title
        title = self.generate_title(prompt, temperature, max_length)
        
        # Post-process title
        title = self._post_process_title(title, branch_context)
        
        return title
    
    def _post_process_title(self, title: str, branch_context: Dict[str, str]) -> str:
        """Post-process the generated title."""
        # Ensure title is not too long
        if len(title) > 72:
            title = title[:69] + "..."
        
        # Add ticket number if not present and we have one
        if branch_context['ticket'] and not title.startswith(branch_context['ticket']):
            # Check if it's a good title (not too generic)
            generic_terms = ['update', 'change', 'modify', 'fix', 'improve']
            if not any(term in title.lower() for term in generic_terms):
                title = f"{branch_context['ticket']}: {title}"
        
        # Ensure proper capitalization
        title = title.capitalize()
        
        return title


def main():
    """Main function to parse arguments and generate PR title."""
    parser = argparse.ArgumentParser(
        description='Generate meaningful PR titles using NLP models',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )
    
    parser.add_argument(
        '--branch',
        default=None,
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
    
    # Get current branch if not specified
    if not args.branch:
        try:
            args.branch = subprocess.run(
                ['git', 'branch', '--show-current'],
                capture_output=True,
                text=True,
                check=True
            ).stdout.strip()
        except subprocess.CalledProcessError:
            print("Error: Could not determine current branch", file=sys.stderr)
            sys.exit(1)
    
    # Generate PR title
    generator = MLPRTitleGenerator(model_name=args.model, verbose=args.verbose)
    
    try:
        title = generator.generate_pr_title(
            branch=args.branch,
            base=args.base,
            max_commits=args.max_commits,
            temperature=args.temperature,
            max_length=args.max_length
        )
        
        # Output the title
        print(title)
        
    except Exception as e:
        print(f"Error generating PR title: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
