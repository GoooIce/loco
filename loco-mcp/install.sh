#!/bin/bash

# Loco MCP Server Installation Script
# This script installs the loco-mcp-server and its dependencies

set -e

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

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

# Function to install Rust
install_rust() {
    print_status "Installing Rust..."
    if command_exists rustup; then
        print_success "Rust is already installed"
        rustup update
    else
        print_status "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        print_success "Rust installed successfully"
    fi
}

# Function to install Python
install_python() {
    local os=$(detect_os)

    print_status "Checking Python installation..."

    if command_exists python3; then
        python_version=$(python3 -c 'import sys; print(".".join(map(str, sys.version_info[:2])))')
        print_success "Python $python_version is already installed"

        # Check if version is >= 3.11
        if python3 -c 'import sys; exit(0 if sys.version_info >= (3, 11) else 1)'; then
            print_success "Python version meets requirements (>=3.11)"
        else
            print_error "Python version is too old. Requires Python 3.11 or later"
            print_status "Current version: $(python3 --version)"
            print_status "Please install Python 3.11+ and try again"
            exit 1
        fi
    else
        print_error "Python 3 is not installed"
        case $os in
            "linux")
                print_status "On Ubuntu/Debian, run: sudo apt update && sudo apt install python3 python3-pip python3-venv"
                print_status "On Fedora, run: sudo dnf install python3 python3-pip"
                print_status "On Arch, run: sudo pacman -S python python-pip"
                ;;
            "macos")
                print_status "Install Python using Homebrew: brew install python@3.11"
                print_status "Or download from https://www.python.org/"
                ;;
            "windows")
                print_status "Download Python from https://www.python.org/"
                ;;
        esac
        exit 1
    fi
}

# Function to install system dependencies
install_system_deps() {
    local os=$(detect_os)

    case $os in
        "linux")
            print_status "Installing system dependencies for Linux..."
            if command_exists apt-get; then
                sudo apt-get update
                sudo apt-get install -y build-essential pkg-config libssl-dev
            elif command_exists dnf; then
                sudo dnf groupinstall -y "Development Tools"
                sudo dnf install -y openssl-devel pkgconfig
            elif command_exists pacman; then
                sudo pacman -S --needed base-devel pkgconf openssl
            else
                print_warning "Could not detect package manager. Please install build tools manually."
            fi
            ;;
        "macos")
            print_status "Checking system dependencies for macOS..."
            if command_exists xcode-select; then
                xcode-select --install 2>/dev/null || print_success "Xcode command line tools are already installed"
            fi

            if command_exists brew; then
                print_success "Homebrew is already installed"
            else
                print_status "Installing Homebrew..."
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            fi
            ;;
        "windows")
            print_status "On Windows, please ensure you have:"
            print_status "- Microsoft Visual Studio Build Tools (latest)"
            print_status "- Rust installer from https://rustup.rs/"
            print_status "- Python installer from https://www.python.org/"
            ;;
    esac
}

# Function to setup Python environment
setup_python_env() {
    print_status "Setting up Python environment..."

    # Create virtual environment if it doesn't exist
    if [ ! -d "venv" ]; then
        python3 -m venv venv
        print_success "Virtual environment created"
    else
        print_status "Virtual environment already exists"
    fi

    # Activate virtual environment
    source venv/bin/activate

    # Upgrade pip
    print_status "Upgrading pip..."
    pip install --upgrade pip

    # Install development tools
    print_status "Installing development tools..."
    pip install maturin[patchelf] black ruff mypy pytest pytest-asyncio pytest-cov

    print_success "Python environment setup complete"
}

# Function to install loco-mcp-server
install_loco_mcp() {
    print_status "Installing loco-mcp-server..."

    # Install loco-bindings first
    print_status "Building and installing loco-bindings..."
    cd loco-bindings

    if [ ! -f "pyproject.toml" ]; then
        print_error "pyproject.toml not found in loco-bindings directory"
        exit 1
    fi

    # Build and install the Rust bindings
    maturin develop --release

    if [ $? -eq 0 ]; then
        print_success "loco-bindings installed successfully"
    else
        print_error "Failed to install loco-bindings"
        exit 1
    fi

    cd ..

    # Install loco-mcp-server
    print_status "Installing loco-mcp-server..."
    cd loco-mcp-server

    if [ ! -f "pyproject.toml" ]; then
        print_error "pyproject.toml not found in loco-mcp-server directory"
        exit 1
    fi

    pip install -e .

    if [ $? -eq 0 ]; then
        print_success "loco-mcp-server installed successfully"
    else
        print_error "Failed to install loco-mcp-server"
        exit 1
    fi

    cd ..
}

