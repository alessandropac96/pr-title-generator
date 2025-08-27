#!/bin/bash

# PR Title Generator Installation Script
# This script installs the PR title generator as a global CLI tool

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

print_status "Installing PR Title Generator..."

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    print_error "Python 3 is required but not installed."
    print_error "Please install Python 3.7 or higher and try again."
    exit 1
fi

PYTHON_VERSION=$(python3 -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
print_status "Found Python $PYTHON_VERSION"

# Check if pip is available
if ! command -v pip3 &> /dev/null; then
    print_error "pip3 is required but not installed."
    print_error "Please install pip3 and try again."
    exit 1
fi

# Install Python dependencies
print_status "Installing Python dependencies..."
pip3 install -r "$SCRIPT_DIR/requirements.txt"

# Create the CLI script
CLI_SCRIPT="$SCRIPT_DIR/generate-pr-title"
cat > "$CLI_SCRIPT" << 'EOF'
#!/bin/bash

# PR Title Generator CLI Wrapper
# This script allows calling the PR title generator from anywhere

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Run the main Python script
python3 "$SCRIPT_DIR/main.py" "$@"
EOF

# Make the CLI script executable
chmod +x "$CLI_SCRIPT"

# Determine installation directory
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    INSTALL_DIR="/usr/local/bin"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    if [[ -d "/usr/local/bin" ]]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="$HOME/.local/bin"
        # Add to PATH if not already there
        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
            print_warning "Added $INSTALL_DIR to PATH in ~/.bashrc"
            print_warning "Please restart your terminal or run: source ~/.bashrc"
        fi
    fi
else
    print_error "Unsupported operating system: $OSTYPE"
    exit 1
fi

# Create symlink
SYMLINK="$INSTALL_DIR/generate-pr-title"
if [[ -L "$SYMLINK" ]]; then
    print_status "Removing existing symlink..."
    rm "$SYMLINK"
fi

print_status "Creating symlink in $INSTALL_DIR..."
if [[ "$EUID" -eq 0 ]]; then
    # Running as root
    ln -sf "$CLI_SCRIPT" "$SYMLINK"
else
    # Not running as root, try to create symlink
    if [[ -w "$INSTALL_DIR" ]]; then
        ln -sf "$CLI_SCRIPT" "$SYMLINK"
    else
        print_warning "Cannot write to $INSTALL_DIR without sudo"
        print_warning "Creating symlink in user directory instead..."
        USER_INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$USER_INSTALL_DIR"
        ln -sf "$CLI_SCRIPT" "$USER_INSTALL_DIR/generate-pr-title"
        
        # Add to PATH if not already there
        if [[ ":$PATH:" != *":$USER_INSTALL_DIR:"* ]]; then
            if [[ -f "$HOME/.bashrc" ]]; then
                echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
                print_warning "Added $USER_INSTALL_DIR to PATH in ~/.bashrc"
                print_warning "Please restart your terminal or run: source ~/.bashrc"
            elif [[ -f "$HOME/.zshrc" ]]; then
                echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
                print_warning "Added $USER_INSTALL_DIR to PATH in ~/.zshrc"
                print_warning "Please restart your terminal or run: source ~/.zshrc"
            fi
        fi
    fi
fi

print_success "Installation completed successfully!"
print_status "You can now use the command: generate-pr-title"
print_status "Examples:"
echo "  generate-pr-title                    # Generate title for current branch"
echo "  generate-pr-title --verbose          # Enable verbose output"
echo "  generate-pr-title --branch feature/auth  # Generate for specific branch"
echo "  generate-pr-title --model phi-2      # Use different ML model"

# Test if the command is available
if command -v generate-pr-title &> /dev/null; then
    print_success "Command 'generate-pr-title' is available in PATH"
else
    print_warning "Command 'generate-pr-title' may not be available in current shell"
    print_warning "Please restart your terminal or run: source ~/.bashrc (or ~/.zshrc)"
fi