# Function to verify installation
verify_installation() {
    print_status "Verifying installation..."

    # Test loco-bindings import
    python3 -c "
import loco_bindings
print('✅ loco-bindings imported successfully')
print(f'✅ Version: {getattr(loco_bindings, \"__version__\", \"unknown\")}')
" || {
        print_error "Failed to import loco-bindings"
        exit 1
    }

    # Test loco-mcp-server import
    python3 -c "
import loco_mcp_server
print('✅ loco-mcp-server imported successfully')
" || {
        print_error "Failed to import loco-mcp-server"
        exit 1
    }

    # Test CLI command
    if command_exists loco-mcp-server; then
        print_success "loco-mcp-server CLI command is available"
    else
        print_warning "loco-mcp-server CLI command not found in PATH"
        print_status "You may need to add the virtual environment's bin directory to your PATH"
        print_status "Run: source venv/bin/activate && which loco-mcp-server"
    fi

    print_success "Installation verification complete!"
}

# Function to create wrapper script
create_wrapper_script() {
    local install_dir="$HOME/.local/bin"

    print_status "Creating wrapper script..."

    # Create .local/bin directory if it doesn't exist
    mkdir -p "$install_dir"

    # Create wrapper script
    cat > "$install_dir/loco-mcp-server" << 'EOF'
#!/bin/bash
# Loco MCP Server wrapper script

# Find the virtual environment
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_PATH=""
POSSIBLE_PATHS=(
    "$HOME/loco-mcp/venv"
    "$HOME/.local/share/loco-mcp/venv"
    "./venv"
    "../venv"
)

for path in "${POSSIBLE_PATHS[@]}"; do
    if [ -d "$path" ]; then
        VENV_PATH="$path"
        break
    fi
done

if [ -z "$VENV_PATH" ]; then
    echo "Error: Could not find loco-mcp virtual environment"
    echo "Please run the installation script again"
    exit 1
fi

# Activate virtual environment and run the server
source "$VENV_PATH/bin/activate"
exec python -m loco_mcp_server "$@"
EOF

    chmod +x "$install_dir/loco-mcp-server"

    # Add to PATH if not already there
    if [[ ":$PATH:" != *":$install_dir:"* ]]; then
        print_status "Adding $install_dir to PATH..."
        echo "export PATH=\"$install_dir:\$PATH\"" >> "$HOME/.bashrc"
        echo "export PATH=\"$install_dir:\$PATH\"" >> "$HOME/.zshrc" 2>/dev/null || true

        print_status "PATH updated. You may need to restart your shell or run:"
        print_status "export PATH=\"$install_dir:\$PATH\""
    fi

    print_success "Wrapper script created: $install_dir/loco-mcp-server"
}

# Function to display next steps
show_next_steps() {
    print_success "Installation completed successfully!"
    echo
    echo "Next steps:"
    echo "1. Activate the virtual environment:"
    echo "   source venv/bin/activate"
    echo
    echo "2. Start the MCP server:"
    echo "   loco-mcp-server"
    echo "   # or"
    echo "   python -m loco_mcp_server"
    echo
    echo "3. Test the installation:"
    echo "   python -c \"import loco_bindings; print('✅ Success!')\""
    echo
    echo "4. Configure Claude Code to connect to the server"
    echo "   (See README.md for configuration details)"
    echo
    echo "5. Start generating loco-rs code with Claude Code!"
    echo
}

# Main installation function
main() {
    echo "🚀 Loco MCP Server Installation Script"
    echo "======================================"
    echo

    print_status "This script will install:"
    echo "  • Rust (if not already installed)"
    echo "  • Python 3.11+ (if not already installed)"
    echo "  • System dependencies"
    echo "  • loco-bindings (Rust-Python bindings)"
    echo "  • loco-mcp-server (MCP server)"
    echo

    read -p "Do you want to continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi

    # Check if we're in the right directory
    if [ ! -f "loco-bindings/Cargo.toml" ] || [ ! -f "loco-mcp-server/pyproject.toml" ]; then
        print_error "Please run this script from the loco-mcp root directory"
        print_error "The directory should contain loco-bindings/ and loco-mcp-server/ subdirectories"
        exit 1
    fi

    # Installation steps
    install_system_deps
    install_rust
    install_python
    setup_python_env
    install_loco_mcp
    verify_installation
    create_wrapper_script
    show_next_steps
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Loco MCP Server Installation Script"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --verify      Verify existing installation"
        echo "  --uninstall    Uninstall loco-mcp-server"
        echo
        exit 0
        ;;
    --verify)
        print_status "Verifying existing installation..."
        verify_installation
        exit 0
        ;;
    --uninstall)
        print_status "Uninstalling loco-mcp-server..."
        pip uninstall -y loco-mcp-server loco-bindings 2>/dev/null || true
        rm -f "$HOME/.local/bin/loco-mcp-server"
        print_success "Uninstallation complete"
        exit 0
        ;;
    "")
        main
        ;;
    *)
        print_error "Unknown option: $1"
        print_status "Use --help for available options"
        exit 1
        ;;
esac